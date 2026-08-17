//! Needle searches for the GFM autolink post-pass.
//!
//! Everything here is a pure scan over one string: the block-level
//! pre-flight that decides whether the post-pass runs at all, and the
//! per-text-node search for the earliest valid candidate.

use std::sync::LazyLock;

use memchr::memmem;

use super::{AutolinkScan, Candidate};

/// Searchers for the multi-byte autolink needles, built once for the process.
///
/// The one-shot `memmem::find` rebuilds its SIMD prefilter on every call, and
/// this scan runs over every text node of every document — on short prose
/// nodes that setup cost dominated the search itself.
static WWW_FINDER: LazyLock<memmem::Finder<'static>> =
    LazyLock::new(|| memmem::Finder::new("www."));

/// The three schemes share one needle: `http://`, `https://` and `ftp://`
/// all end in `://` and differ only in the name in front. One pass for
/// `://` therefore finds every scheme candidate, in position order, in
/// place of three separate whole-node substring scans.
static SCHEME_FINDER: LazyLock<memmem::Finder<'static>> =
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
/// nodes they appear verbatim in `content` too. The one indirection is
/// entities: `&colon;`/`&commat;`/`&#119;` decode into candidate bytes that
/// the raw source spells with `&`, so `&` both keeps the pass on and forces
/// the `www.` search back on.
///
/// The probes are ordered by how cheaply they reject. `@` and `&` fall out
/// of one vectorized byte scan, and asking for the whole `://` rather than
/// a bare `:` is what makes the gate bite: prose is full of `Note:` and
/// `1:1`, and every one of those used to drag a block through the node-tree
/// walk, the text coalescing, and a per-node substring search.
pub(in crate::parser::inline) fn may_contain_autolink(content: &str) -> Option<AutolinkScan> {
    let bytes = content.as_bytes();
    if memchr::memchr2(b'@', b'&', bytes).is_some() {
        // An `&` may expand to anything, so the `www.` search stays on.
        return Some(AutolinkScan { may_have_www: true });
    }
    if WWW_FINDER.find(bytes).is_some() {
        return Some(AutolinkScan { may_have_www: true });
    }
    SCHEME_FINDER.find(bytes).map(|_| AutolinkScan { may_have_www: false })
}

/// Finds the earliest valid autolink candidate in `value`.
pub(super) fn find_candidate(value: &str, scan: AutolinkScan) -> Option<Candidate> {
    crate::profile_span_detail!("parser::gfm_autolink_scan");
    let bytes = value.as_bytes();
    let mut best: Option<Candidate> = None;

    // One `memchr2` decides whether the scheme and email scans can match at
    // all. Together with the block's `www.` verdict it means the ordinary
    // prose node — the overwhelming majority — leaves this function after a
    // single vectorized byte scan, with no substring searcher touched.
    let has_scheme_or_email = memchr::memchr2(b':', b'@', bytes).is_some();
    if !has_scheme_or_email && !scan.may_have_www {
        return None;
    }

    if has_scheme_or_email {
        let mut from = 0;
        while let Some(offset) = SCHEME_FINDER.find(&bytes[from..]) {
            let at = from + offset;
            if let Some(prefix_len) = scheme_prefix_len(bytes, at) {
                // `at + 3 - prefix_len` steps back over the scheme name to
                // the first byte of the candidate.
                if let Some(candidate) = validate_url(value, at + 3 - prefix_len, prefix_len) {
                    best = Some(candidate);
                    break;
                }
            }
            from = at + 3;
        }

        let mut from = 0;
        while let Some(offset) = memchr::memchr(b'@', &bytes[from..]) {
            let at = from + offset;
            if let Some(candidate) = validate_email(value, at) {
                if best.as_ref().is_none_or(|current| candidate.start < current.start) {
                    best = Some(candidate);
                }
                break;
            }
            from = at + 1;
        }
    }

    if scan.may_have_www {
        let mut from = 0;
        while let Some(offset) = WWW_FINDER.find(&bytes[from..]) {
            let at = from + offset;
            if let Some(candidate) = validate_url(value, at, 4) {
                if best.as_ref().is_none_or(|current| candidate.start < current.start) {
                    best = Some(candidate);
                }
                break;
            }
            from = at + 4;
        }
    }

    best
}

