// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Structural parser for one JSDoc block, emitting Binary AST directly.
//!
//! Mirrors the typed-AST `crates/ox_jsdoc/src/parser/context.rs` but
//! replaces every `ArenaBox::new_in(JsdocXxx { ... })` with a `write_*`
//! call against a [`BinaryWriter`].
//!
//! Two phases:
//!
//! 1. **Parse** — `parse_block_into_data` walks the source text and
//!    collects intermediate plain-data structs (no allocation in the
//!    arena-tracked binary buffer). All slices borrow from the original
//!    source text.
//! 2. **Emit** — `emit_block` walks those structs in DFS pre-order and
//!    invokes the matching `write_*` helper. Each emission knows its
//!    parent's `NodeIndex` because we strictly write top-down.
//!
//! The split keeps the `children_bitmask` upfront (which the writer needs)
//! while still allowing the parser to make local decisions (e.g. drop
//! empty lists).
//!
//! `parsed_type` emission is already integrated through the side-table path
//! used by `TagData`; ongoing work here is about keeping the structural parse
//! and emit layers aligned with the format and typed-AST behavior.

use oxc_allocator::{Allocator, StringBuilder, Vec as ArenaVec};
use oxc_span::Span;

/// Path C-2: arena-allocated inline-tag list. Was `SmallVec<[_; 2]>` but
/// SmallVec implements `Drop` (for the spilled-heap case), which prevents
/// the surrounding `TagData` from itself being placed in `ArenaVec`. The
/// `InlineTagData` payload is `Copy`, so `ArenaVec` is the cleanest fit.
type InlineTagsVec<'arena, 'a> = ArenaVec<'arena, InlineTagData<'a>>;
use crate::writer::nodes::comment_ast::{
    write_jsdoc_block, write_jsdoc_block_compat_tail, write_jsdoc_description_line,
    write_jsdoc_description_line_compat, write_jsdoc_generic_tag_body, write_jsdoc_identifier,
    write_jsdoc_inline_tag, write_jsdoc_namepath_source, write_jsdoc_parameter_name,
    write_jsdoc_tag, write_jsdoc_tag_compat_tail, write_jsdoc_tag_name, write_jsdoc_tag_name_value,
    write_jsdoc_text, write_jsdoc_type_line, write_jsdoc_type_line_compat, write_jsdoc_type_source,
};
use crate::writer::{BinaryWriter, StringField};

use super::ParseOptions;
use super::checkpoint::{Checkpoint, FenceState, QuoteKind};
use super::diagnostics::{DiagnosticKind, ParserDiagnosticKind, TypeDiagnosticKind};
use super::scanner;
use super::type_data::TypeNodeData;

// ---------------------------------------------------------------------------
// Diagnostic + parsed-data types
// ---------------------------------------------------------------------------

/// One diagnostic emitted while parsing a comment.
///
/// The parser stores the diagnostic kind + an optional source span so the
/// caller can decide how to format the human-readable message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedDiagnostic {
    /// Either a structural-parser or a type-parser diagnostic kind.
    pub kind: DiagnosticKind,
    /// Source span the diagnostic refers to, when known.
    pub span: Option<Span>,
}

impl ParsedDiagnostic {
    /// Convenience: get the static message string for this diagnostic.
    #[inline]
    #[must_use]
    pub const fn message(&self) -> &'static str {
        self.kind.message()
    }
}

/// Inline tag body format mirroring `ox_jsdoc::ast::JsdocInlineTagFormat`.
///
/// Stored as a `u8` in the binary record's Common Data slot (3 bits used).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InlineTagFormatData {
    /// `{@link target}` — no separator, no display text.
    Plain = 0,
    /// `{@link target|text}` — pipe separator.
    Pipe = 1,
    /// `{@link target text}` — whitespace separator.
    Space = 2,
    /// `{@link prefix:body}` — prefix-style.
    Prefix = 3,
    /// Could not classify.
    Unknown = 4,
}

#[derive(Debug, Clone, Copy)]
struct TypeSourceData<'a> {
    span: Span,
    raw: &'a str,
}

#[derive(Debug, Clone, Copy)]
struct TypeLineData<'a> {
    span: Span,
    raw_type: &'a str,
    delimiter: &'a str,
    post_delimiter: &'a str,
    initial: &'a str,
}

#[derive(Debug, Clone, Copy)]
struct DescriptionLineData<'a> {
    span: Span,
    description: &'a str,
    delimiter: &'a str,
    post_delimiter: &'a str,
    initial: &'a str,
}

#[derive(Debug, Clone, Copy)]
struct InlineTagData<'a> {
    span: Span,
    tag_name_span: Span,
    tag_name: &'a str,
    namepath_or_url: Option<&'a str>,
    text: Option<&'a str>,
    format: InlineTagFormatData,
    raw_body: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
struct TagNameValueData<'a> {
    span: Span,
    raw: &'a str,
}

