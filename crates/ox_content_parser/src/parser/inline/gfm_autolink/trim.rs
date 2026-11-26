//! Trailing punctuation that belongs to the sentence, not the URL.
//!
//! GFM spec "Autolinks (extension)", extended autolink path validation:
//! `?!.,:*_~'"` at the end are dropped, a `)` is dropped while the link has
//! more closing than opening parentheses, and a trailing `&name;` that looks
//! like an entity reference is dropped whole.

/// Trims the candidate `value[start..end]` and returns its new end.
///
/// Each rule used to re-read the candidate from the front for every byte it
/// judged: a `)` recounted both kinds of parenthesis, and a `;` searched
/// back for its `&` and then re-checked the name in between. A link followed
/// by a run of either therefore cost O(n²) — 125 KB of `)` after `www.a.b`
/// took 4.0 s, of `;` 0.55 s, and of `&a;)` 1.0 s, each growing x16 for
/// every x4 of input. One trim now reads the candidate once: the
/// parentheses are counted when the first `)` is met and kept up to date as
/// the end moves, and an entity name is only ever walked from its `;`.
pub(super) fn trim_trailing_punctuation(value: &str, start: usize, mut end: usize) -> usize {
    let bytes = value.as_bytes();
    // `(` and `)` in `value[start..end]`, counted on first use.
    let mut parentheses: Option<(usize, usize)> = None;
    loop {
        if end <= start {
            return end;
        }
        match bytes[end - 1] {
            b'?' | b'!' | b'.' | b',' | b':' | b'*' | b'_' | b'~' | b'\'' | b'"' => end -= 1,
            b')' => {
                let (opens, closes) =
                    parentheses.unwrap_or_else(|| count_parentheses(&bytes[start..end]));
                if closes <= opens {
                    return end;
                }
                // Nothing trimmed between two `)` holds a parenthesis, so
                // dropping this one is the only change to either count.
                parentheses = Some((opens, closes - 1));
                end -= 1;
            }
            b';' => end = entity_start(bytes, start, end).unwrap_or(end - 1),
            _ => return end,
        }
    }
}

fn count_parentheses(candidate: &[u8]) -> (usize, usize) {
    let mut opens = 0;
    let mut closes = 0;
    for &byte in candidate {
        opens += usize::from(byte == b'(');
        closes += usize::from(byte == b')');
    }
    (opens, closes)
}

/// The `&` of an entity-like `&name;` that ends at `end`, when
/// `value[start..end]` ends in one.
///
/// The name is the alphanumeric run in front of the `;`. It has to be
/// non-empty and the byte in front of it has to be the `&` — which is then
/// the last `&` in the candidate, the one the rule is defined on.
fn entity_start(bytes: &[u8], start: usize, end: usize) -> Option<usize> {
    let semicolon = end - 1;
    let mut name = semicolon;
    while name > start && bytes[name - 1].is_ascii_alphanumeric() {
        name -= 1;
    }
    (name < semicolon && name > start && bytes[name - 1] == b'&').then(|| name - 1)
}
