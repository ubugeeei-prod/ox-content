// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Parser state snapshots.
//!
//! Verbatim port of `crates/ox_jsdoc/src/parser/checkpoint.rs`. Checkpoints
//! allow speculative parsing without cloning the arena. Rewind restores
//! scalar parser state and truncates diagnostics emitted after the
//! checkpoint.

/// Quote mode used while scanning nested constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteKind {
    /// `'...'` single-quoted string.
    Single,
    /// `"..."` double-quoted string.
    Double,
    /// `` `...` `` template literal.
    Backtick,
}

/// Active fenced code block state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FenceState {
    /// Number of backticks that opened the current fence.
    pub tick_count: u8,
}

/// Rewindable parser state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Checkpoint {
    /// Parser byte offset relative to the parsed comment slice.
    pub offset: u32,
    /// Current nested `{...}` depth.
    pub brace_depth: u16,
    /// Current nested `[...]` depth.
    pub bracket_depth: u16,
    /// Current nested `(...)` depth.
    pub paren_depth: u16,
    /// Active quote context, if scanning inside a quoted region.
    pub quote: Option<QuoteKind>,
    /// Active fenced code block context, if any.
    pub fence: Option<FenceState>,
    /// Diagnostic list length at the time the checkpoint was captured.
    pub diagnostics_len: usize,
}
