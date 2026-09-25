use ox_content_ast::{BlockQuote, Node, Span};

use super::Parser;
use super::line_scan::{line_end as scan_line_end, line_terminator_end};
use super::reference::{closes_paragraph_context, fence_open, is_fence_close};
use super::spans::SourceMap;
use super::whitespace::is_blank;
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
        let mut source_map = SourceMap::default();

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
            if ws_cursor >= bytes.len() || matches!(bytes[ws_cursor], b'\n' | b'\r') {
                break;
            }

            let line_end = scan_line_end(bytes, line_start);
            let line_next = line_terminator_end(bytes, line_end);
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
                let generated_start = inner.len();
                for _ in 0..indent_columns {
                    inner.push(' ');
                }
                let stripped_trimmed = &after_gt[ws_len..];
                inner.push_str(stripped_trimmed);
                inner.push('\n');
                let content_start =
                    line_start + trimmed_offset + 1 + Self::quote_marker_space_bytes(after_gt);
                let source_len =
                    line_end.saturating_sub(content_start) + line_next.saturating_sub(line_end);
                source_map.push_line(
                    generated_start,
                    indent_columns + stripped_trimmed.len() + 1,
                    content_start,
                    source_len,
                );

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
                    && !is_blank(stripped_trimmed)
                    && indent_columns < 4
                    && !stripped_trimmed.starts_with('#')
                    && !closes_paragraph_context(stripped_trimmed);

                // Advance past this line (and the trailing newline if any).
                self.position = line_next;
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
                let generated_start = inner.len();
                lazy_lines.insert(inner.len() as u32);
                inner.push_str(line);
                inner.push('\n');
                source_map.push_line(
                    generated_start,
                    line.len() + 1,
                    line_start,
                    line.len() + line_next.saturating_sub(line_end),
                );
                self.position = line_next;
            } else {
                // Line doesn't start with `>`, block quote ends
                break;
            }
        }

        // Recursively parse the inner content from the same arena — no copy.
        let inner_str = inner.into_bump_str();
        let sub_parser = self.sub_parser_with_lazy_lines(inner_str, lazy_lines);
        let sub_doc = sub_parser.parse()?;
        let mut children = sub_doc.children;
        for child in &mut children {
            source_map.remap_node_spans(child);
        }

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::BlockQuote(self.allocator.boxed(BlockQuote { children, span }))))
    }

    /// Lines that must not lazily continue a block quote paragraph even
    /// though they cannot interrupt one: a bare list marker opens an
    /// (empty) list block when the quote marker is imagined present.
    fn quote_lazy_blocked(trimmed: &str) -> bool {
        let line = trimmed.trim_end();
        Self::try_parse_list_line(line) && {
            let after_digits = line.trim_start_matches(|ch: char| ch.is_ascii_digit());
            let after_marker = after_digits.trim_start_matches(['-', '*', '+', '.', ')']);
            is_blank(after_marker)
        }
    }

    fn quote_marker_space_bytes(after_gt: &str) -> usize {
        usize::from(after_gt.as_bytes().first() == Some(&b' '))
    }
}
