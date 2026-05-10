// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Node record layout (24 bytes per node) and Node Data bit packing.
//!
//! See `design/007-binary-ast/format.md#nodes-section-24-bytesnode` and
//! `format.md#node-data-bit-packing-32-bit` for the full specifications.
//!
//! ```text
//! byte  0     : Kind            (u8)
//! byte  1     : Common Data     (u8, lower 6 bit; upper 2 bit reserved)
//! byte  2-3   : padding         (u16, zero-fill)
//! byte  4-7   : Pos             (u32, UTF-16 offset relative to root sourceText)
//! byte  8-11  : End             (u32, UTF-16 offset relative to root sourceText)
//! byte 12-15  : Node Data       (u32, 2-bit type tag + 30-bit payload)
//! byte 16-19  : parent_index    (u32, 0 = sentinel)
//! byte 20-23  : next_sibling    (u32, 0 = no sibling)
//! ```

use core::fmt;

/// Size of one node record in bytes (fixed for the entire format).
pub const NODE_RECORD_SIZE: usize = 24;

// ---------------------------------------------------------------------------
// Field byte offsets within a node record.
// ---------------------------------------------------------------------------

/// Offset of the `Kind` byte.
pub const KIND_OFFSET: usize = 0;
/// Offset of the `Common Data` byte (lower 6 bit).
pub const COMMON_DATA_OFFSET: usize = 1;
/// Offset of the 2-byte alignment padding (zero-fill).
pub const PADDING_OFFSET: usize = 2;
/// Offset of the `Pos` field (u32, UTF-16 code units).
pub const POS_OFFSET: usize = 4;
/// Offset of the `End` field (u32, UTF-16 code units).
pub const END_OFFSET: usize = 8;
/// Offset of the `Node Data` field (u32, type tag + payload).
pub const NODE_DATA_OFFSET: usize = 12;
/// Offset of the `parent_index` field (u32, 0 = sentinel).
pub const PARENT_INDEX_OFFSET: usize = 16;
/// Offset of the `next_sibling` field (u32, 0 = no sibling).
pub const NEXT_SIBLING_OFFSET: usize = 20;

// ---------------------------------------------------------------------------
// Common Data masks (byte 1).
// ---------------------------------------------------------------------------

/// Bit mask isolating the 6-bit Common Data payload (`bits[5:0]`).
pub const COMMON_DATA_MASK: u8 = 0b0011_1111;
/// Bit mask isolating the reserved upper bits of byte 1 (`bits[7:6]`).
pub const COMMON_DATA_RESERVED_MASK: u8 = 0b1100_0000;
/// Number of Common Data bits actually used (6 of 8).
pub const COMMON_DATA_BITS: u8 = 6;

// ---------------------------------------------------------------------------
// Node Data bit packing (32-bit u32: 2-bit type tag + 30-bit payload).
// ---------------------------------------------------------------------------

/// Right shift amount for extracting the 2-bit type tag from Node Data.
pub const TYPE_TAG_SHIFT: u32 = 30;
/// Mask for the 2-bit type tag after shifting (`0b11`).
pub const TYPE_TAG_MASK: u32 = 0b11;
/// Mask for the 30-bit payload portion of Node Data.
pub const PAYLOAD_MASK: u32 = 0x3FFF_FFFF;
/// Maximum value the 30-bit payload can carry.
pub const PAYLOAD_MAX: u32 = PAYLOAD_MASK;

/// Type tag values stored in the upper 2 bits of Node Data.
///
/// Discriminants are stable per the format spec. String-leaf nodes use
/// `TypeTag::String` (long strings, via String Offsets table) or
/// `TypeTag::StringInline` (short strings ≤ 255 bytes, payload packed as
/// `(offset: u22, length: u8)`); Extended-type records reach their string
/// slots via inline [`crate::format::string_field::StringField`] entries
/// (no offsets-table indirection).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeTag {
    /// `0b00` - payload is a 30-bit Children bitmask (visitor order).
    Children = 0b00,
    /// `0b01` - payload is a 30-bit String Offsets index. Used for
    /// string-leaf nodes whose value is too long (≥ 256 bytes) or whose
    /// String Data offset exceeds the inline 22-bit range; ED-internal
    /// string slots embed StringField directly instead.
    String = 0b01,
    /// `0b10` - payload is a 30-bit byte offset into the Extended Data section.
    Extended = 0b10,
    /// `0b11` - payload is a packed `(offset: u22, length: u8)` directly
    /// pointing into the String Data section. Skips the String Offsets table
    /// indirection; preferred for short string-leaf values when the offset
    /// fits the 22-bit range. See [`pack_string_inline`] / [`unpack_string_inline`].
    StringInline = 0b11,
}

