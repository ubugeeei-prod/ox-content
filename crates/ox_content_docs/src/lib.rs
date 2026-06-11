//! Source code documentation generator for Ox Content.
//!
//! This crate provides functionality similar to `cargo doc`,
//! generating documentation from source code using OXC parser
//! for JavaScript/TypeScript files.

#![deny(clippy::disallowed_macros)]
#![cfg_attr(test, allow(clippy::disallowed_macros))]

/// Lightweight RAII span guard used internally by the docs generator modules.
///
/// Compiles to nothing when the `profile` feature is disabled (the default)
/// so non-profiling builds pay zero overhead. Under `--features profile`,
/// expands to `ox_content_profiler::ScopeGuard::enter(name)` which records the
/// scope timing + allocation delta into the thread-local span tree. See
/// `ox_content_parser::profile_span` for the same pattern in the parser.
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

mod config;
mod data;
mod extractor;
mod generator;
mod graph;
mod markdown;
mod model;
mod nav;
mod normalize;
mod output;
mod string_builder;

pub use config::DocsConfig;
pub use data::generate_docs_data_json;
pub use extractor::{
    DocExtractor, DocItem, DocItemKind, DocTag, ExtractError, ExtractResult, ParamDoc, TypeParamDoc,
};
pub use generator::{
    collect_source_files, extract_docs_from_directories, DocsGenerator, ExtractedDocModule,
    GenerateError, GenerateResult,
};
pub use graph::{
    build_export_graph, extract_docs_from_entry_points, DocsDiagnostic, DocsDiagnosticCode,
    EntryPointDocsOptions, EntryPointSpec, EntrypointDocsModule, EntrypointModule, ExportGraph,
    ExportKind, ExportSource, ExternalDocsOptions, ExternalPackageSource, GraphError, GraphOptions,
    PublicExport, ResolvedModule,
};
pub use markdown::{
    generate_markdown, MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkStyle,
    MarkdownPathStrategy, MarkdownRenderStyle, MarkdownSingleEntryRoot,
};
pub use model::{
    ApiDocEntry, ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc,
    ApiTypeParamDoc,
};
pub use nav::{
    generate_nav_code, generate_nav_metadata, generate_nav_metadata_from_docs,
    generate_nav_metadata_from_docs_with_options, DocsNavItem, DocsNavMetadataOptions,
};
pub use normalize::{
    normalize_doc_item, normalize_doc_items, NormalizedDocEntry, NormalizedDocKind,
    NormalizedMember, NormalizedMemberKind, NormalizedParamDoc, NormalizedReturnDoc,
    NormalizedThrowsDoc, NormalizedTypeParam,
};
pub use output::{write_docs_output, DocsOutputError, DocsOutputOptions, DocsOutputResult};