#[derive(Debug, Clone, Copy)]
enum TagValueData<'a> {
    Parameter { span: Span, path: &'a str, optional: bool, default_value: Option<&'a str> },
    Namepath { span: Span, raw: &'a str },
    Identifier { span: Span, name: &'a str },
    Raw { span: Span, value: &'a str },
}

#[derive(Debug)]
struct GenericTagBodyData<'a> {
    span: Span,
    has_dash_separator: bool,
    type_source: Option<TypeSourceData<'a>>,
    value: Option<TagValueData<'a>>,
    description: Option<&'a str>,
}

#[derive(Debug)]
enum TagBodyData<'a> {
    Generic(GenericTagBodyData<'a>),
}

#[derive(Debug)]
struct TagData<'arena, 'a> {
    span: Span,
    tag_name_span: Span,
    tag_name: &'a str,
    optional: bool,
    default_value: Option<&'a str>,
    description: Option<&'a str>,
    /// Source-text-relative `(start, end)` byte offsets covering the raw
    /// description region (with `*` prefix and blank lines intact).
    /// `None` when the tag has no description. Emitted in compat-mode wire
    /// format per `design/008-oxlint-oxfmt-support/README.md` §4.2.
    description_raw_span: Option<(u32, u32)>,
    raw_body: Option<&'a str>,
    raw_type: Option<TypeSourceData<'a>>,
    name: Option<TagNameValueData<'a>>,
    /// Path C-2: `true` when this tag has a parsed type expression.
    /// The actual `TypeNodeData` lives in [`BlockData::tags_parsed_types`]
    /// (a side table) so `TagData` can stay non-Drop and live in `ArenaVec`.
    has_parsed_type: bool,
    body: Option<TagBodyData<'a>>,
    /// Path C: arena-allocated. Walked once during emit, no need for std heap.
    description_lines: ArenaVec<'arena, DescriptionLineData<'a>>,
    type_lines: ArenaVec<'arena, TypeLineData<'a>>,
    inline_tags: InlineTagsVec<'arena, 'a>,
    header_initial: &'a str,
    header_delimiter: &'a str,
    header_post_delimiter: &'a str,
    header_line_end: &'a str,
    /// `post_tag` source-preserving slot (compat mode).
    post_tag: &'a str,
    /// `post_type` source-preserving slot (compat mode).
    post_type: &'a str,
    /// `post_name` source-preserving slot (compat mode).
    post_name: &'a str,
}

#[derive(Debug)]
struct BlockData<'arena, 'a> {
    span: Span,
    description: Option<&'a str>,
    /// Source-text-relative `(start, end)` byte offsets covering the raw
    /// block description region (with `*` prefix and blank lines intact).
    /// Same shape as [`TagData::description_raw_span`].
    description_raw_span: Option<(u32, u32)>,
    description_lines: ArenaVec<'arena, DescriptionLineData<'a>>,
    inline_tags: InlineTagsVec<'arena, 'a>,
    /// Path C-2: arena-allocated. `TagData` is now non-Drop (parsed_type
    /// moved to side table below) so `ArenaVec` accepts it.
    tags: ArenaVec<'arena, TagData<'arena, 'a>>,
    /// Side table: `(tag_index, parsed_type)` pairs for tags with
    /// `has_parsed_type = true`. Sorted by `tag_index` ascending. Empty
    /// when `parse_types: false` (the default), which keeps the emit
    /// loop's lookup branch fully predictable.
    tags_parsed_types: Vec<(u32, Box<TypeNodeData<'a>>)>,
    line_end: &'a str,
    delimiter_line_break: &'a str,
    preterminal_line_break: &'a str,
    /// 0-based line index of the closing `*/` line (compat mode).
    end_line: u32,
    description_start_line: Option<u32>,
    description_end_line: Option<u32>,
    last_description_line: Option<u32>,
    has_preterminal_description: u8,
    has_preterminal_tag_description: Option<u8>,
}

// ---------------------------------------------------------------------------
// ParserContext
// ---------------------------------------------------------------------------

/// Stateful parser for one JSDoc block, producing intermediate data that
/// the [`emit_block`] free function later flushes to a [`BinaryWriter`].
///
/// Mirrors `ParserContext` in the typed-AST parser (same offset/depth/
/// quote/fence state) but stores diagnostics as plain
/// [`ParsedDiagnostic`] so the binary parser does not depend on
/// `oxc_diagnostics`.
pub struct ParserContext<'arena, 'a> {
    /// Path C: arena reference for allocating intermediate parse data
    /// (description_lines, tags, etc.) into the bump allocator instead of
    /// the std heap. The arena is the same one the writer uses; allocations
    /// here live until the writer drops it.
    pub(crate) arena: &'arena Allocator,
    /// Complete source slice for one JSDoc block.
    pub(crate) source_text: &'a str,
    /// Absolute byte offset of `source_text` in the original file.
    pub(crate) base_offset: u32,
    /// Current parser offset relative to `source_text`.
    pub(crate) offset: u32,
    /// Feature switches for this parse.
    pub(crate) options: ParseOptions,
    /// Diagnostics emitted while parsing this comment.
    pub(crate) diagnostics: Vec<ParsedDiagnostic>,
    /// Current nested `{...}` depth for speculative scanners.
    pub(crate) brace_depth: u16,
    /// Current nested `[...]` depth for speculative scanners.
    pub(crate) bracket_depth: u16,
    /// Current nested `(...)` depth for speculative scanners.
    pub(crate) paren_depth: u16,
    /// Active quote context for speculative scanners.
    pub(crate) quote: Option<QuoteKind>,
    /// Active fenced code context for speculative scanners.
    pub(crate) fence: Option<FenceState>,
}

impl<'arena, 'a> ParserContext<'arena, 'a> {
    /// Create a parser context for one complete comment block.
    #[must_use]
    pub fn new(
        arena: &'arena Allocator,
        source_text: &'a str,
        base_offset: u32,
        options: ParseOptions,
    ) -> Self {
        Self {
            arena,
            source_text,
            base_offset,
            offset: 0,
            options,
            diagnostics: Vec::new(),
            brace_depth: 0,
            bracket_depth: 0,
            paren_depth: 0,
            quote: None,
            fence: None,
        }
    }

    /// Capture rewindable parser state.
    #[must_use]
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            offset: self.offset,
            brace_depth: self.brace_depth,
            bracket_depth: self.bracket_depth,
            paren_depth: self.paren_depth,
            quote: self.quote,
            fence: self.fence,
            diagnostics_len: self.diagnostics.len(),
        }
    }

    /// Restore a previous checkpoint and discard diagnostics emitted after it.
    pub fn rewind(&mut self, checkpoint: Checkpoint) {
        self.offset = checkpoint.offset;
        self.brace_depth = checkpoint.brace_depth;
        self.bracket_depth = checkpoint.bracket_depth;
        self.paren_depth = checkpoint.paren_depth;
        self.quote = checkpoint.quote;
        self.fence = checkpoint.fence;
        self.diagnostics.truncate(checkpoint.diagnostics_len);
    }

    fn diag(&mut self, kind: ParserDiagnosticKind) {
        self.diagnostics.push(ParsedDiagnostic { kind: DiagnosticKind::Parser(kind), span: None });
    }

    /// Push a type-parser diagnostic.
    pub(crate) fn type_diag(&mut self, kind: TypeDiagnosticKind) {
        self.diagnostics.push(ParsedDiagnostic { kind: DiagnosticKind::Type(kind), span: None });
    }

    fn absolute_end(&self) -> Option<u32> {
        let len = u32::try_from(self.source_text.len()).ok()?;
        self.base_offset.checked_add(len)
    }
}

/// Outcome of [`parse_block_into_data`]: either a parsed block + diagnostics,
/// or just diagnostics when the input could not be parsed at all (not a
/// JSDoc block, unclosed, span overflow).
#[derive(Debug)]
pub struct ParsedBlock<'arena, 'a> {
    block: Option<BlockData<'arena, 'a>>,
    diagnostics: Vec<ParsedDiagnostic>,
}

impl<'arena, 'a> ParsedBlock<'arena, 'a> {
    /// Diagnostics produced during parsing.
    #[must_use]
    pub fn diagnostics(&self) -> &[ParsedDiagnostic] {
        &self.diagnostics
    }

    /// `true` when at least one parse-failure diagnostic was emitted.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        self.block.is_none()
    }
}

/// Parse one JSDoc block into intermediate data. Use [`emit_block`] to
/// flush the result into a [`BinaryWriter`].
///
/// Path C: takes the writer's arena so intermediate data structures
/// (`description_lines`, `tags`, etc.) bump-allocate from the same arena
/// instead of the std heap. The result `ParsedBlock<'arena, 'a>` borrows
/// from both the source text (`'a`) and the arena (`'arena`); typical
/// callers pass `&parse_to_bytes`'s arena or `&parse`'s caller-supplied
/// arena.
pub fn parse_block_into_data<'arena, 'a>(
    arena: &'arena Allocator,
    source_text: &'a str,
    base_offset: u32,
    options: ParseOptions,
) -> ParsedBlock<'arena, 'a> {
    let mut ctx = ParserContext::new(arena, source_text, base_offset, options);
    if ctx.absolute_end().is_none() {
        ctx.diag(ParserDiagnosticKind::SpanOverflow);
        return ParsedBlock { block: None, diagnostics: ctx.diagnostics };
    }
    if !scanner::is_jsdoc_block(source_text) {
        ctx.diag(ParserDiagnosticKind::NotAJSDocBlock);
        return ParsedBlock { block: None, diagnostics: ctx.diagnostics };
    }
    if !scanner::has_closing_block(source_text) {
        ctx.diag(ParserDiagnosticKind::UnclosedBlockComment);
        return ParsedBlock { block: None, diagnostics: ctx.diagnostics };
    }

    let end = ctx.absolute_end().expect("checked above");
    let span = Span::new(ctx.base_offset, end);

    let scan = scanner::logical_lines(source_text, base_offset);
    let (desc_range, tag_sections) = ctx.partition_sections(&scan);

    let desc_lines_slice = &scan.lines[desc_range.start..desc_range.end];
    let desc_margins_slice = &scan.margins[desc_range.start..desc_range.end];
    let parsed_desc = ctx.parse_description_lines(desc_lines_slice, desc_margins_slice);
    let (tags, tags_parsed_types) = ctx.parse_tag_sections(&tag_sections);

    let line_count = scan.lines.len() as u32;
    let end_line = if line_count > 0 { line_count - 1 } else { 0 };

    let delimiter_line_break = if scan.lines.len() <= 1 { "" } else { "\n" };
    let preterminal_line_break = if scan.lines.len() <= 1 {
        ""
    } else if scan.margins[scan.lines.len() - 1].is_content_empty {
        "\n"
    } else {
        ""
    };
    let block_line_end = if scan.margins.is_empty() { "" } else { scan.margins[0].line_end };

    let mut description_start_line: Option<u32> = None;
    let mut description_end_line: Option<u32> = None;
    let mut last_description_line: Option<u32> = None;
    let mut has_preterminal_description: u8 = 0;
    let mut has_preterminal_tag_description: Option<u8> = None;

    let has_tags = !tag_sections.is_empty();
    for (i, m) in desc_margins_slice.iter().enumerate() {
        if !m.is_content_empty {
            let idx = i as u32;
            if description_start_line.is_none() {
                description_start_line = Some(idx);
            }
            description_end_line = Some(idx);
        }
    }
    if has_tags {
        last_description_line = Some((desc_range.end - desc_range.start) as u32);
    } else if !scan.lines.is_empty() {
        last_description_line = Some(end_line);
    }
    if !scan.lines.is_empty() && !scan.margins[scan.lines.len() - 1].is_content_empty {
        if has_tags {
            has_preterminal_tag_description = Some(1);
        } else {
            has_preterminal_description = 1;
        }
    }

    let description_raw_span =
        description_raw_span_from_block_lines(&parsed_desc.lines, base_offset);

    let block = BlockData {
        span,
        description: parsed_desc.text,
        description_raw_span,
        description_lines: parsed_desc.lines,
        inline_tags: parsed_desc.inline_tags,
        tags,
        tags_parsed_types,
        line_end: block_line_end,
        delimiter_line_break,
        preterminal_line_break,
        end_line,
        description_start_line,
        description_end_line,
        last_description_line,
        has_preterminal_description,
        has_preterminal_tag_description,
    };

    ParsedBlock { block: Some(block), diagnostics: ctx.diagnostics }
}

