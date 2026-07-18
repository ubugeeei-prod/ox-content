//! Link reference definitions (CommonMark).
//!
//! Definitions (`[label]: destination "title"`) are usable anywhere in the
//! document, including before their definition site, so the root parser
//! runs a collection pre-pass over the source before block parsing starts.
//! The pre-pass tracks just enough block structure to avoid false
//! positives — fenced code regions, indented code lines, and block quote
//! markers — while actual removal of definition paragraphs from the output
//! happens later during regular block parsing via
//! [`Parser::try_parse_definition_node`].

use std::rc::Rc;

use compact_str::CompactString;
use ox_content_ast::{Definition, Node, Span};
use rustc_hash::FxHashMap;

use super::Parser;

mod scan;

use scan::{line_end_if_blank_after, next_blank_line, skip_ws_one_newline, strip_quote_markers};
// The block quote collector shares the fence and paragraph-context
// helpers for its lazy-continuation tracking.
pub(super) use scan::{closes_paragraph_context, fence_open, is_fence_close};

#[derive(Debug)]
pub(super) struct ReferenceDef<'a> {
    pub url: &'a str,
    pub title: Option<&'a str>,
}

pub(super) type ReferenceMap<'a> = FxHashMap<CompactString, ReferenceDef<'a>>;

/// One parsed definition and the bytes it consumed.
pub(super) struct ParsedDefinition<'a> {
    pub label: &'a str,
    pub url: &'a str,
    pub title: Option<&'a str>,
    pub consumed: usize,
}

impl<'a> Parser<'a> {
    /// Normalizes a reference label: trim, collapse internal whitespace to
    /// single spaces, lowercase (approximating Unicode case fold).
    pub(super) fn normalize_reference_label(label: &str) -> CompactString {
        let mut out = CompactString::default();
        let mut pending_space = false;
        for ch in label.chars() {
            if ch.is_whitespace() {
                pending_space = !out.is_empty();
            } else {
                if pending_space {
                    out.push(' ');
                    pending_space = false;
                }
                for lowered in ch.to_lowercase() {
                    // Case folding (not just lowercasing): the sharp s
                    // folds to "ss", so [SS] and [ẞ] label-match.
                    if lowered == 'ß' {
                        out.push_str("ss");
                    } else {
                        out.push(lowered);
                    }
                }
            }
        }
        out
    }

