//! Arena allocator for Ox Content.
//!
//! This crate provides a high-performance arena allocator based on bumpalo,
//! designed for efficient memory management during parsing operations.

#![deny(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

use std::ops::Deref;

pub use bumpalo::Bump;

/// Arena allocator wrapper for Ox Content.
///
/// This type wraps bumpalo's `Bump` allocator to provide fast, arena-based
/// allocation for AST nodes and other parsing-related data structures.
#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}

impl Allocator {
    /// Creates a new allocator with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Creates a new allocator with the specified capacity in bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { bump: Bump::with_capacity(capacity) }
    }

    /// Creates a new allocator pre-sized for parsing a Markdown source of
    /// the given length. The capacity is a heuristic (`source_len * 8`
    /// bytes, with a 4 KB floor) that covers the typical AST footprint
    /// for real-world Markdown without growing through bumpalo's
    /// chunk-doubling path — on a fresh [`Self::new`], that path accounts
    /// for ~10 global allocations on a 64 KB document.
    ///
    /// Callers that already know the input length should prefer this over
    /// [`Self::new`]: same fallible-only allocator API, but typically one
    /// arena chunk for the whole parse + render pipeline.
    #[must_use]
    pub fn for_source_len(source_len: usize) -> Self {
        // The 8× factor is empirical: across the bundled corpora
        // (rust-book / vite / vue / typescript-handbook) the AST + render
        // output combined comes in between 5× and 7× of the source
        // length. 8× errs slightly on the over-allocation side so the
        // first chunk almost always suffices.
        const BYTES_PER_INPUT_BYTE: usize = 8;
        const MIN_CAPACITY: usize = 4 * 1024;
        let capacity = source_len.saturating_mul(BYTES_PER_INPUT_BYTE).max(MIN_CAPACITY);
        Self::with_capacity(capacity)
    }

    /// Returns the underlying bump allocator.
    #[must_use]
    pub fn bump(&self) -> &Bump {
        &self.bump
    }

    /// Allocates a value in the arena and returns a reference to it.
    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.bump.alloc(val)
    }

    /// Allocates a string in the arena.
    pub fn alloc_str(&self, s: &str) -> &str {
        self.bump.alloc_str(s)
    }

    /// Creates a new `Vec` in the arena.
    pub fn new_vec<T>(&self) -> Vec<'_, T> {
        Vec::new_in(&self.bump)
    }

    /// Creates a new `Vec` in the arena with the given capacity.
    pub fn new_vec_with_capacity<T>(&self, capacity: usize) -> Vec<'_, T> {
        Vec::with_capacity_in(capacity, &self.bump)
    }

    /// Creates a new `String` in the arena.
    pub fn new_string(&self) -> String<'_> {
        String::new_in(&self.bump)
    }

    /// Creates a new `String` in the arena from a `&str`.
    pub fn new_string_from(&self, s: &str) -> String<'_> {
        String::from_str_in(s, &self.bump)
    }

    /// Resets the allocator, freeing all allocated memory.
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Returns the total bytes allocated in this arena.
    #[must_use]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }
}

impl Deref for Allocator {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.bump
    }
}

/// A boxed value allocated in an arena.
pub type Box<'a, T> = bumpalo::boxed::Box<'a, T>;

/// A vector allocated in an arena.
pub type Vec<'a, T> = bumpalo::collections::Vec<'a, T>;

/// A string allocated in an arena.
pub type String<'a> = bumpalo::collections::String<'a>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator_creation() {
        let allocator = Allocator::new();
        assert_eq!(allocator.allocated_bytes(), 0);
    }

    #[test]
    fn test_alloc_value() {
        let allocator = Allocator::new();
        let value = allocator.alloc(42);
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_alloc_str() {
        let allocator = Allocator::new();
        let s = allocator.alloc_str("hello");
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_arena_vec() {
        let allocator = Allocator::new();
        let mut vec = allocator.new_vec();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_arena_string() {
        let allocator = Allocator::new();
        let mut s = allocator.new_string();
        s.push_str("hello");
        s.push_str(" world");
        assert_eq!(s.as_str(), "hello world");
    }
}
