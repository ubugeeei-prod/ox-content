// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! String table (String Offsets + String Data) constants.
//!
//! See `design/007-binary-ast/format.md#string-table` for the full layout.
//!
//! - **String Offsets**: 8 bytes per string (`u32 start, u32 end`).
//!   Used by string-leaf nodes (TypeTag::String) and the diagnostics section.
//! - **String Data**: contiguous UTF-8 bytes; strings are referenced either
//!   by an index into the String Offsets table (string-leaf path, capped at
//!   2³⁰−2 by the 30-bit Node Data payload + the
//!   [`crate::format::node_record::STRING_PAYLOAD_NONE_SENTINEL`] sentinel)
//!   or by an inline [`crate::format::string_field::StringField`] slot
//!   (Extended Data path) which embeds `(offset, length)` directly without
//!   the indirection.

/// Size of one String Offsets entry in bytes (`u32 start` + `u32 end`).
pub const STRING_OFFSET_ENTRY_SIZE: usize = 8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_size_is_8() {
        assert_eq!(STRING_OFFSET_ENTRY_SIZE, 8);
    }
}
