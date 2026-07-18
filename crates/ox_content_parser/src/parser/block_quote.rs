use memchr::memchr;
use ox_content_ast::{BlockQuote, Node, Span};

use super::reference::{closes_paragraph_context, fence_open, is_fence_close};
use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

impl<'a> Parser<'a> {
    /// Parses a block quote by stripping quote markers into arena storage.
    ///
    /// The nested parser needs a contiguous source string without the leading
    /// `>` markers. Building that string directly in the bump arena avoids the
    /// old two-step path of filling a system `String` and then copying it into
    /// arena storage before recursive parsing.
    pub(super) fn parse_block_quote(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_block_quote");
        self.nesting_depth += 1;

        // Collect lines belonging to this block quote and strip the `>` prefix.
        // Write straight into a bump-allocated `String` so we don't pay for
        // `String::new` (system allocator) followed by `alloc_str` (copy to
        // arena) on every block quote. Capacity is intentionally small —
        // bumpalo will grow it if needed, and oversizing wastes arena
        // bytes that can't be reclaimed until reset.
        let bytes = self.source.as_bytes();
        let mut inner = ox_content_allocator::String::with_capacity_in(128, self.allocator.bump());
        // Lazy continuation applies only while the quote's last block is
        // an open paragraph: track blank lines and fenced code regions of
        // the stripped content to know when that is the case.
        let mut fence: Option<(u8, usize)> = None;
        let mut paragraph_open = false;
        let mut lazy_lines = rustc_hash::FxHashSet::default();

        loop {
            if self.position >= bytes.len() {
                break;
            }

            let line_start = self.position;
            let mut ws_cursor = line_start;
            while ws_cursor < bytes.len() && matches!(bytes[ws_cursor], b' ' | b'\t') {
                ws_cursor += 1;
            }

            // Blank line ends the block quote
            if ws_cursor >= bytes.len() || bytes[ws_cursor] == b'\n' {
                break;
            }

            let line_end =
                memchr(b'\n', &bytes[line_start..]).map_or(bytes.len(), |off| line_start + off);
            let line = &self.source[line_start..line_end];
            let trimmed_offset = ws_cursor - line_start;
            let trimmed = &line[trimmed_offset..];

            if let Some(after_gt) = trimmed.strip_prefix('>') {
                // The marker consumes `>` plus one column of following
                // whitespace. Expanding the rest of that whitespace run to
                // spaces (with original column arithmetic) keeps tab stops
                // aligned through the re-parse: `>\t\tfoo` becomes six
                // spaces + foo, i.e. indented code with two extra columns.
                let mut column = 0usize;
                for &byte in &line.as_bytes()[..trimmed_offset] {
                    column = if byte == b'\t' { (column / 4 + 1) * 4 } else { column + 1 };
                }
                let after_marker_column = column + 1;
                let ws_bytes = after_gt.as_bytes();
                let mut ws_len = 0usize;
                let mut ws_end_column = after_marker_column;
                while ws_len < ws_bytes.len() && matches!(ws_bytes[ws_len], b' ' | b'\t') {
                    ws_end_column = if ws_bytes[ws_len] == b'\t' {
                        (ws_end_column / 4 + 1) * 4
                    } else {
                        ws_end_column + 1
                    };
                    ws_len += 1;
                }
                let indent_columns = if ws_len > 0 {
                    ws_end_column.saturating_sub(after_marker_column + 1)
                } else {
                    0
                };
                for _ in 0..indent_columns {
                    inner.push(' ');
                }
                let stripped_trimmed = &after_gt[ws_len..];
                inner.push_str(stripped_trimmed);
                inner.push('\n');

                match fence {
                    Some((fence_byte, fence_len)) => {
                        if is_fence_close(stripped_trimmed, fence_byte, fence_len) {
                            fence = None;
                        }
                    }
                    None => fence = fence_open(stripped_trimmed),
                }
                // Lazy continuation only ever extends an open paragraph:
                // blank lines close it, indented code is not a paragraph,
                // and heading/thematic lines close it too. Deeper markers
                // (nested quotes, list items) keep a paragraph open.
                paragraph_open = fence.is_none()
                    && !stripped_trimmed.trim().is_empty()
                    && indent_columns < 4
                    && !stripped_trimmed.starts_with('#')
                    && !closes_paragraph_context(stripped_trimmed);

                // Advance past this line (and the trailing newline if any).
                self.position = if line_end < bytes.len() { line_end + 1 } else { line_end };
            } else if fence.is_none()
                && paragraph_open
                && !Self::quote_lazy_blocked(trimmed)
                && !self.line_starts_block()
            {
                // Lazy continuation: the line joins the quote's open
                // paragraph as if the `>` marker were present. Keeping the
                // original indentation means the re-parse still sees it as
                // paragraph continuation (an indented `- x` stays text),
                // and recording the offset stops setext reinterpretation.
                lazy_lines.insert(inner.len() as u32);
                inner.push_str(line);
                inner.push('\n');
                self.position = if line_end < bytes.len() { line_end + 1 } else { line_end };
            } else {
                // Line doesn't start with `>`, block quote ends
                break;
            }
        }

        // Recursively parse the inner content from the same arena — no copy.
        let inner_str = inner.into_bump_str();
        let sub_parser = self.sub_parser_with_lazy_lines(inner_str, lazy_lines);
        let sub_doc = sub_parser.parse()?;

        self.nesting_depth -= 1;

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::BlockQuote(BlockQuote { children: sub_doc.children, span })))
    }

    /// Lines that must not lazily continue a block quote paragraph even
    /// though they cannot interrupt one: a bare list marker opens an
    /// (empty) list block when the quote marker is imagined present.
    fn quote_lazy_blocked(trimmed: &str) -> bool {
        let line = trimmed.trim_end();
        Self::try_parse_list_line(line) && {
            let after_digits = line.trim_start_matches(|ch: char| ch.is_ascii_digit());
            let after_marker = after_digits.trim_start_matches(['-', '*', '+', '.', ')']);
            after_marker.trim().is_empty()
        }
    }
}
