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

mod html;
mod render;

pub use html::{CodeAnnotationSyntax, HtmlRenderer, HtmlRendererOptions};
pub use render::{RenderError, RenderResult, Renderer};
