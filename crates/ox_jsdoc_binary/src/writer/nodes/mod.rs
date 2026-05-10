// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Per-Kind `write_*` helpers (Phase 1.0b: signatures only).
//!
//! There is one `write_*` function per concrete node Kind (15 comment AST
//! kinds + 45 TypeNode kinds = 60 helpers, organized in two submodules).
//! `Sentinel` and `NodeList` are emitted internally by [`super::BinaryWriter`]
//! and have no dedicated helper.
//!
//! Each helper:
//! 1. Resolves a [`NodeIndex`] for the new node (= `nodes_buffer.len() / 24`),
//! 2. Writes a 24-byte node record (`Kind` + Common Data + padding +
//!    Pos/End + Node Data + parent_index + next_sibling),
//! 3. For Extended-type Kinds, reserves Extended Data via
//!    [`super::ExtendedDataBuilder::reserve`] and writes the per-Kind
//!    payload,
//! 4. Backpatches the previous sibling's `next_sibling` field if needed,
//! 5. Returns the new [`NodeIndex`] so the caller can wire it as a child.
//!
//! Phase 1.0b only ships the function signatures with `unimplemented!()`
//! bodies — Phase 1.1a will fill them in following the per-Kind layouts in
//! `format::node_record` and the spec text in
//! `design/007-binary-ast/format.md`.

pub mod comment_ast;
pub mod type_node;

use core::num::NonZeroU32;

/// Index into the **Nodes** section.
///
/// Newtype around `u32` (with `0` reserved for the `node[0]` sentinel; the
/// wrapper itself uses `NonZeroU32` storage so `Option<NodeIndex>` is
/// 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeIndex(NonZeroU32);

impl NodeIndex {
    /// Construct a [`NodeIndex`] from a raw `u32`. Returns `None` when
    /// `value == 0` (the sentinel slot).
    #[inline]
    #[must_use]
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(nz) => Some(NodeIndex(nz)),
            None => None,
        }
    }

    /// Get the raw `u32` index.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_index_round_trips() {
        for raw in [1u32, 2, 100, u32::MAX] {
            let idx = NodeIndex::new(raw).unwrap();
            assert_eq!(idx.as_u32(), raw);
        }
    }

    #[test]
    fn node_index_zero_is_sentinel() {
        assert!(NodeIndex::new(0).is_none());
    }
}
