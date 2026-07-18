//! Indented code blocks (CommonMark "Indented code blocks").
//!
//! A run of lines indented at least four columns forms a code block whose
//! content is every line with the first four columns of indentation
//! removed. Interior blank lines are part of the block (contributing
//! whatever remains after the same four-column strip); leading and
//! trailing blank lines are not.

use ox_content_ast::{CodeBlock, Node, Span};

use super::Parser;
use crate::error::ParseResult;

impl<'a> Parser<'a> {
    /// Parses an indented code block starting at `start` (the beginning of
    /// a non-blank line whose indentation is at least four columns).
    pub(super) fn parse_indented_code(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        let mut value = self.allocator.new_string();
        let mut pos = start;
        let mut end = start;
        // Blank lines are buffered until the next sufficiently indented
        // line proves they are interior; trailing ones stay unconsumed.
        let mut pending_blank_lines = 0usize;
        let mut pending_blank_start = start;

        while pos < self.source.len() {
            let line_start = pos;
            let Some(first_non_ws) = self.first_non_whitespace_in_line(line_start) else {
                if pending_blank_lines == 0 {
                    pending_blank_start = line_start;
                }
                pending_blank_lines += 1;
                pos = self.next_line_start(line_start);
                continue;
            };

            if indent_width(self.source.as_bytes(), line_start, first_non_ws) < 4 {
                break;
            }
            let content_start = strip_indent_columns(self.source.as_bytes(), line_start, 4);

            if pending_blank_lines > 0 {
                let mut blank_pos = pending_blank_start;
                for _ in 0..pending_blank_lines {
                    let blank_content = strip_indent_columns(self.source.as_bytes(), blank_pos, 4);
                    let blank_line = self.line_at(blank_pos);
                    let blank_end = blank_pos + blank_line.len();
                    value.push_str(&self.source[blank_content.min(blank_end)..blank_end]);
                    value.push('\n');
                    blank_pos = self.next_line_start(blank_pos);
                }
                pending_blank_lines = 0;
            }

            let line = self.line_at(line_start);
            let line_end = line_start + line.len();
            value.push_str(&self.source[content_start..line_end]);
            value.push('\n');
            pos = self.next_line_start(line_start);
            end = pos;
        }

        self.position = end;
        let span = Span::new(start as u32, end as u32);
        Ok(Some(Node::CodeBlock(CodeBlock {
            lang: None,
            meta: None,
            value: value.into_bump_str(),
            span,
        })))
    }

    /// Returns the indentation width in columns of the current line, where
    /// a tab advances to the next multiple of four.
    pub(super) fn line_indent_width(&self, line_start: usize, first_non_ws: usize) -> usize {
        indent_width(self.source.as_bytes(), line_start, first_non_ws)
    }
}

fn indent_width(bytes: &[u8], line_start: usize, first_non_ws: usize) -> usize {
    let mut columns = 0usize;
    for &byte in &bytes[line_start..first_non_ws] {
        match byte {
            b'\t' => columns = (columns / 4 + 1) * 4,
            _ => columns += 1,
        }
    }
    columns
}

/// Returns the byte offset just past the first `columns` columns of
/// indentation on the line at `line_start`. Tabs advance to the next tab
/// stop; since tab stops are multiples of four, stripping four columns
/// never lands inside a tab.
fn strip_indent_columns(bytes: &[u8], line_start: usize, columns: usize) -> usize {
    let mut column = 0usize;
    let mut i = line_start;
    while column < columns && i < bytes.len() {
        match bytes[i] {
            b' ' => {
                column += 1;
                i += 1;
            }
            b'\t' => {
                column = (column / 4 + 1) * 4;
                i += 1;
            }
            _ => break,
        }
    }
    i
}
