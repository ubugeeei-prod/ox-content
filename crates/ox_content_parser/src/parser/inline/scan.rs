//! Scanning for the next byte that can begin an inline construct.
//!
//! This is the parser's hottest loop: it classifies every byte of every
//! block's inline content. Ten markers is too many for the `memchr` family
//! and too scattered for a SWAR fold, so the portable path is a 256-entry
//! flag table read eight bytes at a time — one load per byte.
//!
//! Both SIMD paths replace that with a nibble-pair classifier that needs no
//! per-byte load at all, and they run the *same* classifier: aarch64 uses
//! `vqtbl1q_u8` and x86-64 `pshufb`, which are the same 16-entry table
//! lookup. Every path is checked against the flag table by the differential
//! tests below, for all 256 byte values at every offset.

#[allow(unused_imports)]
use crate::profile_span_detail;

/// Lookup table: `INLINE_SPECIAL[b] == 1` iff the byte can begin an inline
/// construct handled by `parse_inline_special`.
///
/// This is the definition the vectorized paths must agree with, and the
/// scan every target without one falls back to. The table deliberately
/// stores flags instead of enum variants: the scalar scan ORs eight entries
/// at a time, which gives LLVM a branch-free loop for long runs of normal
/// text. The actual parser decision still happens only after a candidate
/// byte is found.
static INLINE_SPECIAL: [u8; 256] = {
    let mut t = [0u8; 256];
    t[b'*' as usize] = 1;
    t[b'_' as usize] = 1;
    t[b'`' as usize] = 1;
    t[b'[' as usize] = 1;
    t[b'!' as usize] = 1;
    t[b'~' as usize] = 1;
    t[b'\\' as usize] = 1;
    t[b'<' as usize] = 1;
    t[b'\n' as usize] = 1;
    t[b'&' as usize] = 1;
    t
};

/// How far `next_inline_special` walks byte-at-a-time before switching to the
/// chunked scan. Eight is the only length measured that never regressed:
/// longer prefixes bought a little more on code-dense corpora and lost
/// 12-16% on prose-dense ones.
const SHORT_RUN_PREFIX: usize = 8;

/// Nibble-pair classifier tables for the vectorized paths.
///
/// A byte is special iff `LOW[b & 0x0F] & HIGH[b >> 4]` is nonzero. The ten
/// markers fall into six (high nibble, low-nibble set) groups — `\n`;
/// `!`/`&`/`*`; `<`; `[`/`\`/`_`; `` ` ``; `~` — and each group owns one
/// bit, so the AND is exact: it admits no byte outside the set, and every
/// byte >= 0x80 maps to a zero high-nibble entry.
///
/// A 16-entry table is what a vector shuffle can hold, which is the whole
/// point: `vqtbl1q_u8` / `pshufb` look up all sixteen lanes in one
/// instruction, where the flag table needs sixteen loads.
const LOW_NIBBLE: [u8; 16] =
    [0x10, 0x02, 0, 0, 0, 0, 0x02, 0, 0, 0, 0x03, 0x08, 0x0C, 0, 0x20, 0x08];
const HIGH_NIBBLE: [u8; 16] = [0x01, 0, 0x02, 0x04, 0, 0x08, 0x10, 0x20, 0, 0, 0, 0, 0, 0, 0, 0];

#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[inline]
fn next_special_neon(bytes: &[u8], from: usize) -> usize {
    use std::arch::aarch64::*;
    let end = bytes.len();
    let mut i = from;
    unsafe {
        let low = vld1q_u8(LOW_NIBBLE.as_ptr());
        let high = vld1q_u8(HIGH_NIBBLE.as_ptr());
        let nibble = vdupq_n_u8(0x0F);
        while i + 16 <= end {
            let v = vld1q_u8(bytes.as_ptr().add(i));
            let lo = vqtbl1q_u8(low, vandq_u8(v, nibble));
            let hi = vqtbl1q_u8(high, vshrq_n_u8(v, 4));
            // `vtstq_u8` turns the per-lane AND into the all-ones/all-zeros
            // form the nibble-narrowing mask extraction needs.
            let m = vtstq_u8(lo, hi);
            let narrow = vshrn_n_u16(vreinterpretq_u16_u8(m), 4);
            let mask = vget_lane_u64(vreinterpret_u64_u8(narrow), 0);
            if mask != 0 {
                return i + (mask.trailing_zeros() / 4) as usize;
            }
            i += 16;
        }
    }
    while i < end && INLINE_SPECIAL[bytes[i] as usize] == 0 {
        i += 1;
    }
    i
}

