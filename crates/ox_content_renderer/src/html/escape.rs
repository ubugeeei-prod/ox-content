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

mod nibble;

use nibble::{ESCAPE_NIBBLES, NibbleTables, URL_ESCAPE_NIBBLES, first_flagged_simd};

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
    tables: &NibbleTables,
) -> usize {
    if let Some(found) = first_flagged_simd(bytes, from, tables) {
        return found;
    }
    let len = bytes.len();
    let mut i = from;

    while i + 8 <= len {
        let word = u64::from_le_bytes(copy_eight(bytes, i));
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
        let word = u64::from_le_bytes(copy_eight(bytes, base));
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

#[inline]
fn copy_eight(bytes: &[u8], from: usize) -> [u8; 8] {
    let mut chunk = [0u8; 8];
    chunk.copy_from_slice(&bytes[from..from + 8]);
    chunk
}

/// Appends `src` to `out`, copying short runs inline instead of handing them
/// to `memmove`.
///
/// Half of the byte runs this module copies are under 16 bytes long — text
/// nodes are short, and every escape replacement is 5 bytes or fewer. At that
/// size libc's `memmove` spends most of its time picking a strategy, and
/// `push_str` cannot avoid it: any length the compiler cannot see lowers to a
/// `memcpy` call. Two overlapping loads and stores cover every run up to 16
/// bytes with no length-dependent branching inside the copy.
#[allow(unsafe_code)]
#[inline]
fn push_run(out: &mut String, src: &str) {
    let len = src.len();
    if len > 16 {
        out.push_str(src);
        return;
    }
    out.reserve(len);

    // SAFETY: `reserve` above guarantees `len` spare bytes past the current
    // length, and every write below lands inside `[0, len)` of that spare
    // region, so `set_len` covers exactly the bytes written. `src` is a `&str`
    // appended at the end of `out`, which is a char boundary, so the result
    // stays valid UTF-8. The two pointers cannot overlap: `out` is borrowed
    // uniquely, so no live `&str` can alias its buffer.
    unsafe {
        let vec = out.as_mut_vec();
        let at = vec.len();
        let dst = vec.as_mut_ptr().add(at);
        let src = src.as_ptr();
        if len >= 8 {
            std::ptr::copy_nonoverlapping(src, dst, 8);
            std::ptr::copy_nonoverlapping(src.add(len - 8), dst.add(len - 8), 8);
        } else if len >= 4 {
            std::ptr::copy_nonoverlapping(src, dst, 4);
            std::ptr::copy_nonoverlapping(src.add(len - 4), dst.add(len - 4), 4);
        } else if len > 0 {
            *dst = *src;
            *dst.add(len / 2) = *src.add(len / 2);
            *dst.add(len - 1) = *src.add(len - 1);
        }
        vec.set_len(at + len);
    }
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
    tables: &NibbleTables,
) {
    // The invariant: bytes in `s[start..i]` have not been copied yet, and
    // everything before `start` has already been emitted in escaped form.
    let bytes = s.as_bytes();
    let mut start = 0usize;

    loop {
        let i = first_flagged(bytes, start, &mask_of, flags, tables);
        if i >= bytes.len() {
            break;
        }
        if start < i {
            push_run(out, &s[start..i]);
        }
        push_run(out, table[bytes[i] as usize]);
        start = i + 1;
    }

    if start < bytes.len() {
        push_run(out, &s[start..]);
    }
}

#[inline]
pub(super) fn write_escaped_into(out: &mut String, s: &str) {
    crate::profile_span_detail!("renderer::escape_text");
    // `reserve(s.len())` covers the no-escape case exactly and reduces growth
    // even when replacements make the final output longer.
    out.reserve(s.len());
    escape_into(out, s, escape_mask, &ESCAPE_FLAG, &ESCAPE_TABLE, &ESCAPE_NIBBLES);
}

pub(super) fn write_url_escaped_into(out: &mut String, s: &str) {
    crate::profile_span_detail!("renderer::escape_url");
    // Same scanner as `write_escaped_into`, but with URL attribute semantics.
    // Ampersand stays HTML-escaped because the result is written inside an
    // HTML attribute, while spaces and tag delimiters are percent encoded to
    // keep the URL value itself stable.
    escape_into(out, s, url_escape_mask, &URL_ESCAPE_FLAG, &URL_ESCAPE_TABLE, &URL_ESCAPE_NIBBLES);
}

#[cfg(test)]
mod tests;