    pub(super) fn lookup_reference(&self, raw_label: &str) -> Option<&ReferenceDef<'a>> {
        if self.definitions.is_empty() {
            return None;
        }
        self.definitions.get(&Self::normalize_reference_label(raw_label))
    }

    /// Parses a single definition at the start of `text`. `text` must not
    /// span a blank line (callers cut at paragraph boundaries).
    pub(super) fn parse_reference_definition(&self, text: &'a str) -> Option<ParsedDefinition<'a>> {
        let bytes = text.as_bytes();
        let mut i = 0;
        while i < bytes.len() && bytes[i] == b' ' {
            i += 1;
        }
        if i > 3 || bytes.get(i) != Some(&b'[') {
            return None;
        }

        let label_start = i + 1;
        let mut j = label_start;
        loop {
            if j - label_start > 1000 {
                return None;
            }
            match bytes.get(j)? {
                b'\\' if bytes.get(j + 1).is_some_and(u8::is_ascii_punctuation) => j += 2,
                b']' => break,
                b'[' => return None,
                _ => j += 1,
            }
        }
        let label = &text[label_start..j];
        if label.trim().is_empty() {
            return None;
        }
        // With footnotes enabled, `[^label]:` belongs to the footnote
        // parser; treating it as a link reference here would turn every
        // `[^label]` in the document into a link.
        if self.options.footnotes && label.starts_with('^') {
            return None;
        }
        if bytes.get(j + 1) != Some(&b':') {
            return None;
        }

        let dest_start = skip_ws_one_newline(bytes, j + 2)?;
        let (raw_url, after_dest) = super::inline::parse_link_destination(text, dest_start)?;
        if raw_url.is_empty() && bytes.get(dest_start) != Some(&b'<') {
            return None;
        }

        // End of the destination line, in case the title turns out absent
        // or invalid: only spaces/tabs may follow on that line.
        let dest_line_end = line_end_if_blank_after(bytes, after_dest);

        let mut k = after_dest;
        let mut ws_between = false;
        while matches!(bytes.get(k), Some(b' ' | b'\t')) {
            k += 1;
            ws_between = true;
        }
        let title_on_next_line = bytes.get(k) == Some(&b'\n');
        if title_on_next_line {
            k += 1;
            ws_between = true;
            while matches!(bytes.get(k), Some(b' ' | b'\t')) {
                k += 1;
            }
        }

        if ws_between {
            if let Some((raw_title, after_title)) = super::inline::parse_link_title(text, k) {
                if let Some(end) = line_end_if_blank_after(bytes, after_title) {
                    return Some(ParsedDefinition {
                        label,
                        url: self.unescape_reference_component(raw_url),
                        title: Some(self.unescape_reference_component(raw_title)),
                        consumed: end,
                    });
                }
            }
        }

        // No (valid) title: the definition is still good if its
        // destination line ends cleanly.
        let end = dest_line_end?;
        Some(ParsedDefinition {
            label,
            url: self.unescape_reference_component(raw_url),
            title: None,
            consumed: end,
        })
    }

    /// Consumes one definition at the current block position, emitting the
    /// AST node. Returns `None` when the position does not start a
    /// definition (the caller falls through to paragraph parsing).
    pub(super) fn try_parse_definition_node(&mut self) -> Option<Node<'a>> {
        let start = self.position;
        // Definitions cannot contain blank lines; cut the candidate region
        // at the next one so the destination/title scanners stay in
        // paragraph bounds.
        let region_end = next_blank_line(self.source.as_bytes(), start);
        let parsed = self.parse_reference_definition(&self.source[start..region_end])?;

        let identifier =
            self.allocator.alloc_str(Self::normalize_reference_label(parsed.label).as_str());
        let end = start + parsed.consumed;
        self.position = end;
        Some(Node::Definition(Definition {
            identifier,
            label: Some(parsed.label),
            url: parsed.url,
            title: parsed.title,
            span: Span::new(start as u32, end as u32),
        }))
    }

    /// Collection pre-pass over the whole source. Tracks fenced code
    /// regions and strips block quote markers so definitions inside block
    /// quotes are found; definition-shaped text inside fenced or indented
    /// code is skipped.
    pub(super) fn collect_reference_definitions(&self) -> ReferenceMap<'a> {
        let mut map = ReferenceMap::default();
        let bytes = self.source.as_bytes();
        let mut pos = 0;
        let mut fence: Option<(u8, usize)> = None;
        // A definition can only start where a paragraph could start; a
        // bracket line directly under a paragraph (or lazy list/HTML
        // continuation) is continuation text, not a definition.
        let mut paragraph_open = false;

        while pos < bytes.len() {
            let line_end = memchr::memchr(b'\n', &bytes[pos..]).map_or(bytes.len(), |o| pos + o);
            let line = &self.source[pos..line_end];
            let stripped = strip_quote_markers(line);
            let trimmed = stripped.trim_start_matches([' ', '\t']);
            let indent = stripped.len() - trimmed.len();

            if let Some((fence_byte, fence_len)) = fence {
                if is_fence_close(trimmed, fence_byte, fence_len) {
                    fence = None;
                }
                pos = line_end + 1;
                continue;
            }
            if let Some(open) = fence_open(trimmed) {
                fence = Some(open);
                paragraph_open = false;
                pos = line_end + 1;
                continue;
            }
            if trimmed.is_empty() {
                paragraph_open = false;
                pos = line_end + 1;
                continue;
            }

            if !paragraph_open && indent <= 3 && trimmed.starts_with('[') {
                // Candidate: join the stripped lines of this paragraph
                // chunk and parse as many definitions as it holds.
                let (chunk, line_starts) = self.join_stripped_chunk(pos);
                let mut offset = 0;
                while let Some(parsed) = self.parse_reference_definition(&chunk[offset..]) {
                    map.entry(Self::normalize_reference_label(parsed.label))
                        .or_insert(ReferenceDef { url: parsed.url, title: parsed.title });
                    offset += parsed.consumed;
                }
                // Skip the source lines the parsed prefix covered so fence
                // tracking stays aligned (definition text can't open
                // fences). A leftover suffix starts a paragraph.
                let consumed_lines = chunk[..offset].matches('\n').count();
                if consumed_lines > 0 {
                    pos = line_starts.get(consumed_lines).copied().unwrap_or(bytes.len());
                    continue;
                }
            }

            paragraph_open = !closes_paragraph_context(trimmed);
            pos = line_end + 1;
        }

        map
    }

    /// Joins the block-quote-stripped lines of the paragraph chunk that
    /// starts at `pos` (stopping at a blank line), returning the joined
    /// text and each line's start offset in the original source.
    fn join_stripped_chunk(
        &self,
        mut pos: usize,
    ) -> (&'a str, ox_content_allocator::Vec<'a, usize>) {
        let bytes = self.source.as_bytes();
        let mut joined = self.allocator.new_string();
        let mut line_starts = self.allocator.new_vec();
        while pos < bytes.len() {
            let line_end = memchr::memchr(b'\n', &bytes[pos..]).map_or(bytes.len(), |o| pos + o);
            let line = &self.source[pos..line_end];
            let stripped = strip_quote_markers(line);
            if stripped.trim().is_empty() {
                break;
            }
            line_starts.push(pos);
            joined.push_str(stripped);
            joined.push('\n');
            pos = line_end + 1;
        }
        line_starts.push(pos.min(bytes.len()));
        (joined.into_bump_str(), line_starts)
    }

    fn unescape_reference_component(&self, raw: &'a str) -> &'a str {
        self.unescape_link_component(raw)
    }
}

impl<'a> Parser<'a> {
    /// Builds the shared reference map for a root parser.
    pub(super) fn build_definitions(&self) -> Rc<ReferenceMap<'a>> {
        // Cheap bail: no `[` anywhere means no definitions.
        if !self.source.contains('[') {
            return Rc::new(ReferenceMap::default());
        }
        Rc::new(self.collect_reference_definitions())
    }
}
