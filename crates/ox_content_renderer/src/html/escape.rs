//! Allocation-aware escaping helpers for text and URL attributes.
//!
//! These routines are on the renderer's hottest path. They use static byte lookup
//! tables so long runs of safe text can be copied in chunks while only the bytes that
//! affect HTML parsing are replaced.

// Per-byte HTML-escape mapping. `ESCAPE_FLAG[b] == 1` and
// `ESCAPE_TABLE[b]` is the replacement string when `b` must be escaped;
// otherwise the flag is 0 and the entry is `""`. Splitting flag/string
// lets the inner scan use a plain integer OR over 8-byte chunks (which
// LLVM vectorizes) instead of branching on a string comparison.
static ESCAPE_TABLE: [&str; 256] = {
    let mut table: [&str; 256] = [""; 256];
    table[b'&' as usize] = "&amp;";
    table[b'<' as usize] = "&lt;";
    table[b'>' as usize] = "&gt;";
    table[b'"' as usize] = "&quot;";
    table[b'\'' as usize] = "&#39;";
    table
};

static ESCAPE_FLAG: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'&' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'>' as usize] = 1;
    t[b'"' as usize] = 1;
    t[b'\'' as usize] = 1;
    t
};

static URL_ESCAPE_TABLE: [&str; 256] = {
    let mut table: [&str; 256] = [""; 256];
    table[b'&' as usize] = "&amp;";
    table[b'<' as usize] = "%3C";
    table[b'>' as usize] = "%3E";
    table[b'"' as usize] = "%22";
    table[b' ' as usize] = "%20";
    table
};

static URL_ESCAPE_FLAG: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'&' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'>' as usize] = 1;
    t[b'"' as usize] = 1;
    t[b' ' as usize] = 1;
    t
};

#[inline]
pub(super) fn write_escaped_into(out: &mut String, s: &str) {
    let bytes = s.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;
    out.reserve(s.len());

    // 8-byte chunk fast-skip — the OR over 8 lookups vectorizes well and
    // dominates the inner loop on long no-escape runs (most plain text).
    while i + 8 <= bytes.len() {
        let chunk = &bytes[i..i + 8];
        let mask = ESCAPE_FLAG[chunk[0] as usize]
            | ESCAPE_FLAG[chunk[1] as usize]
            | ESCAPE_FLAG[chunk[2] as usize]
            | ESCAPE_FLAG[chunk[3] as usize]
            | ESCAPE_FLAG[chunk[4] as usize]
            | ESCAPE_FLAG[chunk[5] as usize]
            | ESCAPE_FLAG[chunk[6] as usize]
            | ESCAPE_FLAG[chunk[7] as usize];
        if mask == 0 {
            i += 8;
            continue;
        }
        break;
    }

    while i < bytes.len() {
        let b = bytes[i];
        if ESCAPE_FLAG[b as usize] != 0 {
            if start < i {
                out.push_str(&s[start..i]);
            }
            out.push_str(ESCAPE_TABLE[b as usize]);
            i += 1;
            start = i;
            while i + 8 <= bytes.len() {
                let chunk = &bytes[i..i + 8];
                let mask = ESCAPE_FLAG[chunk[0] as usize]
                    | ESCAPE_FLAG[chunk[1] as usize]
                    | ESCAPE_FLAG[chunk[2] as usize]
                    | ESCAPE_FLAG[chunk[3] as usize]
                    | ESCAPE_FLAG[chunk[4] as usize]
                    | ESCAPE_FLAG[chunk[5] as usize]
                    | ESCAPE_FLAG[chunk[6] as usize]
                    | ESCAPE_FLAG[chunk[7] as usize];
                if mask == 0 {
                    i += 8;
                    continue;
                }
                break;
            }
            continue;
        }
        i += 1;
    }

    if start < bytes.len() {
        out.push_str(&s[start..]);
    }
}

pub(super) fn write_url_escaped_into(out: &mut String, s: &str) {
    let bytes = s.as_bytes();
    let mut start = 0usize;
    let mut i = 0usize;

    while i + 8 <= bytes.len() {
        let chunk = &bytes[i..i + 8];
        let mask = URL_ESCAPE_FLAG[chunk[0] as usize]
            | URL_ESCAPE_FLAG[chunk[1] as usize]
            | URL_ESCAPE_FLAG[chunk[2] as usize]
            | URL_ESCAPE_FLAG[chunk[3] as usize]
            | URL_ESCAPE_FLAG[chunk[4] as usize]
            | URL_ESCAPE_FLAG[chunk[5] as usize]
            | URL_ESCAPE_FLAG[chunk[6] as usize]
            | URL_ESCAPE_FLAG[chunk[7] as usize];
        if mask == 0 {
            i += 8;
            continue;
        }
        break;
    }

    while i < bytes.len() {
        let b = bytes[i];
        if URL_ESCAPE_FLAG[b as usize] != 0 {
            if start < i {
                out.push_str(&s[start..i]);
            }
            out.push_str(URL_ESCAPE_TABLE[b as usize]);
            i += 1;
            start = i;
            while i + 8 <= bytes.len() {
                let chunk = &bytes[i..i + 8];
                let mask = URL_ESCAPE_FLAG[chunk[0] as usize]
                    | URL_ESCAPE_FLAG[chunk[1] as usize]
                    | URL_ESCAPE_FLAG[chunk[2] as usize]
                    | URL_ESCAPE_FLAG[chunk[3] as usize]
                    | URL_ESCAPE_FLAG[chunk[4] as usize]
                    | URL_ESCAPE_FLAG[chunk[5] as usize]
                    | URL_ESCAPE_FLAG[chunk[6] as usize]
                    | URL_ESCAPE_FLAG[chunk[7] as usize];
                if mask == 0 {
                    i += 8;
                    continue;
                }
                break;
            }
            continue;
        }
        i += 1;
    }

    if start < bytes.len() {
        out.push_str(&s[start..]);
    }
}
