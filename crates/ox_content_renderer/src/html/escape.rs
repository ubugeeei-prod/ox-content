//! Allocation-aware escaping helpers for text and URL attributes.
//!
//! These routines are on the renderer's hottest path — escaping accounts for
//! roughly a seventh of a full parse+render pipeline on the bundled corpora.
//! They scan for the handful of bytes that must be replaced using SWAR
//! (SIMD-within-a-register) word tests, so long runs of safe text are proven
//! clean eight bytes at a time and then copied with one `push_str`.

// Replacement strings for the bytes that must be escaped. `ESCAPE_FLAG[b]`
// answers "does byte `b` need replacing" for the scalar tail; the hot loop
// uses the SWAR word tests below instead of per-byte lookups.
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

const ONES: u64 = 0x0101_0101_0101_0101;
const HIGH: u64 = 0x8080_8080_8080_8080;

const fn splat(byte: u8) -> u64 {
    (byte as u64) * ONES
}

/// Sets `0x80` in every byte lane of `word` that is zero.
///
/// The classic bit-twiddling zero test. It can also light up a lane holding
/// `0x01` when a lower lane borrowed into it, which is why every caller below
/// only ever consumes the *lowest* set bit: a spurious lane can only sit
/// above a genuine zero lane that produced the borrow, so the lowest set bit
/// is always a true match.
#[inline]
const fn has_zero(word: u64) -> u64 {
    word.wrapping_sub(ONES) & !word & HIGH
}

/// Nonzero iff `word` holds any of `&`, `<`, `>`, `"`, `'`.
///
/// Five needles fold into three zero tests: `&`(0x26)/`'`(0x27) differ only in
/// bit 0 and `<`(0x3C)/`>`(0x3E) only in bit 1, so forcing that bit on maps
/// each pair onto a single value. `y | 0x01 == 0x27` holds for exactly
/// {0x26, 0x27} and `y | 0x02 == 0x3E` for exactly {0x3C, 0x3E}, so the fold
/// admits no bytes beyond the five.
#[inline]
const fn escape_mask(word: u64) -> u64 {
    has_zero((word | splat(0x01)) ^ splat(b'\''))
        | has_zero((word | splat(0x02)) ^ splat(b'>'))
        | has_zero(word ^ splat(b'"'))
}

/// Nonzero iff `word` holds any of `&`, `<`, `>`, `"`, ` `.
///
/// Same fold: `<`/`>` share bit 1, and so do ` `(0x20)/`"`(0x22), leaving `&`
/// on its own.
#[inline]
const fn url_escape_mask(word: u64) -> u64 {
    has_zero((word | splat(0x02)) ^ splat(b'>'))
        | has_zero((word | splat(0x02)) ^ splat(b'"'))
        | has_zero(word ^ splat(b'&'))
}

/// Byte offset of the lowest flagged lane in a nonzero mask.
#[inline]
const fn first_flagged_lane(mask: u64) -> usize {
    (mask.trailing_zeros() / 8) as usize
}

/// Offset of the first byte at or after `from` that needs replacing, or
/// `bytes.len()` when the rest is clean.
///
/// Whole words are cleared by `mask_of`. What is left over when the length
/// is not a multiple of eight used to be walked a byte at a time, and that
/// walk is most of the work: the strings reaching these escapers have a
/// median length of 15 bytes on the bundled corpora, so a *typical* call
/// tested one word and then seven bytes one by one, ending on a loop the
/// branch predictor cannot learn. Re-reading the final eight bytes and
/// discarding the lanes the loop already cleared replaces that walk with
/// one more word test.
///
/// Masking off the low lanes gives up `has_zero`'s "lowest set bit is
/// always a true match" guarantee — a masked-off lane that *is* a match can
/// borrow into the lane above it — so surviving lanes are confirmed against
/// `flags` before being reported. That check is off the hot path: it only
/// runs on the sub-word tail of a string that still holds a match.
#[inline]
fn first_flagged(
    bytes: &[u8],
    from: usize,
    mask_of: impl Fn(u64) -> u64,
    flags: &[u8; 256],
) -> usize {
    let len = bytes.len();
    let mut i = from;

    while i + 8 <= len {
        // `unwrap` is unreachable: the slice is exactly 8 bytes wide.
        let word = u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        let mask = mask_of(word);
        if mask != 0 {
            return i + first_flagged_lane(mask);
        }
        i += 8;
    }

    if i < len && len >= 8 {
        // `base < i` here: the loop above only stops with bytes left when
        // fewer than eight remain, so the re-read always overlaps.
        let base = len - 8;
        let word = u64::from_le_bytes(bytes[base..len].try_into().unwrap());
        let mut mask = mask_of(word) & (u64::MAX << ((i - base) * 8));
        while mask != 0 {
            let lane = base + first_flagged_lane(mask);
            if flags[bytes[lane] as usize] != 0 {
                return lane;
            }
            mask &= mask - 1;
        }
        return len;
    }

    while i < len && flags[bytes[i] as usize] == 0 {
        i += 1;
    }
    i
}

