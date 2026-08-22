//! AST definitions for Ox Content Markdown parser.
//!
//! This crate defines the Abstract Syntax Tree (AST) for Markdown documents,
//! designed to be compatible with mdast (Markdown AST) specification while
//! providing efficient arena-based allocation.

#![deny(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

mod ast;
mod span;
mod visit;

pub use ast::*;
pub use span::*;
pub use visit::*;

/// The AST must own nothing outside the arena.
///
/// [`ox_content_allocator::Vec`] never runs its elements' destructors, which
/// is what keeps dropping a parsed document from walking the whole tree. A
/// field that owned heap memory — a `std::string::String`, a `Box`, an `Rc` —
/// would therefore leak it on every parse instead of being reclaimed with the
/// `Bump`. Every field in [`Node`] is either `Copy` or another arena vector,
/// and this is where that stops being a convention: giving any node a
/// heap-owning field makes `Node` need dropping and fails the build here.
///
/// The check has to name concrete types. Written generically inside
/// `Vec::new_in` it is simply never forced, so it would pass while leaking.
const _AST_IS_ARENA_ONLY: () = {
    assert!(!std::mem::needs_drop::<Node<'static>>());
    assert!(!std::mem::needs_drop::<Document<'static>>());
};
