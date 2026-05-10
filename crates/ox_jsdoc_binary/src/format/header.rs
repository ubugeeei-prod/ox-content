// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Header section (40 bytes, fixed).
//!
//! See `design/007-binary-ast/format.md#header-40-bytes` for the full layout.
//!
//! ```text
//! byte 0       : version (upper 4-bit major + lower 4-bit minor)
//! byte 1       : flags   (bit0=compat_mode, bit1-7=reserved)
//! byte 2-3     : reserved (zero-fill)
//! byte 4-7     : root_array_offset    (u32)
//! byte 8-11    : string_offsets_offset(u32)
//! byte 12-15   : string_data_offset   (u32)
//! byte 16-19   : extended_data_offset (u32)
//! byte 20-23   : diagnostics_offset   (u32)
//! byte 24-27   : nodes_offset         (u32)
//! byte 28-31   : node_count           (u32)
//! byte 32-35   : source_text_length   (u32)
//! byte 36-39   : root_count           (u32)
//! ```
//!
//! All multi-byte integers are little-endian (per the format conventions).

/// Header size in bytes (fixed).
pub const HEADER_SIZE: usize = 40;

// ---------------------------------------------------------------------------
// Field byte offsets within the header.
// ---------------------------------------------------------------------------

/// Offset of the version byte (`bits[7:4]` major, `bits[3:0]` minor).
pub const VERSION_OFFSET: usize = 0;
/// Offset of the flag byte.
pub const FLAGS_OFFSET: usize = 1;
/// Offset of the 2-byte reserved region (capability flags etc.).
pub const RESERVED_OFFSET: usize = 2;
/// Offset of the root index array start (u32).
pub const ROOT_ARRAY_OFFSET_FIELD: usize = 4;
/// Offset of the String Offsets section start (u32).
pub const STRING_OFFSETS_OFFSET_FIELD: usize = 8;
/// Offset of the String Data section start (u32).
pub const STRING_DATA_OFFSET_FIELD: usize = 12;
/// Offset of the Extended Data section start (u32).
pub const EXTENDED_DATA_OFFSET_FIELD: usize = 16;
/// Offset of the Diagnostics section start (u32).
pub const DIAGNOSTICS_OFFSET_FIELD: usize = 20;
/// Offset of the Nodes section start (u32).
pub const NODES_OFFSET_FIELD: usize = 24;
/// Offset of the total node count (u32).
pub const NODE_COUNT_FIELD: usize = 28;
/// Offset of the total source text length in UTF-8 bytes (u32).
pub const SOURCE_TEXT_LENGTH_FIELD: usize = 32;
/// Offset of the batch root count N (u32).
pub const ROOT_COUNT_FIELD: usize = 36;

// ---------------------------------------------------------------------------
// Protocol version (4-bit major + 4-bit minor packed into byte 0).
// ---------------------------------------------------------------------------

/// Major version supported by this implementation.
pub const SUPPORTED_MAJOR: u8 = 1;
/// Minor version supported by this implementation.
pub const SUPPORTED_MINOR: u8 = 0;

/// Bit shift for extracting the major version from byte 0.
pub const MAJOR_SHIFT: u8 = 4;
/// Bit mask for extracting the minor version from byte 0.
pub const MINOR_MASK: u8 = 0x0F;

/// Pack the (major, minor) tuple into the single version byte.
#[inline]
#[must_use]
pub const fn pack_version(major: u8, minor: u8) -> u8 {
    (major << MAJOR_SHIFT) | (minor & MINOR_MASK)
}

/// Extract the major version number from byte 0.
#[inline]
#[must_use]
pub const fn major(version_byte: u8) -> u8 {
    version_byte >> MAJOR_SHIFT
}

/// Extract the minor version number from byte 0.
#[inline]
#[must_use]
pub const fn minor(version_byte: u8) -> u8 {
    version_byte & MINOR_MASK
}

/// The version byte that this implementation writes (`SUPPORTED_MAJOR.SUPPORTED_MINOR`).
pub const SUPPORTED_VERSION_BYTE: u8 = pack_version(SUPPORTED_MAJOR, SUPPORTED_MINOR);

// ---------------------------------------------------------------------------
// Flag bit definitions (byte 1).
// ---------------------------------------------------------------------------

/// `bit0` of the flag byte: when set, the buffer carries jsdoccomment-compat
/// extension data on `JsdocBlock` / `JsdocTag` / `JsdocDescriptionLine` /
/// `JsdocTypeLine` (see `design/007-binary-ast/encoding.md`).
pub const COMPAT_MODE_BIT: u8 = 0b0000_0001;

