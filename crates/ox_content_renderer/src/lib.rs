//! Markdown renderer for Ox Content.
//!
//! This crate provides a renderer that converts Markdown AST to HTML
//! and other output formats.
//!
//! # Example
//!
//! ```
//! use ox_content_allocator::Allocator;
//! use ox_content_parser::Parser;
//! use ox_content_renderer::HtmlRenderer;
//!
//! let allocator = Allocator::new();
//! let source = "# Hello World\n\nThis is a paragraph.";
//! let parser = ox_content_parser::Parser::new(&allocator, source);
//! let document = parser.parse().unwrap();
//!
//! let mut renderer = HtmlRenderer::new();
//! let html = renderer.render(&document);
//! ```

#![deny(clippy::disallowed_macros)]
#![cfg_attr(test, allow(clippy::disallowed_macros))]

/// Lightweight RAII span guard used internally by the renderer modules.
///
/// Compiles to nothing without the `profile` feature so non-profiling
/// builds pay zero overhead. See `ox_content_parser::profile_span` for the
/// same pattern on the parser side.
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

pub(crate) use profile_span;

#[cfg(feature = "frameworks")]
pub mod frameworks;
mod html;
mod render;

#[cfg(feature = "frameworks")]
pub use frameworks::{
    escape_svelte_markup, render_framework_component_code, FrameworkCodegenError,
    FrameworkCodegenTarget, FrameworkComponentIsland,
};
pub use html::{CodeAnnotationSyntax, HtmlRenderer, HtmlRendererOptions};
pub use render::{RenderError, RenderResult, Renderer};
