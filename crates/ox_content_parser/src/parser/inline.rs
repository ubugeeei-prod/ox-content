use ox_content_allocator::Vec;
use ox_content_ast::{Node, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

impl<'a> Parser<'a> {
    pub(super) fn parse_inline(
        &self,
        content: &'a str,
        offset: usize,
    ) -> ParseResult<Vec<'a, Node<'a>>> {
        profile_span!("parser::parse_inline");
        let mut children = self.allocator.new_vec();
        let mut pos = 0;
        let bytes = content.as_bytes();

        while pos < content.len() {
            let start = pos;
            pos = next_inline_special(bytes, pos);

            if pos > start {
                Self::push_text(&mut children, &content[start..pos], offset + start, offset + pos);
            }
            if pos >= content.len() {
                break;
            }

            self.parse_inline_special(content, offset, &mut children, &mut pos)?;
        }

        Ok(children)
    }

    fn parse_inline_special(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        let bytes = content.as_bytes();
        match bytes[*pos] {
            b'\\' if *pos + 1 < content.len() && bytes[*pos + 1] == b'\n' => {
                let span = Span::new((offset + *pos) as u32, (offset + *pos + 2) as u32);
                children.push(Node::Break(ox_content_ast::Break { span }));
                *pos += 2;
            }
            b'<' => Self::parse_inline_html_or_text(content, offset, children, pos),
            b'\\' if *pos + 1 < content.len() => {
                *pos += 1;
                let span_start = offset + *pos - 1;
                Self::push_text(children, &content[*pos..*pos + 1], span_start, offset + *pos + 1);
                *pos += 1;
            }
            b'~' if self.options.strikethrough
                && *pos + 1 < content.len()
                && bytes[*pos + 1] == b'~' =>
            {
                self.parse_strikethrough(content, offset, children, pos)?;
            }
            b'*' | b'_' => self.parse_delimited(content, offset, children, pos)?,
            b'`' => Self::parse_inline_code(content, offset, children, pos),
            b'[' => self.parse_link(content, offset, children, pos)?,
            b'!' => Self::parse_image(content, offset, children, pos),
            _ => {
                Self::push_text(
                    children,
                    &content[*pos..*pos + 1],
                    offset + *pos,
                    offset + *pos + 1,
                );
                *pos += 1;
            }
        }
        Ok(())
    }

    fn parse_inline_html_or_text(
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) {
        if let Some((html, end)) = Self::parse_inline_html(content, *pos, offset) {
            children.push(Node::Html(html));
            *pos = end;
        } else {
            Self::push_text(children, "<", offset + *pos, offset + *pos + 1);
            *pos += 1;
        }
    }

    fn parse_strikethrough(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        let bytes = content.as_bytes();
        let inner_start = *pos + 2;
        let mut inner_end = inner_start;

        while inner_end + 1 < content.len() {
            if bytes[inner_end] == b'~' && bytes[inner_end + 1] == b'~' {
                let inner_children =
                    self.parse_inline(&content[inner_start..inner_end], offset + inner_start)?;
                let span = Span::new((offset + *pos) as u32, (offset + inner_end + 2) as u32);
                children
                    .push(Node::Delete(ox_content_ast::Delete { children: inner_children, span }));
                *pos = inner_end + 2;
                return Ok(());
            }
            inner_end += 1;
        }

        Self::push_text(children, &content[*pos..*pos + 2], offset + *pos, offset + *pos + 2);
        *pos += 2;
        Ok(())
    }

    fn parse_delimited(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        let bytes = content.as_bytes();
        let marker = bytes[*pos];
        let count = Self::marker_run_len(bytes, *pos, marker);
        let inner_start = *pos + count;

        if let Some(inner_end) = Self::find_marker_run(bytes, inner_start, marker, count) {
            let inner_children =
                self.parse_inline(&content[inner_start..inner_end], offset + inner_start)?;
            let span = Span::new((offset + *pos) as u32, (offset + inner_end + count) as u32);
            if count >= 2 {
                children
                    .push(Node::Strong(ox_content_ast::Strong { children: inner_children, span }));
            } else {
                children.push(Node::Emphasis(ox_content_ast::Emphasis {
                    children: inner_children,
                    span,
                }));
            }
            *pos = inner_end + count;
        } else {
            Self::push_text(
                children,
                &content[*pos..*pos + count],
                offset + *pos,
                offset + *pos + count,
            );
            *pos += count;
        }
        Ok(())
    }

    fn parse_inline_code(
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) {
        *pos += 1;
        let code_start = *pos;
        while *pos < content.len() && content.as_bytes()[*pos] != b'`' {
            *pos += 1;
        }

        if *pos < content.len() {
            let span = Span::new((offset + code_start - 1) as u32, (offset + *pos + 1) as u32);
            children.push(Node::InlineCode(ox_content_ast::InlineCode {
                value: &content[code_start..*pos],
                span,
            }));
            *pos += 1;
        } else {
            Self::push_text(
                children,
                &content[code_start - 1..],
                offset + code_start - 1,
                offset + content.len(),
            );
        }
    }
}

/// Lookup table: `INLINE_SPECIAL[b] == 1` iff the byte is one of the
/// inline-special markers.
static INLINE_SPECIAL: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'*' as usize] = 1;
    t[b'_' as usize] = 1;
    t[b'`' as usize] = 1;
    t[b'[' as usize] = 1;
    t[b'!' as usize] = 1;
    t[b'~' as usize] = 1;
    t[b'\\' as usize] = 1;
    t[b'<' as usize] = 1;
    t
};

#[inline]
fn next_inline_special(bytes: &[u8], from: usize) -> usize {
    let mut i = from;
    let end = bytes.len();

    while i + 8 <= end {
        let chunk = &bytes[i..i + 8];
        let mask = INLINE_SPECIAL[chunk[0] as usize]
            | INLINE_SPECIAL[chunk[1] as usize]
            | INLINE_SPECIAL[chunk[2] as usize]
            | INLINE_SPECIAL[chunk[3] as usize]
            | INLINE_SPECIAL[chunk[4] as usize]
            | INLINE_SPECIAL[chunk[5] as usize]
            | INLINE_SPECIAL[chunk[6] as usize]
            | INLINE_SPECIAL[chunk[7] as usize];
        if mask != 0 {
            break;
        }
        i += 8;
    }
    while i < end && INLINE_SPECIAL[bytes[i] as usize] == 0 {
        i += 1;
    }
    i
}
