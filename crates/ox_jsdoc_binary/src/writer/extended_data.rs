// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Extended Data buffer builder used by [`super::BinaryWriter`].
//!
//! Manages a single byte buffer onto which per-Kind Extended Data records
//! are appended. Each new record is prefixed with zero-fill padding so its
//! starting byte offset is divisible by
//! [`crate::format::extended_data::EXTENDED_DATA_ALIGNMENT`] (8 bytes).
//!
//! See `design/007-binary-ast/format.md#extended-data-section` for the
//! per-Kind layouts.

use core::num::NonZeroU32;

use oxc_allocator::{Allocator, Vec as ArenaVec};

use crate::format::extended_data::EXTENDED_DATA_ALIGNMENT;

/// Byte offset into the Extended Data section.
///
/// Newtype wrapper to avoid mixing up Extended Data offsets with String
/// Offsets indices or node indices. Stored as `offset + 1` internally so the
/// type can use `NonZeroU32` for niche optimization (`Option<ExtOffset>` is
/// 4 bytes).
///
/// The wire representation in Node Data uses the *raw* offset packed into
/// 30 bits (see `format::node_record::PAYLOAD_MASK`); offset 0 is a valid
/// position because the very first record sits at byte 0 of the section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExtOffset(NonZeroU32);

impl ExtOffset {
    /// Construct an [`ExtOffset`] from the raw byte offset.
    ///
    /// Returns `None` only when `offset + 1` overflows `u32` (i.e. `offset`
    /// is `u32::MAX`).
    #[inline]
    #[must_use]
    pub const fn from_u32(offset: u32) -> Option<Self> {
        match NonZeroU32::new(offset.wrapping_add(1)) {
            Some(nz) => Some(ExtOffset(nz)),
            None => None,
        }
    }

    /// Get the raw byte offset.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0.get() - 1
    }
}

/// Builder that appends Extended Data records with the required 8-byte
/// alignment.
pub struct ExtendedDataBuilder<'arena> {
    /// Concatenated Extended Data records.
    pub(crate) buffer: ArenaVec<'arena, u8>,
}

impl<'arena> ExtendedDataBuilder<'arena> {
    /// Create an empty builder backed by the supplied arena.
    #[must_use]
    pub fn new(arena: &'arena Allocator) -> Self {
        ExtendedDataBuilder { buffer: ArenaVec::new_in(arena) }
    }

    /// Truncate the builder back to its post-[`Self::new`] state, keeping the
    /// arena-backed buffer capacity. Used by
    /// [`crate::writer::BinaryWriter::reset`] to recycle a writer across
    /// per-comment parse calls.
    pub(crate) fn reset(&mut self) {
        self.buffer.truncate(0);
    }

    /// Reserve `size` bytes for a new record, returning the resulting
    /// [`ExtOffset`].
    ///
    /// Inserts zero-fill padding before the record so the offset is
    /// 8-byte aligned. The returned offset points to the first reserved
    /// byte; the reserved bytes themselves are zeroed and can be patched
    /// in place by the caller using indexing.
    pub fn reserve(&mut self, size: usize) -> ExtOffset {
        let aligned_offset = self.next_aligned_offset();
        let new_len = aligned_offset + size;
        // `Vec::resize(len, 0)` lowers to `memset` for `Vec<u8>`, which is
        // measurably tighter than `extend(repeat_n(...))` because the
        // latter retains the iterator dispatch.
        self.buffer.resize(new_len, 0);
        ExtOffset::from_u32(aligned_offset as u32).expect("Extended Data offset overflow")
    }

    /// Mutable view of `len` bytes starting at `offset` in the underlying
    /// buffer. Helper for callers that want to write multi-byte fields
    /// after [`Self::reserve`].
    #[inline]
    pub fn slice_mut(&mut self, offset: ExtOffset, len: usize) -> &mut [u8] {
        let start = offset.as_u32() as usize;
        &mut self.buffer[start..start + len]
    }

    /// Combined [`Self::reserve`] + [`Self::slice_mut`]: reserve `size`
    /// bytes and return both the resulting offset and a mutable slice
    /// pointing at the just-allocated zone. Saves one bounds check + one
    /// arithmetic round-trip on the per-Kind `write_*` hot path.
    #[inline]
    pub fn reserve_mut(&mut self, size: usize) -> (ExtOffset, &mut [u8]) {
        let aligned_offset = self.next_aligned_offset();
        let new_len = aligned_offset + size;
        self.buffer.resize(new_len, 0);
        let off =
            ExtOffset::from_u32(aligned_offset as u32).expect("Extended Data offset overflow");
        let slice = &mut self.buffer[aligned_offset..new_len];
        (off, slice)
    }

    /// Total Extended Data section size in bytes (includes padding).
    #[inline]
    #[must_use]
    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    /// Whether the buffer is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Round the current buffer length up to the next 8-byte boundary.
    /// Equivalent to `unaligned.div_ceil(8) * 8`, but the bitwise form
    /// compiles to two instructions vs the more involved div+mul lowering
    /// the generic version produces (rustc 1.x doesn't reliably constant
    /// fold the `align = 8` for the helper here even though it does at
    /// the call site).
    #[inline]
    fn next_aligned_offset(&self) -> usize {
        const _: () = assert!(EXTENDED_DATA_ALIGNMENT == 8);
        (self.buffer.len() + 7) & !7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ext_offset_round_trips() {
        for raw in [0u32, 1, 7, 8, 0x3FFF_FFFE] {
            let off = ExtOffset::from_u32(raw).unwrap();
            assert_eq!(off.as_u32(), raw);
        }
    }

    #[test]
    fn ext_offset_zero_is_representable() {
        let off = ExtOffset::from_u32(0).unwrap();
        assert_eq!(off.as_u32(), 0);
    }

    #[test]
    fn ext_offset_rejects_u32_max() {
        assert!(ExtOffset::from_u32(u32::MAX).is_none());
    }

    #[test]
    fn reserve_first_record_starts_at_zero() {
        let arena = Allocator::default();
        let mut b = ExtendedDataBuilder::new(&arena);
        let off = b.reserve(2);
        assert_eq!(off.as_u32(), 0);
        assert_eq!(b.size(), 2);
    }

    #[test]
    fn reserve_pads_to_8_byte_alignment() {
        let arena = Allocator::default();
        let mut b = ExtendedDataBuilder::new(&arena);
        // First reserve 2 bytes (so the buffer ends at offset 2)
        let _ = b.reserve(2);
        // Second reserve must round up to offset 8 (6 bytes padding inserted)
        let off = b.reserve(2);
        assert_eq!(off.as_u32(), 8);
        assert_eq!(b.size(), 10);
    }

    #[test]
    fn slice_mut_round_trip() {
        let arena = Allocator::default();
        let mut b = ExtendedDataBuilder::new(&arena);
        let off = b.reserve(4);
        b.slice_mut(off, 4).copy_from_slice(&[1, 2, 3, 4]);
        assert_eq!(&b.buffer[..], &[1, 2, 3, 4]);
    }
}
