use memchr::{memchr, memchr3};
use ox_content_allocator::Vec;
use ox_content_ast::{Image, Node, Span, Text};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span_detail;

impl<'a> Parser<'a> {
    pub(super) fn parse_image(
        &self,
        content: &'a str,
        offset: usize,
        children: &mut Vec<'a, Node<'a>>,
        pos: &mut usize,
    ) -> ParseResult<()> {
        profile_span_detail!("parser::inline_image");
        let bytes = content.as_bytes();
        if *pos + 1 >= content.len() || bytes[*pos + 1] != b'[' {
            Self::push_text(children, "!", offset + *pos, offset + *pos + 1);
            *pos += 1;
            return Ok(());
        }

        let image_start = *pos;

        // Nothing can close this bracket, so skip the balanced scan that
        // would walk to the end of the content to reach the same verdict.
        // Same fallback as below, reached without the walk.
        if !self.has_closer_from(content, *pos + 2, b']') {
            Self::push_text(children, "![", offset + image_start, offset + image_start + 2);
            *pos = image_start + 2;
            return Ok(());
        }

        let alt_start = *pos + 2;
        let close = self.matching_bracket(content, alt_start);
        if close < content.len() {
            let raw_alt = &content[alt_start..close];
            // The alt text is parsed only once what follows the `]` has
            // made this an image. Doing it first parsed the text of every
            // bracket that turned out not to be one, and that text holds
            // the brackets nested in it: each level parsed all the levels
            // below it again, 2^depth in total.
            if let Some(tail) = self.link_tail(content, close, raw_alt) {
                let alt = self.flatten_image_alt(raw_alt, offset + alt_start)?;
                children.push(Node::Image(self.allocator.boxed(Image {
                    url: tail.url,
                    alt,
                    title: tail.title,
                    span: Span::new((offset + image_start) as u32, (offset + tail.end) as u32),
                })));
                *pos = tail.end;
                return Ok(());
            }
        }

        // No valid inline image here: `![` is literal text and the rest of
        // the bracketed run is re-parsed for other inline markup.
        Self::push_text(children, "![", offset + image_start, offset + image_start + 2);
        *pos = image_start + 2;
        Ok(())
    }

    /// Builds an image's `alt` attribute: the bracket text parsed as
    /// inlines and flattened to plain text (links contribute their text,
    /// code its literal content). Plain text stays zero-copy.
    fn flatten_image_alt(&self, raw: &'a str, offset: usize) -> ParseResult<&'a str> {
        let bytes = raw.as_bytes();
        // Alt text is usually tiny. A single scalar classifier avoids three
        // separate search setup costs there; larger spans still use memchr's
        // vectorized searches.
        let needs_parse = if bytes.len() < 16 {
            bytes.iter().any(|byte| matches!(byte, b'[' | b'*' | b'_' | b'`' | b'\\' | b'&' | b'<'))
        } else {
            memchr3(b'[', b'*', b'_', bytes).is_some()
                || memchr3(b'`', b'\\', b'&', bytes).is_some()
                || memchr(b'<', bytes).is_some()
        };
        if !needs_parse {
            return Ok(raw);
        }
        let nodes = self.parse_inline(raw, offset)?;
        let mut out = self.allocator.new_string();
        flatten_inline_text(&nodes, &mut out);
        Ok(out.into_bump_str())
    }

    /// Child slots to reserve for `content_len` bytes of inline content.
    ///
    /// A bump-allocated `Vec` cannot extend the block it owns — bumpalo
    /// hands back a fresh region and memcpies — so growing copies every
    /// node so far at each doubling step and abandons the old block in the
    /// arena. Measured over the bundled corpora the node count tracks the
    /// content length closely (p90 ≈ one node per 20 bytes in every length
    /// bucket), so reserving that covers most blocks in one allocation and
    /// still uses ~3% *less* arena than growing did. The floor keeps short
    /// spans at bumpalo's own minimum; the ceiling stops a long paragraph
    /// from reserving a kilobyte it will not fill.
    pub(super) fn inline_children_capacity(content_len: usize) -> usize {
        const BYTES_PER_NODE: usize = 20;
        (content_len / BYTES_PER_NODE).clamp(4, 12)
    }

    pub(super) fn push_text(
        children: &mut Vec<'a, Node<'a>>,
        value: &'a str,
        start: usize,
        end: usize,
    ) {
        profile_span_detail!("parser::push_text");
        children.push(Node::Text(Text { value, span: Span::new(start as u32, end as u32) }));
    }

    pub(super) fn marker_run_len(bytes: &[u8], start: usize, marker: u8) -> usize {
        let mut count = 1;
        while start + count < bytes.len() && bytes[start + count] == marker {
            count += 1;
        }
        count
    }

    /// Returns the byte after a closed code span that opens at `start`.
    ///
    /// Inline constructs outside code spans use this to skip over backtick
    /// regions while scanning for their own closing delimiter. An unmatched
    /// opener stays literal and therefore is not skipped.
    pub(super) fn closed_code_span_end(bytes: &[u8], start: usize) -> Option<usize> {
        let open_len = Self::marker_run_len(bytes, start, b'`');
        let mut cursor = start + open_len;
        while cursor < bytes.len() {
            let relative = memchr::memchr(b'`', &bytes[cursor..])?;
            cursor += relative;
            let close_len = Self::marker_run_len(bytes, cursor, b'`');
            if close_len == open_len {
                return Some(cursor + close_len);
            }
            cursor += close_len;
        }
        None
    }

    /// Reports whether `closer` occurs at or after `from` in `content`.
    ///
    /// The balanced scans (`matching_bracket` for `]`, `skip_braces` for `}`)
    /// only report that nothing closed after walking to the end of the
    /// content, so a run of unclosed openers pays one full walk each and
    /// costs O(n²). The position of the last closer settles it for every
    /// opener in the slice at once, so the run costs one scan in total.
    pub(super) fn has_closer_from(&self, content: &'a str, from: usize, closer: u8) -> bool {
        // Short inline slices usually ask this once. A direct scan avoids a
        // hash lookup and insertion for every link or image while bounding
        // any repeated scan to a small fixed amount of text.
        if content.len() <= 256 {
            return content
                .as_bytes()
                .get(from..)
                .is_some_and(|tail| memchr(closer, tail).is_some());
        }

        let key = (content.as_ptr() as usize, content.len(), closer);

        let cached = self.last_closer.borrow().get(&key).copied();
        let last = if let Some(last) = cached {
            last
        } else {
            let last = memchr::memrchr(closer, content.as_bytes());
            self.last_closer.borrow_mut().insert(key, last);
            last
        };

        last.is_some_and(|last| last >= from)
    }
}

/// Flattens inline nodes to their plain-text content (image `alt` rules).
fn flatten_inline_text(nodes: &[Node<'_>], out: &mut ox_content_allocator::String<'_>) {
    for node in nodes {
        match node {
            Node::Text(n) => out.push_str(n.value),
            Node::InlineCode(n) => out.push_str(n.value),
            Node::Emphasis(n) => flatten_inline_text(&n.children, out),
            Node::Strong(n) => flatten_inline_text(&n.children, out),
            Node::Delete(n) => flatten_inline_text(&n.children, out),
            Node::Superscript(n) => flatten_inline_text(&n.children, out),
            Node::Subscript(n) => flatten_inline_text(&n.children, out),
            Node::Link(n) => flatten_inline_text(&n.children, out),
            Node::Image(n) => out.push_str(n.alt),
            Node::Break(_) => out.push('\n'),
            _ => {}
        }
    }
}