// ---------------------------------------------------------------------------
// Section partitioning + parsing helpers (live on ParserContext)
// ---------------------------------------------------------------------------

/// Index range into the parallel lines/margins arrays for description lines.
#[derive(Debug, Clone, Copy)]
struct DescLineRange {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
struct TagSection<'a> {
    tag_name: &'a str,
    tag_name_start: u32,
    tag_name_end: u32,
    body_lines: Vec<scanner::LogicalLine<'a>>,
    end: u32,
    header_initial: &'a str,
    header_delimiter: &'a str,
    header_post_delimiter: &'a str,
    header_line_end: &'a str,
}

#[derive(Debug)]
struct ParsedDescription<'arena, 'a> {
    text: Option<&'a str>,
    lines: ArenaVec<'arena, DescriptionLineData<'a>>,
    inline_tags: InlineTagsVec<'arena, 'a>,
}

#[derive(Debug, Clone, Copy)]
struct NormalizedText<'a> {
    text: &'a str,
    span: Span,
}

#[inline]
fn is_indented_code_block(
    content_leading_whitespace: usize,
    margin: &scanner::MarginInfo<'_>,
) -> bool {
    if margin.delimiter.is_empty() {
        return false;
    }

    // Same threshold as `oxc_jsdoc`: one conventional space after `*`
    // plus four Markdown indented-code spaces.
    let spaces_after_star = margin.post_delimiter.len() + content_leading_whitespace;
    spaces_after_star >= 5
}

#[derive(Debug)]
struct ParsedTagBody<'arena, 'a> {
    raw_body: &'a str,
    raw_type: Option<TypeSourceData<'a>>,
    name: Option<TagNameValueData<'a>>,
    optional: bool,
    default_value: Option<&'a str>,
    description: Option<&'a str>,
    type_lines: ArenaVec<'arena, TypeLineData<'a>>,
    description_lines: ArenaVec<'arena, DescriptionLineData<'a>>,
    inline_tags: InlineTagsVec<'arena, 'a>,
    body: GenericTagBodyData<'a>,
}

impl<'arena, 'a> ParserContext<'arena, 'a> {
    fn partition_sections(
        &self,
        scan: &scanner::ScanResult<'a>,
    ) -> (DescLineRange, Vec<TagSection<'a>>) {
        let lines = &scan.lines;
        let margins = &scan.margins;
        let mut desc_end = 0usize;
        let mut tag_sections = Vec::new();
        let mut current_tag: Option<TagSection<'a>> = None;
        let mut in_fence = false;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = trim_start_fast(line.content);
            let trimmed_delta = line.content.len() - trimmed.len();
            // `trimmed_delta <= line.content.len()` and source offsets fit
            // in u32 by encoder precondition; the panic check on
            // `try_from(...).unwrap()` is dead weight on this hot loop
            // (`partition_sections` runs once per logical line). Pattern #9.
            let trimmed_start = line.content_start + trimmed_delta as u32;
            let m = &margins[idx];
            let tag_header = if !in_fence
                && trimmed.starts_with('@')
                && !is_indented_code_block(trimmed_delta, m)
            {
                parse_tag_header(trimmed, trimmed_start)
            } else {
                None
            };

            if let Some((tag_name, tag_name_start, body_start)) = tag_header {
                if let Some(section) = current_tag.take() {
                    tag_sections.push(section);
                }
                let mut body_lines = Vec::new();
                if let Some((body, body_abs_start)) = body_start {
                    // `body` is `trim_start`ed by `parse_tag_header`, so it
                    // begins with a non-whitespace byte and is therefore
                    // never whitespace-only.
                    body_lines.push(scanner::LogicalLine {
                        content: body,
                        content_start: body_abs_start,
                        is_content_empty: false,
                    });
                }
                current_tag = Some(TagSection {
                    tag_name,
                    tag_name_start,
                    // `tag_name` is a sub-slice of the source text; its
                    // length fits in u32 by the same precondition that
                    // bounds source offsets. Pattern #9.
                    tag_name_end: tag_name_start + tag_name.len() as u32,
                    body_lines,
                    end: line.content_end(),
                    header_initial: m.initial,
                    header_delimiter: m.delimiter,
                    header_post_delimiter: m.post_delimiter,
                    header_line_end: m.line_end,
                });
            } else if !in_fence {
                if let Some(section) = current_tag.as_mut() {
                    section.body_lines.push(*line);
                    section.end = line.content_end();
                } else {
                    desc_end = idx + 1;
                }
            } else if let Some(section) = current_tag.as_mut() {
                section.body_lines.push(*line);
                section.end = line.content_end();
            } else {
                desc_end = idx + 1;
            }

            if self.options.fence_aware && trimmed.starts_with("```") {
                in_fence = !in_fence;
            }
        }

