//! Small HTML attribute scanner used by root-absolute URL rewriting.
//!
//! The renderer does not parse full HTML here; it only needs to find `href` and `src`
//! attribute values inside raw HTML tags so SSG base paths can be applied consistently.

pub(super) fn is_html_attr_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}

pub(super) fn is_html_attr_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':')
}

pub(super) fn html_attr_value_range(
    html: &str,
    bytes: &[u8],
    name_end: usize,
) -> Option<(usize, usize)> {
    let mut cursor = name_end;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'=') {
        return None;
    }
    cursor += 1;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }

    if let quote @ (b'"' | b'\'') = bytes.get(cursor).copied()? {
        let value_start = cursor + 1;
        let value_end = html[value_start..]
            .bytes()
            .position(|byte| byte == quote)
            .map(|offset| value_start + offset)?;
        Some((value_start, value_end))
    } else {
        let value_start = cursor;
        let mut value_end = value_start;
        while value_end < bytes.len()
            && !bytes[value_end].is_ascii_whitespace()
            && bytes[value_end] != b'>'
        {
            value_end += 1;
        }
        Some((value_start, value_end))
    }
}
