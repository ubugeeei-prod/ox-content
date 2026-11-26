//! The GFM autolink candidate search as it stood before the scan started
//! keeping what it found from one link to the next: a fresh, windowed
//! search of every remaining suffix, each candidate validated from scratch.
//!
//! Kept as it was written — only the block-level pre-flight and the
//! profiling hooks are gone — so `gfm_autolink_scaling.rs` can hold the
//! rewritten search to it, input by input.

use memchr::memmem;

pub struct Candidate {
    pub start: usize,
    pub end: usize,
    pub href_prefix: &'static str,
}

const SCHEMES: [&str; 3] = ["https", "http", "ftp"];
const LONGEST_SCHEME: usize = 5;
const FIRST_WINDOW: usize = 512;

/// Finds the earliest valid autolink candidate in `value`.
pub fn find_candidate(value: &str) -> Option<Candidate> {
    let bytes = value.as_bytes();
    let mut window = FIRST_WINDOW.min(bytes.len());

    loop {
        let found = search_within(value, window);

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

fn search_within(value: &str, window: usize) -> Option<Candidate> {
    let bytes = value.as_bytes();
    let mut best: Option<Candidate> = None;

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

    let limit = search_limit(window, best.as_ref(), 0, 4);
    let mut from = 0;
    while from < limit {
        let Some(offset) = memmem::find(&bytes[from..limit], b"www.") else {
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

    let limit = search_limit(window, best.as_ref(), LONGEST_SCHEME, 3);
    let mut from = 0;
    while from < limit {
        let Some(offset) = memmem::find(&bytes[from..limit], b"://") else {
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

fn search_limit(len: usize, best: Option<&Candidate>, lookbehind: usize, needle: usize) -> usize {
    best.map_or(len, |candidate| (candidate.start + lookbehind + needle).min(len))
}

fn scheme_prefix_len(bytes: &[u8], at: usize) -> Option<usize> {
    SCHEMES.iter().find(|name| bytes[..at].ends_with(name.as_bytes())).map(|name| name.len() + 3)
}

fn valid_boundary(value: &str, start: usize) -> bool {
    value[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| ch.is_whitespace() || matches!(ch, '*' | '_' | '~' | '(' | '\'' | '"'))
}

fn validate_url(value: &str, start: usize, prefix_len: usize) -> Option<Candidate> {
    if !valid_boundary(value, start) {
        return None;
    }
    let bytes = value.as_bytes();
    let domain_start = start + prefix_len;
    let mut domain_end = domain_start;
    while domain_end < bytes.len()
        && (bytes[domain_end].is_ascii_alphanumeric()
            || matches!(bytes[domain_end], b'-' | b'_' | b'.'))
    {
        domain_end += 1;
    }
    let domain = value[domain_start..domain_end].trim_end_matches('.');
    if domain.split('.').count() < 2
        || domain.rsplit('.').take(2).any(|segment| segment.is_empty() || segment.contains('_'))
    {
        return None;
    }

    let end = trim_trailing_punctuation(value, start, scan_url_end(value, domain_end));
    (end > domain_start).then_some(Candidate {
        start,
        end,
        href_prefix: if prefix_len == 4 { "http://" } else { "" },
    })
}

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

const fn ends_url(ch: char) -> bool {
    matches!(
        ch,
        '\u{3000}'..='\u{303F}'
        | '\u{FF01}'..='\u{FF0F}'
        | '\u{FF1A}'..='\u{FF20}'
        | '\u{FF3B}'..='\u{FF40}'
        | '\u{FF5B}'..='\u{FF65}'
    )
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
