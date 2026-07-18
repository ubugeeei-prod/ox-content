//! GFM autolink extension: bare `www.`, `http(s)://`, and email
//! addresses in plain text become links (GFM spec "Autolinks
//! (extension)").
//!
//! Runs as a post-pass over a parsed inline sequence so it cannot
//! interfere with emphasis pairing, and recurses into emphasis-like
//! containers while never descending into existing links.

use memchr::memmem;
use ox_content_allocator::Vec;
use ox_content_ast::{Link, Node, Span, Text};

use crate::parser::Parser;

struct Candidate {
    start: usize,
    end: usize,
    href_prefix: &'static str,
}

impl<'a> Parser<'a> {
    pub(in crate::parser) fn apply_gfm_autolinks(&self, children: &mut Vec<'a, Node<'a>>) {
        // Entity, escape, and unpaired-delimiter handling fragment plain
        // prose into adjacent text nodes; autolinks must see the joined
        // run (`...?q=x&hl=en` is one URL despite the `&` split).
        self.coalesce_adjacent_text(children);
        let mut i = 0;
        while i < children.len() {
            match &mut children[i] {
                Node::Emphasis(node) => self.apply_gfm_autolinks(&mut node.children),
                Node::Strong(node) => self.apply_gfm_autolinks(&mut node.children),
                Node::Delete(node) => self.apply_gfm_autolinks(&mut node.children),
                Node::Text(text) => {
                    let value = text.value;
                    if let Some(candidate) = find_candidate(value) {
                        let span_start = text.span.start;
                        let link_value = &value[candidate.start..candidate.end];
                        let url: &'a str = if candidate.href_prefix.is_empty() {
                            link_value
                        } else {
                            let mut url = self.allocator.new_string_from(candidate.href_prefix);
                            url.push_str(link_value);
                            url.into_bump_str()
                        };

                        let link_span = Span::new(
                            span_start + candidate.start as u32,
                            span_start + candidate.end as u32,
                        );
                        let mut link_children = self.allocator.new_vec();
                        link_children.push(Node::Text(Text { value: link_value, span: link_span }));
                        let link_node = Node::Link(Link {
                            url,
                            title: None,
                            children: link_children,
                            span: link_span,
                        });

                        let after = &value[candidate.end..];
                        let after_node = (!after.is_empty()).then(|| {
                            Node::Text(Text {
                                value: after,
                                span: Span::new(span_start + candidate.end as u32, text.span.end),
                            })
                        });

                        if candidate.start == 0 {
                            children[i] = link_node;
                        } else {
                            text.value = &value[..candidate.start];
                            text.span = Span::new(span_start, span_start + candidate.start as u32);
                            i += 1;
                            children.insert(i, link_node);
                        }
                        if let Some(after_node) = after_node {
                            children.insert(i + 1, after_node);
                        }
                        // Continue scanning in the remainder text node.
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}

impl<'a> Parser<'a> {
    fn coalesce_adjacent_text(&self, children: &mut Vec<'a, Node<'a>>) {
        let mut i = 0;
        while i + 1 < children.len() {
            if !matches!(children[i], Node::Text(_)) || !matches!(children[i + 1], Node::Text(_)) {
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < children.len() && matches!(children[j], Node::Text(_)) {
                j += 1;
            }
            let mut merged = self.allocator.new_string();
            let mut span = Span::new(0, 0);
            for (index, node) in children[i..j].iter().enumerate() {
                if let Node::Text(text) = node {
                    if index == 0 {
                        span = text.span;
                    }
                    span = Span::new(span.start, text.span.end);
                    merged.push_str(text.value);
                }
            }
            let merged_node = Node::Text(Text { value: merged.into_bump_str(), span });
            children.drain(i..j);
            children.insert(i, merged_node);
            i += 1;
        }
    }
}

/// Finds the earliest valid autolink candidate in `value`.
fn find_candidate(value: &str) -> Option<Candidate> {
    let mut best: Option<Candidate> = None;
    for (needle, _href, is_email) in [
        ("www.", "http://", false),
        ("http://", "", false),
        ("https://", "", false),
        ("ftp://", "", false),
        ("@", "mailto:", true),
    ] {
        let mut from = 0;
        while let Some(offset) = memmem::find(&value.as_bytes()[from..], needle.as_bytes()) {
            let at = from + offset;
            let candidate = if is_email {
                validate_email(value, at)
            } else {
                validate_url(value, at, needle.len())
            };
            if let Some(candidate) = candidate {
                if best.as_ref().is_none_or(|current| candidate.start < current.start) {
                    best = Some(candidate);
                }
                break;
            }
            from = at + needle.len();
        }
    }
    best
}

/// Start-of-text, whitespace, or `*`, `_`, `~`, `(` may precede an
/// autolink.
fn valid_boundary(value: &str, start: usize) -> bool {
    value[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| ch.is_whitespace() || matches!(ch, '*' | '_' | '~' | '('))
}

fn validate_url(value: &str, start: usize, prefix_len: usize) -> Option<Candidate> {
    if !valid_boundary(value, start) {
        return None;
    }
    let bytes = value.as_bytes();
    // Validate the domain: alphanumerics, `-`, `_`, `.`; at least one
    // dot; no underscore in the last two segments.
    let domain_start = start + prefix_len;
    let mut domain_end = domain_start;
    while domain_end < bytes.len()
        && (bytes[domain_end].is_ascii_alphanumeric()
            || matches!(bytes[domain_end], b'-' | b'_' | b'.'))
    {
        domain_end += 1;
    }
    // Trailing dots belong to the surrounding sentence, not the domain.
    let domain = value[domain_start..domain_end].trim_end_matches('.');
    if domain.split('.').count() < 2
        || domain.rsplit('.').take(2).any(|segment| segment.is_empty() || segment.contains('_'))
    {
        return None;
    }

    // The link runs to whitespace or `<`, then trailing punctuation is
    // trimmed (unbalanced `)` and entity-like `&x;` suffixes included).
    let mut end = domain_end;
    while end < bytes.len() && !bytes[end].is_ascii_whitespace() && bytes[end] != b'<' {
        end += 1;
    }
    let end = trim_trailing_punctuation(value, start, end);
    (end > domain_start).then_some(Candidate {
        start,
        end,
        href_prefix: if prefix_len == 4 { "http://" } else { "" },
    })
}

fn trim_trailing_punctuation(value: &str, start: usize, mut end: usize) -> usize {
    let bytes = value.as_bytes();
    loop {
        if end <= start {
            return end;
        }
        match bytes[end - 1] {
            b'?' | b'!' | b'.' | b',' | b':' | b'*' | b'_' | b'~' | b'\'' | b'"' => end -= 1,
            b')' => {
                let opens = value[start..end].bytes().filter(|&b| b == b'(').count();
                let closes = value[start..end].bytes().filter(|&b| b == b')').count();
                if closes > opens {
                    end -= 1;
                } else {
                    return end;
                }
            }
            b';' => {
                // Strip an entity-like `&name;` suffix entirely.
                let entity_start = value[start..end - 1].rfind('&').map(|found| start + found);
                match entity_start {
                    Some(amp)
                        if value[amp + 1..end - 1]
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric())
                            && amp + 1 < end - 1 =>
                    {
                        end = amp;
                    }
                    _ => end -= 1,
                }
            }
            _ => return end,
        }
    }
}

fn validate_email(value: &str, at: usize) -> Option<Candidate> {
    let bytes = value.as_bytes();
    // Local part: alphanumerics plus `.`, `-`, `_`, `+`.
    let mut start = at;
    while start > 0 {
        let byte = bytes[start - 1];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+') {
            start -= 1;
        } else {
            break;
        }
    }
    if start == at || !valid_boundary(value, start) {
        return None;
    }

    // Domain: alphanumerics plus `.`, `-`, `_`, with at least one dot;
    // trailing dots are trimmed; a trailing `-` or `_` invalidates.
    let mut end = at + 1;
    while end < bytes.len()
        && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'.' | b'-' | b'_'))
    {
        end += 1;
    }
    while end > at + 1 && bytes[end - 1] == b'.' {
        end -= 1;
    }
    if end <= at + 1 || matches!(bytes[end - 1], b'-' | b'_') {
        return None;
    }
    if !value[at + 1..end].contains('.') {
        return None;
    }
    Some(Candidate { start, end, href_prefix: "mailto:" })
}
