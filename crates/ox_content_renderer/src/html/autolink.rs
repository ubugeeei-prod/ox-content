//! Bare-URL autolink scanner.
//!
//! Text nodes can optionally turn recognized URL prefixes into anchors. The scanner
//! first indexes possible leading bytes so most prose is skipped without repeated
//! prefix checks, then validates word boundaries and trims punctuation around matches.

/// Case-insensitive index over the first byte of every registered autolink
/// pattern, used to skip the long runs of text that can't begin a URL.
///
/// The default patterns (`http://`, `https://`) share the single leading
/// letter `h`, so [`Self::next`] collapses to a `memchr2` over `{b'h', b'H'}`
/// — letting the scanner jump straight to candidate offsets instead of
/// testing the word-boundary + prefix at every byte. Up to three distinct
/// leading bytes keep the SIMD `memchr` fast path; beyond that (rare, only
/// with many custom schemes) it falls back to a 256-entry lookup table.
pub(super) struct FirstByteIndex {
    table: [bool; 256],
    needles: [u8; 3],
    needle_len: usize,
    overflow: bool,
}

impl FirstByteIndex {
    /// Builds a compact candidate-start index for the configured patterns.
    ///
    /// Each pattern contributes both lowercase and uppercase variants of its
    /// first byte so later prefix checks can stay case-insensitive without
    /// lowercasing the whole text node. One to three distinct bytes are stored
    /// in `needles` for `memchr`, `memchr2`, or `memchr3`; larger custom
    /// pattern sets keep correctness by falling back to the table scan.
    pub(super) fn from_patterns(patterns: &[String]) -> Self {
        let mut table = [false; 256];
        let mut needles = [0u8; 3];
        let mut needle_len = 0usize;
        let mut overflow = false;
        for pat in patterns {
            let Some(&first) = pat.as_bytes().first() else {
                continue;
            };
            for cand in [first.to_ascii_lowercase(), first.to_ascii_uppercase()] {
                if table[cand as usize] {
                    continue;
                }
                table[cand as usize] = true;
                if needle_len < needles.len() {
                    needles[needle_len] = cand;
                }
                // Count past the array so >3 distinct bytes trips `overflow`.
                needle_len += 1;
            }
        }
        if needle_len > needles.len() {
            overflow = true;
        }
        Self { table, needles, needle_len, overflow }
    }

    /// Byte offset of the next possible pattern start within `hay`, or `None`.
    #[inline]
    fn next(&self, hay: &[u8]) -> Option<usize> {
        if self.overflow {
            // Rare custom-pattern case. The table is still a single indexed
            // load per byte, but the common one/two/three-needle cases use
            // memchr's specialized search loops instead.
            return hay.iter().position(|&b| self.table[b as usize]);
        }
        match self.needle_len {
            1 => memchr::memchr(self.needles[0], hay),
            2 => memchr::memchr2(self.needles[0], self.needles[1], hay),
            3 => memchr::memchr3(self.needles[0], self.needles[1], self.needles[2], hay),
            _ => None,
        }
    }
}

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
    patterns: &[String],
    index: &FirstByteIndex,
) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut base = from;
    while base < bytes.len() {
        let rel = index.next(&bytes[base..])?;
        let i = base + rel;
        // Word boundary: the previous byte must not be ASCII alphanumeric.
        let is_boundary = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if is_boundary {
            for pat in patterns {
                let pat_bytes = pat.as_bytes();
                if pat_bytes.is_empty() {
                    continue;
                }
                if i + pat_bytes.len() <= bytes.len()
                    && bytes[i..i + pat_bytes.len()].eq_ignore_ascii_case(pat_bytes)
                {
                    let url_start = i;
                    let mut url_end = i + pat_bytes.len();
                    while url_end < bytes.len() && is_url_byte(bytes[url_end]) {
                        url_end += 1;
                    }
                    // Require at least one byte beyond the scheme/prefix
                    // so `"http://"` on its own isn't auto-linked.
                    if url_end == i + pat_bytes.len() {
                        continue;
                    }
                    url_end = trim_trailing_punct(bytes, url_start, url_end);
                    return Some((url_start, url_end));
                }
            }
        }
        base = i + 1;
    }
    None
}

#[inline]
fn is_url_byte(byte: u8) -> bool {
    !matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'<' | b'>' | b'"' | b'\'' | b'`')
}

fn trim_trailing_punct(bytes: &[u8], start: usize, mut end: usize) -> usize {
    while end > start {
        let b = bytes[end - 1];
        match b {
            b'.' | b',' | b';' | b':' | b'!' | b'?' => end -= 1,
            b')' | b']' | b'}' => {
                let (open, close) = match b {
                    b')' => (b'(', b')'),
                    b']' => (b'[', b']'),
                    _ => (b'{', b'}'),
                };
                // Strip the closing bracket only when it has no unmatched
                // partner inside the URL — a single pass over the slice is
                // simpler than two `filter().count()` walks and avoids the
                // `naive_bytecount` clippy lint.
                let mut opens = 0usize;
                let mut closes = 0usize;
                for &x in &bytes[start..end - 1] {
                    if x == open {
                        opens += 1;
                    } else if x == close {
                        closes += 1;
                    }
                }
                if closes >= opens {
                    end -= 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }
    end
}