/// Shared scan/copy loop for both escapers.
///
/// `mask_of` proves a whole 8-byte word clean in one test; when a word is
/// dirty its mask names the exact byte, so no byte is ever examined twice.
/// `flags`/`table` drive the sub-word tail and the replacement itself.
#[inline]
fn escape_into(
    out: &mut String,
    s: &str,
    mask_of: impl Fn(u64) -> u64,
    flags: &[u8; 256],
    table: &[&'static str; 256],
) {
    // The invariant: bytes in `s[start..i]` have not been copied yet, and
    // everything before `start` has already been emitted in escaped form.
    let bytes = s.as_bytes();
    let mut start = 0usize;

    loop {
        let i = first_flagged(bytes, start, &mask_of, flags);
        if i >= bytes.len() {
            break;
        }
        if start < i {
            out.push_str(&s[start..i]);
        }
        out.push_str(table[bytes[i] as usize]);
        start = i + 1;
    }

    if start < bytes.len() {
        out.push_str(&s[start..]);
    }
}

#[inline]
pub(super) fn write_escaped_into(out: &mut String, s: &str) {
    crate::profile_span_detail!("renderer::escape_text");
    // `reserve(s.len())` covers the no-escape case exactly and reduces growth
    // even when replacements make the final output longer.
    out.reserve(s.len());
    escape_into(out, s, escape_mask, &ESCAPE_FLAG, &ESCAPE_TABLE);
}

pub(super) fn write_url_escaped_into(out: &mut String, s: &str) {
    crate::profile_span_detail!("renderer::escape_url");
    // Same scanner as `write_escaped_into`, but with URL attribute semantics.
    // Ampersand stays HTML-escaped because the result is written inside an
    // HTML attribute, while spaces and tag delimiters are percent encoded to
    // keep the URL value itself stable.
    escape_into(out, s, url_escape_mask, &URL_ESCAPE_FLAG, &URL_ESCAPE_TABLE);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Straightforward byte-at-a-time escaper the SWAR scanner must match.
    ///
    /// Every escaped byte is ASCII, so copying the rest through verbatim at
    /// the byte level keeps multi-byte sequences intact.
    fn reference(s: &str, flags: &[u8; 256], table: &[&'static str; 256]) -> String {
        let mut out: Vec<u8> = Vec::new();
        for byte in s.bytes() {
            if flags[byte as usize] != 0 {
                out.extend_from_slice(table[byte as usize].as_bytes());
            } else {
                out.push(byte);
            }
        }
        String::from_utf8(out).expect("escaping only rewrites ASCII bytes")
    }

    fn check(s: &str) {
        let mut text = String::new();
        write_escaped_into(&mut text, s);
        assert_eq!(text, reference(s, &ESCAPE_FLAG, &ESCAPE_TABLE), "text escape: {s:?}");

        let mut url = String::new();
        write_url_escaped_into(&mut url, s);
        assert_eq!(url, reference(s, &URL_ESCAPE_FLAG, &URL_ESCAPE_TABLE), "url escape: {s:?}");
    }

    #[test]
    fn matches_reference_on_fixtures() {
        for case in [
            "",
            "a",
            "&",
            "<>",
            "plain ascii text with no escapes at all",
            "&<>\"'",
            "a&b<c>d\"e'f",
            "exactly-8b",
            "seven77",
            "&&&&&&&&&&&&&&&&",
            "trailing escape &",
            "& leading escape",
            // The folded masks admit no extra bytes, but these neighbours are
            // the ones a sloppy fold would leak: 0x21 0x23 0x25 0x3D 0x3F.
            "!#%=?",
            "!#%=? &<>\"' !#%=?",
        ] {
            check(case);
        }
    }

    #[test]
    fn matches_reference_on_borrow_propagation_shapes() {
        // `has_zero` can flag a 0x01 lane that borrowed from a real match
        // below it. These interleavings put such bytes directly after a match
        // inside the same word, which is where a wrong lane index would show.
        for filler in ["!", "#", "%", "=", "?", "\x01", "\x00"] {
            for needle in ["&", "<", ">", "\"", "'", " "] {
                for lead in 0..9 {
                    let mut s = "x".repeat(lead);
                    s.push_str(needle);
                    for _ in 0..8 {
                        s.push_str(filler);
                    }
                    s.push_str(needle);
                    check(&s);
                }
            }
        }
    }

    #[test]
    fn matches_reference_across_the_overlapping_tail_read() {
        // The sub-word tail is covered by re-reading the last eight bytes
        // with the already-cleared lanes masked off, which is exactly where
        // a masked-off match can borrow into the lane above it. The pairs
        // below are the ones that can do it: after the mask folds, `<`/`>`
        // leave a `0x01` lane on a following `=` or `?`, and `"` on a
        // following `#`. One is placed at every offset of every length so
        // each lands on both sides of the tail boundary.
        for pair in ["<=", "<?", ">=", ">?", "\"#"] {
            for len in 2..40usize {
                for at in 0..len - 1 {
                    let mut s = "x".repeat(len);
                    s.replace_range(at..at + 2, pair);
                    check(&s);
                }
            }
        }
    }

    #[test]
    fn matches_reference_on_pseudorandom_bytes() {
        // xorshift over the printable-plus-needles range; deterministic so a
        // failure is reproducible.
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        let alphabet: Vec<u8> = (0x20u8..0x7f).chain(*b"&<>\"' ").collect();
        for len in 0..200 {
            let mut s = String::with_capacity(len);
            for _ in 0..len {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                s.push(alphabet[(state % alphabet.len() as u64) as usize] as char);
            }
            check(&s);
        }
    }

    #[test]
    fn matches_reference_on_multibyte_utf8() {
        check("日本語のテキスト");
        check("emoji 🎉 and <tags> mixed");
        check("café & naïve — \"quoted\"");
    }
}
