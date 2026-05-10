// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

#![allow(clippy::all, clippy::cargo, clippy::nursery, clippy::pedantic)]

//! Binary AST for ox_jsdoc.
//!
//! This crate hosts the Binary AST format specification ([`format`]), the
//! parser-integrated binary writer ([`writer`]), the parser entry point
//! ([`parser`]), the Rust-side lazy decoder ([`decoder`]), and the
//! UTF-8 → UTF-16 position converter ([`utf16`]) used at emit time to
//! satisfy the wire-format requirement that Pos/End are UTF-16 code units.
//!
//! The Binary AST replaces the previous JSON serialization path between the
//! Rust parser and JS bindings. The full design lives under
//! `design/007-binary-ast/`. The format specification itself is in
//! `design/007-binary-ast/format.md`; this crate is the Rust reference
//! implementation for that spec.

pub mod decoder;
pub mod format;
pub mod parser;
pub mod writer;
