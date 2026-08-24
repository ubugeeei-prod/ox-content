//! AST definitions for Ox Content Markdown parser.
//!
//! This crate defines the Abstract Syntax Tree (AST) for Markdown documents,
//! designed to be compatible with mdast (Markdown AST) specification while
//! providing efficient arena-based allocation.

#![deny(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

mod ast;
mod mdx;
mod span;
mod visit;

pub use ast::*;
pub use mdx::*;
pub use span::*;
pub use visit::*;

/// The AST must own nothing outside the arena.
///
/// [`ox_content_allocator::Vec`] never runs its elements' destructors, which
/// is what keeps dropping a parsed document from walking the whole tree. A
/// field that owned heap memory — a `std::string::String`, a `std::boxed::Box`,
/// an `Rc` — would therefore leak it on every parse instead of being reclaimed
/// with the `Bump`. Large variants are interned behind
/// [`ox_content_allocator::Box`], which is a thin pointer with no destructor;
/// every other field is `Copy` or another arena vector. This is where that
/// stops being a convention: giving any node a heap-owning field makes `Node`
/// need dropping and fails the build here.
///
/// The check has to name concrete types. Written generically inside
/// `Vec::new_in` it is simply never forced, so it would pass while leaking.
const _AST_IS_ARENA_ONLY: () = {
    assert!(!std::mem::needs_drop::<Node<'static>>());
    assert!(!std::mem::needs_drop::<Document<'static>>());
    // Boxed container variants keep `Node` at the size of Text/Html:
    // `&str` + `Span` is 24 bytes, plus the enum discriminant rounds the
    // slot to 32. Paragraph/Emphasis/etc. still exist, just behind a
    // pointer, so the arrays that dominate a parse pack two cells per
    // cache line instead of carrying 40-byte Vec+Span holes.
    assert!(std::mem::size_of::<Node<'static>>() <= 32);
    assert!(std::mem::size_of::<Paragraph<'static>>() > std::mem::size_of::<Node<'static>>());
    assert!(std::mem::size_of::<Definition<'static>>() > std::mem::size_of::<Node<'static>>());
    assert!(
        std::mem::size_of::<MdxJsxFlowElement<'static>>() > std::mem::size_of::<Node<'static>>()
    );
    assert!(
        std::mem::size_of::<MdxJsxTextElement<'static>>() > std::mem::size_of::<Node<'static>>()
    );
};

#[cfg(test)]
mod mdx_tests;
