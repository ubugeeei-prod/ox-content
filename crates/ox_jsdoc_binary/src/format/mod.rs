// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Binary AST format specification (constants, layouts, type definitions).
//!
//! The format is documented in detail in
//! `design/007-binary-ast/format.md`; this module provides the canonical
//! Rust constants and types corresponding to that spec.
//!
//! Layout overview:
//!
//! ```text
//! [Header (40 bytes)] [Root Index Array (12N)] [String Offsets (8K)]
//! [String Data] [Extended Data] [Diagnostics (4 + 8M)] [Nodes (24P)]
//! ```
//!
//! Each section has its own submodule that exposes the constants required by
//! both the writer (Phase 1.1a) and the lazy decoder (Phase 1.1b).

pub mod diagnostics;
pub mod extended_data;
pub mod header;
pub mod kind;
pub mod node_record;
pub mod root_index;
pub mod string_field;
pub mod string_table;
