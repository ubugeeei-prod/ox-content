//! Incremental Markdown parsing and rendering for streaming UI.
//!
//! This crate keeps the one-shot parser and renderer fast paths untouched. It
//! owns the state needed by append-only streaming use cases and feeds only
//! stable Markdown prefixes through the existing core parser/renderer.

mod boundary;
mod parser;
mod provisional;
mod renderer;
mod result;

pub use boundary::stable_prefix_len;
pub use parser::IncrementalParser;
pub use provisional::complete_provisional_markdown;
pub use renderer::IncrementalHtmlRenderer;
pub use result::{IncrementalParseResult, IncrementalRenderOptions, IncrementalRenderResult};
