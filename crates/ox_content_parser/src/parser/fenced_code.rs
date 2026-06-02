use memchr::memchr;
use ox_content_ast::{Node, Span};

use super::Parser;
use crate::error::{ParseError, ParseResult};
#[allow(unused_imports)]
use crate::profile_span;

impl<'a> Parser<'a> {
    /// Finds the body end and cursor position after a closing fence.
    ///
    /// This helper is used only by the zero-copy fenced-code path where the
    /// opening fence has no indentation. In that case body lines do not need
    /// stripping, so the parser can search line starts in the original source
    /// and return a borrowed body slice. The tuple separates the end of code
    /// content from the end of the closing fence line.
    pub(super) fn find_fenced_close(
        &self,
        fence_char: char,
        fence_len: usize,
        body_start: usize,
    ) -> (usize, usize) {
        let bytes = self.source.as_bytes();
        let fence_byte = fence_char as u8;
        let mut line_start = body_start;

        while line_start < bytes.len() {
            // Skip up to 3 leading spaces.
            let mut cursor = line_start;
            let max_indent_end = (line_start + 3).min(bytes.len());
            while cursor < max_indent_end && bytes[cursor] == b' ' {
                cursor += 1;
            }

            // Count the run of `fence_char`.
            let fence_start = cursor;
            while cursor < bytes.len() && bytes[cursor] == fence_byte {
                cursor += 1;
            }
            let count = cursor - fence_start;

            if count >= fence_len {
                // Found the closing fence. Body ends at `line_start`;
                // fence line ends at the next `\n` (inclusive) or EOF.
                let after_fence = match memchr(b'\n', &bytes[cursor..]) {
                    Some(off) => cursor + off + 1,
                    None => bytes.len(),
                };
                return (line_start, after_fence);
            }

            // Not a closing fence — move to the next line.
            line_start = match memchr(b'\n', &bytes[line_start..]) {
                Some(off) => line_start + off + 1,
                None => bytes.len(),
            };
        }

        // No closing fence; consume everything as body.
        (bytes.len(), bytes.len())
    }

    /// Parses a fenced code block.
    pub(super) fn parse_fenced_code(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_fenced_code");
        let opening_indent = self.calc_indentation(start).min(3);
        for _ in 0..opening_indent {
            if self.peek() == Some(' ') {
                self.advance();
            }
        }

        let Some(fence_char) = self.peek() else {
            return Err(ParseError::UnexpectedEof { span: Span::new(start as u32, start as u32) });
        };
        let mut fence_len = 0;

        while self.peek() == Some(fence_char) {
            fence_len += 1;
            self.advance();
        }

        // Parse info string (language)
        self.skip_whitespace();
        let info_start = self.position;
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
        let info = self.source[info_start..self.position].trim();
        let (lang, meta) = if info.is_empty() {
            (None, None)
        } else if let Some(space_idx) = info.find(' ') {
            (Some(&info[..space_idx]), Some(&info[space_idx + 1..]))
        } else {
            (Some(info), None)
        };

        // Skip newline after info string
        if self.peek() == Some('\n') {
            self.advance();
        }

        // Fast path: when the opening fence has no indentation, the body
        // lines need no indent stripping — we can find the closing fence
        // by scanning the source and emit a zero-copy `&str` slice. This
        // is the overwhelmingly common case (almost no code block in the
        // wild is indented), and it removes both the per-line copy into a
        // growing `String` *and* the trailing `alloc_str` (which used to
        // double-allocate every code block's body).
        let body_start = self.position;
        let span;
        let value: &'a str = if opening_indent == 0 {
            let (body_end, fence_line_end) =
                self.find_fenced_close(fence_char, fence_len, body_start);
            self.position = fence_line_end;
            span = Span::new(start as u32, self.position as u32);
            &self.source[body_start..body_end]
        } else {
            // Indented opening fence: lines may need leading-space stripping,
            // so a borrowed source slice would be wrong. Materialize only this
            // uncommon case, and write directly into a bump-allocated string
            // so the final AST can borrow it without a second arena copy.
            let remaining_estimate = self.source.len().saturating_sub(self.position);
            let mut value = ox_content_allocator::String::with_capacity_in(
                remaining_estimate.min(8 * 1024),
                self.allocator.bump(),
            );

            loop {
                if self.is_at_end() {
                    break;
                }

                let line_start = self.position;
                let line = self.line_at(line_start);
                let line_indent = Self::indentation_columns(line);

                if line_indent <= 3 {
                    self.position = line_start;
                    // `line_indent` was just computed from the same leading
                    // whitespace `calc_indentation(line_start)` would re-scan,
                    // and is already <= 3, so the `.min(3)` was a no-op.
                    let indent_to_skip = line_indent;
                    for _ in 0..indent_to_skip {
                        if self.peek() == Some(' ') {
                            self.advance();
                        }
                    }

                    // Check for closing fence
                    let mut closing_fence_len = 0;
                    while self.peek() == Some(fence_char) {
                        closing_fence_len += 1;
                        self.advance();
                    }

                    if closing_fence_len >= fence_len {
                        // Skip rest of line
                        while let Some(ch) = self.peek() {
                            if ch == '\n' {
                                self.advance();
                                break;
                            }
                            self.advance();
                        }
                        break;
                    }
                }

                // Not a closing fence, reset and consume line
                self.position = line_start;
                let next_line = self.next_line_start(line_start);
                let stripped = Self::strip_indent_columns(line, opening_indent);
                value.push_str(stripped);
                if self.source.as_bytes().get(line_start + line.len()) == Some(&b'\n') {
                    value.push('\n');
                }
                self.position = next_line;
            }

            span = Span::new(start as u32, self.position as u32);
            value.into_bump_str()
        };

        Ok(Some(Node::CodeBlock(ox_content_ast::CodeBlock { lang, meta, value, span })))
    }
}