/// SSSE3 sibling of [`next_special_neon`]. `pshufb` is the same 16-entry
/// table lookup `vqtbl1q_u8` performs, so both paths run the identical
/// classifier and are covered by the same differential tests.
///
/// # Safety
///
/// The caller must have verified SSSE3 support. Every load is bounded by
/// the `i + 16 <= end` guard.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "ssse3")]
unsafe fn next_special_ssse3(bytes: &[u8], from: usize) -> usize {
    use std::arch::x86_64::*;
    let end = bytes.len();
    let mut i = from;
    unsafe {
        let low = _mm_loadu_si128(LOW_NIBBLE.as_ptr().cast());
        let high = _mm_loadu_si128(HIGH_NIBBLE.as_ptr().cast());
        let nibble = _mm_set1_epi8(0x0F);
        while i + 16 <= end {
            let v = _mm_loadu_si128(bytes.as_ptr().add(i).cast());
            // `_mm_srli_epi16` shifts 16-bit lanes, so the neighbouring
            // byte's low bits ride along; masking leaves the high nibble.
            let lo = _mm_shuffle_epi8(low, _mm_and_si128(v, nibble));
            let hi = _mm_shuffle_epi8(high, _mm_and_si128(_mm_srli_epi16(v, 4), nibble));
            let m = _mm_and_si128(lo, hi);
            // `movemask` of "lane is zero" sets a bit per *clean* byte, so
            // the complement's lowest set bit is the first marker.
            // `movemask` fills only the low 16 bits, so the cast is exact
            // and the complement below stays inside them.
            let clean = _mm_movemask_epi8(_mm_cmpeq_epi8(m, _mm_setzero_si128()));
            let flagged = !clean & 0xFFFF;
            if flagged != 0 {
                return i + flagged.trailing_zeros() as usize;
            }
            i += 16;
        }
    }
    next_inline_special_scalar(bytes, i)
}

#[cfg(target_arch = "aarch64")]
#[inline]
pub(super) fn next_inline_special(bytes: &[u8], from: usize) -> usize {
    next_special_neon(bytes, from)
}

