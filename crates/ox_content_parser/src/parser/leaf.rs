use memchr::memchr;
use ox_content_ast::{Node, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

impl<'a> Parser<'a> {
    /// Cheap recognizer for ATX heading starts used by block dispatch.
    ///
    /// The caller has already found the first non-whitespace byte. Requiring
    /// `line_start == trimmed_start` preserves the current rule that headings
    /// are not indented, while `is_atx_heading_prefix` validates the marker
    /// with byte checks and without allocating a trimmed line.
    pub(super) fn try_parse_heading_start(&self, line_start: usize, trimmed_start: usize) -> bool {
        line_start == trimmed_start
            && is_atx_heading_prefix(&self.source.as_bytes()[trimmed_start..])
    }

    pub(super) fn try_parse_thematic_break_line(line: &str) -> bool {
        let bytes = line.trim().as_bytes();
        if bytes.len() < 3 {
            return false;
        }
        let first = bytes[0];
        if !matches!(first, b'-' | b'*' | b'_') {
            return false;
        }
        let mut count = 0u32;
        for &b in bytes {
            if b == first {
                count += 1;
            } else if b != b' ' && b != b'\t' {
                return false;
            }
        }
        count >= 3
    }

    /// Checks whether a line begins a fenced code block.
    ///
    /// `line` is used only for indentation, and `trimmed` is the caller's
    /// already-sliced view starting at the first non-whitespace byte. This
    /// avoids recomputing `trim_start` in both `parse_block` and
    /// `line_starts_block`.
    pub(super) fn try_parse_fenced_code_at(line: &str, trimmed: &str) -> bool {
        if Self::indentation_columns(line) > 3 {
            return false;
        }

        let trimmed = trimmed.as_bytes();
        trimmed.len() >= 3
            && ((trimmed[0] == b'`' && trimmed[1] == b'`' && trimmed[2] == b'`')
                || (trimmed[0] == b'~' && trimmed[1] == b'~' && trimmed[2] == b'~'))
    }

    pub(super) fn indentation_columns(line: &str) -> usize {
        let mut indent = 0;
        for &b in line.as_bytes() {
            match b {
                b' ' => indent += 1,
                b'\t' => indent += 4,
                _ => break,
            }
        }
        indent
    }

    /// Parses a heading.
    pub(super) fn parse_heading(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_heading");
        let bytes = self.source.as_bytes();
        let mut depth = 0u8;
        // `#` is ASCII, so count the leading run with direct byte compares
        // instead of routing each through `peek()`/`advance()`.
        while self.position < bytes.len() && bytes[self.position] == b'#' {
            depth += 1;
            self.position += 1;
        }

        self.skip_whitespace();

        let content_start = self.position;
        // The heading content runs to the end of the line; find it in one
        // memchr scan rather than a per-char peek/advance walk.
        let content_end = memchr(b'\n', &bytes[content_start..])
            .map_or(self.source.len(), |off| content_start + off);
        self.position = content_end;

        // Skip trailing hashes and whitespace
        let content = self.source[content_start..content_end].trim_end();
        let content = content.trim_end_matches('#').trim_end();

        // Consume newline
        if self.peek() == Some('\n') {
            self.advance();
        }

        let span = Span::new(start as u32, self.position as u32);

        // Parse inline content
        let children = if !content.is_empty() {
            self.parse_inline(content, content_start)?
        } else {
            self.allocator.new_vec()
        };

        Ok(Some(Node::Heading(ox_content_ast::Heading { depth, children, span })))
    }

    /// Parses a thematic break.
    pub(super) fn parse_thematic_break(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        // Skip to (and past) the end of the current line. `consume_line`
        // advances to `line_end + 1`, or to EOF when there's no newline —
        // exactly the two positions the old peek/advance loop produced.
        self.consume_line();

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::ThematicBreak(ox_content_ast::ThematicBreak { span })))
    }
}

fn is_atx_heading_prefix(bytes: &[u8]) -> bool {
    // Count at most six leading hashes with direct byte checks. The following
    // byte must be whitespace, newline, or EOF, which lets the dispatcher
    // reject `#not-heading` without materializing a line string.
    let mut hashes = 0;
    while hashes < bytes.len() && bytes[hashes] == b'#' {
        hashes += 1;
        if hashes > 6 {
            return false;
        }
    }
    if hashes == 0 {
        return false;
    }
    matches!(bytes.get(hashes), None | Some(b' ' | b'\t' | b'\n'))
}
