use memchr::memchr;
use ox_content_ast::{Node, Paragraph, Span};

use super::Parser;
use crate::error::{ParseError, ParseResult};
#[allow(unused_imports)]
use crate::profile_span;

impl<'a> Parser<'a> {
    pub(super) fn parse_block(&mut self) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_block");
        self.skip_blank_lines();

        if self.is_at_end() {
            return Ok(None);
        }

        // Check nesting depth
        if self.nesting_depth > self.options.max_nesting_depth {
            return Err(ParseError::NestingTooDeep {
                span: Span::new(self.position as u32, self.position as u32),
                max_depth: self.options.max_nesting_depth,
            });
        }

        let start = self.position;
        let line = self.current_line();
        let trimmed = line.trim_start();
        let first = trimmed.as_bytes().first().copied();

        // Try to parse different block types. Each dispatch arm hands the
        // already-scanned `line` / `trimmed` slices to the matching
        // `*_at` helper so the inner check doesn't re-run `memchr` +
        // `trim_start` on the same line.
        match first {
            Some(b'#') if self.try_parse_heading_at(trimmed) => return self.parse_heading(start),
            Some(b'-' | b'*') => {
                if Self::try_parse_thematic_break_line(line) {
                    return self.parse_thematic_break(start);
                }
                if Self::try_parse_list_line(trimmed) {
                    return self.parse_list(start);
                }
            }
            Some(b'_') if Self::try_parse_thematic_break_line(line) => {
                return self.parse_thematic_break(start);
            }
            Some(b'>') => return self.parse_block_quote(start),
            Some(b'`' | b'~') if Self::try_parse_fenced_code_at(line, trimmed) => {
                return self.parse_fenced_code(start);
            }
            Some(b'<') if self.try_parse_html_block() => return self.parse_html_block(start),
            Some(b'+' | b'0'..=b'9') if Self::try_parse_list_line(trimmed) => {
                return self.parse_list(start);
            }
            _ => {}
        }

        if self.options.tables && memchr(b'|', line.as_bytes()).is_some() && self.try_parse_table()
        {
            return self.parse_table(start);
        }

        // Default: parse as paragraph
        self.parse_paragraph(start)
    }

    pub(super) fn parse_paragraph(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_paragraph");
        let bytes = self.source.as_bytes();

        // `parse_block` is the sole caller and only reaches here after
        // `skip_blank_lines` + its block dispatch — the very checks
        // `line_starts_block` re-runs — have already classified the current
        // line as a non-blank, non-block paragraph line. So consume the first
        // line unconditionally instead of re-deriving that verdict with
        // another `current_line` memchr + `trim_start` + dispatch (+ table
        // `memchr`). This also removes the infinite-loop hazard the two
        // dispatchers guard against: by always advancing past line one we can
        // never return `Ok(None)` without progress on a non-blank line.
        let mut content_end = if let Some(off) = memchr(b'\n', &bytes[start..]) {
            start + off + 1
        } else {
            self.source.len()
        };
        self.position = content_end;

        loop {
            if self.is_at_end() {
                break;
            }

            // Check for blank line (paragraph end): scan whitespace and
            // peek the next byte. Cheaper than the prior
            // `skip_whitespace` + `peek` + reset dance.
            let line_start = self.position;
            let mut cursor = line_start;
            while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
            if cursor >= bytes.len() || bytes[cursor] == b'\n' {
                break;
            }

            // Check for block-level element that would end paragraph.
            if self.line_starts_block() {
                break;
            }

            // Consume one line via memchr.
            content_end = if let Some(off) = memchr(b'\n', &bytes[line_start..]) {
                line_start + off + 1
            } else {
                self.source.len()
            };
            self.position = content_end;
        }

        let content = self.source[start..content_end].trim();
        if content.is_empty() {
            return Ok(None);
        }

        let span = Span::new(start as u32, content_end as u32);

        // Parse inline content
        let children = self.parse_inline(content, start)?;

        Ok(Some(Node::Paragraph(Paragraph { children, span })))
    }
}
