//! Source code documentation generator for Ox Content.
//!
//! This crate provides functionality similar to `cargo doc`,
//! generating documentation from source code using OXC parser
//! for JavaScript/TypeScript files.

mod config;
mod data;
mod extractor;
mod generator;
mod graph;
mod markdown;
mod nav;
mod normalize;
mod output;

pub use config::DocsConfig;
pub use data::generate_docs_data_json;
pub use extractor::{
    DocExtractor, DocItem, DocItemKind, DocTag, ExtractError, ExtractResult, ParamDoc,
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
    generate_markdown, ApiDocEntry, ApiDocMember, ApiDocModule, ApiDocTag, ApiParamDoc,
    ApiReturnDoc, MarkdownDocsOptions, MarkdownLinkStyle,
};
pub use nav::{generate_nav_code, generate_nav_metadata, DocsNavItem};
pub use normalize::{
    normalize_doc_item, normalize_doc_items, NormalizedDocEntry, NormalizedDocKind,
    NormalizedMember, NormalizedMemberKind, NormalizedParamDoc, NormalizedReturnDoc,
};
pub use output::{write_docs_output, DocsOutputError, DocsOutputOptions, DocsOutputResult};
