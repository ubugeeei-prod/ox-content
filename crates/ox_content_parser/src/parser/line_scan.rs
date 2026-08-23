//! Newline scanning for the document pre-pass.
//!
//! The pre-pass walks every line of the document — blanks and one-word lines
//! included — so it asks "where does this line end" tens of thousands of times.
//! At that access pattern a `memchr` call spends most of its time on per-call
//! setup rather than on scanning: walking lines is ~90% of the pre-pass.
//!
//! Short tails and portable targets use a SWAR (SIMD-within-a-register) word
//! test, which beats `memchr` by a third on the short-line case that dominated
//! the original measurement. aarch64 additionally runs a 32-byte NEON scan on
//! the rest: rust-book-style prose (median line ~57 bytes, nearly half of
//! lines 64+) pays four to eight SWAR iterations per line, and those documents
//! are exactly the ones that cannot bail out of the pre-pass because they
//! hold link reference definitions.
//!
//! That advantage is specific to this access pattern. Block parsing walks the
//! long prose lines of a paragraph, where `memchr`'s wider SIMD step overtakes
//! an eight-byte word test — converting those call sites measured 4-11% slower,
//! so they deliberately keep using `memchr`.

const ONES: u64 = 0x0101_0101_0101_0101;
const HIGH: u64 = 0x8080_8080_8080_8080;
const NEWLINES: u64 = (b'\n' as u64) * ONES;

/// Sets `0x80` in every byte lane of `word` that is zero.
///
/// A lane holding `0x01` can also light up when a lower lane borrowed into it,
/// so callers must only consume the *lowest* set bit: a spurious lane can only
/// sit above a genuine zero lane, which means the lowest set bit is always a
/// true match.
#[inline]
const fn has_zero(word: u64) -> u64 {
    word.wrapping_sub(ONES) & !word & HIGH
}

