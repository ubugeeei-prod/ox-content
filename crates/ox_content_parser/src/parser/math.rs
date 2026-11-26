//! Opt-in `$...$` inline and `$$...$$` display math nodes.

use ox_content_allocator::Vec;
use ox_content_ast::{InlineMath, MathBlock, Node, Span};

use super::Parser;
use super::line_scan::{is_line_ending_byte, next_line_start};
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span_detail;

mod closer;

use self::closer::math_block_close;

impl<'a> Parser<'a> {
    pub(super) fn try_parse_math_block_at(
        &self,
        line_start: usize,
        line: &str,
        trimmed: &str,
    ) -> bool {
        let trimmed_offset = trimmed.as_ptr() as usize - line.as_ptr() as usize;
        self.options.math
            && Self::indentation_columns(line) <= 3
            && trimmed.starts_with("$$")
            && self.has_math_block_close_from(line_start + trimmed_offset + 2)
    }

    pub(super) fn parse_math_block(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span_detail!("parser::math_block");
        let line = self.line_at(start);
        let trimmed_offset = line.len() - line.trim_start_matches([' ', '\t']).len();
        let open = start + trimmed_offset;
        let Some(close) = math_block_close(self.source.as_bytes(), open + 2) else {
            return self.parse_paragraph(start, None);
        };
        let close_end = close + 2;
        self.position = next_line_start(self.source.as_bytes(), close_end);
        let value = &self.source[open + 2..close];
        Ok(Some(Node::MathBlock(
            self.allocator
                .boxed(MathBlock { value, span: Span::new(start as u32, self.position as u32) }),
        )))
    }

    pub(super) fn parse_inline_math(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) {
        profile_span_detail!("parser::inline_math");
        let bytes = content.as_bytes();
        let open_len = if bytes.get(*pos + 1) == Some(&b'$') { 2 } else { 1 };
        if !can_open_inline(bytes, *pos, open_len)
            && !self.can_open_digit_prefixed_inline_math(bytes, *pos, open_len)
        {
            Self::push_text(children, &content[*pos..*pos + 1], offset + *pos, offset + *pos + 1);
            *pos += 1;
            return;
        }

        let inner_start = *pos + open_len;
        if let Some(candidate) = self.inline_math_closer(bytes, inner_start, open_len) {
            let span = Span::new((offset + *pos) as u32, (offset + candidate + open_len) as u32);
            children.push(Node::InlineMath(InlineMath {
                value: &content[inner_start..candidate],
                span,
            }));
            *pos = candidate + open_len;
            return;
        }

        Self::push_text(
            children,
            &content[*pos..*pos + open_len],
            offset + *pos,
            offset + *pos + open_len,
        );
        *pos += open_len;
    }

    fn can_open_digit_prefixed_inline_math(
        &self,
        bytes: &'a [u8],
        index: usize,
        open_len: usize,
    ) -> bool {
        let next = bytes.get(index + open_len).copied();
        let prev = index.checked_sub(1).and_then(|prev| bytes.get(prev).copied());
        matches!(next, Some(b'0'..=b'9'))
            && !matches!(prev, Some(b'0'..=b'9'))
            && self.has_closing_inline_math(bytes, index, open_len)
    }

    fn has_closing_inline_math(&self, bytes: &'a [u8], index: usize, open_len: usize) -> bool {
        let inner_start = index + open_len;
        self.inline_math_closer(bytes, inner_start, open_len)
            .is_some_and(|closer| self.has_emphasis_marker_before(bytes, inner_start, closer))
    }
}

fn can_open_inline(bytes: &[u8], index: usize, open_len: usize) -> bool {
    let next = bytes.get(index + open_len).copied();
    let prev = index.checked_sub(1).and_then(|prev| bytes.get(prev).copied());
    !matches!(next, None | Some(b' ' | b'\t' | b'\n' | b'0'..=b'9'))
        && !matches!(prev, Some(b'0'..=b'9'))
}

fn can_close_inline(bytes: &[u8], index: usize, close_len: usize) -> bool {
    let prev = index.checked_sub(1).and_then(|prev| bytes.get(prev).copied());
    let next = bytes.get(index + close_len).copied();
    !matches!(prev, None | Some(b' ' | b'\t' | b'\n')) && !matches!(next, Some(b'0'..=b'9'))
}

fn is_escaped_marker(bytes: &[u8], pos: usize) -> bool {
    let mut count = 0usize;
    let mut cursor = pos;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        count += 1;
        cursor -= 1;
    }
    count % 2 == 1
}

fn is_line_end(bytes: &[u8], index: usize) -> bool {
    let mut cursor = index;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b' ' | b'\t' => cursor += 1,
            byte if is_line_ending_byte(byte) => return true,
            _ => return false,
        }
    }
    true
}
