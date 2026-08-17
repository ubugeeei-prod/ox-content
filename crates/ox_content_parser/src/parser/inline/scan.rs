#[allow(unused_imports)]
use crate::profile_span_detail;

/// Lookup table: `INLINE_SPECIAL[b] == 1` iff the byte can begin an inline
/// construct handled by `parse_inline_special`.
///
/// The table deliberately stores flags instead of enum variants. The scan in
/// `next_inline_special` ORs eight table entries at a time, which gives LLVM a
/// simple branch-free loop for long runs of normal text. The actual parser
/// decision still happens only after a candidate byte is found.
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

#[inline]
pub(super) fn next_inline_special(bytes: &[u8], from: usize) -> usize {
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
