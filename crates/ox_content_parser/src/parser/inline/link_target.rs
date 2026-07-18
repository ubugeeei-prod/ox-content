//! Inline link destination and title parsing (CommonMark "Links").
//!
//! Handles the `(...)` part of `[text](dest "title")`: pointy-bracket and
//! bare destinations, the three title quoting styles, backslash escapes,
//! and the surrounding whitespace rules. Escaped destinations/titles are
//! unescaped into the arena so the AST keeps borrowing from parser memory.

use crate::parser::Parser;

pub(in crate::parser) struct LinkTarget<'a> {
    pub url: &'a str,
    pub title: Option<&'a str>,
    /// Byte index in the inline content just past the closing `)`.
    pub end: usize,
}

impl<'a> Parser<'a> {
    /// Parses a link target starting at the `(` at `open`. Returns `None`
    /// when the parenthesized run is not a valid destination/title pair,
    /// in which case the bracket text falls back to literal parsing.
    pub(in crate::parser) fn parse_link_target(
        &self,
        content: &'a str,
        open: usize,
    ) -> Option<LinkTarget<'a>> {
        let bytes = content.as_bytes();
        let mut i = skip_ws(bytes, open + 1);

        let (raw_url, after_dest) = parse_destination(content, i)?;
        i = skip_ws(bytes, after_dest);

        let mut title = None;
        // A title needs whitespace between it and the destination.
        if i > after_dest {
            if let Some((raw_title, after_title)) = parse_title(content, i) {
                title = Some(self.unescape_component(raw_title));
                i = skip_ws(bytes, after_title);
            }
        }

        if bytes.get(i) != Some(&b')') {
            return None;
        }
        Some(LinkTarget { url: self.unescape_component(raw_url), title, end: i + 1 })
    }

    /// Removes backslashes that escape ASCII punctuation and decodes
    /// entity/numeric character references (both apply inside link
    /// destinations and titles). Returns the input slice untouched when
    /// nothing decodes; otherwise the copy is allocated in the arena.
    fn unescape_component(&self, raw: &'a str) -> &'a str {
        let bytes = raw.as_bytes();
        let mut i = 0;
        let mut start = 0;
        let mut out: Option<ox_content_allocator::String<'a>> = None;
        while i < bytes.len() {
            match bytes[i] {
                b'\\' if i + 1 < bytes.len() && bytes[i + 1].is_ascii_punctuation() => {
                    let out = out.get_or_insert_with(|| self.allocator.new_string());
                    out.push_str(&raw[start..i]);
                    start = i + 1;
                    i += 2;
                }
                b'&' => {
                    if let Some((value, len)) = super::entity::scan_entity(&raw[i..]) {
                        let out = out.get_or_insert_with(|| self.allocator.new_string());
                        out.push_str(&raw[start..i]);
                        match value {
                            super::entity::EntityValue::Named(expansion) => {
                                out.push_str(expansion);
                            }
                            super::entity::EntityValue::Char(ch) => out.push(ch),
                        }
                        i += len;
                        start = i;
                    } else {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }
        match out {
            Some(mut out) => {
                out.push_str(&raw[start..]);
                out.into_bump_str()
            }
            None => raw,
        }
    }
}

/// Parses a destination at `i`: either `<...>` (may contain spaces, no
/// newlines or unescaped angle brackets) or a bare run without whitespace
/// or control characters and with balanced unescaped parentheses.
fn parse_destination(content: &str, i: usize) -> Option<(&str, usize)> {
    let bytes = content.as_bytes();
    if bytes.get(i) == Some(&b'<') {
        let mut j = i + 1;
        loop {
            match bytes.get(j)? {
                b'\\' if is_escape(bytes, j) => j += 2,
                b'>' => return Some((&content[i + 1..j], j + 1)),
                b'<' | b'\n' => return None,
                _ => j += 1,
            }
        }
    }

    let mut depth = 0usize;
    let mut j = i;
    while j < bytes.len() {
        match bytes[j] {
            b'\\' if is_escape(bytes, j) => j += 2,
            b'(' => {
                depth += 1;
                j += 1;
            }
            b')' if depth == 0 => break,
            b')' => {
                depth -= 1;
                j += 1;
            }
            byte if byte.is_ascii_whitespace() || byte.is_ascii_control() => break,
            _ => j += 1,
        }
    }
    if depth > 0 {
        return None;
    }
    Some((&content[i..j], j))
}

/// Parses a `"..."`, `'...'`, or `(...)` title starting at `i`.
fn parse_title(content: &str, i: usize) -> Option<(&str, usize)> {
    let bytes = content.as_bytes();
    let (closer, nested_open) = match bytes.get(i)? {
        b'"' => (b'"', None),
        b'\'' => (b'\'', None),
        b'(' => (b')', Some(b'(')),
        _ => return None,
    };

    let mut j = i + 1;
    loop {
        match bytes.get(j)? {
            b'\\' if is_escape(bytes, j) => j += 2,
            byte if *byte == closer => return Some((&content[i + 1..j], j + 1)),
            byte if nested_open == Some(*byte) => return None,
            _ => j += 1,
        }
    }
}

fn is_escape(bytes: &[u8], i: usize) -> bool {
    i + 1 < bytes.len() && bytes[i + 1].is_ascii_punctuation()
}

fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while matches!(bytes.get(i), Some(b' ' | b'\t' | b'\n' | b'\r')) {
        i += 1;
    }
    i
}
