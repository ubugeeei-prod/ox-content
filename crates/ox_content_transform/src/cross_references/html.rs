//! Attribute and text helpers for the cross-reference pass.
//!
//! These reproduce the semantics of the regular expressions the TypeScript
//! implementation used, including the parts that look accidental. Where a rule
//! is surprising it is spelled out, because "the regex did that" stops being an
//! answer once the regex is gone.

use super::types::CrossReferenceKind;
use crate::features::escape::{escape_html_attr, escape_html_text};

/// `id` prefixes that name a kind. Anything else is not a cross-reference.
pub(super) fn expected_kind(id: &str) -> Option<CrossReferenceKind> {
    let prefix = id.split('-').next().unwrap_or("").to_ascii_lowercase();
    match prefix.as_str() {
        "fig" | "figure" => Some(CrossReferenceKind::Figure),
        "tbl" | "table" => Some(CrossReferenceKind::Table),
        "sec" | "section" => Some(CrossReferenceKind::Section),
        _ => None,
    }
}

pub(super) fn should_track_target(id: &str) -> bool {
    expected_kind(id).is_some()
}

/// Reads one attribute out of a raw attribute string.
///
/// A bare attribute (`<img hidden>`) reads as an empty string, which is how the
/// caller tells "absent" from "present and empty".
pub(super) fn read_attr(attrs: &str, name: &str) -> Option<String> {
    // The original tried the quoted form across the whole string before
    // considering a bare attribute, so `id id="x"` reads as `x`, not as the
    // empty string the leading bare `id` would give. Two passes, same order.
    if let Some(value) = read_quoted_attr(attrs, name) {
        return Some(value);
    }
    read_bare_attr(attrs, name).then(String::new)
}

fn read_quoted_attr(attrs: &str, name: &str) -> Option<String> {
    let bytes = attrs.as_bytes();
    let mut cursor = 0usize;
    while let Some(found) = crate::html_scan::find_ci(attrs, cursor, name) {
        cursor = found + 1;
        if !preceded_by_boundary(bytes, found) {
            continue;
        }
        let mut probe = found + name.len();
        while probe < bytes.len() && bytes[probe].is_ascii_whitespace() {
            probe += 1;
        }
        if probe >= bytes.len() || bytes[probe] != b'=' {
            continue;
        }
        probe += 1;
        while probe < bytes.len() && bytes[probe].is_ascii_whitespace() {
            probe += 1;
        }
        if probe >= bytes.len() || (bytes[probe] != b'"' && bytes[probe] != b'\'') {
            continue;
        }
        let quote = bytes[probe] as char;
        let start = probe + 1;
        if let Some(end) = attrs[start..].find(quote).map(|rel| start + rel) {
            return Some(decode_html(&attrs[start..end]));
        }
    }
    None
}

fn read_bare_attr(attrs: &str, name: &str) -> bool {
    let bytes = attrs.as_bytes();
    let mut cursor = 0usize;
    while let Some(found) = crate::html_scan::find_ci(attrs, cursor, name) {
        cursor = found + 1;
        if !preceded_by_boundary(bytes, found) {
            continue;
        }
        let after = found + name.len();
        if after == bytes.len() || bytes[after].is_ascii_whitespace() {
            return true;
        }
    }
    false
}

/// `(?:^|\s)` — the name starts the string or follows whitespace.
fn preceded_by_boundary(bytes: &[u8], at: usize) -> bool {
    at == 0 || bytes[at - 1].is_ascii_whitespace()
}

/// Appends `name="value"` unless the attribute is already there.
pub(super) fn append_attr(attrs: &str, name: &str, value: &str) -> String {
    if read_attr(attrs, name).is_some() {
        return attrs.to_string();
    }
    let mut out = String::with_capacity(attrs.len() + name.len() + value.len() + 4);
    out.push_str(attrs);
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    escape_html_attr(value, &mut out);
    out.push('"');
    out
}

pub(super) fn append_data_attrs(
    attrs: &str,
    kind: CrossReferenceKind,
    number: &str,
    text: &str,
) -> String {
    let with_kind = append_attr(attrs, "data-ox-xref-kind", kind.as_str());
    let with_number = append_attr(&with_kind, "data-ox-xref-number", number);
    append_attr(&with_number, "data-ox-xref-label", text)
}

pub(super) fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    escape_html_text(value, &mut out);
    out
}

pub(super) fn escape_attr(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    escape_html_attr(value, &mut out);
    out
}

/// Percent-encodes for use as a URL fragment, matching `encodeURIComponent`.
///
/// The unreserved set is the one `encodeURIComponent` leaves alone:
/// alphanumerics and `-_.!~*'()`.
pub(super) fn escape_url_fragment(value: &str) -> String {
    const UNRESERVED: &[u8] = b"-_.!~*'()";
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || UNRESERVED.contains(&byte) {
            out.push(byte as char);
        } else {
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            out.push('%');
            out.push(HEX[usize::from(byte >> 4)] as char);
            out.push(HEX[usize::from(byte & 0x0f)] as char);
        }
    }
    out
}

/// Strips tags and collapses whitespace, the way a reader would read it.
pub(super) fn text_content(html: &str) -> String {
    let without_anchors = strip_header_anchors(html);
    let mut stripped = String::with_capacity(without_anchors.len());
    let bytes = without_anchors.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] == b'<' {
            // `<[^>]+>` needs at least one character before the `>`.
            if let Some(rel) = without_anchors[cursor + 1..].find('>')
                && rel > 0
            {
                cursor = cursor + 1 + rel + 1;
                continue;
            }
        }
        let ch_len = char_len(&without_anchors[cursor..]);
        stripped.push_str(&without_anchors[cursor..cursor + ch_len]);
        cursor += ch_len;
    }
    decode_html(&collapse_whitespace(&stripped))
}

/// Drops `<a class="header-anchor" …>…</a>`, which is chrome, not title text.
fn strip_header_anchors(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    while let Some(open) = crate::html_scan::find_ci(html, cursor, "<a") {
        // `<a\b`: `<address` is not an anchor.
        if html.as_bytes().get(open + 2).is_some_and(|byte| is_word_byte(*byte)) {
            out.push_str(&html[cursor..open + 2]);
            cursor = open + 2;
            continue;
        }
        let Some(open_end) = html[open..].find('>').map(|rel| open + rel + 1) else {
            break;
        };
        let attrs = &html[open + 2..open_end - 1];
        if !is_header_anchor(attrs) {
            out.push_str(&html[cursor..open + 2]);
            cursor = open + 2;
            continue;
        }
        let Some(close) = crate::html_scan::find_ci(html, open_end, "</a>") else {
            break;
        };
        out.push_str(&html[cursor..open]);
        cursor = close + 4;
    }
    out.push_str(&html[cursor..]);
    out
}

fn is_header_anchor(attrs: &str) -> bool {
    read_attr(attrs, "class").is_some_and(|value| value == "header-anchor")
}

fn collapse_whitespace(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut in_space = false;
    for ch in value.chars() {
        if ch.is_whitespace() {
            in_space = true;
            continue;
        }
        if in_space && !out.is_empty() {
            out.push(' ');
        }
        in_space = false;
        out.push(ch);
    }
    out
}

/// Undoes the five entities the writer emits. Order matters: `&amp;` last, so
/// `&amp;lt;` comes back as `&lt;` rather than `<`.
pub(super) fn decode_html(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// JavaScript's `\w`: ASCII letters, digits, and underscore.
pub(super) fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

pub(super) fn char_len(rest: &str) -> usize {
    rest.chars().next().map_or(1, char::len_utf8)
}
