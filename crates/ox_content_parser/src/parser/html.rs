use ox_content_ast::{Html, Node, Span};

use super::Parser;
use crate::error::ParseResult;
#[allow(unused_imports)]
use crate::profile_span;

impl<'a> Parser<'a> {
    pub(super) fn try_parse_html_block(&self) -> bool {
        let line = self.remaining().lines().next().unwrap_or("");
        Self::parse_html_block_tag_name(line).is_some() || line.trim_start().starts_with("<!--")
    }

    pub(super) fn parse_html_block(&mut self, start: usize) -> ParseResult<Option<Node<'a>>> {
        profile_span!("parser::parse_html_block");
        let line = self.remaining().lines().next().unwrap_or("");

        if line.trim_start().starts_with("<!--") {
            loop {
                let consumed = self.consume_line();
                if consumed.contains("-->") || self.is_at_end() {
                    break;
                }
            }

            let span = Span::new(start as u32, self.position as u32);
            let value = &self.source[start..self.position];
            return Ok(Some(Node::Html(Html { value, span })));
        }

        // `parse_html_block_tag_name` returns a borrowed slice — no allocation.
        let Some(tag_name) = Self::parse_html_block_tag_name(line) else {
            return Ok(None);
        };

        if Self::is_type1_html_block_tag(tag_name) {
            // Type 1 blocks (`<pre>`, `<script>`, `<style>`, `<textarea>`)
            // close on the first line containing `</tag`. The previous
            // implementation built a `String` per parse + lowercased every
            // line; instead we scan bytes directly with case-insensitive
            // compare. For a single 25KB document with ~38 blocks this
            // turns ~hundreds of allocations into zero.
            let tag_bytes = tag_name.as_bytes();
            loop {
                let consumed = self.consume_line();
                if ascii_contains_closing_tag(consumed, tag_bytes) || self.is_at_end() {
                    break;
                }
            }
        } else {
            self.consume_line();
            // Walk forward line-by-line until a blank line. Previously we
            // peeked with `remaining().lines().next()` THEN consumed the
            // line, which walked each line twice. Consume first and inspect
            // the returned slice so we only scan once.
            while !self.is_at_end() {
                let line_start = self.position;
                let consumed = self.consume_line();
                if consumed.is_empty()
                    || consumed.bytes().all(|b| b == b' ' || b == b'\t' || b == b'\r')
                {
                    // Roll back so the outer `parse_block` loop sees the
                    // blank line and skips it via `skip_blank_lines`.
                    self.position = line_start;
                    break;
                }
            }
        }

        let span = Span::new(start as u32, self.position as u32);
        let value = &self.source[start..self.position];
        Ok(Some(Node::Html(Html { value, span })))
    }

    /// Returns the borrowed tag-name slice when `line` begins with one of
    /// the recognized HTML block tags. The slice points back into `line`'s
    /// underlying storage, so there's no allocation. Callers that need
    /// case-insensitive comparison should use `eq_ignore_ascii_case` directly
    /// — the source casing is preserved.
    pub(super) fn parse_html_block_tag_name(line: &str) -> Option<&str> {
        let trimmed = line.trim_start();
        let after_open = trimmed.strip_prefix('<')?;
        let after_slash = after_open.strip_prefix('/').unwrap_or(after_open);
        let mut tag_len = 0;

        for byte in after_slash.as_bytes() {
            if byte.is_ascii_alphanumeric() || *byte == b'-' {
                tag_len += 1;
            } else {
                break;
            }
        }

        if tag_len == 0 {
            return None;
        }

        let tag_name = &after_slash[..tag_len];
        let next = after_slash.as_bytes().get(tag_len).copied();

        if let Some(byte) = next {
            if !matches!(byte, b' ' | b'\t' | b'>' | b'/') {
                return None;
            }
        }

        if !Self::is_supported_html_block_tag(tag_name) {
            return None;
        }

        Some(tag_name)
    }

    pub(super) fn is_supported_html_block_tag(tag_name: &str) -> bool {
        [
            "article",
            "aside",
            "blockquote",
            "details",
            "dialog",
            "div",
            "figcaption",
            "figure",
            "footer",
            "header",
            "main",
            "nav",
            "ol",
            "p",
            "pre",
            "script",
            "section",
            "style",
            "summary",
            "table",
            "tbody",
            "td",
            "tfoot",
            "th",
            "thead",
            "textarea",
            "tr",
            "ul",
        ]
        .iter()
        .any(|candidate| tag_name.eq_ignore_ascii_case(candidate))
    }

    pub(super) fn is_type1_html_block_tag(tag_name: &str) -> bool {
        ["pre", "script", "style", "textarea"]
            .iter()
            .any(|candidate| tag_name.eq_ignore_ascii_case(candidate))
    }
}

pub(super) fn ascii_contains_closing_tag(haystack: &str, tag: &[u8]) -> bool {
    let bytes = haystack.as_bytes();
    if bytes.len() < tag.len() + 2 {
        return false;
    }
    let limit = bytes.len() - (tag.len() + 1);
    let mut i = 0;
    while i <= limit {
        if bytes[i] == b'<' && bytes[i + 1] == b'/' {
            let candidate = &bytes[i + 2..i + 2 + tag.len()];
            if candidate.eq_ignore_ascii_case(tag) {
                return true;
            }
        }
        i += 1;
    }
    false
}
