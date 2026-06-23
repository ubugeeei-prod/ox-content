//! Node.js bindings for Ox Content.
//!
//! This crate provides NAPI bindings for using Ox Content from Node.js,
//! including raw-buffer AST transfer for JavaScript interoperability.

mod collection_bindings;
mod docs_bindings;
mod docs_graph_types;
mod docs_markdown_types;
mod docs_source_types;
mod framework_codegen;
mod i18n_bindings;
mod incremental;
mod incremental_result;
mod incremental_types;
mod lint;
mod mermaid_bindings;
mod og_image_bindings;
mod parse_bindings;
mod search_bindings;
mod ssg_bindings;
mod ssg_page_types;
mod ssg_theme_types;
mod transform_bindings;

pub use collection_bindings::*;
pub use docs_bindings::{
    build_export_graph_napi, collect_docs_source_files, extract_docs_from_directories_napi,
    extract_docs_from_entry_points_napi, extract_file_doc_entries, extract_file_docs,
    generate_docs_data_json_napi, generate_docs_markdown, generate_docs_nav_code,
    generate_docs_nav_metadata, generate_docs_nav_metadata_from_docs_napi, write_generated_docs,
};
pub use docs_graph_types::*;
pub use docs_markdown_types::*;
pub use docs_source_types::*;
pub use framework_codegen::*;
pub use i18n_bindings::*;
pub use incremental::{IncrementalMarkdownParser, IncrementalMarkdownRenderer};
pub use incremental_types::{
    IncrementalMarkdownParseResult, IncrementalMarkdownRenderResult, JsIncrementalParseOptions,
    JsIncrementalRenderOptions,
};
pub use lint::*;
pub use mermaid_bindings::*;
pub use og_image_bindings::*;
pub use parse_bindings::*;
pub use search_bindings::*;
pub use ssg_bindings::*;
pub use ssg_page_types::*;
pub use ssg_theme_types::*;
pub use transform_bindings::*;

use ox_content_allocator::Allocator;

pub(crate) fn create_allocator_for_source(source: &str) -> Allocator {
    // NAPI parse/render calls know the full Markdown string length before
    // parsing. Use the shared source-length heuristic so synchronous native
    // calls start with one appropriately sized bump chunk instead of growing
    // from `Bump::new()` while JavaScript is blocked.
    Allocator::for_source_len(source.len())
}

#[cfg(test)]
mod tests;
