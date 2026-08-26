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
mod module_routes;
mod nav;
mod normalize;
mod openapi;
mod output;
mod string_builder;

pub use config::DocsConfig;
pub use data::generate_docs_data_json;
pub use extractor::{
    DocExtractor, DocItem, DocItemKind, DocTag, ExtractError, ExtractResult, ParamDoc, TypeParamDoc,
};
pub use generator::{
    DocsGenerator, ExtractedDocModule, GenerateError, GenerateResult, collect_source_files,
    extract_docs_from_directories,
};
pub use graph::{
    DocsDiagnostic, DocsDiagnosticCode, EntryPointDocsOptions, EntryPointSpec,
    EntrypointDocsModule, EntrypointModule, ExportGraph, ExportKind, ExportSource,
    ExternalDocsOptions, ExternalPackageSource, GraphError, GraphOptions, PublicExport,
    ResolvedModule, build_export_graph, extract_docs_from_entry_points,
};
pub use markdown::{
    MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkStyle, MarkdownPathStrategy,
    MarkdownRenderStyle, MarkdownSingleEntryRoot, generate_markdown,
};
pub use model::{
    ApiDocEntry, ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc,
    ApiTypeParamDoc,
};
pub use nav::{
    DocsNavItem, DocsNavMetadataOptions, generate_nav_code, generate_nav_metadata,
    generate_nav_metadata_from_docs, generate_nav_metadata_from_docs_with_options,
};
pub use normalize::{
    NormalizedDocEntry, NormalizedDocKind, NormalizedMember, NormalizedMemberKind,
    NormalizedParamDoc, NormalizedReturnDoc, NormalizedThrowsDoc, NormalizedTypeParam,
    normalize_doc_item, normalize_doc_items,
};
pub use openapi::{
    GeneratedOpenApiDocs, OpenApiDocsError, OpenApiDocsOptions, OpenApiDocsResult,
    OpenApiSpecInput, generate_openapi_docs,
};
pub use output::{DocsOutputError, DocsOutputOptions, DocsOutputResult, write_docs_output};
