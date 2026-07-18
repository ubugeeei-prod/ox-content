//! GFM footnotes.
//!
//! A definition is a block starting with `[^label]:`; its content is the
//! rest of that line plus any following lines indented at least four
//! columns (or blank lines between them). A reference is the inline
//! `[^label]`, which only becomes a [`FootnoteReference`] when a matching
//! definition exists somewhere in the document — otherwise it stays
//! literal text, as GFM specifies.
//!
//! Because references may appear before their definitions, the root
//! parser collects the label set in a pre-pass ([`Parser::build_footnote_labels`])
//! and shares it with sub-parsers, mirroring how link reference
//! definitions work.

use std::rc::Rc;

use compact_str::CompactString;
use ox_content_ast::{FootnoteDefinition, Node, Span};
use rustc_hash::FxHashSet;

use super::Parser;
use crate::error::ParseResult;

pub(super) type FootnoteLabels = FxHashSet<CompactString>;

/// Reads a `[^label]:` opener at the start of `text`.
///
/// Returns the raw label (without the `^`) and the byte offset just past
/// the colon. Labels may not contain brackets, whitespace, or span lines.
pub(super) fn parse_footnote_opener(text: &str) -> Option<(&str, usize)> {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    // More than three leading spaces makes it indented code, not a block start.
    if i > 3 || bytes.get(i) != Some(&b'[') || bytes.get(i + 1) != Some(&b'^') {
        return None;
    }

    let label_start = i + 2;
    let mut j = label_start;
    loop {
        match bytes.get(j)? {
            b']' => break,
            b'[' | b'\n' => return None,
            byte if byte.is_ascii_whitespace() => return None,
            _ => j += 1,
        }
    }
    if j == label_start || bytes.get(j + 1) != Some(&b':') {
        return None;
    }

    Some((&text[label_start..j], j + 2))
}

/// Normalizes a footnote label for matching. GFM matches footnote labels
/// case-insensitively, like link labels.
pub(super) fn normalize_footnote_label(label: &str) -> CompactString {
    CompactString::from(label.to_lowercase())
}

/// Byte length of the definition body starting at `content_start`:
/// the remainder of the opening line plus indented continuation lines.
fn definition_body_len(source: &str, content_start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut cursor = memchr::memchr(b'\n', &bytes[content_start..])
        .map_or(source.len(), |offset| content_start + offset + 1);
    // Trailing blank lines only belong to the definition when an indented
    // line follows, so remember where the last real content ended.
    let mut end = cursor;

    while cursor < source.len() {
        let line_end =
            memchr::memchr(b'\n', &bytes[cursor..]).map_or(source.len(), |o| cursor + o + 1);
        let line = &source[cursor..line_end];
        let trimmed = line.trim_start_matches([' ', '\t']);

        if trimmed.trim_end().is_empty() {
            cursor = line_end;
            continue;
        }
        // Four columns of indent continue the definition; anything less
        // starts a new block.
        if indent_columns(line) < 4 {
            break;
        }
        cursor = line_end;
        end = line_end;
    }

    end - content_start
}

fn indent_columns(line: &str) -> usize {
    let mut columns = 0;
    for byte in line.bytes() {
        match byte {
            b' ' => columns += 1,
            b'\t' => columns += 4 - (columns % 4),
            _ => break,
        }
    }
    columns
}

/// Strips up to four columns of indentation from every line but the first,
/// which the caller has already positioned past the `[^label]:` opener.
fn dedent_body<'a>(allocator: &'a ox_content_allocator::Allocator, body: &str) -> &'a str {
    // Arena-owned like the block quote / list item sub-sources, so the
    // dedented copy lives as long as the AST that borrows from it.
    let mut out = ox_content_allocator::String::with_capacity_in(body.len(), allocator.bump());
    for (index, line) in body.split_inclusive('\n').enumerate() {
        if index == 0 {
            out.push_str(line.trim_start_matches([' ', '\t']));
            continue;
        }
        let mut columns = 0;
        let mut consumed = 0;
        for byte in line.bytes() {
            if columns >= 4 {
                break;
            }
            match byte {
                b' ' => columns += 1,
                b'\t' => columns += 4 - (columns % 4),
                _ => break,
            }
            consumed += 1;
        }
        out.push_str(&line[consumed..]);
    }
    out.into_bump_str()
}

