//! SSE2 newline scan for x86-64, where SSE2 is part of the target baseline.

#![allow(unsafe_code)]

/// Finds the next line terminator with one inline vector probe before moving
/// longer scans out of line. Most Markdown lines finish in that first vector;
/// keeping only the probe inline avoids cloning the full scan into every block
/// parser call site.
#[inline]
pub(super) fn line_end(bytes: &[u8], from: usize) -> usize {
    let rest = if from + 16 <= bytes.len() {
        // SAFETY: the length check above proves the vector is in bounds.
        let mask = unsafe { newline_lanes(bytes, from) };
        if mask != 0 {
            return from + mask.trailing_zeros() as usize;
        }
        from + 16
    } else {
        from
    };
    line_end_rest(bytes, rest)
}

#[inline(never)]
fn line_end_rest(bytes: &[u8], from: usize) -> usize {
    if bytes.len() >= 16 {
        return line_end_sse2(bytes, from);
    }

    let mut at = from;
    while at < bytes.len() && !super::is_line_ending_byte(bytes[at]) {
        at += 1;
    }
    at
}

#[inline]
fn line_end_sse2(bytes: &[u8], from: usize) -> usize {
    let end = bytes.len();
    debug_assert!(from <= end && end >= 16);
    let mut at = from;

    // SAFETY: stepped loads are guarded by their bounds checks. The tail load
    // starts at `end - 16`, which exists because this function requires a
    // document at least one vector wide.
    unsafe {
        while at + 32 <= end {
            let mask = newline_lanes(bytes, at) | (newline_lanes(bytes, at + 16) << 16);
            if mask != 0 {
                return at + mask.trailing_zeros() as usize;
            }
            at += 32;
        }
        if at + 16 <= end {
            let mask = newline_lanes(bytes, at);
            if mask != 0 {
                return at + mask.trailing_zeros() as usize;
            }
            at += 16;
        }
        if at < end {
            let base = end - 16;
            let mask = newline_lanes(bytes, base) & (u32::MAX << (at - base));
            if mask != 0 {
                return base + mask.trailing_zeros() as usize;
            }
            return end;
        }
    }
    at
}

/// Returns a bit mask for `\n` and `\r` in the vector at `at`.
///
/// # Safety
/// `at + 16 <= bytes.len()`.
#[inline]
unsafe fn newline_lanes(bytes: &[u8], at: usize) -> u32 {
    use std::arch::x86_64::{
        _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8,
    };

    debug_assert!(at + 16 <= bytes.len());
    // SAFETY: x86-64 guarantees SSE2 and the caller guarantees 16 readable
    // bytes. The load has no alignment requirement.
    let mask = unsafe {
        let vector = _mm_loadu_si128(bytes.as_ptr().add(at).cast());
        let newline = _mm_cmpeq_epi8(vector, _mm_set1_epi8(b'\n'.cast_signed()));
        let carriage = _mm_cmpeq_epi8(vector, _mm_set1_epi8(b'\r'.cast_signed()));
        _mm_movemask_epi8(_mm_or_si128(newline, carriage))
    };
    mask.cast_unsigned()
}