        if let Some(section) = current_tag {
            tag_sections.push(section);
        }
        (DescLineRange { start: 0, end: desc_end }, tag_sections)
    }

    fn parse_tag_sections(
        &mut self,
        sections: &[TagSection<'a>],
    ) -> (ArenaVec<'arena, TagData<'arena, 'a>>, Vec<(u32, Box<TypeNodeData<'a>>)>) {
        let mut tags = ArenaVec::with_capacity_in(sections.len(), self.arena);
        // Side table: stays empty when parse_types is off (the default), so
        // emit-loop's lookup branch is a single `peek().is_none()` predict.
        let mut parsed_types: Vec<(u32, Box<TypeNodeData<'a>>)> = Vec::new();
        for (i, section) in sections.iter().enumerate() {
            let (tag, opt_pt) = self.parse_jsdoc_tag(section);
            if let Some(pt) = opt_pt {
                parsed_types.push((i as u32, pt));
            }
            tags.push(tag);
        }
        (tags, parsed_types)
    }

    fn parse_jsdoc_tag(
        &mut self,
        section: &TagSection<'a>,
    ) -> (TagData<'arena, 'a>, Option<Box<TypeNodeData<'a>>>) {
        let normalized = self.normalize_lines(&section.body_lines);
        // Note 4 (compat-mode only): jsdoccomment customizes per-tag tokenization
        // — `defaultNoTypes` skips the `{type}` step and `defaultNoNames` skips
        // the name token. Mirror those rules so compat-mode AST shape matches
        // `commentParserToESTree()`. Outside compat mode keep the uniform
        // behavior (parser shape contract for non-jsdoccomment users).
        let (skip_types, skip_names) = if self.options.compat_mode {
            (jsdoccomment_no_type(section.tag_name), jsdoccomment_no_name(section.tag_name))
        } else {
            (false, false)
        };
        let parsed_body =
            normalized.map(|n| self.parse_generic_tag_body(n, skip_types, skip_names));

        let (
            raw_type,
            name,
            optional,
            default_value,
            description,
            type_lines,
            description_lines,
            inline_tags,
            body,
            raw_body,
        ) = if let Some(p) = parsed_body {
            (
                p.raw_type,
                p.name,
                p.optional,
                p.default_value,
                p.description,
                p.type_lines,
                p.description_lines,
                p.inline_tags,
                Some(TagBodyData::Generic(p.body)),
                Some(p.raw_body),
            )
        } else {
            (
                None,
                None,
                false,
                None,
                None,
                ArenaVec::new_in(self.arena),
                ArenaVec::new_in(self.arena),
                InlineTagsVec::new_in(self.arena),
                None,
                None,
            )
        };

        // parsedType: parse the {...} type expression when enabled.
        let parsed_type = if self.options.parse_types {
            raw_type.and_then(|ts| {
                let mode = self.options.type_parse_mode;
                self.parse_type_expression(ts.raw, ts.span.start + 1, mode)
            })
        } else {
            None
        };

        let has_parsed_type = parsed_type.is_some();
        let description_raw_span = description_raw_span_from_tag(
            &description_lines,
            &section.body_lines,
            self.base_offset,
        );
        let tag = TagData {
            span: Span::new(section.tag_name_start, section.end),
            tag_name_span: Span::new(section.tag_name_start, section.tag_name_end),
            tag_name: section.tag_name,
            optional,
            default_value,
            description,
            description_raw_span,
            raw_body,
            raw_type,
            name,
            has_parsed_type,
            body,
            description_lines,
            type_lines,
            inline_tags,
            header_initial: section.header_initial,
            header_delimiter: section.header_delimiter,
            header_post_delimiter: section.header_post_delimiter,
            header_line_end: section.header_line_end,
            post_tag: " ",
            post_type: " ",
            post_name: " ",
        };
        (tag, parsed_type)
    }

    fn parse_generic_tag_body(
        &mut self,
        normalized: NormalizedText<'a>,
        skip_types: bool,
        skip_names: bool,
    ) -> ParsedTagBody<'arena, 'a> {
        let mut cursor = 0usize;
        let bytes = normalized.text.as_bytes();
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        let type_source = if !skip_types && bytes.get(cursor) == Some(&b'{') {
            match find_matching_type_end(normalized.text, cursor) {
                Some(end) => {
                    let raw = &normalized.text[cursor + 1..end];
                    let span = relative_span(normalized.span, cursor as u32, (end + 1) as u32);
                    cursor = end + 1;
                    Some(TypeSourceData { span, raw })
                }
                None => {
                    self.diag(ParserDiagnosticKind::UnclosedTypeExpression);
                    None
                }
            }
        } else {
            None
        };

        let mut type_lines: ArenaVec<'arena, TypeLineData<'a>> = ArenaVec::new_in(self.arena);
        if let Some(ts) = type_source {
            type_lines.push(TypeLineData {
                span: ts.span,
                raw_type: ts.raw,
                delimiter: "",
                post_delimiter: "",
                initial: "",
            });
        }

        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        // jsdoccomment's `@see {@link ...}` rule: when the body after `{type}`
        // starts with an inline `{@link ...}`, skip name extraction so the
        // link expression stays in the description.
        let see_with_link = skip_names || see_starts_with_link(&normalized.text[cursor..]);

        let value = if see_with_link {
            None
        } else {
            let token_end = find_value_end(normalized.text, cursor);
            if token_end > cursor {
                let token = &normalized.text[cursor..token_end];
                let span = relative_span(normalized.span, cursor as u32, token_end as u32);
                cursor = token_end;
                Some(parse_tag_value_token(token, span))
            } else {
                None
            }
        };
        let (name, optional, default_value) = tag_value_name(value.as_ref());

        let mut separator = false;
        let mut remainder_start = cursor + leading_whitespace_len(&normalized.text[cursor..]);
        let mut remainder = &normalized.text[remainder_start..];
        if let Some(rest) = remainder.strip_prefix("- ") {
            separator = true;
            remainder_start += 2;
            remainder = rest;
        } else if remainder == "-" {
            separator = true;
            remainder_start = normalized.text.len();
            remainder = "";
        }

        let description_span =
            relative_span(normalized.span, remainder_start as u32, normalized.text.len() as u32);
        let parsed_desc = self.parse_description_text(remainder, description_span);

        ParsedTagBody {
            raw_body: normalized.text,
            raw_type: type_source,
            name,
            optional,
            default_value,
            description: parsed_desc.text,
            type_lines,
            description_lines: parsed_desc.lines,
            inline_tags: parsed_desc.inline_tags,
            body: GenericTagBodyData {
                span: normalized.span,
                has_dash_separator: separator,
                type_source,
                value,
                description: parsed_desc.text,
            },
        }
    }

    fn parse_description_lines(
        &mut self,
        lines: &[scanner::LogicalLine<'a>],
        margins: &[scanner::MarginInfo<'a>],
    ) -> ParsedDescription<'arena, 'a> {
        let mut description_lines: ArenaVec<'arena, DescriptionLineData<'a>> =
            ArenaVec::new_in(self.arena);
        for (line, margin) in lines.iter().zip(margins.iter()) {
            if margin.is_content_empty {
                continue;
            }
            description_lines.push(DescriptionLineData {
                span: Span::new(line.content_start, line.content_end()),
                description: trim_end_fast(line.content),
                delimiter: margin.delimiter,
                post_delimiter: margin.post_delimiter,
                initial: margin.initial,
            });
        }

        let Some(normalized) = self.normalize_lines(lines) else {
            return ParsedDescription {
                text: None,
                lines: description_lines,
                inline_tags: InlineTagsVec::new_in(self.arena),
            };
        };
        let mut description = self.parse_description_text(normalized.text, normalized.span);
        description.lines = description_lines;
        description
    }

    fn parse_description_text(
        &mut self,
        text: &'a str,
        span: Span,
    ) -> ParsedDescription<'arena, 'a> {
        let mut lines: ArenaVec<'arena, DescriptionLineData<'a>> = ArenaVec::new_in(self.arena);
        let mut inline_tags = InlineTagsVec::new_in(self.arena);

        if text.bytes().all(|b| b == b' ' || b == b'\t' || b == b'\n' || b == b'\r') {
            return ParsedDescription { text: None, lines, inline_tags };
        }

        lines.push(DescriptionLineData {
            span,
            description: text,
            delimiter: "",
            post_delimiter: "",
            initial: "",
        });

        // Fast path: most descriptions contain no `{@…}` inline tag at
        // all, but the original `find("{@")` loop still pays a Boyer-Moore
        // scan over the entire text. A single byte search for `@` is
        // SIMD-accelerated via `memchr::memchr` and lets us skip the scan
        // loop entirely when the description has no `@` to anchor on.
        // (`[u8]::contains` doesn't currently lower to memchr; using it
        // explicitly here matches `scanner::logical_lines`.)
        if memchr::memchr(b'@', text.as_bytes()).is_none() {
            return ParsedDescription { text: Some(text), lines, inline_tags };
        }

        let mut cursor = 0usize;
        while let Some(rel_start) = text[cursor..].find("{@") {
            let inline_start = cursor + rel_start;
            let Some(rel_end) = text[inline_start + 2..].find('}') else {
                self.diag(ParserDiagnosticKind::UnclosedInlineTag);
                break;
            };
            let inline_end = inline_start + 2 + rel_end;
            let inside = &text[inline_start + 2..inline_end];
            let Some((tag_name, body)) = parse_inline_tag_header(inside) else {
                self.diag(ParserDiagnosticKind::InvalidInlineTagStart);
                cursor = inline_start + 2;
                continue;
            };

            let inline_span = relative_span(span, inline_start as u32, (inline_end + 1) as u32);
            let tag_name_start = inline_start + 2;
            let tag_name_end = tag_name_start + tag_name.len();
            let (np_or_url, link_text, format) = parse_inline_tag_body(body);
            inline_tags.push(InlineTagData {
                span: inline_span,
                tag_name_span: relative_span(span, tag_name_start as u32, tag_name_end as u32),
                tag_name,
                namepath_or_url: np_or_url,
                text: link_text,
                format,
                raw_body: if body.is_empty() { None } else { Some(body) },
            });
            cursor = inline_end + 1;
        }

        ParsedDescription { text: Some(text), lines, inline_tags }
    }

    fn normalize_lines(
        &mut self,
        lines: &[scanner::LogicalLine<'a>],
    ) -> Option<NormalizedText<'a>> {
        let first_index = lines.iter().position(|line| !line.is_content_empty)?;
        let last_index = lines.iter().rposition(|line| !line.is_content_empty)?;
        let lines = &lines[first_index..=last_index];
        let first = &lines[0];
        let last = &lines[lines.len() - 1];
        let span = Span::new(first.content_start, last.content_end());

        if lines.len() == 1 {
            return Some(NormalizedText { text: trim_end_fast(lines[0].content), span });
        }

        // Build the joined text directly in the arena via `StringBuilder`
        // — single copy of each line's bytes, no heap-side staging buffer.
        // Pre-compute the exact capacity so the builder never resizes
        // (resize on bumpalo means re-alloc + copy, defeating the purpose).
        //
        // `+ 1` per line covers the inter-line `\n` separator; the count
        // overshoots by 1 (no separator after the last line) but
        // pre-allocating one extra byte is cheaper than a branch in the
        // hot loop.
        let capacity: usize = lines.iter().map(|l| l.content.len() + 1).sum();
        let mut builder = StringBuilder::with_capacity_in(capacity, self.arena);
        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                builder.push_str("\n");
            }
            builder.push_str(trim_end_fast(line.content));
        }
        // `into_str()` finalizes the builder and yields a `&'arena str`
        // that lives as long as the allocator. This replaces the prior
        // `scratch: String` + `arena.alloc_str(&scratch)` two-step
        // (which itself replaced an unsound `unsafe transmute` of a
        // reused heap String — the original UAF).
        let arena_str: &str = builder.into_str();
        // SAFETY: widening `&'arena str` to `&'a str` is sound because the
        // arena is the lifetime upper bound on every borrow in
        // `BlockData<'arena, 'a>` — anything that consumes the returned
        // text is itself bounded by the arena, so the bytes remain valid
        // for the duration of every read. Mirrors how source slices
        // (`&'a str` from `source_text`) flow through the same fields.
        let widened: &'a str = unsafe { std::mem::transmute::<&str, &'a str>(arena_str) };
        Some(NormalizedText { text: widened, span })
    }
}

