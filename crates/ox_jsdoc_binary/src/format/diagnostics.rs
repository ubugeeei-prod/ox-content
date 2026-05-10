// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Diagnostics section constants.
//!
//! See `design/007-binary-ast/format.md#diagnostics-section-4--8m-bytes` for
//! the full layout.
//!
//! ```text
//! byte 0-3      : M (u32)            -- diagnostic count
//! per entry (8 bytes, sorted ascending by root_index):
//!   byte 0-3    : root_index    (u32) -- index into the root array
//!   byte 4-7    : message_index (u32) -- String Offsets index for the message
//! ```

/// Size in bytes of the diagnostic count header at the start of the section.
pub const COUNT_HEADER_SIZE: usize = 4;

/// Size of one diagnostic entry in bytes (`u32 root_index` + `u32 message_index`).
pub const DIAGNOSTIC_ENTRY_SIZE: usize = 8;

/// Byte offset of `root_index` within a diagnostic entry.
pub const ROOT_INDEX_OFFSET: usize = 0;
/// Byte offset of `message_index` within a diagnostic entry.
pub const MESSAGE_INDEX_OFFSET: usize = 4;

/// Compute the total Diagnostics-section size for `m` entries.
#[inline]
#[must_use]
pub const fn section_size(m: usize) -> usize {
    COUNT_HEADER_SIZE + DIAGNOSTIC_ENTRY_SIZE * m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_size_is_8() {
        assert_eq!(DIAGNOSTIC_ENTRY_SIZE, 8);
    }

    #[test]
    fn entry_field_offsets_partition_8_bytes() {
        let layout: &[(usize, usize)] = &[(ROOT_INDEX_OFFSET, 4), (MESSAGE_INDEX_OFFSET, 4)];
        let mut cursor = 0usize;
        for (offset, size) in layout {
            assert_eq!(*offset, cursor);
            cursor += size;
        }
        assert_eq!(cursor, DIAGNOSTIC_ENTRY_SIZE);
    }

    #[test]
    fn section_size_for_zero_entries_is_just_count_header() {
        assert_eq!(section_size(0), COUNT_HEADER_SIZE);
    }

    #[test]
    fn section_size_examples() {
        // Spec example: M=2 -> 4 + 16 = 20 bytes
        assert_eq!(section_size(2), 20);
        // M=10 -> 84 bytes; M=100 -> 804 bytes
        assert_eq!(section_size(10), 84);
        assert_eq!(section_size(100), 804);
    }
}
