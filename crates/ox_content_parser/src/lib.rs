//! High-performance Markdown parser for Ox Content.
//!
//! This crate provides a fast, arena-allocated Markdown parser following
//! the CommonMark specification with GFM extensions.
//!
//! # Features
//!
//! - Arena-based allocation for zero-copy parsing
//! - CommonMark compliant with GFM extensions
//! - Pluggable architecture for custom syntax extensions
//!
//! # Example
//!
//! ```
//! use ox_content_allocator::Allocator;
//! use ox_content_parser::Parser;
//!
//! let allocator = Allocator::new();
//! let source = "# Hello World\n\nThis is a paragraph.";
//! let parser = Parser::new(&allocator, source);
//! let document = parser.parse();
//! ```

#![deny(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented
    )
)]

/// Lightweight RAII span guard used internally by the parser modules.
///
/// Compiles to `let _ = ();` when the `profile` feature is disabled (the
/// default) so call sites pay nothing. Under `--features profile`, expands
/// to `ox_content_profiler::ScopeGuard::enter(name)` which records the
/// scope timing + allocation delta into the thread-local span tree.
#[cfg(feature = "profile")]
macro_rules! profile_span {
    ($name:literal) => {
        let __ox_profile_guard = ::ox_content_profiler::ScopeGuard::enter($name);
    };
}

#[cfg(not(feature = "profile"))]
macro_rules! profile_span {
    ($name:literal) => {};
}

/// Micro-span variant of [`profile_span!`] for per-node / per-line hot
/// paths, where the guard itself costs as much as the work it measures.
///
/// Records only when the profiler's detail gate is open on top of the
/// regular enable gate (`ox_content_profiler::scope::enable_detail`), so
/// default profiling runs keep phase-level accuracy and `--detail` runs
/// trade it for the full trace.
#[cfg(feature = "profile")]
macro_rules! profile_span_detail {
    ($name:literal) => {
        let __ox_profile_guard = ::ox_content_profiler::ScopeGuard::enter_detail($name);
    };
}

#[cfg(not(feature = "profile"))]
macro_rules! profile_span_detail {
    ($name:literal) => {};
}

pub(crate) use {profile_span, profile_span_detail};

mod error;
mod parser;

pub use error::{ParseError, ParseResult};
pub use parser::{Parser, ParserOptions};

/// Parses Markdown source into an AST.
///
/// This is a convenience function that creates a parser with default options.
pub fn parse<'a>(
    allocator: &'a ox_content_allocator::Allocator,
    source: &'a str,
) -> ParseResult<ox_content_ast::Document<'a>> {
    Parser::new(allocator, source).parse()
}
