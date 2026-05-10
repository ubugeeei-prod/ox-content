// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! String field encoding (6 bytes per slot).
//!
//! Replaces the indirect "String Offsets" table by materializing every
//! string reference inline as a [`StringField`] holding `(offset: u32,
//! length: u16)` — readers slice the String Data section directly without
//! the extra hop.
//!
//! Wire layout (little-endian, **not** naturally aligned):
//!
//! ```text
//! byte 0-3 : offset (u32)
//! byte 4-5 : length (u16)
//! ```
//!
//! Each slot is 6 bytes. Records in Extended Data may pack consecutive
//! StringField slots back-to-back; the surrounding Extended Data record's
//! 8-byte alignment guarantees that the first slot's u32 sits on at least a
//! 2-byte boundary, but later slots may straddle 4/8-byte boundaries.
//! Decoders therefore must use byte-level reads (`from_le_bytes` on a
//! `[u8; 4]` / `[u8; 2]` slice) rather than relying on natural alignment.

/// Size of a single [`StringField`] in the on-wire encoding (6 bytes).
pub const STRING_FIELD_SIZE: usize = 6;

/// Offset value used by [`StringField::NONE`].
///
/// `u32::MAX` is reserved as the "absent string" sentinel; since the String
/// Data section is bounded by [`crate::format::node_record::PAYLOAD_MAX`]
/// (~1 GiB) in practice, no real string ever has this offset.
pub const STRING_FIELD_NONE_OFFSET: u32 = u32::MAX;

/// Length value used by [`StringField::NONE`]. Always zero so that the None
/// sentinel is distinguishable from any valid empty-string field, which uses
/// some valid `(offset, 0)` pair pointing into the data buffer (typically
/// the COMMON_STRINGS prelude entry for `""`).
pub const STRING_FIELD_NONE_LENGTH: u16 = 0;

/// On-wire string reference: a `(offset, length)` pair into the String Data
/// section.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringField {
    /// Byte offset within the String Data section where this string starts.
    pub offset: u32,
    /// UTF-8 byte length of the referenced string.
    pub length: u16,
}

impl StringField {
    /// Sentinel meaning "no string" (`Option<&str> = None`). Encoded as
    /// `(offset = u32::MAX, length = 0)` so it never collides with a valid
    /// reference.
    pub const NONE: Self =
        Self { offset: STRING_FIELD_NONE_OFFSET, length: STRING_FIELD_NONE_LENGTH };

    /// Construct a `StringField` from raw `(offset, length)` parts.
    #[inline]
    #[must_use]
    pub const fn new(offset: u32, length: u16) -> Self {
        Self { offset, length }
    }

    /// Whether this `StringField` is the [`Self::NONE`] sentinel.
    #[inline]
    #[must_use]
    pub const fn is_none(self) -> bool {
        self.offset == STRING_FIELD_NONE_OFFSET && self.length == STRING_FIELD_NONE_LENGTH
    }

    /// Write the 6-byte little-endian encoding into `dst`.
    ///
    /// `dst` must be at least [`STRING_FIELD_SIZE`] bytes long; the write
    /// debug-asserts on a too-short slice.
    #[inline]
    pub fn write_le(self, dst: &mut [u8]) {
        debug_assert!(
            dst.len() >= STRING_FIELD_SIZE,
            "StringField::write_le slice too short ({} bytes)",
            dst.len()
        );
        dst[0..4].copy_from_slice(&self.offset.to_le_bytes());
        dst[4..6].copy_from_slice(&self.length.to_le_bytes());
    }

    /// Read a 6-byte little-endian encoded `StringField` from `src`.
    #[inline]
    #[must_use]
    pub fn read_le(src: &[u8]) -> Self {
        debug_assert!(
            src.len() >= STRING_FIELD_SIZE,
            "StringField::read_le slice too short ({} bytes)",
            src.len()
        );
        let offset = u32::from_le_bytes([src[0], src[1], src[2], src[3]]);
        let length = u16::from_le_bytes([src[4], src[5]]);
        Self { offset, length }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_is_6_bytes() {
        assert_eq!(STRING_FIELD_SIZE, 6);
    }

    #[test]
    fn round_trip_through_write_read() {
        let cases = [
            StringField::new(0, 0),
            StringField::new(1, 5),
            StringField::new(0x1234_5678, 0xABCD),
            StringField::new(u32::MAX - 1, u16::MAX),
            StringField::NONE,
        ];
        let mut buf = [0u8; STRING_FIELD_SIZE];
        for field in cases {
            buf.fill(0);
            field.write_le(&mut buf);
            let decoded = StringField::read_le(&buf);
            assert_eq!(decoded, field, "round trip {field:?}");
        }
    }

    #[test]
    fn none_sentinel_distinct_from_empty_string() {
        let none = StringField::NONE;
        assert!(none.is_none());
        let empty = StringField::new(0, 0);
        assert!(!empty.is_none(), "empty (offset=0, length=0) is a valid string, not NONE");
    }

    #[test]
    fn write_le_lays_out_little_endian() {
        let field = StringField::new(0x0403_0201, 0x0605);
        let mut buf = [0u8; STRING_FIELD_SIZE];
        field.write_le(&mut buf);
        assert_eq!(buf, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }
}
