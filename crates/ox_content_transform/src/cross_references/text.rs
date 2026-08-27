//! Deciding which parts of the HTML are prose that `@ref` may be rewritten in.
//!
//! Three nested exclusions, applied outside-in: HTML comments and verbatim
//! elements are skipped whole, then tags are skipped within what remains, then
//! citation groups (`[@smith2020]`) are left for the citation pass.

use super::html::{char_len, is_word_byte};

/// Elements whose contents are never prose.
const PROTECTED_TAGS: [&str; 6] = ["pre", "code", "script", "style", "textarea", "a"];

/// Runs `replacer` over every stretch of prose in `html`.
pub(super) fn transform_text(html: &str, mut replacer: impl FnMut(&str) -> String) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;

    while let Some(region) = next_protected_region(html, cursor) {
        out.push_str(&transform_text_outside_tags(&html[cursor..region.start], &mut replacer));
        out.push_str(&html[region.start..region.end]);
        cursor = region.end;
    }
    out.push_str(&transform_text_outside_tags(&html[cursor..], &mut replacer));
    out
}

struct Region {
    start: usize,
    end: usize,
}

/// The next comment or verbatim element at or after `from`.
///
/// One forward scan, not one per tag name. Searching for each of the six names
/// separately means every name that does not occur again scans to the end of
/// the document, once per region — which made the pass quadratic in the number
/// of protected elements. Here each `<` is examined once.
///
/// A verbatim element runs to its *first* matching close tag: the original
/// pattern was lazy, so nesting is not tracked.
fn next_protected_region(html: &str, from: usize) -> Option<Region> {
    let bytes = html.as_bytes();
    let mut cursor = from;

    while let Some(rel) = memchr::memchr(b'<', &bytes[cursor..]) {
        let start = cursor + rel;
        cursor = start + 1;

        if html[start..].starts_with("<!--") {
            let end = html[start + 4..].find("-->").map_or(html.len(), |at| start + 4 + at + 3);
            return Some(Region { start, end });
        }

        let Some(tag) = protected_tag_at(html, start) else {
            continue;
        };
        let close = format!("</{tag}>");
        let Some(close_at) = crate::html_scan::find_ci(html, start + 1 + tag.len(), &close) else {
            // No close tag: the original pattern simply failed to match, so the
            // rest of the document stays prose.
            continue;
        };
        return Some(Region { start, end: close_at + close.len() });
    }
    None
}

/// The protected tag opening at `start`, if any, matching `<name\b`.
fn protected_tag_at(html: &str, start: usize) -> Option<&'static str> {
    let rest = html.get(start + 1..)?;
    PROTECTED_TAGS.into_iter().find(|tag| {
        rest.len() >= tag.len()
            && rest[..tag.len()].eq_ignore_ascii_case(tag)
            && !rest.as_bytes().get(tag.len()).is_some_and(|byte| is_word_byte(*byte))
    })
}

/// Applies `replacer` to the non-tag parts of a stretch of HTML.
///
/// A "tag" is `<` … `>` with at least one character between, matching the
/// original split pattern; a bare `<` is prose.
fn transform_text_outside_tags(segment: &str, replacer: &mut impl FnMut(&str) -> String) -> String {
    let mut out = String::with_capacity(segment.len());
    let mut text_start = 0usize;
    let mut cursor = 0usize;
    let bytes = segment.as_bytes();

    while cursor < bytes.len() {
        if bytes[cursor] == b'<'
            && let Some(rel) = segment[cursor + 1..].find('>')
            && rel > 0
        {
            let tag_end = cursor + 1 + rel + 1;
            out.push_str(&replacer(&segment[text_start..cursor]));
            out.push_str(&segment[cursor..tag_end]);
            cursor = tag_end;
            text_start = tag_end;
            continue;
        }
        cursor += char_len(&segment[cursor..]);
    }
    out.push_str(&replacer(&segment[text_start..]));
    out
}

/// Applies `replacer` to prose outside citation groups.
///
/// A citation group is `[…]` on one line whose body starts with `@` or `-@`.
/// Those belong to the citation pass, which runs later over the same text.
pub(super) fn transform_text_outside_citation_groups(
    text: &str,
    mut replacer: impl FnMut(&str) -> String,
) -> String {
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    let mut scan = 0usize;
    let bytes = text.as_bytes();

    while scan < bytes.len() {
        if bytes[scan] != b'[' {
            scan += char_len(&text[scan..]);
            continue;
        }
        // `[^\]\n]+` — at least one character, no `]` or newline inside.
        let Some(close) = text[scan + 1..].find([']', '\n']).map(|rel| scan + 1 + rel) else {
            break;
        };
        if bytes[close] != b']' || close == scan + 1 {
            scan += char_len(&text[scan..]);
            continue;
        }
        let body = text[scan + 1..close].trim();
        if !body.starts_with('@') && !body.starts_with("-@") {
            // Not a citation group; the scan continues past this `[` only, so a
            // later `[` inside the same span still gets its chance.
            scan += 1;
            continue;
        }
        out.push_str(&replacer(&text[cursor..scan]));
        out.push_str(&text[scan..=close]);
        cursor = close + 1;
        scan = close + 1;
    }
    out.push_str(&replacer(&text[cursor..]));
    out
}

/// One `@identifier` found in prose.
pub(super) struct TextReference {
    /// Byte range of the whole match, prefix character included.
    pub start: usize,
    pub end: usize,
    /// The prefix character, or empty at the start of the segment.
    pub prefix_len: usize,
    pub id_start: usize,
}

/// Finds `@identifier` occurrences, matching `(^|[^\w@/[])@([A-Za-z][A-Za-z0-9_-]*)\b`.
///
/// The prefix character is part of the match and therefore consumed, so
/// `@fig-1@fig-2` yields only the first — the second `@` sits where a prefix
/// would have to be, and `@` is excluded from that class.
pub(super) fn find_text_references(text: &str) -> Vec<TextReference> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        let Some(rel) = memchr::memchr(b'@', &bytes[cursor..]) else {
            break;
        };
        let at = cursor + rel;
        let prefix_len = if at == 0 {
            0
        } else {
            let prefix_start = prev_char_start(text, at);
            let prefix = bytes[prefix_start];
            if is_word_byte(prefix) || prefix == b'@' || prefix == b'/' || prefix == b'[' {
                cursor = at + 1;
                continue;
            }
            at - prefix_start
        };

        let id_start = at + 1;
        if !bytes.get(id_start).is_some_and(u8::is_ascii_alphabetic) {
            cursor = at + 1;
            continue;
        }
        let mut id_end = id_start;
        while bytes
            .get(id_end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'-')
        {
            id_end += 1;
        }
        if !word_boundary_at(bytes, id_end) {
            cursor = at + 1;
            continue;
        }

        found.push(TextReference { start: at - prefix_len, end: id_end, prefix_len, id_start });
        cursor = id_end;
    }
    found
}

/// `\b` between `at - 1` and `at`, using JavaScript's `\w`.
///
/// The identifier may end in `-`, which is not a word character, so the
/// boundary then requires the *next* character to be one — `@fig-` before a
/// space does not match, while `@fig-x` does.
fn word_boundary_at(bytes: &[u8], at: usize) -> bool {
    let before = at.checked_sub(1).is_some_and(|index| is_word_byte(bytes[index]));
    let after = bytes.get(at).is_some_and(|byte| is_word_byte(*byte));
    before != after
}

fn prev_char_start(text: &str, at: usize) -> usize {
    let mut index = at - 1;
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    index
}
