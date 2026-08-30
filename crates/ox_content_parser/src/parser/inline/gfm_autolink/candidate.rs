//! The per-text-node search for the earliest autolink candidate.
//!
//! Split out of `scan.rs` so that file stays the needle definitions and the
//! block-level pre-flight: this one is the search strategy and nothing else.

use super::AutolinkScan;
use super::Candidate;
use super::scan::{
    LONGEST_SCHEME, SCHEME_FINDER, WWW_FINDER, scheme_prefix_len, validate_email, validate_url,
};

/// Where the doubling search starts. A node that holds a candidate almost
/// always holds one within a few hundred bytes: it was split at the previous
/// one.
const FIRST_WINDOW: usize = 512;

/// Finds the earliest valid autolink candidate in `value`.
///
/// The search grows a window instead of reading the whole node. Two of the
/// three finders normally match nothing, and an unmatched finder walks to the
/// end — while `apply_gfm_autolinks` calls this once per remaining suffix of
/// a paragraph. Unbounded, that is one full pass per link: 1 MiB of
/// `a@b.com` lines took 3.6 s and grew x15 for every x4 of input. Doubling
/// costs O(distance to the next candidate), and a node with no candidate at
/// all is never split, so its one full pass is paid once.
pub(super) fn find_candidate(value: &str, scan: AutolinkScan) -> Option<Candidate> {
    crate::profile_span_detail!("parser::gfm_autolink_scan");
    let bytes = value.as_bytes();
    let mut window = FIRST_WINDOW.min(bytes.len());

    loop {
        let found = search_within(value, window, scan);

        // A window edge can cut a needle in half, hiding a candidate that
        // starts before the one just found. Accept only when no earlier
        // candidate could have had its needle beyond the edge.
        if let Some(candidate) = found.as_ref()
            && (window == bytes.len() || earlier_needle_bound(bytes, candidate.start) <= window)
        {
            return found;
        }
        if window == bytes.len() {
            return found;
        }
        window = window.saturating_mul(2).min(bytes.len());
    }
}

/// The furthest a needle can sit for a candidate starting before `start`.
///
/// A scheme name is at most `https`; `www.` is the candidate's own first
/// bytes; an email's `@` ends an unbroken run of local-part bytes, so the run
/// out of `start` is as far as it can reach.
fn earlier_needle_bound(bytes: &[u8], start: usize) -> usize {
    let mut local_part_end = start;
    while local_part_end < bytes.len() && is_local_part_byte(bytes[local_part_end]) {
        local_part_end += 1;
    }
    (start + LONGEST_SCHEME + 3).max(start + 4).max(local_part_end + 1)
}

fn is_local_part_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+')
}

/// The earliest candidate whose needle lies fully inside `bytes[..window]`.
fn search_within(value: &str, window: usize, scan: AutolinkScan) -> Option<Candidate> {
    let bytes = value.as_bytes();
    let mut best: Option<Candidate> = None;

    // Each finder is additionally capped by what the ones before it found: a
    // candidate starting at or after the best cannot win.
    let mut from = 0;
    let limit = window;
    while from < limit {
        let Some(offset) = memchr::memchr(b'@', &bytes[from..limit]) else {
            break;
        };
        let at = from + offset;
        if let Some(candidate) = validate_email(value, at) {
            best = Some(candidate);
            break;
        }
        from = at + 1;
    }

    if scan.may_have_www {
        // A `www.` candidate starts at the needle itself.
        let limit = search_limit(window, best.as_ref(), 0, 4);
        let mut from = 0;
        while from < limit {
            let Some(offset) = WWW_FINDER.find(&bytes[from..limit]) else {
                break;
            };
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

    // The candidate starts at the scheme name, which is at most `https` —
    // five bytes — in front of the `://`.
    let limit = search_limit(window, best.as_ref(), LONGEST_SCHEME, 3);
    let mut from = 0;
    while from < limit {
        let Some(offset) = SCHEME_FINDER.find(&bytes[from..limit]) else {
            break;
        };
        let at = from + offset;
        if let Some(prefix_len) = scheme_prefix_len(bytes, at) {
            // `at + 3 - prefix_len` steps back over the scheme name to the
            // first byte of the candidate.
            if let Some(candidate) = validate_url(value, at + 3 - prefix_len, prefix_len) {
                if best.as_ref().is_none_or(|current| candidate.start < current.start) {
                    best = Some(candidate);
                }
                break;
            }
        }
        from = at + 3;
    }

    best
}

/// How far a finder still has to look to beat `best`.
///
/// A candidate starting at or after the best one cannot win, and a needle
/// sits at most `lookbehind` bytes after the start of its candidate, so
/// nothing past that point is worth scanning. Without a best yet, the whole
/// node is in play.
fn search_limit(len: usize, best: Option<&Candidate>, lookbehind: usize, needle: usize) -> usize {
    best.map_or(len, |candidate| (candidate.start + lookbehind + needle).min(len))
}
