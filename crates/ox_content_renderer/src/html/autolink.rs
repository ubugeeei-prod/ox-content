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
    /// Per-first-byte second-byte filter: `second[b]` holds the lowercased
    /// byte every pattern starting with `b` continues with, or `ANY_SECOND`
    /// when patterns disagree (or the pattern is a single byte). Prose hits
    /// on a frequent first byte — `h` matches "the", "here", … — are
    /// rejected with one comparison instead of full prefix checks.
    second: [u8; 256],
    /// A byte every pattern contains, when one exists. Since a match always
    /// begins with a full pattern verbatim (prefix compare is
    /// case-insensitive, so only caseless bytes qualify), a text node
    /// without this byte cannot contain a match at all — one `memchr` proves
    /// it. The default patterns (`http://`, `https://`) gate on `:`, which
    /// is far rarer in prose than their first byte `h`; most text nodes
    /// skip the candidate walk entirely.
    gate: Option<u8>,
    /// Bytes that must follow `gate` for a match to be possible, when every
    /// pattern agrees on them. The gate byte alone is a weak filter: `:` is
    /// everywhere in prose (`Note:`, `1:1`, `Listing 3-2:`) and each hit used
    /// to drag the node through the full candidate walk. The default
    /// patterns both continue `//` after their `:`, so requiring `://`
    /// rejects all of those with a two-byte compare.
    gate_tail: [u8; 2],
    gate_tail_len: usize,
}

/// Sentinel in `FirstByteIndex::second`: no single second byte filters
/// candidates starting with this first byte.
const ANY_SECOND: u8 = 0xFF;

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
        let mut second = [0u8; 256];
        for pat in patterns {
            let Some(&first) = pat.as_bytes().first() else {
                continue;
            };
            // 0 = unset, ANY_SECOND = conflicting/absent, else the required
            // lowercased second byte. The default patterns (`http://`,
            // `https://`) agree on `t`, so `h` candidates filter on it.
            let pat_second = pat.as_bytes().get(1).map_or(ANY_SECOND, u8::to_ascii_lowercase);
            for cand in [first.to_ascii_lowercase(), first.to_ascii_uppercase()] {
                let entry = &mut second[cand as usize];
                *entry = match *entry {
                    0 => pat_second,
                    prev if prev == pat_second => prev,
                    _ => ANY_SECOND,
                };
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

        // Pick the gate byte: preferred candidates first (roughly ordered by
        // rarity in prose), then any other caseless byte the first pattern
        // holds. Alphabetic bytes never qualify — the prefix compare is
        // case-insensitive, so a letter's presence proves nothing about the
        // other case.
        let gate_candidates = patterns.first().map_or(&[][..], |pat| pat.as_bytes());
        let qualifies = |byte: u8| {
            !byte.is_ascii_alphabetic() && patterns.iter().all(|pat| pat.as_bytes().contains(&byte))
        };
        let gate = b":/@."
            .iter()
            .copied()
            .find(|&byte| qualifies(byte))
            .or_else(|| gate_candidates.iter().copied().find(|&byte| qualifies(byte)));

        // Extend the gate with the bytes that follow it in every pattern.
        // Only caseless bytes qualify, for the same reason the gate byte
        // itself must be caseless: the prefix compare is case-insensitive,
        // so a letter's presence proves nothing about the other case.
        let mut gate_tail = [0u8; 2];
        let mut gate_tail_len = 0usize;
        if let Some(gate_byte) = gate {
            if let Some(first) = patterns.first() {
                let first = first.as_bytes();
                if let Some(at) = memchr::memchr(gate_byte, first) {
                    let mut len = 0usize;
                    while len < gate_tail.len()
                        && at + 1 + len < first.len()
                        && !first[at + 1 + len].is_ascii_alphabetic()
                    {
                        len += 1;
                    }
                    // Shrink until every pattern holds the whole needle; a
                    // pattern set that disagrees falls back to the bare byte.
                    while len > 0 {
                        let needle = &first[at..=at + len];
                        if patterns
                            .iter()
                            .all(|pat| memchr::memmem::find(pat.as_bytes(), needle).is_some())
                        {
                            gate_tail[..len].copy_from_slice(&first[at + 1..=at + len]);
                            gate_tail_len = len;
                            break;
                        }
                        len -= 1;
                    }
                }
            }
        }

        Self { table, needles, needle_len, overflow, second, gate, gate_tail, gate_tail_len }
    }

    /// Whether `haystack` can hold a pattern match at all.
    ///
    /// One `memchr` for the gate byte, then a compare against the bytes that
    /// must follow it. With no gate byte configured this is vacuously true,
    /// and with no tail it degrades to exactly the old single-byte probe.
    pub(super) fn may_match(&self, haystack: &[u8]) -> bool {
        let Some(gate) = self.gate else {
            return true;
        };
        let tail = &self.gate_tail[..self.gate_tail_len];
        let mut from = 0;
        while let Some(off) = memchr::memchr(gate, &haystack[from..]) {
            let at = from + off;
            if haystack[at + 1..].starts_with(tail) {
                return true;
            }
            from = at + 1;
        }
        false
    }

    /// True when the byte after a first-byte hit rules the candidate out
    /// without running any full prefix comparison.
    #[inline]
    fn rejects_second(&self, first: u8, after: Option<&u8>) -> bool {
        let expected = self.second[first as usize];
        if expected == ANY_SECOND {
            return false;
        }
        match after {
            Some(&byte) => byte.to_ascii_lowercase() != expected,
            // Pattern needs a second byte but the text ends here.
            None => true,
        }
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