// ---------------------------------------------------------------------------
// StringInline payload bit packing (within the 30-bit Node Data payload).
// ---------------------------------------------------------------------------

/// Number of bits the inline String payload reserves for the data-buffer
/// offset (high portion of the 30-bit payload).
pub const STRING_INLINE_OFFSET_BITS: u32 = 22;
/// Number of bits the inline String payload reserves for the UTF-8 byte
/// length (low portion of the 30-bit payload).
pub const STRING_INLINE_LENGTH_BITS: u32 = 8;
/// Maximum offset that fits in the inline-payload offset field (4 MB - 1).
pub const STRING_INLINE_OFFSET_MAX: u32 = (1u32 << STRING_INLINE_OFFSET_BITS) - 1;
/// Maximum UTF-8 byte length that fits in the inline-payload length field
/// (255). Strings of length 256 or more must take the String Offsets path.
pub const STRING_INLINE_LENGTH_MAX: u32 = (1u32 << STRING_INLINE_LENGTH_BITS) - 1;
/// Bit mask isolating the length portion of an inline payload.
pub const STRING_INLINE_LENGTH_MASK: u32 = STRING_INLINE_LENGTH_MAX;

/// Pack `(offset, length)` into the 30-bit inline-payload layout.
///
/// `offset` MUST be ≤ [`STRING_INLINE_OFFSET_MAX`] (caller-checked).
/// Length is stored in the low 8 bits, offset in the upper 22 bits.
#[inline]
#[must_use]
pub const fn pack_string_inline(offset: u32, length: u8) -> u32 {
    debug_assert!(offset <= STRING_INLINE_OFFSET_MAX);
    (offset << STRING_INLINE_LENGTH_BITS) | (length as u32)
}

/// Unpack a 30-bit inline payload into `(offset, length)`.
#[inline]
#[must_use]
pub const fn unpack_string_inline(payload: u32) -> (u32, u8) {
    let offset = payload >> STRING_INLINE_LENGTH_BITS;
    let length = (payload & STRING_INLINE_LENGTH_MASK) as u8;
    (offset, length)
}

/// Error returned by [`TypeTag::from_u32`] when the value is outside `0b00 - 0b11`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTypeTag(pub u32);

impl fmt::Display for InvalidTypeTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Node Data type tag 0b{:02b}", self.0)
    }
}

impl TypeTag {
    /// Convert the raw 2-bit tag value (`0..=3`) into a [`TypeTag`].
    #[inline]
    pub const fn from_u32(value: u32) -> Result<Self, InvalidTypeTag> {
        match value {
            0b00 => Ok(TypeTag::Children),
            0b01 => Ok(TypeTag::String),
            0b10 => Ok(TypeTag::Extended),
            0b11 => Ok(TypeTag::StringInline),
            other => Err(InvalidTypeTag(other)),
        }
    }
}

/// Sentinel for "absent" stored in a 30-bit String payload (`0x3FFF_FFFF`).
///
/// The encoder writes this when an `Option<&str>` is `None`. The same value
/// happens to coincide with [`PAYLOAD_MAX`], so encoders **must** never assign
/// a real string the index `2^30 - 1`; in practice this is enforced by the
/// String Table being capped at the u16 limit (see `string_table.rs`).
pub const STRING_PAYLOAD_NONE_SENTINEL: u32 = PAYLOAD_MAX;

