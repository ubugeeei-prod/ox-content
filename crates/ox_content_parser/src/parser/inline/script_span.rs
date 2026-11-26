use memchr::memchr2;
use ox_content_allocator::Vec;
use ox_content_ast::{Node, Span};

use super::Parser;
use crate::error::ParseResult;

impl<'a> Parser<'a> {
    pub(super) fn parse_superscript_span(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        self.parse_script_span(content, offset, children, pos, ScriptKind::Superscript)
    }

    pub(super) fn parse_subscript_span(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        self.parse_script_span(content, offset, children, pos, ScriptKind::Subscript)
    }

    fn parse_script_span(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
        kind: ScriptKind,
    ) -> ParseResult<()> {
        let marker = kind.marker();
        let inner_start = *pos + 1;
        let bytes = content.as_bytes();
        let mut scan_from = inner_start;

        while let Some(relative) = memchr2(marker, b'`', &bytes[scan_from..]) {
            let inner_end = scan_from + relative;
            if bytes[inner_end] == b'`' {
                if let Some(end) = self.closed_code_span_end(bytes, inner_end) {
                    scan_from = end;
                } else {
                    scan_from = inner_end + Self::marker_run_len(bytes, inner_end, b'`');
                }
                continue;
            }
            if is_escaped_marker(bytes, inner_end) {
                scan_from = inner_end + 1;
                continue;
            }
            let inner = &content[inner_start..inner_end];
            if inner.is_empty()
                || inner.contains('\n')
                || inner.chars().next().is_some_and(char::is_whitespace)
                || inner.chars().next_back().is_some_and(char::is_whitespace)
            {
                break;
            }

            let inner_children = self.parse_inline(inner, offset + inner_start)?;
            let span = Span::new((offset + *pos) as u32, (offset + inner_end + 1) as u32);
            let node = match kind {
                ScriptKind::Superscript => Node::Superscript(
                    self.allocator
                        .boxed(ox_content_ast::Superscript { children: inner_children, span }),
                ),
                ScriptKind::Subscript => Node::Subscript(
                    self.allocator
                        .boxed(ox_content_ast::Subscript { children: inner_children, span }),
                ),
            };
            children.push(node);
            *pos = inner_end + 1;
            return Ok(());
        }

        Self::push_text(children, &content[*pos..*pos + 1], offset + *pos, offset + *pos + 1);
        *pos += 1;
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum ScriptKind {
    Superscript,
    Subscript,
}

impl ScriptKind {
    const fn marker(self) -> u8 {
        match self {
            Self::Superscript => b'^',
            Self::Subscript => b'~',
        }
    }
}

pub(super) fn same_marker_neighbor(bytes: &[u8], pos: usize, marker: u8) -> bool {
    pos.checked_sub(1).is_some_and(|prev| bytes[prev] == marker)
        || bytes.get(pos + 1).is_some_and(|next| *next == marker)
}

fn is_escaped_marker(bytes: &[u8], pos: usize) -> bool {
    let mut backslashes = 0;
    let mut i = pos;
    while let Some(prev) = i.checked_sub(1) {
        if bytes[prev] != b'\\' {
            break;
        }
        backslashes += 1;
        i = prev;
    }
    backslashes % 2 == 1
}
