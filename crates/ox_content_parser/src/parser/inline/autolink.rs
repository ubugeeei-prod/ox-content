//! CommonMark autolinks: `<scheme:uri>` and `<email@host>`.
//!
//! These are the core-spec autolinks (absolute URIs and email addresses
//! between angle brackets), independent from the GFM bare-URL extension.

use memchr::memchr;
use ox_content_ast::{Link, Node, Span};

use crate::parser::Parser;

/// Validation-only scan: returns the index just past `>` when an
/// autolink starts at `pos`, without building nodes.
pub(in crate::parser) fn autolink_end(content: &str, pos: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let inner_start = pos + 1;
    let close = memchr(b'>', bytes.get(inner_start..)?)? + inner_start;
    let inner = &content[inner_start..close];
    if inner.is_empty()
        || inner
            .bytes()
            .any(|byte| byte == b'<' || byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return None;
    }
    (is_absolute_uri(inner) || is_email_address(inner)).then_some(close + 1)
}

impl<'a> Parser<'a> {
    /// Tries to parse an autolink at `pos` (which points at `<`).
    /// Returns the link node and the position just past the closing `>`.
    pub(in crate::parser) fn parse_autolink(
        &self,
        content: &'a str,
        pos: usize,
        offset: usize,
    ) -> Option<(Node<'a>, usize)> {
        let bytes = content.as_bytes();
        let inner_start = pos + 1;
        let close = memchr(b'>', bytes.get(inner_start..)?)? + inner_start;
        let inner = &content[inner_start..close];

        if inner.is_empty()
            || inner
                .bytes()
                .any(|byte| byte == b'<' || byte.is_ascii_whitespace() || byte.is_ascii_control())
        {
            return None;
        }

        let url = if is_absolute_uri(inner) {
            inner
        } else if is_email_address(inner) {
            let mut url = self.allocator.new_string_from("mailto:");
            url.push_str(inner);
            url.into_bump_str()
        } else {
            return None;
        };

        let mut children = self.allocator.new_vec();
        children.push(Node::Text(ox_content_ast::Text {
            value: inner,
            span: Span::new((offset + inner_start) as u32, (offset + close) as u32),
        }));
        let link = Link {
            url,
            title: None,
            children,
            span: Span::new((offset + pos) as u32, (offset + close + 1) as u32),
        };
        Some((Node::Link(link), close + 1))
    }
}

/// `scheme:rest` where scheme is 2–32 characters, starts with a letter,
/// and continues with letters, digits, `+`, `.`, or `-`.
fn is_absolute_uri(s: &str) -> bool {
    let Some(colon) = s.find(':') else {
        return false;
    };
    let scheme = &s[..colon];
    (2..=32).contains(&scheme.len())
        && scheme.as_bytes()[0].is_ascii_alphabetic()
        && scheme
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'-'))
}

/// The spec's email autolink shape: `local@domain` with dot-separated
/// domain labels of up to 63 characters that don't start or end with `-`.
fn is_email_address(s: &str) -> bool {
    let Some((local, domain)) = s.split_once('@') else {
        return false;
    };
    if local.is_empty()
        || !local
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".!#$%&'*+/=?^_`{|}~-".contains(&byte))
    {
        return false;
    }
    !domain.is_empty()
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && !label.starts_with('-')
                && !label.ends_with('-')
        })
}