/// Length of the whole `scheme://` prefix ending at the `://` that starts
/// at `at`, or `None` when the bytes in front are not a known scheme.
fn scheme_prefix_len(bytes: &[u8], at: usize) -> Option<usize> {
    SCHEMES.iter().find(|name| bytes[..at].ends_with(name.as_bytes())).map(|name| name.len() + 3)
}

/// Start-of-text, whitespace, or `*`, `_`, `~`, `(` may precede an
/// autolink.
fn valid_boundary(value: &str, start: usize) -> bool {
    value[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| ch.is_whitespace() || matches!(ch, '*' | '_' | '~' | '('))
}

fn validate_url(value: &str, start: usize, prefix_len: usize) -> Option<Candidate> {
    if !valid_boundary(value, start) {
        return None;
    }
    let bytes = value.as_bytes();
    // Validate the domain: alphanumerics, `-`, `_`, `.`; at least one
    // dot; no underscore in the last two segments.
    let domain_start = start + prefix_len;
    let mut domain_end = domain_start;
    while domain_end < bytes.len()
        && (bytes[domain_end].is_ascii_alphanumeric()
            || matches!(bytes[domain_end], b'-' | b'_' | b'.'))
    {
        domain_end += 1;
    }
    // Trailing dots belong to the surrounding sentence, not the domain.
    let domain = value[domain_start..domain_end].trim_end_matches('.');
    if domain.split('.').count() < 2
        || domain.rsplit('.').take(2).any(|segment| segment.is_empty() || segment.contains('_'))
    {
        return None;
    }

    // The link runs to whitespace or `<`, then trailing punctuation is
    // trimmed (unbalanced `)` and entity-like `&x;` suffixes included).
    let mut end = domain_end;
    while end < bytes.len() && !bytes[end].is_ascii_whitespace() && bytes[end] != b'<' {
        end += 1;
    }
    let end = trim_trailing_punctuation(value, start, end);
    (end > domain_start).then_some(Candidate {
        start,
        end,
        href_prefix: if prefix_len == 4 { "http://" } else { "" },
    })
}

fn trim_trailing_punctuation(value: &str, start: usize, mut end: usize) -> usize {
    let bytes = value.as_bytes();
    loop {
        if end <= start {
            return end;
        }
        match bytes[end - 1] {
            b'?' | b'!' | b'.' | b',' | b':' | b'*' | b'_' | b'~' | b'\'' | b'"' => end -= 1,
            b')' => {
                let opens = value[start..end].bytes().filter(|&b| b == b'(').count();
                let closes = value[start..end].bytes().filter(|&b| b == b')').count();
                if closes > opens {
                    end -= 1;
                } else {
                    return end;
                }
            }
            b';' => {
                // Strip an entity-like `&name;` suffix entirely.
                let entity_start = value[start..end - 1].rfind('&').map(|found| start + found);
                match entity_start {
                    Some(amp)
                        if value[amp + 1..end - 1]
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric())
                            && amp + 1 < end - 1 =>
                    {
                        end = amp;
                    }
                    _ => end -= 1,
                }
            }
            _ => return end,
        }
    }
}

fn validate_email(value: &str, at: usize) -> Option<Candidate> {
    let bytes = value.as_bytes();
    // Local part: alphanumerics plus `.`, `-`, `_`, `+`.
    let mut start = at;
    while start > 0 {
        let byte = bytes[start - 1];
        if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+') {
            start -= 1;
        } else {
            break;
        }
    }
    if start == at || !valid_boundary(value, start) {
        return None;
    }

    // Domain: alphanumerics plus `.`, `-`, `_`, with at least one dot;
    // trailing dots are trimmed; a trailing `-` or `_` invalidates.
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
    if !value[at + 1..end].contains('.') {
        return None;
    }
    Some(Candidate { start, end, href_prefix: "mailto:" })
}
