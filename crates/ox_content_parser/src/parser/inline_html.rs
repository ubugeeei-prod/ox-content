//! Inline raw HTML (CommonMark "Raw HTML").
//!
//! Only text matching the spec's tag grammar passes through as raw HTML;
//! anything else stays literal text. Open/closing tags may span line
//! endings, attribute names are restricted to the spec's character set,
//! attributes must be separated by whitespace, and comments, processing
//! instructions, declarations, and CDATA sections use their own
//! terminators.

use ox_content_ast::{Html, Span};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_inline_html(
        content: &'a str,
        pos: usize,
        offset: usize,
    ) -> Option<(Html<'a>, usize)> {
        let rest = &content[pos..];
        let bytes = content.as_bytes();

        // Comments, with the 0.31.2 empty forms `<!-->` and `<!--->`.
        for empty in ["<!-->", "<!--->"] {
            if rest.starts_with(empty) {
                let end = pos + empty.len();
                return Some((Self::inline_html_node(content, pos, end, offset), end));
            }
        }
        if let Some(comment_body) = rest.strip_prefix("<!--") {
            let end = comment_body.find("-->").map(|found| pos + 4 + found + 3)?;
            return Some((Self::inline_html_node(content, pos, end, offset), end));
        }
        // Processing instructions, CDATA, then other declarations.
        for (open, close) in [("<?", "?>"), ("<![CDATA[", "]]>")] {
            if let Some(body) = rest.strip_prefix(open) {
                let end = body.find(close).map(|found| pos + open.len() + found + close.len())?;
                return Some((Self::inline_html_node(content, pos, end, offset), end));
            }
        }
        if rest.starts_with("<!") && rest.as_bytes().get(2).is_some_and(u8::is_ascii_alphabetic) {
            let end = rest.find('>').map(|found| pos + found + 1)?;
            return Some((Self::inline_html_node(content, pos, end, offset), end));
        }

        let closing = bytes.get(pos + 1) == Some(&b'/');
        let tag_start = if closing { pos + 2 } else { pos + 1 };
        if !bytes.get(tag_start).is_some_and(u8::is_ascii_alphabetic) {
            return None;
        }

        let tag_end = Self::inline_tag_name_end(bytes, tag_start);
        let mut cursor = tag_end;

        if closing {
            while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\n')) {
                cursor += 1;
            }
            return (bytes.get(cursor) == Some(&b'>')).then(|| {
                let end = cursor + 1;
                (Self::inline_html_node(content, pos, end, offset), end)
            });
        }

        let end = scan_open_tag_rest(bytes, cursor)?;
        Some((Self::inline_html_node(content, pos, end, offset), end))
    }

    fn inline_tag_name_end(bytes: &[u8], start: usize) -> usize {
        let mut end = start + 1;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-') {
            end += 1;
        }
        end
    }

    fn inline_html_node(content: &'a str, start: usize, end: usize, offset: usize) -> Html<'a> {
        Html {
            value: &content[start..end],
            span: Span::new((offset + start) as u32, (offset + end) as u32),
        }
    }
}

/// Scans the remainder of an open tag after its name: whitespace-separated
/// attributes, optional `/`, and the closing `>`. Returns the index just
/// past `>`.
fn scan_open_tag_rest(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    loop {
        let ws_start = cursor;
        while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\n')) {
            cursor += 1;
        }
        match bytes.get(cursor)? {
            b'/' if bytes.get(cursor + 1) == Some(&b'>') => return Some(cursor + 2),
            b'>' => return Some(cursor + 1),
            _ => {
                // Attributes require separating whitespace.
                if cursor == ws_start {
                    return None;
                }
                cursor = scan_attribute(bytes, cursor)?;
            }
        }
    }
}

/// Scans one attribute: a spec-charset name plus an optional
/// `= value` (unquoted, single-, or double-quoted).
fn scan_attribute(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    let first = *bytes.get(cursor)?;
    if !(first.is_ascii_alphabetic() || matches!(first, b'_' | b':')) {
        return None;
    }
    cursor += 1;
    while bytes.get(cursor).is_some_and(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':' | b'-')
    }) {
        cursor += 1;
    }

    let mut look = cursor;
    while matches!(bytes.get(look), Some(b' ' | b'\t' | b'\n')) {
        look += 1;
    }
    if bytes.get(look) != Some(&b'=') {
        // Boolean attribute; the (skipped) whitespace still separates the
        // next attribute because the caller re-scans from `cursor`.
        return Some(cursor);
    }
    cursor = look + 1;
    while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\n')) {
        cursor += 1;
    }

    if let quote @ (b'"' | b'\'') = *bytes.get(cursor)? {
        cursor += 1;
        while *bytes.get(cursor)? != quote {
            cursor += 1;
        }
        return Some(cursor + 1);
    }

    let value_start = cursor;
    while bytes.get(cursor).is_some_and(|byte| {
        !matches!(byte, b' ' | b'\t' | b'\n' | b'"' | b'\'' | b'=' | b'<' | b'>' | b'`')
    }) {
        cursor += 1;
    }
    (cursor > value_start).then_some(cursor)
}