/// Pack a `(TypeTag, payload)` pair into a single Node Data u32.
///
/// The `payload` is masked to 30 bits; passing a larger value silently
/// truncates the upper bits. Encoders should validate the range beforehand.
#[inline]
#[must_use]
pub const fn pack_node_data(tag: TypeTag, payload: u32) -> u32 {
    ((tag as u32) << TYPE_TAG_SHIFT) | (payload & PAYLOAD_MASK)
}

/// Extract the raw 2-bit type tag from a Node Data u32.
#[inline]
#[must_use]
pub const fn type_tag_bits(node_data: u32) -> u32 {
    (node_data >> TYPE_TAG_SHIFT) & TYPE_TAG_MASK
}

/// Extract the 30-bit payload from a Node Data u32.
#[inline]
#[must_use]
pub const fn payload(node_data: u32) -> u32 {
    node_data & PAYLOAD_MASK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_record_size_is_24() {
        assert_eq!(NODE_RECORD_SIZE, 24);
    }

    #[test]
    fn field_offsets_partition_24_bytes() {
        let layout: &[(usize, usize, &str)] = &[
            (KIND_OFFSET, 1, "kind"),
            (COMMON_DATA_OFFSET, 1, "common_data"),
            (PADDING_OFFSET, 2, "padding"),
            (POS_OFFSET, 4, "pos"),
            (END_OFFSET, 4, "end"),
            (NODE_DATA_OFFSET, 4, "node_data"),
            (PARENT_INDEX_OFFSET, 4, "parent_index"),
            (NEXT_SIBLING_OFFSET, 4, "next_sibling"),
        ];
        let mut cursor = 0usize;
        for (offset, size, name) in layout {
            assert_eq!(*offset, cursor, "{name} starts at expected offset");
            cursor += size;
        }
        assert_eq!(cursor, NODE_RECORD_SIZE, "fields cover all 24 bytes");
    }

    #[test]
    fn common_data_masks_are_complementary() {
        assert_eq!(COMMON_DATA_MASK | COMMON_DATA_RESERVED_MASK, 0xFF);
        assert_eq!(COMMON_DATA_MASK & COMMON_DATA_RESERVED_MASK, 0x00);
        assert_eq!(COMMON_DATA_MASK.count_ones(), COMMON_DATA_BITS as u32);
    }

    #[test]
    fn node_data_pack_unpack_round_trip() {
        for tag in [TypeTag::Children, TypeTag::String, TypeTag::Extended, TypeTag::StringInline] {
            for &payload_value in &[0u32, 1, 0xABCD, PAYLOAD_MAX, PAYLOAD_MAX - 1] {
                let nd = pack_node_data(tag, payload_value);
                let extracted_tag = TypeTag::from_u32(type_tag_bits(nd)).unwrap();
                let extracted_payload = payload(nd);
                assert_eq!(extracted_tag, tag, "tag round trip");
                assert_eq!(
                    extracted_payload, payload_value,
                    "payload round trip for {tag:?} / 0x{payload_value:08X}"
                );
            }
        }
    }

    #[test]
    fn payload_truncation_when_value_exceeds_30_bits() {
        let nd = pack_node_data(TypeTag::String, 0xFFFF_FFFF);
        assert_eq!(payload(nd), PAYLOAD_MAX);
        assert_eq!(type_tag_bits(nd), TypeTag::String as u32);
    }

    #[test]
    fn payload_max_is_2_pow_30_minus_1() {
        assert_eq!(PAYLOAD_MAX, (1u32 << 30) - 1);
        assert_eq!(STRING_PAYLOAD_NONE_SENTINEL, PAYLOAD_MAX);
    }

    #[test]
    fn type_tag_from_u32_rejects_out_of_range() {
        assert!(TypeTag::from_u32(4).is_err());
        assert_eq!(TypeTag::from_u32(99).unwrap_err().0, 99);
    }

    #[test]
    fn pack_keeps_payload_within_30_bits_when_in_range() {
        // Top bits of payload must not bleed into the tag field.
        let nd = pack_node_data(TypeTag::Children, PAYLOAD_MAX);
        assert_eq!(type_tag_bits(nd), TypeTag::Children as u32);
        assert_eq!(payload(nd), PAYLOAD_MAX);
    }
}
