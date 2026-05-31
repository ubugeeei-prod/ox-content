use memchr::{memchr, memchr2};
use ox_content_allocator::Vec;
use ox_content_ast::{Image, Link, Node, Span, Text};

use super::Parser;
use crate::error::ParseResult;

impl<'a> Parser<'a> {
    pub(super) fn parse_link(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        let bytes = content.as_bytes();
        let link_start = *pos;
        *pos += 1;
        let text_start = *pos;
        *pos = Self::scan_balanced(bytes, *pos, b'[', b']');

        if *pos < content.len()
            && bytes[*pos] == b']'
            && *pos + 1 < content.len()
            && bytes[*pos + 1] == b'('
        {
            let link_text = &content[text_start..*pos];
            *pos += 2;
            let url_start = *pos;
            *pos = Self::scan_balanced(bytes, *pos, b'(', b')');

            if *pos < content.len() && bytes[*pos] == b')' {
                let url = &content[url_start..*pos];
                *pos += 1;
                let children_nodes = self.parse_inline(link_text, offset + text_start)?;
                children.push(Node::Link(Link {
                    url,
                    title: None,
                    children: children_nodes,
                    span: Span::new((offset + link_start) as u32, (offset + *pos) as u32),
                }));
            } else {
                Self::push_text(
                    children,
                    &content[link_start..*pos],
                    offset + link_start,
                    offset + *pos,
                );
            }
        } else {
            Self::push_text(children, "[", offset + link_start, offset + link_start + 1);
            *pos = link_start + 1;
        }
        Ok(())
    }

    pub(super) fn parse_image(
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) {
        let bytes = content.as_bytes();
        if *pos + 1 >= content.len() || bytes[*pos + 1] != b'[' {
            Self::push_text(children, "!", offset + *pos, offset + *pos + 1);
            *pos += 1;
            return;
        }

        let image_start = *pos;
        *pos += 2;
        let alt_start = *pos;
        *pos = Self::scan_balanced(bytes, *pos, b'[', b']');

        if *pos < content.len()
            && bytes[*pos] == b']'
            && *pos + 1 < content.len()
            && bytes[*pos + 1] == b'('
        {
            let alt = &content[alt_start..*pos];
            *pos += 2;
            let url_start = *pos;
            *pos = Self::scan_balanced(bytes, *pos, b'(', b')');

            if *pos < content.len() && bytes[*pos] == b')' {
                let url = &content[url_start..*pos];
                *pos += 1;
                children.push(Node::Image(Image {
                    url,
                    alt,
                    title: None,
                    span: Span::new((offset + image_start) as u32, (offset + *pos) as u32),
                }));
            } else {
                Self::push_text(
                    children,
                    &content[image_start..*pos],
                    offset + image_start,
                    offset + *pos,
                );
            }
        } else {
            Self::push_text(children, "![", offset + image_start, offset + image_start + 2);
            *pos = image_start + 2;
        }
    }

    pub(super) fn push_text(
        children: &mut Vec<'a, Node<'a>>,
        value: &'a str,
        start: usize,
        end: usize,
    ) {
        children.push(Node::Text(Text { value, span: Span::new(start as u32, end as u32) }));
    }

    pub(super) fn marker_run_len(bytes: &[u8], start: usize, marker: u8) -> usize {
        let mut count = 1;
        while start + count < bytes.len() && bytes[start + count] == marker {
            count += 1;
        }
        count
    }

    pub(super) fn find_marker_run(
        bytes: &[u8],
        mut cursor: usize,
        marker: u8,
        min_count: usize,
    ) -> Option<usize> {
        while cursor < bytes.len() {
            // Skip directly to the next marker byte instead of inspecting every
            // intervening byte; the original `else { cursor += 1 }` arm was a
            // pure no-op skip, so the marker positions visited are identical.
            let off = memchr(marker, &bytes[cursor..])?;
            cursor += off;
            let count = Self::marker_run_len(bytes, cursor, marker);
            if count >= min_count {
                return Some(cursor);
            }
            cursor += count;
        }
        None
    }

    fn scan_balanced(bytes: &[u8], mut cursor: usize, open: u8, close: u8) -> usize {
        let mut depth = 1;
        while cursor < bytes.len() {
            // Only `open`/`close` change depth, so jump straight to the next one
            // via memchr2; the skipped bytes were a no-op in the original loop.
            let Some(off) = memchr2(open, close, &bytes[cursor..]) else {
                return bytes.len();
            };
            cursor += off;
            if bytes[cursor] == open {
                depth += 1;
            } else {
                depth -= 1;
                // Stop AT the closing delimiter (matching the original, which
                // skipped its trailing `cursor += 1` once depth hit 0).
                if depth == 0 {
                    return cursor;
                }
            }
            cursor += 1;
        }
        cursor
    }
}
