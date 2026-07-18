use memchr::memchr;
use ox_content_ast::{BlockQuote, Node, Span};

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
                // Strip the optional single space after `>`
                let stripped = after_gt.strip_prefix(' ').unwrap_or(after_gt);
                inner.push_str(stripped);
                inner.push('\n');

                // Advance past this line (and the trailing newline if any).
                self.position = if line_end < bytes.len() { line_end + 1 } else { line_end };
            } else {
                // Line doesn't start with `>`, block quote ends
                break;
            }
        }

        // Recursively parse the inner content from the same arena — no copy.
        let inner_str = inner.into_bump_str();
        let sub_parser = self.sub_parser(inner_str);
        let sub_doc = sub_parser.parse()?;

        self.nesting_depth -= 1;

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::BlockQuote(BlockQuote { children: sub_doc.children, span })))
    }
}