// ---------------------------------------------------------------------------
// Local parsing helpers (no Self)
// ---------------------------------------------------------------------------

fn parse_tag_header(line: &str, line_start: u32) -> Option<(&str, u32, Option<(&str, u32)>)> {
    let stripped = line.strip_prefix('@')?;
    // Tag-name characters are ASCII-only by spec, so byte position search
    // beats `chars().take_while().sum::<len_utf8>()` (no UTF-8 decoding,
    // single-byte is_ascii_alphanumeric LUT).
    let name_len = stripped
        .as_bytes()
        .iter()
        .position(|&b| !(b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'!')))
        .unwrap_or(stripped.len());
    if name_len == 0 {
        return None;
    }
    let tag_name = &stripped[..name_len];
    let body = trim_start_fast(&stripped[name_len..]);
    let body_start = if body.is_empty() {
        None
    } else {
        // `body_delta <= line.len()` and source offsets fit in u32 by
        // encoder precondition. Pattern #9.
        let body_delta = line.len() - body.len();
        Some((body, line_start + body_delta as u32))
    };
    Some((tag_name, line_start + 1, body_start))
}

fn parse_inline_tag_header(inside: &str) -> Option<(&str, &str)> {
    let trimmed = trim_fast(inside);
    let name_len = trimmed
        .as_bytes()
        .iter()
        .position(|&b| !(b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'!')))
        .unwrap_or(trimmed.len());
    if name_len == 0 {
        return None;
    }
    let tag_name = &trimmed[..name_len];
    let body = trim_fast(&trimmed[name_len..]);
    Some((tag_name, body))
}

fn parse_inline_tag_body(body: &str) -> (Option<&str>, Option<&str>, InlineTagFormatData) {
    if body.is_empty() {
        return (None, None, InlineTagFormatData::Plain);
    }
    if let Some((target, text)) = body.split_once('|') {
        return (non_empty_trimmed(target), non_empty_trimmed(text), InlineTagFormatData::Pipe);
    }
    if let Some((target, text)) = body.split_once(char::is_whitespace) {
        return (non_empty_trimmed(target), non_empty_trimmed(text), InlineTagFormatData::Space);
    }
    (Some(body), None, InlineTagFormatData::Plain)
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = trim_fast(value);
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

fn leading_whitespace_len(value: &str) -> usize {
    value.len() - trim_start_fast(value).len()
}

fn find_matching_type_end(text: &str, start: usize) -> Option<usize> {
    // `{` and `}` are single-byte ASCII chars and unambiguous within UTF-8
    // (continuation bytes never collide with ASCII), so byte iteration is
    // both correct and avoids per-step UTF-8 decoding.
    //
    // Use `memchr2` to skip directly to the next brace; on
    // typescript-checker.ts most `{Type}` bodies are 5-30 bytes, which is
    // around memchr's SIMD break-even, but the per-comment frequency
    // (tens of `{...}` per file) makes the cumulative win worthwhile.
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut i = start;
    while let Some(off) = memchr::memchr2(b'{', b'}', &bytes[i..]) {
        i += off;
        if bytes[i] == b'{' {
            depth += 1;
        } else {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn find_value_end(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    if start >= bytes.len() {
        return start;
    }
    if bytes[start] == b'[' {
        // Bracket-depth scan: `[` and `]` are single-byte ASCII so byte loop
        // suffices.
        let mut depth = 0usize;
        let mut i = start;
        while i < bytes.len() {
            match bytes[i] {
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        return i + 1;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        return text.len();
    }
    // Tag value tokens terminate at any whitespace. ASCII bytes are checked
    // inline (LUT, no UTF-8 decode); the moment a non-ASCII byte appears we
    // fall back to char_indices to preserve the original Unicode-whitespace
    // semantics.
    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];
        if b < 0x80 {
            if (b as char).is_whitespace() {
                return i;
            }
            i += 1;
        } else {
            for (idx, ch) in text[i..].char_indices() {
                if ch.is_whitespace() {
                    return i + idx;
                }
            }
            return text.len();
        }
    }
    text.len()
}

fn parse_tag_value_token(token: &str, span: Span) -> TagValueData<'_> {
    if token.starts_with('[') && token.ends_with(']') {
        let inner = &token[1..token.len() - 1];
        let (path, default_value) =
            inner.split_once('=').map_or((inner, None), |(p, v)| (p, Some(v)));
        return TagValueData::Parameter { span, path, optional: true, default_value };
    }
    if token
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '.' | '[' | ']'))
    {
        return TagValueData::Parameter { span, path: token, optional: false, default_value: None };
    }
    if token.contains(['.', '#', '~', '/', ':', '"', '\'', '(']) {
        return TagValueData::Namepath { span, raw: token };
    }
    if token.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$')) {
        return TagValueData::Identifier { span, name: token };
    }
    TagValueData::Raw { span, value: token }
}

fn tag_value_name<'a>(
    value: Option<&TagValueData<'a>>,
) -> (Option<TagNameValueData<'a>>, bool, Option<&'a str>) {
    match value {
        Some(TagValueData::Parameter { span, path, optional, default_value }) => {
            (Some(TagNameValueData { span: *span, raw: path }), *optional, *default_value)
        }
        Some(TagValueData::Namepath { span, raw }) => {
            (Some(TagNameValueData { span: *span, raw }), false, None)
        }
        Some(TagValueData::Identifier { span, name }) => {
            (Some(TagNameValueData { span: *span, raw: name }), false, None)
        }
        Some(TagValueData::Raw { span, value }) => {
            (Some(TagNameValueData { span: *span, raw: value }), false, None)
        }
        None => (None, false, None),
    }
}

fn relative_span(base: Span, relative_start: u32, relative_end: u32) -> Span {
    let start = base.start.saturating_add(relative_start);
    let end = base.start.saturating_add(relative_end);
    Span::new(start.min(base.end), end.min(base.end).max(start.min(base.end)))
}

// ---------------------------------------------------------------------------
// jsdoccomment compatibility: per-tag tokenization rules (Note 4)
// ---------------------------------------------------------------------------
//
// Mirrors `defaultNoTypes` / `defaultNoNames` / `hasSeeWithLink` from
// `@es-joy/jsdoccomment/src/parseComment.js`. Only consulted when
// `ParseOptions::compat_mode` is enabled — basic-mode parses keep the
// uniform `{type} name description` extraction that downstream typed
// consumers rely on.

/// Tags whose body has no `{type}` segment in jsdoccomment. Sorted for
/// `binary_search` (the lookup runs once per block tag in compat mode).
const NO_TYPE_TAGS: &[&str] = &[
    "default",
    "defaultvalue",
    "description",
    "example",
    "file",
    "fileoverview",
    "license",
    "overview",
    "see",
    "summary",
];

/// Tags whose body has no `name` segment in jsdoccomment. Sorted.
const NO_NAME_TAGS: &[&str] = &[
    "access",
    "author",
    "default",
    "defaultvalue",
    "description",
    "example",
    "exception",
    "file",
    "fileoverview",
    "kind",
    "license",
    "overview",
    "return",
    "returns",
    "since",
    "summary",
    "throws",
    "variation",
    "version",
];

/// `defaultNoTypes` lookup.
fn jsdoccomment_no_type(tag: &str) -> bool {
    NO_TYPE_TAGS.binary_search(&tag).is_ok()
}

