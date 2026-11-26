//! Needle searches for the GFM autolink post-pass.
//!
//! Everything here is a pure scan over one string: the block-level
//! pre-flight that decides whether the post-pass runs at all, and the
//! pieces a candidate is validated with once its needle has been found.

use std::sync::LazyLock;

use memchr::{memchr, memmem};

use super::AutolinkScan;
use super::trim::trim_trailing_punctuation;

/// Searchers for the multi-byte autolink needles, built once for the process.
///
/// The one-shot `memmem::find` rebuilds its SIMD prefilter on every call, and
/// this scan runs over every text node of every document — on short prose
/// nodes that setup cost dominated the search itself.
pub(super) static WWW_FINDER: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("www."));

/// The three schemes share one needle: `http://`, `https://` and `ftp://`
/// all end in `://` and differ only in the name in front. One pass for
/// `://` therefore finds every scheme candidate, in position order, in
/// place of three separate whole-node substring scans.
pub(super) static SCHEME_FINDER: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("://"));

/// Scheme names accepted in front of `://`, longest first so `https://`
/// is not mistaken for a `ttp`-suffixed shorter name.
const SCHEMES: [&str; 3] = ["https", "http", "ftp"];

/// Cheap pre-flight over a block's raw inline content: can the autolink
/// pass possibly rewrite anything here, and if so does it need the `www.`
/// search at all?
///
/// Every candidate needs `://` (scheme), `@` (email), or `www.` — and none
/// of those bytes are inline-special, so if they appear in the parsed text
/// nodes they appear verbatim in `content` too. `&` used to keep the pass
/// on for entity-decoded needles, but that made every Rust-doc paragraph
/// containing `` `&str` `` pay the coalesce + rewrite walk. GFM spec
/// examples always include a verbatim `www.` / `://` / `@` alongside `&`
/// in a URL.
///
/// `://` inside a markdown destination (`](https://…)`) cannot become a
/// GFM autolink — the inline parser already turned it into a Link — so it
/// does not keep the pass on.
pub(in crate::parser::inline) fn may_contain_autolink(content: &str) -> Option<AutolinkScan> {
    let bytes = content.as_bytes();
    if memchr::memchr(b'@', bytes).is_some() {
        return Some(AutolinkScan { may_have_www: true });
    }
    if WWW_FINDER.find(bytes).is_some() {
        return Some(AutolinkScan { may_have_www: true });
    }
    has_bare_scheme(bytes).then_some(AutolinkScan { may_have_www: false })
}

/// True when `://` appears as a bare URL, not only as `](http://…)` /
/// `](https://…)` / `](ftp://…)`.
fn has_bare_scheme(bytes: &[u8]) -> bool {
    let mut from = 0;
    while let Some(offset) = SCHEME_FINDER.find(&bytes[from..]) {
        let at = from + offset;
        if !scheme_is_markdown_destination(bytes, at) {
            return true;
        }
        from = at + 3;
    }
    false
}

fn scheme_is_markdown_destination(bytes: &[u8], colon_slash_slash: usize) -> bool {
    for name in SCHEMES {
        let Some(start) = colon_slash_slash.checked_sub(name.len()) else {
            continue;
        };
        if start >= 2
            && bytes[start - 2] == b']'
            && bytes[start - 1] == b'('
            && bytes[start..colon_slash_slash].eq_ignore_ascii_case(name.as_bytes())
        {
            return true;
        }
    }
    false
}

/// Length of the whole `scheme://` prefix ending at the `://` that starts
/// at `at`, or `None` when the bytes in front are not a known scheme.
pub(super) fn scheme_prefix_len(bytes: &[u8], at: usize) -> Option<usize> {
    SCHEMES.iter().find(|name| bytes[..at].ends_with(name.as_bytes())).map(|name| name.len() + 3)
}

/// Length of the `scheme://` prefix that `bytes[at..]` begins with, or
/// `None` when it does not begin with one.
pub(super) fn scheme_prefix_at(bytes: &[u8], at: usize) -> Option<usize> {
    let rest = bytes.get(at..)?;
    SCHEMES
        .iter()
        .find(|name| {
            rest.strip_prefix(name.as_bytes()).is_some_and(|after| after.starts_with(b"://"))
        })
        .map(|name| name.len() + 3)
}

/// Start-of-text, whitespace, or common delimiter punctuation may precede an
/// autolink.
pub(super) fn valid_boundary(value: &str, start: usize) -> bool {
    value[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| ch.is_whitespace() || matches!(ch, '*' | '_' | '~' | '(' | '\'' | '"'))
}

/// Where the bare URL that starts at `start`, with its host at
/// `domain_start`, ends.
///
/// The link runs to whitespace, `<`, or CJK sentence punctuation, then
/// trailing punctuation is trimmed (unbalanced `)` and entity-like `&x;`
/// suffixes included). A valid host ends in a byte none of those rules can
/// remove and holds no `&`, so the trimmed end never falls back into it:
/// whether a candidate is a link is settled by its boundary and its host
/// alone, and the end is only worth computing for the candidate that wins.
pub(super) fn url_end(value: &str, start: usize, domain_start: usize) -> usize {
    trim_trailing_punctuation(value, start, scan_url_end(value, domain_start))
}

/// Extends a URL from `from` to the offset where it stops.
///
/// Non-ASCII characters normally belong to the URL — an IRI carries them
/// verbatim (`/wiki/日本語`) — with CJK sentence punctuation the exception.
fn scan_url_end(value: &str, from: usize) -> usize {
    let bytes = value.as_bytes();
    let mut end = from;
    while end < bytes.len() {
        let byte = bytes[end];
        if byte.is_ascii() {
            if byte.is_ascii_whitespace() || byte == b'<' {
                break;
            }
            end += 1;
            continue;
        }
        let Some(ch) = value.get(end..).and_then(|rest| rest.chars().next()) else {
            break;
        };
        if ends_url(ch) {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

/// Punctuation that ends a bare URL the way ASCII whitespace does.
///
/// The GFM autolink extension defines trailing-punctuation trimming for
/// ASCII only, and CJK prose puts no space between a URL and the `。` that
/// closes the sentence — so without this the rest of the sentence is
/// swallowed into the link.
///
/// Mirrored by the bare-URL scanner in `ox_content_renderer`.
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

/// First byte of the email local part that ends at the `@` at `at`:
/// alphanumerics plus `.`, `-`, `_`, `+`, read no further back than `floor`.
///
/// Returns `at` itself when no local-part byte precedes the `@`.
pub(super) fn local_part_start(bytes: &[u8], floor: usize, at: usize) -> usize {
    let mut start = at;
    while start > floor
        && (bytes[start - 1].is_ascii_alphanumeric()
            || matches!(bytes[start - 1], b'.' | b'-' | b'_' | b'+'))
    {
        start -= 1;
    }
    start
}

/// End of the email domain that follows the `@` at `at`, or `None` when
/// what follows is not one.
///
/// Domain: alphanumerics plus `.`, `-`, `_`, with at least one dot;
/// trailing dots are trimmed; a trailing `-` or `_` invalidates.
pub(super) fn email_domain_end(bytes: &[u8], at: usize) -> Option<usize> {
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
    memchr(b'.', &bytes[at + 1..end]).map(|_| end)
}