/// Byte offset of the newline ending the line that starts at `from`, or
/// `bytes.len()` when the last line is unterminated.
#[inline]
pub(in crate::parser) fn line_end(bytes: &[u8], from: usize) -> usize {
    #[cfg(target_arch = "aarch64")]
    {
        // One NEON vector is the breakeven against SWAR. Shorter remainders
        // stay on the word test so a 5-byte last line does not pay a 16-byte
        // overlapping reload.
        if bytes.len() - from >= 16 {
            return line_end_neon(bytes, from);
        }
    }
    line_end_swar(bytes, from)
}

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[inline]
fn line_end_neon(bytes: &[u8], from: usize) -> usize {
    use std::arch::aarch64::*;
    let end = bytes.len();
    let mut i = from;
    unsafe {
        let nl = vdupq_n_u8(b'\n');
        let classify = |v: uint8x16_t| {
            let m = vceqq_u8(v, nl);
            vget_lane_u64(vreinterpret_u64_u8(vshrn_n_u16(vreinterpretq_u16_u8(m), 4)), 0)
        };
        while i + 32 <= end {
            let m0 = classify(vld1q_u8(bytes.as_ptr().add(i)));
            if m0 != 0 {
                return i + (m0.trailing_zeros() / 4) as usize;
            }
            let m1 = classify(vld1q_u8(bytes.as_ptr().add(i + 16)));
            if m1 != 0 {
                return i + 16 + (m1.trailing_zeros() / 4) as usize;
            }
            i += 32;
        }
        while i + 16 <= end {
            let mask = classify(vld1q_u8(bytes.as_ptr().add(i)));
            if mask != 0 {
                return i + (mask.trailing_zeros() / 4) as usize;
            }
            i += 16;
        }
        if i < end && end >= 16 {
            // Overlapping tail: re-read the last vector and drop the lanes
            // the loops already cleared. `vceqq_u8` is exact per lane.
            let base = end - 16;
            let mask =
                classify(vld1q_u8(bytes.as_ptr().add(base))) & (u64::MAX << ((i - base) * 4));
            if mask != 0 {
                return base + (mask.trailing_zeros() / 4) as usize;
            }
            return end;
        }
    }
    while i < end && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

/// Portable eight-byte word scan used for short remainders and non-aarch64.
#[inline]
fn line_end_swar(bytes: &[u8], from: usize) -> usize {
    let end = bytes.len();
    let mut i = from;

    while i + 8 <= end {
        // `unwrap` is unreachable: the slice is exactly 8 bytes wide.
        let word = u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        let mask = has_zero(word ^ NEWLINES);
        if mask != 0 {
            return i + (mask.trailing_zeros() / 8) as usize;
        }
        i += 8;
    }

    while i < end && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

/// Byte offset of the next line's start — just past the newline, or `bytes.len()`
/// at end of input.
#[inline]
pub(in crate::parser) fn next_line_start(bytes: &[u8], from: usize) -> usize {
    let end = line_end(bytes, from);
    if end < bytes.len() { end + 1 } else { end }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_memchr_at_every_offset() {
        // Newlines at every position across the word boundary and the tail, so
        // both the SWAR lane index and the scalar remainder get exercised.
        for len in 0..40usize {
            for newline_at in 0..len {
                let mut buffer = [b'x'; 40];
                buffer[newline_at] = b'\n';
                let bytes = &buffer[..len];
                for from in 0..=len {
                    let expected =
                        memchr::memchr(b'\n', &bytes[from..]).map_or(len, |off| from + off);
                    assert_eq!(
                        line_end(bytes, from),
                        expected,
                        "len {len}, newline at {newline_at}, from {from}"
                    );
                    let expected_start = if expected < len { expected + 1 } else { expected };
                    assert_eq!(next_line_start(bytes, from), expected_start);
                }
            }
        }
    }

    #[test]
    fn matches_memchr_with_no_newline() {
        let buffer = [b'x'; 40];
        for len in 0..=buffer.len() {
            let bytes = &buffer[..len];
            for from in 0..=len {
                assert_eq!(line_end(bytes, from), len);
                assert_eq!(next_line_start(bytes, from), len);
            }
        }
    }

    #[test]
    fn matches_memchr_on_borrow_propagation_shapes() {
        // `0x0B` is the byte whose lane holds `0x01` after the newline xor, so
        // it is the one that can light up spuriously behind a real newline.
        for filler in [0x0Bu8, 0x00, 0x01, b'x'] {
            for lead in 0..12usize {
                let mut buffer = [filler; 40];
                buffer[lead] = b'\n';
                let bytes = &buffer[..];
                let expected = memchr::memchr(b'\n', bytes).unwrap_or(bytes.len());
                assert_eq!(line_end(bytes, 0), expected, "filler {filler:#x}, newline at {lead}");
            }
        }
    }

    #[test]
    fn matches_memchr_on_long_lines() {
        // 32-byte NEON loop + overlapping 16-byte tail: newlines past the
        // first vector, and unterminated 80-byte last lines.
        for len in 16..96usize {
            for newline_at in 0..len {
                let mut buffer = [b'x'; 96];
                buffer[newline_at] = b'\n';
                let bytes = &buffer[..len];
                for from in 0..=len {
                    let expected =
                        memchr::memchr(b'\n', &bytes[from..]).map_or(len, |off| from + off);
                    assert_eq!(
                        line_end(bytes, from),
                        expected,
                        "len {len}, newline at {newline_at}, from {from}"
                    );
                }
            }
            let unterminated = [b'x'; 96];
            let bytes = &unterminated[..len];
            for from in 0..=len {
                assert_eq!(line_end(bytes, from), len, "unterminated len {len} from {from}");
            }
        }
    }

    #[test]
    fn matches_memchr_on_multiline_walk() {
        let source = "alpha\nbeta\n\ngamma delta epsilon\n\tindented\nlast line without newline";
        let bytes = source.as_bytes();
        // Walk both scanners in lockstep so a divergence fails on the line it
        // happens, rather than as a mismatch between two collected lists.
        let mut pos = 0;
        let mut expected_pos = 0;
        let mut lines = 0;
        while expected_pos < bytes.len() {
            let expected = memchr::memchr(b'\n', &bytes[expected_pos..])
                .map_or(bytes.len(), |off| expected_pos + off);
            assert_eq!(line_end(bytes, pos), expected, "line {lines}");
            pos = next_line_start(bytes, pos);
            expected_pos = if expected < bytes.len() { expected + 1 } else { expected };
            assert_eq!(pos, expected_pos, "line {lines}");
            lines += 1;
        }
        assert_eq!(lines, 6);
    }
}