/// `defaultNoNames` lookup.
fn jsdoccomment_no_name(tag: &str) -> bool {
    NO_NAME_TAGS.binary_search(&tag).is_ok()
}

/// jsdoccomment's `hasSeeWithLink` rule: when a body starts with
/// `{@link <name>}`, skip name extraction so the inline-tag stays in the
/// description. The match is intentionally conservative — only fires on
/// the literal `{@link …}` prefix, leaving `@see {Foo}` / `@see Foo`
/// alone (parsed normally).
fn see_starts_with_link(after_type: &str) -> bool {
    let trimmed = after_type.trim_start();
    let Some(rest) = trimmed.strip_prefix("{@link") else {
        return false;
    };
    let Some(closing_offset) = rest.find('}') else {
        return false;
    };
    closing_offset > 0
}

// ---------------------------------------------------------------------------
// Whitespace trim helpers (ASCII fast path + Unicode fallback)
//
// JSDoc source is overwhelmingly ASCII, so the byte-based `trim_ascii_*`
// path covers the common case. We only fall back to the Unicode-aware
// `trim_*` when the post-trim boundary still touches a non-ASCII byte
// (i.e. a UTF-8 multi-byte character that might encode Unicode whitespace
// such as U+00A0 NBSP or U+3000 IDEOGRAPHIC SPACE). UTF-8 invariant
// guarantees ASCII trimming never bisects a multi-byte character (ASCII
// whitespace bytes are all < 0x80; multi-byte lead/continuation bytes are
// all ≥ 0x80).
// ---------------------------------------------------------------------------

#[inline(always)]
fn trim_start_fast(s: &str) -> &str {
    let t = s.trim_ascii_start();
    if !t.is_empty() && t.as_bytes()[0] >= 0x80 { t.trim_start() } else { t }
}

#[inline(always)]
fn trim_end_fast(s: &str) -> &str {
    let t = s.trim_ascii_end();
    if !t.is_empty() && t.as_bytes()[t.len() - 1] >= 0x80 { t.trim_end() } else { t }
}

#[inline(always)]
fn trim_fast(s: &str) -> &str {
    let t = s.trim_ascii();
    let bytes = t.as_bytes();
    if bytes.is_empty() {
        return t;
    }
    let needs_unicode = bytes[0] >= 0x80 || bytes[bytes.len() - 1] >= 0x80;
    if needs_unicode { t.trim() } else { t }
}

// ---------------------------------------------------------------------------
// Emit phase — walk parsed data and write to BinaryWriter
// ---------------------------------------------------------------------------

/// Write a parsed JSDoc block into `writer` and return the assigned root
/// node index. The caller is responsible for adding a corresponding entry
/// to the Root index array via [`BinaryWriter::push_root`].
pub fn emit_block<'arena>(
    writer: &mut BinaryWriter<'arena>,
    parsed: &ParsedBlock<'_, '_>,
) -> Option<u32> {
    let block = parsed.block.as_ref()?;
    let compat = writer.compat_mode();
    let preserve_ws = writer.preserve_whitespace_span();
    Some(emit_block_inner(writer, block, compat, preserve_ws))
}

fn opt_string(writer: &mut BinaryWriter<'_>, value: Option<&str>) -> Option<StringField> {
    value.map(|s| writer.intern_string(s))
}

/// `Option<&str>` variant of [`BinaryWriter::intern_source_slice_or_string`]
/// for fields the parser surfaces as borrowed slices (description text,
/// raw body, default value, inline-tag content). Pointer-arithmetic
/// identification picks the zero-copy path automatically; multi-line
/// `normalize_lines` joins (separate allocation) fall through to the
/// unique-string path with no per-call branching at the call site.
fn opt_source_string(writer: &mut BinaryWriter<'_>, value: Option<&str>) -> Option<StringField> {
    value.map(|s| writer.intern_source_slice_or_string(s))
}

