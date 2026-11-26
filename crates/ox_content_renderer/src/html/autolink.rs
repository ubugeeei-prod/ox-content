//! Bare-URL autolink scanner.
//!
//! Text nodes can optionally turn recognized URL prefixes into anchors. The scanner
//! first indexes possible leading bytes so most prose is skipped without repeated
//! prefix checks, then validates word boundaries and trims punctuation around matches.

use super::options::AutolinkPatterns;

mod index;

pub(in crate::html) use index::FirstByteIndex;

/// Scans `s` from `from` for the next position that begins one of the
/// registered URL prefixes at a word boundary, and returns the
/// `(match_start, url_end)` byte range with trailing punctuation trimmed.
///
/// The boundary rule mirrors common autolinkers: a match is only accepted
/// when the preceding byte (if any) is not an ASCII alphanumeric — so
/// `"see http://x"` matches but `"shttp://x"` doesn't. The URL extends to
/// the next whitespace, `<`, `>`, `"`, `'`, or backtick, and we then strip
/// trailing `.,;:!?` plus an unbalanced `)`, `]`, or `}`.
///
/// `index` skips ahead to the next byte that could start a pattern, so the
/// per-byte boundary and prefix checks below only run at real candidates
/// rather than across every byte of non-URL prose.
pub(super) fn find_autolink_match(
    s: &str,
    from: usize,
    patterns: AutolinkPatterns<'_>,
    index: &FirstByteIndex,
) -> Option<(usize, usize)> {
    match patterns {
        AutolinkPatterns::Defaults(patterns) => find_autolink_match_in(s, from, patterns, index),
        AutolinkPatterns::Custom(patterns) => find_autolink_match_in(s, from, patterns, index),
    }
}

fn find_autolink_match_in<P: AsRef<str>>(
    s: &str,
    from: usize,
    patterns: &[P],
    index: &FirstByteIndex,
) -> Option<(usize, usize)> {
    crate::profile_span_detail!("renderer::autolink_scan");
    let bytes = s.as_bytes();
    let mut base = from;
    while base < bytes.len() {
        let rel = index.next(&bytes[base..])?;
        let i = base + rel;
        // One-byte look-ahead: most prose hits on a frequent candidate byte
        // ("the", "words"…) die here before any prefix comparison.
        if index.rejects_second(bytes[i], bytes.get(i + 1)) {
            base = i + 1;
            continue;
        }
        // Word boundary: the previous byte must not be ASCII alphanumeric.
        let is_boundary = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if is_boundary {
            for pat in patterns {
                let pat = pat.as_ref();
                let pat_bytes = pat.as_bytes();
                if pat_bytes.is_empty() {
                    continue;
                }
                if i + pat_bytes.len() <= bytes.len()
                    && bytes[i..i + pat_bytes.len()].eq_ignore_ascii_case(pat_bytes)
                {
                    let url_start = i;
                    let url_end = scan_url_end(s, i + pat_bytes.len());
                    // Require at least one byte beyond the scheme/prefix
                    // so `"http://"` on its own isn't auto-linked.
                    if url_end == i + pat_bytes.len() {
                        continue;
                    }
                    return Some((url_start, trim_trailing_punct(bytes, url_start, url_end)));
                }
            }
        }
        base = i + 1;
    }
    None
}

/// Extends a URL from `from` to the offset where it stops.
///
/// Non-ASCII characters normally belong to the URL — an IRI carries them
/// verbatim (`/wiki/日本語`) — with CJK sentence punctuation the exception.
fn scan_url_end(s: &str, from: usize) -> usize {
    let bytes = s.as_bytes();
    let mut end = from;
    while end < bytes.len() {
        let byte = bytes[end];
        if byte.is_ascii() {
            if !is_url_byte(byte) {
                break;
            }
            end += 1;
            continue;
        }
        let Some(ch) = s.get(end..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        if ends_url(ch) {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

#[inline]
fn is_url_byte(byte: u8) -> bool {
    !matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'<' | b'>' | b'"' | b'\'' | b'`')
}

/// Punctuation that ends a bare URL the way ASCII whitespace does.
///
/// Trailing-punctuation handling is defined for ASCII only, and CJK prose
/// puts no space between a URL and the `。` that closes the sentence — so
/// without this the rest of the sentence is swallowed into the link.
///
/// Mirrored by the GFM autolink scan in `ox_content_parser`.
const fn ends_url(ch: char) -> bool {
    matches!(
        ch,
        // CJK symbols and punctuation: the ideographic space, 、。〈〉《》
        // 「」『』【】 and friends.
        '\u{3000}'..='\u{303F}'
        // Fullwidth ！＂＃＄％＆＇（）＊＋，－．／
        | '\u{FF01}'..='\u{FF0F}'
        // Fullwidth ：；＜＝＞？＠
        | '\u{FF1A}'..='\u{FF20}'
        // Fullwidth ［＼］＾＿｀
        | '\u{FF3B}'..='\u{FF40}'
        // Fullwidth ｛｜｝～｟｠ and halfwidth ｡｢｣､･
        | '\u{FF5B}'..='\u{FF65}'
    )
}

/// Strips trailing `.,;:!?`, and a closing bracket when the rest of the URL
/// closes at least as many of its kind as it opens.
///
/// The bracket counts are taken once and kept in step as bytes come off the
/// end; recounting the URL for every closing bracket made a run of `)` cost
/// O(n²).
fn trim_trailing_punct(bytes: &[u8], start: usize, mut end: usize) -> usize {
    // `[opens, closes]` of `()`, `[]` and `{}` in `bytes[start..end]`.
    let mut counts = [[0usize; 2]; 3];
    for &byte in &bytes[start..end] {
        if let Some((pair, side)) = bracket(byte) {
            counts[pair][side] += 1;
        }
    }
    while end > start {
        let b = bytes[end - 1];
        match bracket(b) {
            None if matches!(b, b'.' | b',' | b';' | b':' | b'!' | b'?') => end -= 1,
            // `b` itself is one of the closes counted.
            Some((pair, 1)) if counts[pair][1] > counts[pair][0] => {
                counts[pair][1] -= 1;
                end -= 1;
            }
            _ => break,
        }
    }
    end
}

/// The bracket pair `byte` belongs to, and whether it opens (0) or closes (1).
const fn bracket(byte: u8) -> Option<(usize, usize)> {
    match byte {
        b'(' => Some((0, 0)),
        b')' => Some((0, 1)),
        b'[' => Some((1, 0)),
        b']' => Some((1, 1)),
        b'{' => Some((2, 0)),
        b'}' => Some((2, 1)),
        _ => None,
    }
}