impl<'a> Parser<'a> {
    /// Whether `[^` at `position` opens a footnote definition here.
    pub(super) fn at_footnote_definition(&self, start: usize) -> bool {
        self.options.footnotes && parse_footnote_opener(&self.source[start..]).is_some()
    }

    /// Consumes a footnote definition block, emitting its node.
    pub(super) fn try_parse_footnote_definition_node(&mut self) -> ParseResult<Option<Node<'a>>> {
        let start = self.position;
        let Some((label, after_colon)) = parse_footnote_opener(&self.source[start..]) else {
            return Ok(None);
        };

        let identifier =
            self.allocator.alloc_str(normalize_footnote_label(label).as_str()) as &'a str;
        let content_start = start + after_colon;
        let body_len = definition_body_len(self.source, content_start);
        let body =
            dedent_body(self.allocator, &self.source[content_start..content_start + body_len]);

        // The body is a full block context (paragraphs, lists, code), so
        // hand it to a sub-parser rather than treating it as inline text.
        let sub_doc =
            self.sub_parser_with_lazy_lines(body, rustc_hash::FxHashSet::default()).parse()?;
        let mut children = sub_doc.children;
        // Sub-parser spans are relative to `body`; shift them back onto
        // the original source so downstream tooling keeps working ranges.
        for child in &mut children {
            Self::offset_node_spans(child, content_start as u32);
        }
        let end = content_start + body_len;
        self.position = end;

        Ok(Some(Node::FootnoteDefinition(FootnoteDefinition {
            identifier,
            label: Some(label),
            children,
            span: Span::new(start as u32, end as u32),
        })))
    }

    /// Whether a definition exists for `label` (already normalized-able).
    pub(super) fn has_footnote_label(&self, label: &str) -> bool {
        !self.footnote_labels.is_empty()
            && self.footnote_labels.contains(&normalize_footnote_label(label))
    }

    /// Emits a [`FootnoteReference`] for `[^label]` at `pos`. Returns
    /// false (leaving `pos` untouched) when this is not a reference to a
    /// defined footnote, so the caller can fall back to link parsing.
    pub(super) fn try_parse_footnote_reference(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut ox_content_allocator::Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> bool {
        let bytes = content.as_bytes();
        let label_start = *pos + 2;
        let mut end = label_start;
        loop {
            match bytes.get(end) {
                Some(b']') => break,
                // Labels are a single run without brackets or whitespace.
                Some(byte) if !byte.is_ascii_whitespace() && *byte != b'[' => end += 1,
                _ => return false,
            }
        }
        if end == label_start {
            return false;
        }

        let label = &content[label_start..end];
        if !self.has_footnote_label(label) {
            return false;
        }

        let identifier =
            self.allocator.alloc_str(normalize_footnote_label(label).as_str()) as &'a str;
        children.push(Node::FootnoteReference(ox_content_ast::FootnoteReference {
            identifier,
            label: Some(label),
            span: Span::new((offset + *pos) as u32, (offset + end + 1) as u32),
        }));
        *pos = end + 1;
        true
    }

    /// Collects every footnote label in the source so inline references
    /// resolve regardless of definition order.
    pub(super) fn build_footnote_labels(&self) -> Rc<FootnoteLabels> {
        if !self.options.footnotes || !self.source.contains("[^") {
            return Rc::new(FootnoteLabels::default());
        }

        let mut labels = FootnoteLabels::default();
        let bytes = self.source.as_bytes();
        let mut pos = 0;
        let mut fence: Option<(u8, usize)> = None;

        while pos < bytes.len() {
            let line_end = memchr::memchr(b'\n', &bytes[pos..]).map_or(bytes.len(), |o| pos + o);
            let line = &self.source[pos..line_end];
            let trimmed = line.trim_start_matches([' ', '\t']);

            // Definition-shaped text inside fenced code is not a definition.
            if let Some((fence_byte, fence_len)) = fence {
                if super::reference::is_fence_close(trimmed, fence_byte, fence_len) {
                    fence = None;
                }
            } else if let Some(open) = super::reference::fence_open(trimmed) {
                fence = Some(open);
            } else if let Some((label, _)) = parse_footnote_opener(line) {
                labels.insert(normalize_footnote_label(label));
            }

            pos = line_end + 1;
        }

        Rc::new(labels)
    }
}
