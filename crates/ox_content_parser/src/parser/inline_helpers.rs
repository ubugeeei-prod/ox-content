use memchr::{memchr, memchr3};
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
            if let Some(target) = self.parse_link_target(content, *pos + 1) {
                let link_text = &content[text_start..*pos];
                let children_nodes = self.parse_inline(link_text, offset + text_start)?;
                children.push(Node::Link(Link {
                    url: target.url,
                    title: target.title,
                    children: children_nodes,
                    span: Span::new((offset + link_start) as u32, (offset + target.end) as u32),
                }));
                *pos = target.end;
                return Ok(());
            }
        }

        // No valid inline link here: the bracket is literal text and the
        // rest of the bracketed run is re-parsed for other inline markup.
        Self::push_text(children, "[", offset + link_start, offset + link_start + 1);
        *pos = link_start + 1;
        Ok(())
    }

    pub(super) fn parse_image(
        &self,
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
            if let Some(target) = self.parse_link_target(content, *pos + 1) {
                let alt = &content[alt_start..*pos];
                children.push(Node::Image(Image {
                    url: target.url,
                    alt,
                    title: target.title,
                    span: Span::new((offset + image_start) as u32, (offset + target.end) as u32),
                }));
                *pos = target.end;
                return;
            }
        }

        // No valid inline image here: `![` is literal text and the rest of
        // the bracketed run is re-parsed for other inline markup.
        Self::push_text(children, "![", offset + image_start, offset + image_start + 2);
        *pos = image_start + 2;
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

    /// Finds the next emphasis/strong delimiter run of at least `min_count`.
    ///
    /// Only occurrences of `marker` can change the result. Using `memchr` to
    /// jump between marker runs preserves the delimiter positions visited by
    /// the old byte-by-byte loop while making long non-marker spans cheap.
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

    /// Scans a balanced delimiter region and returns the matching close byte.
    ///
    /// Link labels only care about nested `open`/`close` delimiters and
    /// backslash escapes; every other byte is inert. `memchr3` moves directly
    /// to the next byte that can affect the scan, which keeps deeply textual
    /// labels from paying a branch for each character.
    fn scan_balanced(bytes: &[u8], mut cursor: usize, open: u8, close: u8) -> usize {
        let mut depth = 1;
        while cursor < bytes.len() {
            // Only `open`/`close` change depth and `\` can hide one of them,
            // so jump straight to the next such byte; the skipped bytes were
            // a no-op in the original loop.
            let Some(off) = memchr3(open, close, b'\\', &bytes[cursor..]) else {
                return bytes.len();
            };
            cursor += off;
            if bytes[cursor] == b'\\' {
                // An escaped ASCII punctuation byte (which covers both
                // delimiters) is inert for bracket matching.
                let escapes_next =
                    cursor + 1 < bytes.len() && bytes[cursor + 1].is_ascii_punctuation();
                cursor += if escapes_next { 2 } else { 1 };
                continue;
            }
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
