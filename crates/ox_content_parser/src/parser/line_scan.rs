//! Newline scanning for the document pre-pass.
//!
//! The pre-pass walks every line of the document — blanks and one-word lines
//! included — so it asks "where does this line end" tens of thousands of times
//! against very short spans. At that length a `memchr` call spends most of its
//! time on per-call setup rather than on scanning: measured on the bundled
//! corpora, walking lines is ~90% of the pre-pass, and a SWAR
//! (SIMD-within-a-register) word test beats `memchr` there by a third.
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
