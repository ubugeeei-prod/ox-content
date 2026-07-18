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

        // `[^label]` is a footnote reference when the extension is on and
        // a definition exists; otherwise it falls through to normal link
        // handling and may still be a link label.
        if self.options.footnotes
            && bytes.get(*pos + 1) == Some(&b'^')
            && self.try_parse_footnote_reference(content, offset, children, pos)
        {
            return Ok(());
        }

        *pos += 1;
        let text_start = *pos;
        *pos = Self::scan_balanced(content, *pos, b'[', b']');

        if *pos < content.len() && bytes[*pos] == b']' {
            let close = *pos;
            let link_text = &content[text_start..close];
            // Links may not contain other links; when the bracket text
            // parses to one, the outer bracket stays literal and the
            // inner (re-parsed after the fallback) wins.
            let inner_has_link = memchr::memchr(b'[', link_text.as_bytes()).is_some_and(|_| {
                self.parse_inline(link_text, offset + text_start)
                    .is_ok_and(|nodes| contains_link(&nodes))
            });

            // Inline form: [text](dest "title")
            if !inner_has_link && bytes.get(close + 1) == Some(&b'(') {
                if let Some(target) = self.parse_link_target(content, close + 1) {
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

            // Full [text][label] and collapsed [text][] reference forms.
            let mut well_formed_reference = false;
            if !inner_has_link && bytes.get(close + 1) == Some(&b'[') {
                let label_start = close + 2;
                let label_end = Self::scan_balanced(content, label_start, b'[', b']');
                if label_end < content.len() && bytes[label_end] == b']' {
                    well_formed_reference = true;
                    let raw_label = &content[label_start..label_end];
                    let key = if raw_label.trim().is_empty() { link_text } else { raw_label };
                    if let Some(reference) = self.lookup_reference(key) {
                        let (url, title) = (reference.url, reference.title);
                        let children_nodes = self.parse_inline(link_text, offset + text_start)?;
                        children.push(Node::Link(Link {
                            url,
                            title,
                            children: children_nodes,
                            span: Span::new(
                                (offset + link_start) as u32,
                                (offset + label_end + 1) as u32,
                            ),
                        }));
                        *pos = label_end + 1;
                        return Ok(());
                    }
                }
            }

            // Shortcut form: [label]. Suppressed when an explicit (but
            // unknown) [label] followed, which must stay literal.
            if !inner_has_link && !well_formed_reference {
                if let Some(reference) = self.lookup_reference(link_text) {
                    let (url, title) = (reference.url, reference.title);
                    let children_nodes = self.parse_inline(link_text, offset + text_start)?;
                    children.push(Node::Link(Link {
                        url,
                        title,
                        children: children_nodes,
                        span: Span::new((offset + link_start) as u32, (offset + close + 1) as u32),
                    }));
                    *pos = close + 1;
                    return Ok(());
                }
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
    ) -> ParseResult<()> {
        let bytes = content.as_bytes();
        if *pos + 1 >= content.len() || bytes[*pos + 1] != b'[' {
            Self::push_text(children, "!", offset + *pos, offset + *pos + 1);
            *pos += 1;
            return Ok(());
        }

        let image_start = *pos;
        *pos += 2;
        let alt_start = *pos;
        *pos = Self::scan_balanced(content, *pos, b'[', b']');

        if *pos < content.len() && bytes[*pos] == b']' {
            let close = *pos;
            let raw_alt = &content[alt_start..close];
            let alt = self.flatten_image_alt(raw_alt, offset + alt_start)?;

            if bytes.get(close + 1) == Some(&b'(') {
                if let Some(target) = self.parse_link_target(content, close + 1) {
                    children.push(Node::Image(Image {
                        url: target.url,
                        alt,
                        title: target.title,
                        span: Span::new(
                            (offset + image_start) as u32,
                            (offset + target.end) as u32,
                        ),
                    }));
                    *pos = target.end;
                    return Ok(());
                }
            }

            let mut well_formed_reference = false;
            if bytes.get(close + 1) == Some(&b'[') {
                let label_start = close + 2;
                let label_end = Self::scan_balanced(content, label_start, b'[', b']');
                if label_end < content.len() && bytes[label_end] == b']' {
                    well_formed_reference = true;
                    let raw_label = &content[label_start..label_end];
                    let key = if raw_label.trim().is_empty() { raw_alt } else { raw_label };
                    if let Some(reference) = self.lookup_reference(key) {
                        children.push(Node::Image(Image {
                            url: reference.url,
                            alt,
                            title: reference.title,
                            span: Span::new(
                                (offset + image_start) as u32,
                                (offset + label_end + 1) as u32,
                            ),
                        }));
                        *pos = label_end + 1;
                        return Ok(());
                    }
                }
            }

            if !well_formed_reference {
                if let Some(reference) = self.lookup_reference(raw_alt) {
                    children.push(Node::Image(Image {
                        url: reference.url,
                        alt,
                        title: reference.title,
                        span: Span::new((offset + image_start) as u32, (offset + close + 1) as u32),
                    }));
                    *pos = close + 1;
                    return Ok(());
                }
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

    /// Scans a balanced delimiter region and returns the matching close byte.
    ///
    /// Constructs that bind tighter than brackets are skipped whole:
    /// backslash escapes, code spans (an unmatched opener stays literal),
    /// autolinks, and inline raw HTML. This is what makes
    /// `[not a `link](/foo`)` a code span instead of a link.
    fn scan_balanced(content: &str, mut cursor: usize, open: u8, close: u8) -> usize {
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

/// Does any node in the tree contain a link?
fn contains_link(nodes: &[Node<'_>]) -> bool {
    nodes.iter().any(|node| match node {
        Node::Link(_) => true,
        Node::Emphasis(n) => contains_link(&n.children),
        Node::Strong(n) => contains_link(&n.children),
        Node::Delete(n) => contains_link(&n.children),
        _ => false,
    })
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
