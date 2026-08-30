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

        *pos += 2;
        let alt_start = *pos;
        *pos = Self::scan_balanced(content, *pos, b'[', b']');

        if *pos < content.len() && bytes[*pos] == b']' {
            let close = *pos;
            let raw_alt = &content[alt_start..close];
            let alt = self.flatten_image_alt(raw_alt, offset + alt_start)?;

            if bytes.get(close + 1) == Some(&b'(')
                && let Some(target) = self.parse_link_target(content, close + 1)
            {
                children.push(Node::Image(self.allocator.boxed(Image {
                    url: target.url,
                    alt,
                    title: target.title,
                    span: Span::new((offset + image_start) as u32, (offset + target.end) as u32),
                })));
                *pos = target.end;
                return Ok(());
            }

            let mut well_formed_reference = false;
            if bytes.get(close + 1) == Some(&b'[') && self.has_closer_from(content, close + 2, b']')
            {
                let label_start = close + 2;
                let label_end = Self::scan_balanced(content, label_start, b'[', b']');
                if label_end < content.len() && bytes[label_end] == b']' {
                    well_formed_reference = true;
                    let raw_label = &content[label_start..label_end];
                    let key = if raw_label.trim().is_empty() { raw_alt } else { raw_label };
                    if let Some(reference) = self.lookup_reference(key) {
                        children.push(Node::Image(self.allocator.boxed(Image {
                            url: reference.url,
                            alt,
                            title: reference.title,
                            span: Span::new(
                                (offset + image_start) as u32,
                                (offset + label_end + 1) as u32,
                            ),
                        })));
                        *pos = label_end + 1;
                        return Ok(());
                    }
                }
            }

            if !well_formed_reference && let Some(reference) = self.lookup_reference(raw_alt) {
                children.push(Node::Image(self.allocator.boxed(Image {
                    url: reference.url,
                    alt,
                    title: reference.title,
                    span: Span::new((offset + image_start) as u32, (offset + close + 1) as u32),
                })));
                *pos = close + 1;
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
        if memchr3(b'[', b'*', b'_', raw.as_bytes()).is_none()
            && memchr3(b'`', b'\\', b'&', raw.as_bytes()).is_none()
            && memchr(b'<', raw.as_bytes()).is_none()
        {
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

    /// Reports whether `closer` occurs at or after `from` in `content`.
    ///
    /// The balanced scans (`scan_balanced` for `]`, `skip_braces` for `}`)
    /// only report that nothing closed after walking to the end of the
    /// content, so a run of unclosed openers pays one full walk each and
    /// costs O(n²). The position of the last closer settles it for every
    /// opener in the slice at once, so the run costs one scan in total.
    pub(super) fn has_closer_from(&self, content: &'a str, from: usize, closer: u8) -> bool {
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

    /// Scans a balanced delimiter region and returns the matching close byte.
    ///
    /// Constructs that bind tighter than brackets are skipped whole:
    /// backslash escapes, code spans (an unmatched opener stays literal),
    /// autolinks, and inline raw HTML. This is what makes
    /// `[not a `link](/foo`)` a code span instead of a link.
    pub(super) fn scan_balanced(content: &str, mut cursor: usize, open: u8, close: u8) -> usize {
        let bytes = content.as_bytes();
        let mut depth = 1;
        while cursor < bytes.len() {
            match bytes[cursor] {
                b'\\' => {
                    // An escaped ASCII punctuation byte (which covers both
                    // delimiters) is inert for bracket matching.
                    let escapes_next =
                        cursor + 1 < bytes.len() && bytes[cursor + 1].is_ascii_punctuation();
                    cursor += if escapes_next { 2 } else { 1 };
                }
                b'`' => {
                    let run = Self::marker_run_len(bytes, cursor, b'`');
                    cursor += run;
                    let mut scan = cursor;
                    while scan < bytes.len() {
                        let Some(off) = memchr(b'`', &bytes[scan..]) else {
                            break;
                        };
                        scan += off;
                        let closer = Self::marker_run_len(bytes, scan, b'`');
                        if closer == run {
                            cursor = scan + closer;
                            break;
                        }
                        scan += closer;
                    }
                }
                b'<' => {
                    if let Some(end) = super::inline::autolink_end(content, cursor) {
                        cursor = end;
                    } else if let Some((_, end)) = Parser::parse_inline_html(content, cursor, 0) {
                        cursor = end;
                    } else {
                        cursor += 1;
                    }
                }
                byte if byte == open => {
                    depth += 1;
                    cursor += 1;
                }
                byte if byte == close => {
                    depth -= 1;
                    // Stop AT the closing delimiter.
                    if depth == 0 {
                        return cursor;
                    }
                    cursor += 1;
                }
                _ => cursor += 1,
            }
        }
        cursor
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
            Node::Link(n) => flatten_inline_text(&n.children, out),
            Node::Image(n) => out.push_str(n.alt),
            Node::Break(_) => out.push('\n'),
            _ => {}
        }
    }
}
