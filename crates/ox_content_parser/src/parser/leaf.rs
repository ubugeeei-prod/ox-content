use ox_content_ast::{Node, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

impl<'a> Parser<'a> {
    pub(super) fn try_parse_heading_at(&self, trimmed: &str) -> bool {
        let bytes = self.source.as_bytes();
        let pos = self.position;
        if pos < bytes.len() && matches!(bytes[pos], b' ' | b'\t') {
            return false;
        }
        is_atx_heading_prefix(trimmed.as_bytes())
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
        let mut depth = 0u8;
        while self.peek() == Some('#') {
            depth += 1;
            self.advance();
        }

        self.skip_whitespace();

        let content_start = self.position;
        let mut content_end = content_start;

        // Read until end of line
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
            content_end = self.position;
        }

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
        // Skip to end of line
        while let Some(ch) = self.peek() {
            self.advance();
            if ch == '\n' {
                break;
            }
        }

        let span = Span::new(start as u32, self.position as u32);
        Ok(Some(Node::ThematicBreak(ox_content_ast::ThematicBreak { span })))
    }
}

fn is_atx_heading_prefix(bytes: &[u8]) -> bool {
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
