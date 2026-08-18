//! Vector classifiers for the escape scanners.
//!
//! Both needle sets are small and split cleanly by high nibble, which makes
//! them expressible as a pair of 16-entry tables — exactly what a vector
//! shuffle holds. `vqtbl1q_u8` on aarch64 and `pshufb` on x86-64 are the
//! same instruction for this purpose, so both paths run the identical
//! classifier and are covered by the same differential tests in the parent
//! module.

/// Nibble-pair classifier tables for the vector paths, one pair per
/// escaper.
///
/// A byte is flagged iff `low[b & 0x0F] & high[b >> 4]` is nonzero. Both
/// needle sets split cleanly by high nibble — `"`/`&`/`'` and ` `/`"`/`&`
/// sit at `0x2_`, `<`/`>` at `0x3_` — so two bits describe each set
/// exactly, admitting no other byte (digits share the `0x3_` row but not
/// the low nibbles, and every byte >= 0x80 maps to a zero high entry).
///
/// Sixteen entries is what a vector shuffle holds: `vqtbl1q_u8` and
/// `pshufb` classify all sixteen lanes in one instruction.
pub(super) struct NibbleTables {
    pub(super) low: [u8; 16],
    pub(super) high: [u8; 16],
}

pub(super) const ESCAPE_NIBBLES: NibbleTables = NibbleTables {
    //          0     1     2     3  4  5     6     7  8  9  A  B     C  D     E  F
    low: [0, 0, 0x01, 0, 0, 0, 0x01, 0x01, 0, 0, 0, 0, 0x02, 0, 0x02, 0],
    high: [0, 0, 0x01, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub(super) const URL_ESCAPE_NIBBLES: NibbleTables = NibbleTables {
    low: [0x01, 0, 0x01, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0x02, 0, 0x02, 0],
    high: [0, 0, 0x01, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

/// Offset of the first flagged byte in `bytes[from..]`, or `None` when the
/// caller should fall through to the word scan (input shorter than one
/// vector, or no vector path on this target).
#[cfg(target_arch = "aarch64")]
#[allow(unsafe_code)]
#[inline]
fn first_flagged_vector(bytes: &[u8], from: usize, tables: &NibbleTables) -> Option<usize> {
    use std::arch::aarch64::*;
    let len = bytes.len();
    if len < 16 {
        return None;
    }
    let mut i = from;
    unsafe {
        let low = vld1q_u8(tables.low.as_ptr());
        let high = vld1q_u8(tables.high.as_ptr());
        let nibble = vdupq_n_u8(0x0F);
        let classify = |v: uint8x16_t| {
            let lo = vqtbl1q_u8(low, vandq_u8(v, nibble));
            let hi = vqtbl1q_u8(high, vshrq_n_u8(v, 4));
            let m = vtstq_u8(lo, hi);
            vget_lane_u64(vreinterpret_u64_u8(vshrn_n_u16(vreinterpretq_u16_u8(m), 4)), 0)
        };
        while i + 16 <= len {
            let mask = classify(vld1q_u8(bytes.as_ptr().add(i)));
            if mask != 0 {
                return Some(i + (mask.trailing_zeros() / 4) as usize);
            }
            i += 16;
        }
        if i < len {
            // Overlapping tail: re-read the last vector and drop the lanes
            // the loop already cleared. `vtstq_u8` gives exact per-lane
            // answers, so no borrow can leak across the mask.
            let base = len - 16;
            let mask =
                classify(vld1q_u8(bytes.as_ptr().add(base))) & (u64::MAX << ((i - base) * 4));
            if mask != 0 {
                return Some(base + (mask.trailing_zeros() / 4) as usize);
            }
        }
    }
    Some(len)
}

/// SSSE3 sibling of [`first_flagged_vector`]; `pshufb` is the same 16-entry
/// lookup. SSSE3 is not in the x86-64 baseline, so callers detect it.
///
/// # Safety
///
/// The caller must have verified SSSE3 support. Every load is bounded by
/// the surrounding length checks.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
#[target_feature(enable = "ssse3")]
unsafe fn first_flagged_ssse3(bytes: &[u8], from: usize, tables: &NibbleTables) -> Option<usize> {
    use std::arch::x86_64::*;
    let len = bytes.len();
    if len < 16 {
        return None;
    }
    let mut i = from;
    unsafe {
        let low = _mm_loadu_si128(tables.low.as_ptr().cast());
        let high = _mm_loadu_si128(tables.high.as_ptr().cast());
        let nibble = _mm_set1_epi8(0x0F);
        let classify = |v: __m128i| {
            let lo = _mm_shuffle_epi8(low, _mm_and_si128(v, nibble));
            let hi = _mm_shuffle_epi8(high, _mm_and_si128(_mm_srli_epi16(v, 4), nibble));
            let m = _mm_and_si128(lo, hi);
            // `movemask` fills only the low 16 bits, so the complement
            // stays inside them and the widening is exact.
            let clean = _mm_movemask_epi8(_mm_cmpeq_epi8(m, _mm_setzero_si128()));
            u32::from_ne_bytes((!clean & 0xFFFF).to_ne_bytes())
        };
        while i + 16 <= len {
            let mask = classify(_mm_loadu_si128(bytes.as_ptr().add(i).cast()));
            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 16;
        }
        if i < len {
            let base = len - 16;
            let mask = classify(_mm_loadu_si128(bytes.as_ptr().add(base).cast()))
                & (u32::MAX << (i - base));
            if mask != 0 {
                return Some(base + mask.trailing_zeros() as usize);
            }
        }
    }
    Some(len)
}

#[inline]
pub(super) fn first_flagged_simd(
    bytes: &[u8],
    from: usize,
    tables: &NibbleTables,
) -> Option<usize> {
    #[cfg(target_arch = "aarch64")]
    {
        first_flagged_vector(bytes, from, tables)
    }
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("ssse3") {
            // SAFETY: guarded by the detection above.
            #[allow(unsafe_code)]
            unsafe {
                first_flagged_ssse3(bytes, from, tables)
            }
        } else {
            None
        }
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        let _ = (bytes, from, tables);
        None
    }
}