#[cfg(target_arch = "x86_64")]
#[inline]
pub(super) fn next_inline_special(bytes: &[u8], from: usize) -> usize {
    // SSSE3 is not in the x86-64 baseline, so it is detected rather than
    // assumed. `is_x86_feature_detected!` caches its answer in an atomic,
    // and a machine without it keeps the scalar scan unchanged.
    if std::arch::is_x86_feature_detected!("ssse3") {
        // SAFETY: guarded by the detection above.
        #[allow(unsafe_code)]
        unsafe {
            next_special_ssse3(bytes, from)
        }
    } else {
        next_inline_special_scalar(bytes, from)
    }
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
#[inline]
pub(super) fn next_inline_special(bytes: &[u8], from: usize) -> usize {
    next_inline_special_scalar(bytes, from)
}

#[cfg_attr(target_arch = "aarch64", allow(dead_code))]
#[inline]
fn next_inline_special_scalar(bytes: &[u8], from: usize) -> usize {
    profile_span_detail!("parser::inline_scan");
    let end = bytes.len();

    // Text runs are strongly bimodal on the bundled corpora: 53-64% of them
    // end within eight bytes while holding under 5% of the bytes, and 44-52%
    // of the bytes sit in runs of 64 or more. Walking that short prefix one
    // byte at a time means the common run never pays the chunked scan's eight
    // lookups and seven ORs to find a marker two bytes in, while long runs —
    // where the bytes actually are — still reach the wide scan below.
    let quick = if from + SHORT_RUN_PREFIX < end { from + SHORT_RUN_PREFIX } else { end };
    let mut i = from;
    while i < quick && INLINE_SPECIAL[bytes[i] as usize] == 0 {
        i += 1;
    }
    if i < quick {
        return i;
    }
    if i == end {
        return end;
    }

    // Skip eight bytes at a time while the OR of their marker flags is zero.
    // This is not a semantic parser: it only proves that none of those bytes
    // can start inline syntax, so returning the first flagged byte preserves
    // the exact same marker positions as the previous per-byte loop.
    while i + 8 <= end {
        let chunk = &bytes[i..i + 8];
        let mask = INLINE_SPECIAL[chunk[0] as usize]
            | INLINE_SPECIAL[chunk[1] as usize]
            | INLINE_SPECIAL[chunk[2] as usize]
            | INLINE_SPECIAL[chunk[3] as usize]
            | INLINE_SPECIAL[chunk[4] as usize]
            | INLINE_SPECIAL[chunk[5] as usize]
            | INLINE_SPECIAL[chunk[6] as usize]
            | INLINE_SPECIAL[chunk[7] as usize];
        if mask != 0 {
            break;
        }
        i += 8;
    }
    while i < end && INLINE_SPECIAL[bytes[i] as usize] == 0 {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The definition the chunked scan has to agree with.
    fn reference(bytes: &[u8], from: usize) -> usize {
        let mut i = from;
        while i < bytes.len() && INLINE_SPECIAL[bytes[i] as usize] == 0 {
            i += 1;
        }
        i
    }

    #[test]
    fn matches_reference_around_the_prefix_boundary() {
        // A marker at every offset, in inputs long and short enough to exercise
        // the short-run prefix, the prefix/chunk handoff, and the scalar tail.
        for len in 0..40usize {
            for marker_at in 0..len {
                let mut buffer = [b'x'; 40];
                buffer[marker_at] = b'*';
                let bytes = &buffer[..len];
                for from in 0..=len {
                    assert_eq!(
                        next_inline_special(bytes, from),
                        reference(bytes, from),
                        "len {len}, marker at {marker_at}, from {from}"
                    );
                }
            }
        }
    }

    #[test]
    fn matches_reference_with_no_marker() {
        let buffer = [b'x'; 40];
        for len in 0..=buffer.len() {
            let bytes = &buffer[..len];
            for from in 0..=len {
                assert_eq!(next_inline_special(bytes, from), reference(bytes, from));
            }
        }
    }

    #[test]
    fn matches_reference_for_every_byte_value_at_every_offset() {
        // The vectorized classifiers decide from nibble pairs rather than a
        // 256-entry table, so a wrong entry would admit or drop a byte the
        // flag table disagrees with. Check all 256 values, at every offset
        // of a buffer long enough to cross the 16-byte step and land in the
        // scalar tail.
        for value in 0..=255u8 {
            for at in 0..40usize {
                let mut buffer = [b'x'; 40];
                buffer[at] = value;
                let bytes = &buffer[..];
                assert_eq!(
                    next_inline_special(bytes, 0),
                    reference(bytes, 0),
                    "byte {value:#04x} at {at}"
                );
                assert_eq!(
                    next_inline_special_scalar(bytes, 0),
                    reference(bytes, 0),
                    "scalar: byte {value:#04x} at {at}"
                );
            }
        }
    }

    #[test]
    fn matches_reference_on_every_marker_byte() {
        for marker in *b"*_`[!~\\<\n&" {
            for lead in 0..20usize {
                let mut buffer = [b'x'; 40];
                buffer[lead] = marker;
                let bytes = &buffer[..lead + 8];
                assert_eq!(
                    next_inline_special(bytes, 0),
                    reference(bytes, 0),
                    "marker {marker:?} at {lead}"
                );
            }
        }
    }
}
