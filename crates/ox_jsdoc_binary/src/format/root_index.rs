// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Root index array constants (12 bytes per root).
//!
//! See `design/007-binary-ast/format.md#root-index-array-12n-bytes` for the
//! full layout.
//!
//! ```text
//! byte 0-3   : node_index             (u32, 0 = parse failure sentinel)
//! byte 4-7   : source_offset_in_data  (u32, byte offset into String Data)
//! byte 8-11  : base_offset            (u32, original-file absolute offset)
//! ```

/// Size of one root index entry in bytes.
pub const ROOT_INDEX_ENTRY_SIZE: usize = 12;

/// Byte offset of `node_index` within a root index entry.
pub const NODE_INDEX_OFFSET: usize = 0;
/// Byte offset of `source_offset_in_data` within a root index entry.
pub const SOURCE_OFFSET_FIELD: usize = 4;
/// Byte offset of `base_offset` within a root index entry.
pub const BASE_OFFSET_FIELD: usize = 8;

/// Sentinel value for `node_index` indicating that the corresponding comment
/// failed to parse. The `Diagnostics` section is required to contain at least
/// one entry with the matching `root_index` (see
/// `design/007-binary-ast/format.md#root-index-array-12n-bytes` "Required on
/// failure").
pub const PARSE_FAILURE_SENTINEL: u32 = 0;

/// Compute the total root-index-array size for `n` roots.
#[inline]
#[must_use]
pub const fn section_size(n: usize) -> usize {
    ROOT_INDEX_ENTRY_SIZE * n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_size_is_12() {
        assert_eq!(ROOT_INDEX_ENTRY_SIZE, 12);
    }

    #[test]
    fn entry_field_offsets_partition_12_bytes() {
        let layout: &[(usize, usize)] =
            &[(NODE_INDEX_OFFSET, 4), (SOURCE_OFFSET_FIELD, 4), (BASE_OFFSET_FIELD, 4)];
        let mut cursor = 0usize;
        for (offset, size) in layout {
            assert_eq!(*offset, cursor);
            cursor += size;
        }
        assert_eq!(cursor, ROOT_INDEX_ENTRY_SIZE);
    }

    #[test]
    fn parse_failure_sentinel_is_zero() {
        // node[0] is also the all-zero sentinel; reusing `0` for failure is
        // explicitly part of the spec.
        assert_eq!(PARSE_FAILURE_SENTINEL, 0);
    }

    #[test]
    fn section_size_examples() {
        // Spec example: N=2 -> 24 bytes
        assert_eq!(section_size(2), 24);
        // N=1 (single comment): 12 bytes
        assert_eq!(section_size(1), 12);
        // N=0 (degenerate): 0 bytes
        assert_eq!(section_size(0), 0);
    }
}