/// Compute the `description_raw_span` for a `JsdocBlock` from its
/// description-line list. Returns source-text-relative `(start, end)`
/// UTF-8 byte offsets, or `None` when the list is empty.
///
/// The boundary is `description_lines.first().span.start ..
/// description_lines.last().span.end` per
/// `design/008-oxlint-oxfmt-support/README.md` §4.1. For `JsdocBlock` each
/// description line carries an accurate per-`LogicalLine` span, so this
/// straight reduction is correct (no multi-line span correction needed).
fn description_raw_span_from_block_lines(
    description_lines: &[DescriptionLineData<'_>],
    base_offset: u32,
) -> Option<(u32, u32)> {
    let first = description_lines.first()?;
    let last = description_lines.last()?;
    let start = first.span.start.checked_sub(base_offset)?;
    let end = last.span.end.checked_sub(base_offset)?;
    if start > end {
        return None;
    }
    Some((start, end))
}

/// Compute the `description_raw_span` for a `JsdocTag` from the synthetic
/// description line (built by `parse_description_text`) plus the tag's
/// `body_lines`. The synthetic line's `span.start` points at the
/// description's first byte in the original source, but its `span.end` is
/// short by `(line_breaks * margin_chars_lost)` bytes for multi-line
/// bodies because `normalize_lines` joins lines with a single `\n`. We
/// take END from the last non-blank `body_line.content_end` to recover the
/// correct original-source range.
fn description_raw_span_from_tag(
    description_lines: &[DescriptionLineData<'_>],
    body_lines: &[scanner::LogicalLine<'_>],
    base_offset: u32,
) -> Option<(u32, u32)> {
    let first = description_lines.first()?;
    let last_body = body_lines
        .iter()
        .rev()
        .find(|line| !line.content.bytes().all(|b| b == b' ' || b == b'\t'))?;
    let start = first.span.start.checked_sub(base_offset)?;
    let end = last_body.content_end().checked_sub(base_offset)?;
    if start > end {
        return None;
    }
    Some((start, end))
}

fn intern(writer: &mut BinaryWriter<'_>, value: &str) -> StringField {
    writer.intern_string(value)
}

/// Hot-path resolver for the source-preserving line-end strings
/// (`""`, `"\n"`, `"\r\n"`) used throughout `emit_block_inner` and
/// `emit_tag`. Skips the length-bucketed `lookup_common` + 1-slot cache
/// dance by mapping the three known cases to their pre-computed
/// `StringField` directly; falls back to the interner for any other value
/// (e.g. the rare `\r`-only or trimmed line ending).
#[inline]
fn intern_line_end(writer: &mut BinaryWriter<'_>, value: &str) -> StringField {
    use crate::writer::{COMMON_CRLF, COMMON_EMPTY, COMMON_LF, common_string_field};
    match value {
        "" => common_string_field(COMMON_EMPTY),
        "\n" => common_string_field(COMMON_LF),
        "\r\n" => common_string_field(COMMON_CRLF),
        other => writer.intern_string(other),
    }
}

fn emit_block_inner<'arena>(
    writer: &mut BinaryWriter<'arena>,
    block: &BlockData<'_, '_>,
    compat: bool,
    preserve_whitespace_span: bool,
) -> u32 {
    let description_idx = opt_source_string(writer, block.description);

    use crate::writer::{
        COMMON_EMPTY, COMMON_SLASH_STAR, COMMON_SPACE, COMMON_STAR, common_string_field,
    };

    // Branchless bitmask: each `is_empty()` returns bool, cast to u8 (0 or 1).
    // Compute first so `bitmask == 0` doubles as `is_empty_block(block)`,
    // saving 3 redundant `is_empty()` calls vs the previous post_delim check
    // calling `is_empty_block` separately.
    let bitmask: u8 = ((!block.description_lines.is_empty()) as u8)
        | (((!block.tags.is_empty()) as u8) << 1)
        | (((!block.inline_tags.is_empty()) as u8) << 2);

    // Pre-intern all source-preserving strings.
    let post_delim_str =
        if block.delimiter_line_break.is_empty() && bitmask != 0 { " " } else { "" };
    let star = common_string_field(COMMON_STAR);
    let close = common_string_field(COMMON_SLASH_STAR);
    let empty_field = common_string_field(COMMON_EMPTY);
    let post_delim =
        if post_delim_str.is_empty() { empty_field } else { common_string_field(COMMON_SPACE) };
    let line_end = intern_line_end(writer, block.line_end);
    // `initial` reuses the same prelude-cached `""` slot as `post_delim`'s
    // empty branch; one prelude lookup instead of two.
    let initial = empty_field;
    let dlb = intern_line_end(writer, block.delimiter_line_break);
    let plb = intern_line_end(writer, block.preterminal_line_break);

    // Phase 5: pass description_raw_span only when the writer-level opt-in
    // is on; otherwise the per-node `has_description_raw_span` Common Data
    // bit stays clear and the 8-byte slot is omitted.
    let block_description_raw_span =
        if preserve_whitespace_span { block.description_raw_span } else { None };

    let (block_idx, block_ext) = write_jsdoc_block(
        writer,
        block.span,
        0,
        description_idx,
        star,
        post_delim,
        close,
        line_end,
        initial,
        dlb,
        plb,
        bitmask,
        block_description_raw_span,
    );

    if compat {
        write_jsdoc_block_compat_tail(
            writer,
            block_ext,
            block.end_line,
            block.description_start_line,
            block.description_end_line,
            block.last_description_line,
            block.has_preterminal_description,
            block.has_preterminal_tag_description,
        );
    }

    let block_parent = block_idx.as_u32();

    // description_lines list — children are emitted directly under the
    // block (no NodeList wrapper); the writer patches (head, count) into
    // the parent ED slot when the list closes.
    let mut desc_list = writer.begin_node_list_at(
        block_ext,
        crate::writer::nodes::comment_ast::JSDOC_BLOCK_DESC_LINES_SLOT,
    );
    for line in &block.description_lines {
        let child_idx = emit_description_line(writer, line, block_parent, compat);
        writer.record_list_child(&mut desc_list, child_idx);
    }
    writer.finalize_node_list(desc_list);

    let mut tag_list = writer
        .begin_node_list_at(block_ext, crate::writer::nodes::comment_ast::JSDOC_BLOCK_TAGS_SLOT);
    // Path C-2: side-table walk for parsed_type. Both `tags` and
    // `tags_parsed_types` are emitted in tag-index order, so a single
    // peekable iterator avoids the per-tag linear search.
    let mut pt_iter = block.tags_parsed_types.iter().peekable();
    for (i, tag) in block.tags.iter().enumerate() {
        let parsed_type: Option<&TypeNodeData<'_>> = match pt_iter.peek() {
            Some((idx, _)) if (*idx as usize) == i => {
                let (_, pt) = pt_iter.next().unwrap();
                Some(pt.as_ref())
            }
            _ => None,
        };
        let child_idx =
            emit_tag(writer, tag, block_parent, compat, preserve_whitespace_span, parsed_type);
        writer.record_list_child(&mut tag_list, child_idx);
    }
    writer.finalize_node_list(tag_list);

    let mut inline_list = writer.begin_node_list_at(
        block_ext,
        crate::writer::nodes::comment_ast::JSDOC_BLOCK_INLINE_TAGS_SLOT,
    );
    for inline in &block.inline_tags {
        let child_idx = emit_inline_tag(writer, inline, block_parent);
        writer.record_list_child(&mut inline_list, child_idx);
    }
    writer.finalize_node_list(inline_list);

    block_parent
}

fn emit_description_line(
    writer: &mut BinaryWriter<'_>,
    line: &DescriptionLineData<'_>,
    parent_index: u32,
    compat: bool,
) -> u32 {
    // Path A: `line.description` is `line.content.trim_ascii_end()`, i.e. a
    // sub-slice of the source text that was just appended to `data_buffer`
    // via `append_source_text`. Use the zero-copy intern paths so we
    // register the offsets-only entry without re-copying the bytes — the
    // dominant emit-phase cost we identified in
    // `.notes/binary-ast-emit-phase-format-analysis.md`.
    //
    // The byte range is `[span.start, span.start + description.len())`;
    // `span.end` would over-shoot because it includes the trailing
    // whitespace that `trim_end()` removed.
    let desc_byte_end = line.span.start + line.description.len() as u32;
    if compat {
        let desc_field = writer.intern_source_slice(line.span.start, desc_byte_end);
        let delim = opt_string(writer, non_empty_str(line.delimiter));
        let pdelim = opt_string(writer, non_empty_str(line.post_delimiter));
        let init = opt_string(writer, non_empty_str(line.initial));
        write_jsdoc_description_line_compat(
            writer,
            line.span,
            parent_index,
            desc_field,
            delim,
            pdelim,
            init,
        )
        .as_u32()
    } else {
        let desc_idx = writer.intern_source_slice_for_leaf_payload(line.span.start, desc_byte_end);
        write_jsdoc_description_line(writer, line.span, parent_index, desc_idx).as_u32()
    }
}

fn non_empty_str(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}

fn emit_tag(
    writer: &mut BinaryWriter<'_>,
    tag: &TagData<'_, '_>,
    parent_index: u32,
    compat: bool,
    preserve_whitespace_span: bool,
    parsed_type: Option<&TypeNodeData<'_>>,
) -> u32 {
    let default_idx = opt_source_string(writer, tag.default_value);
    let desc_idx = opt_source_string(writer, tag.description);
    let raw_body_idx = opt_source_string(writer, tag.raw_body);

    // bit0 = tag (mandatory, always 1). Remaining bits via branchless OR.
    let bitmask: u8 = 0b0000_0001
        | ((tag.raw_type.is_some() as u8) << 1)
        | ((tag.name.is_some() as u8) << 2)
        | ((tag.has_parsed_type as u8) << 3)
        | ((tag.body.is_some() as u8) << 4)
        | (((!tag.type_lines.is_empty()) as u8) << 5)
        | (((!tag.description_lines.is_empty()) as u8) << 6)
        | (((!tag.inline_tags.is_empty()) as u8) << 7);

    // Phase 5: pass description_raw_span only when the writer-level opt-in
    // is on (mirrors the JsdocBlock path).
    let tag_description_raw_span =
        if preserve_whitespace_span { tag.description_raw_span } else { None };

    let (tag_idx, tag_ext) = write_jsdoc_tag(
        writer,
        tag.span,
        parent_index,
        tag.optional,
        default_idx,
        desc_idx,
        raw_body_idx,
        bitmask,
        tag_description_raw_span,
    );

    if compat {
        // Per-tag compat tail (delimiter strings). Defaults match
        // ox_jsdoc's typed parser convention.
        let post_tag_str = intern(writer, tag.post_tag);
        let post_type_str = intern(writer, tag.post_type);
        let post_name_str = intern(writer, tag.post_name);
        let initial = intern(writer, tag.header_initial);
        let line_end = intern(writer, tag.header_line_end);
        let delim = intern(writer, tag.header_delimiter);
        let pdelim = intern(writer, tag.header_post_delimiter);
        write_jsdoc_tag_compat_tail(
            writer,
            tag_ext,
            delim,
            pdelim,
            post_tag_str,
            post_type_str,
            post_name_str,
            initial,
            line_end,
        );
    }

    let tag_parent = tag_idx.as_u32();

    // Mandatory tag-name child (visitor index 0). Common tag names hit
    // COMMON_STRINGS via the helper; uncommon ones zero-copy off the source.
    let tn_idx = writer.intern_source_or_string_for_leaf_payload(tag.tag_name, tag.tag_name_span);
    let _ = write_jsdoc_tag_name(writer, tag.tag_name_span, tag_parent, tn_idx);

    if let Some(rt) = tag.raw_type.as_ref() {
        let raw_idx = writer.intern_source_or_string_for_leaf_payload(rt.raw, rt.span);
        let _ = write_jsdoc_type_source(writer, rt.span, tag_parent, raw_idx);
    }
    if let Some(name) = tag.name.as_ref() {
        let raw_idx = writer.intern_source_or_string_for_leaf_payload(name.raw, name.span);
        let _ = write_jsdoc_tag_name_value(writer, name.span, tag_parent, raw_idx);
    }
    if let Some(pt) = parsed_type {
        super::type_emit::emit_type_node(writer, pt, tag_parent);
    }

    if let Some(body) = tag.body.as_ref() {
        emit_tag_body(writer, body, tag_parent);
    }

    let mut type_list = writer
        .begin_node_list_at(tag_ext, crate::writer::nodes::comment_ast::JSDOC_TAG_TYPE_LINES_SLOT);
    for tl in &tag.type_lines {
        let child_idx = if compat {
            let raw_field = writer.intern_source_or_string(tl.raw_type, tl.span);
            let delim = opt_string(writer, non_empty_str(tl.delimiter));
            let pdelim = opt_string(writer, non_empty_str(tl.post_delimiter));
            let init = opt_string(writer, non_empty_str(tl.initial));
            write_jsdoc_type_line_compat(
                writer, tl.span, tag_parent, raw_field, delim, pdelim, init,
            )
            .as_u32()
        } else {
            let raw_idx = writer.intern_source_or_string_for_leaf_payload(tl.raw_type, tl.span);
            write_jsdoc_type_line(writer, tl.span, tag_parent, raw_idx).as_u32()
        };
        writer.record_list_child(&mut type_list, child_idx);
    }
    writer.finalize_node_list(type_list);

    let mut desc_list = writer
        .begin_node_list_at(tag_ext, crate::writer::nodes::comment_ast::JSDOC_TAG_DESC_LINES_SLOT);
    for line in &tag.description_lines {
        let child_idx = emit_description_line(writer, line, tag_parent, compat);
        writer.record_list_child(&mut desc_list, child_idx);
    }
    writer.finalize_node_list(desc_list);

    let mut inline_list = writer
        .begin_node_list_at(tag_ext, crate::writer::nodes::comment_ast::JSDOC_TAG_INLINE_TAGS_SLOT);
    for inline in &tag.inline_tags {
        let child_idx = emit_inline_tag(writer, inline, tag_parent);
        writer.record_list_child(&mut inline_list, child_idx);
    }
    writer.finalize_node_list(inline_list);

    tag_idx.as_u32()
}

fn emit_tag_body(writer: &mut BinaryWriter<'_>, body: &TagBodyData<'_>, parent_index: u32) {
    match body {
        TagBodyData::Generic(g) => {
            let desc_idx = opt_source_string(writer, g.description);
            // Children bitmask: bit0 = type_source, bit1 = value (branchless).
            let bm: u8 = (g.type_source.is_some() as u8) | ((g.value.is_some() as u8) << 1);
            let body_idx = write_jsdoc_generic_tag_body(
                writer,
                g.span,
                parent_index,
                g.has_dash_separator,
                desc_idx,
                bm,
            );
            let body_parent = body_idx.as_u32();

            if let Some(ts) = g.type_source.as_ref() {
                let raw_idx = writer.intern_source_or_string_for_leaf_payload(ts.raw, ts.span);
                let _ = write_jsdoc_type_source(writer, ts.span, body_parent, raw_idx);
            }
            if let Some(v) = g.value.as_ref() {
                emit_tag_value(writer, v, body_parent);
            }
        }
    }
}

fn emit_tag_value(writer: &mut BinaryWriter<'_>, value: &TagValueData<'_>, parent_index: u32) {
    match value {
        TagValueData::Parameter { span, path, optional, default_value } => {
            let path_idx = writer.intern_source_or_string(path, *span);
            let dv_idx = opt_source_string(writer, *default_value);
            let _ = write_jsdoc_parameter_name(
                writer,
                *span,
                parent_index,
                *optional,
                path_idx,
                dv_idx,
            );
        }
        TagValueData::Namepath { span, raw } => {
            let raw_idx = writer.intern_source_or_string_for_leaf_payload(raw, *span);
            let _ = write_jsdoc_namepath_source(writer, *span, parent_index, raw_idx);
        }
        TagValueData::Identifier { span, name } => {
            let name_idx = writer.intern_source_or_string_for_leaf_payload(name, *span);
            let _ = write_jsdoc_identifier(writer, *span, parent_index, name_idx);
        }
        TagValueData::Raw { span, value } => {
            let val_idx = writer.intern_source_or_string_for_leaf_payload(value, *span);
            let _ = write_jsdoc_text(writer, *span, parent_index, val_idx);
        }
    }
}

fn emit_inline_tag(
    writer: &mut BinaryWriter<'_>,
    inline: &InlineTagData<'_>,
    parent_index: u32,
) -> u32 {
    let np_idx = opt_source_string(writer, inline.namepath_or_url);
    let text_idx = opt_source_string(writer, inline.text);
    let raw_idx = opt_source_string(writer, inline.raw_body);
    let format = inline.format as u8;
    let idx = write_jsdoc_inline_tag(
        writer,
        inline.span,
        parent_index,
        format,
        np_idx,
        text_idx,
        raw_idx,
    );
    // Note: the inline tag's `tag` child (JsdocTagName) is referenced by the
    // jsdoccomment AST shape but the binary writer's JsdocInlineTag does not
    // currently include a child slot for it. This matches the Rust lazy
    // decoder's surface.
    let _ = inline.tag_name_span;
    let _ = inline.tag_name;
    idx.as_u32()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> ParseOptions {
        ParseOptions::default()
    }

    #[test]
    fn parse_block_returns_none_for_non_jsdoc() {
        let arena = oxc_allocator::Allocator::default();
        let parsed = parse_block_into_data(&arena, "/* plain */", 0, opts());
        assert!(parsed.is_failure());
        assert!(matches!(
            parsed.diagnostics()[0].kind,
            DiagnosticKind::Parser(ParserDiagnosticKind::NotAJSDocBlock)
        ));
    }

    #[test]
    fn parse_block_returns_none_for_unclosed() {
        let arena = oxc_allocator::Allocator::default();
        let parsed = parse_block_into_data(&arena, "/** unclosed", 0, opts());
        assert!(parsed.is_failure());
        assert!(matches!(
            parsed.diagnostics()[0].kind,
            DiagnosticKind::Parser(ParserDiagnosticKind::UnclosedBlockComment)
        ));
    }

    #[test]
    fn parses_top_level_description() {
        let arena = oxc_allocator::Allocator::default();
        let parsed = parse_block_into_data(&arena, "/** ok */", 10, opts());
        assert!(!parsed.is_failure());
        let block = parsed.block.as_ref().unwrap();
        assert_eq!(block.description, Some("ok"));
        assert_eq!(block.description_lines.len(), 1);
    }

    #[test]
    fn parses_param_tag_with_type_value_and_description() {
        let arena = oxc_allocator::Allocator::default();
        let parsed = parse_block_into_data(
            &arena,
            "/**\n * @param {string} id - The user ID\n */",
            0,
            opts(),
        );
        assert!(!parsed.is_failure());
        let block = parsed.block.as_ref().unwrap();
        assert_eq!(block.tags.len(), 1);
        let tag = &block.tags[0];
        assert_eq!(tag.tag_name, "param");
        assert_eq!(tag.description, Some("The user ID"));
        assert!(tag.raw_type.is_some());
        assert_eq!(tag.raw_type.as_ref().unwrap().raw, "string");
        assert!(tag.name.is_some());
        assert_eq!(tag.name.as_ref().unwrap().raw, "id");
    }

    #[test]
    fn parses_tags_like_oxc_jsdoc_tag_splitter() {
        let cases = [
            (
                "backtick_inside_quotes",
                "/**\n * @param {\"'\" | '\"' | '`'} string_start_char desc\n * @returns {number} The index\n */",
                vec!["param", "returns"],
            ),
            (
                "extra_closing_brace",
                "/**\n * @param {AST.SvelteElement | AST.RegularElement} node}\n * @param {{ stop: () => void }} context\n */",
                vec!["param", "param"],
            ),
            (
                "inline_link",
                "/**\n * @param {string} name See {@link Foo} for details\n * @returns {void}\n */",
                vec!["param", "returns"],
            ),
            (
                "at_sign_mid_line",
                "/**\n * @param {string} email user@example.com address\n * @returns {void}\n */",
                vec!["param", "returns"],
            ),
            (
                "braces_inside_quotes",
                "/**\n * \"props\" of the form \"{ [key: string]: { type?: \"String\" | \"Object\" }\"\n * @param {null} node\n * @returns {never}\n */",
                vec!["param", "returns"],
            ),
            (
                "indented_code_block_at_sign",
                "/**\n * @deprecated\n *     @myDecorator\n *     class Foo {}\n * @type {string}\n */",
                vec!["deprecated", "type"],
            ),
            (
                "normal_indent_at_sign",
                "/**\n * @deprecated\n *    @type {string}\n */",
                vec!["deprecated", "type"],
            ),
        ];

        for (name, source, expected) in cases {
            let arena = oxc_allocator::Allocator::default();
            let parsed = parse_block_into_data(&arena, source, 0, opts());
            assert!(!parsed.is_failure(), "{name}");
            let block = parsed.block.as_ref().unwrap();
            let actual = block.tags.iter().map(|section| section.tag_name).collect::<Vec<_>>();

            assert_eq!(actual, expected, "{name}");
        }
    }
}