/// Mask for the reserved flag bits (bit1..=bit7). Decoders must ignore bits
/// they do not understand.
pub const FLAGS_RESERVED_MASK: u8 = 0b1111_1110;

// ---------------------------------------------------------------------------
// Header struct (in-memory representation, populated by the writer).
// ---------------------------------------------------------------------------

/// In-memory representation of the binary Header.
///
/// This struct is only used while constructing or inspecting a buffer; the
/// on-wire representation is the 40-byte little-endian byte sequence
/// described at the top of this module. The struct itself does **not** have a
/// guaranteed `repr(C)` layout because individual fields are written through
/// the offset constants rather than via direct struct copy.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// Packed major+minor version byte.
    pub version: u8,
    /// Flag byte (`bit0=compat_mode`, others reserved).
    pub flags: u8,
    /// Byte offset of the root index array (Header end if N=0).
    pub root_array_offset: u32,
    /// Byte offset of the String Offsets section.
    pub string_offsets_offset: u32,
    /// Byte offset of the String Data section.
    pub string_data_offset: u32,
    /// Byte offset of the Extended Data section.
    pub extended_data_offset: u32,
    /// Byte offset of the Diagnostics section.
    pub diagnostics_offset: u32,
    /// Byte offset of the Nodes section.
    pub nodes_offset: u32,
    /// Number of node records (including the `node[0]` sentinel).
    pub node_count: u32,
    /// Total length of the concatenated source texts in UTF-8 bytes.
    pub source_text_length: u32,
    /// Number of roots N in this buffer (1 for the single-comment case).
    pub root_count: u32,
}

impl Header {
    /// Returns whether the `compat_mode` flag bit is set.
    #[inline]
    #[must_use]
    pub const fn compat_mode(&self) -> bool {
        (self.flags & COMPAT_MODE_BIT) != 0
    }

    /// Returns the major version stored in `self.version`.
    #[inline]
    #[must_use]
    pub const fn major(&self) -> u8 {
        major(self.version)
    }

    /// Returns the minor version stored in `self.version`.
    #[inline]
    #[must_use]
    pub const fn minor(&self) -> u8 {
        minor(self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_size_is_40_bytes() {
        // The on-wire size is fixed at 40 bytes regardless of struct padding.
        // This test pins the *layout constant*, not `size_of::<Header>()`.
        assert_eq!(HEADER_SIZE, 40);
    }

    #[test]
    fn header_field_offsets_cover_40_bytes_without_overlap() {
        // Each field must lie inside the 40-byte header, and the order of
        // offsets must match the spec.
        let offsets: &[(usize, usize)] = &[
            (VERSION_OFFSET, 1),
            (FLAGS_OFFSET, 1),
            (RESERVED_OFFSET, 2),
            (ROOT_ARRAY_OFFSET_FIELD, 4),
            (STRING_OFFSETS_OFFSET_FIELD, 4),
            (STRING_DATA_OFFSET_FIELD, 4),
            (EXTENDED_DATA_OFFSET_FIELD, 4),
            (DIAGNOSTICS_OFFSET_FIELD, 4),
            (NODES_OFFSET_FIELD, 4),
            (NODE_COUNT_FIELD, 4),
            (SOURCE_TEXT_LENGTH_FIELD, 4),
            (ROOT_COUNT_FIELD, 4),
        ];

        let mut cursor = 0usize;
        for (offset, size) in offsets {
            assert_eq!(*offset, cursor, "field at expected position");
            cursor += size;
        }
        assert_eq!(cursor, HEADER_SIZE, "fields fully cover the header");
    }

    #[test]
    fn version_pack_unpack_roundtrip() {
        for major in 0u8..=15 {
            for minor in 0u8..=15 {
                let byte = pack_version(major, minor);
                assert_eq!(super::major(byte), major);
                assert_eq!(super::minor(byte), minor);
            }
        }
    }

    #[test]
    fn supported_version_byte_is_0x10() {
        assert_eq!(SUPPORTED_VERSION_BYTE, 0x10);
        assert_eq!(major(SUPPORTED_VERSION_BYTE), 1);
        assert_eq!(minor(SUPPORTED_VERSION_BYTE), 0);
    }

    #[test]
    fn compat_mode_flag_round_trips() {
        let mut h = Header::default();
        assert!(!h.compat_mode());
        h.flags |= COMPAT_MODE_BIT;
        assert!(h.compat_mode());
        // Setting reserved bits must not affect the compat flag query.
        h.flags |= 0b1010_1010 & FLAGS_RESERVED_MASK;
        assert!(h.compat_mode());
    }
}
