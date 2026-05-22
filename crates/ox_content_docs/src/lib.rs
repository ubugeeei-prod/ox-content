//! Source code documentation generator for Ox Content.
//!
//! This crate provides functionality similar to `cargo doc`,
//! generating documentation from source code using OXC parser
//! for JavaScript/TypeScript files.

mod config;
mod data;
mod extractor;
mod generator;
mod markdown;
mod nav;
mod normalize;

pub use config::DocsConfig;
pub use data::generate_docs_data_json;
pub use extractor::{
    DocExtractor, DocItem, DocItemKind, DocTag, ExtractError, ExtractResult, ParamDoc,
};
pub use generator::{collect_source_files, DocsGenerator, GenerateError, GenerateResult};
pub use markdown::{
    generate_markdown, ApiDocEntry, ApiDocModule, ApiDocTag, ApiParamDoc, ApiReturnDoc,
    MarkdownDocsOptions,
};
pub use nav::{generate_nav_code, generate_nav_metadata, DocsNavItem};
pub use normalize::{
    normalize_doc_item, normalize_doc_items, NormalizedDocEntry, NormalizedDocKind,
    NormalizedParamDoc, NormalizedReturnDoc,
};
