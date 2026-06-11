//! Documentation extraction from source code using OXC parser.

mod context;
mod declarations;
mod driver;
mod items;
mod jsdoc;
mod jsdoc_tags;
mod members;
mod model;
mod params;
mod returns;
mod signatures;
mod tags;
mod type_parameters;
mod types;
mod visit;

#[cfg(test)]
use oxc_span::SourceType;
use rustc_hash::FxHashMap;

use self::jsdoc::ParsedJsdoc;
use self::model::FunctionTypeMetadata;
pub use self::model::{
    DocItem, DocItemKind, DocTag, ExtractError, ExtractResult, ParamDoc, TypeParamDoc,
};

/// Documentation extractor.
pub struct DocExtractor {
    /// Include private items.
    include_private: bool,
    /// Include internal items.
    include_internal: bool,
    /// Include declarations without JSDoc. Used for public entry point exports.
    include_undocumented_declarations: bool,
    /// Capture the verbatim JSDoc comment text into [`DocItem::jsdoc`].
    ///
    /// Only the raw `extract_file_docs` NAPI path reads it; the normalize-bound
    /// paths (directory + entry-point extraction) discard it, so they opt out
    /// to skip a per-comment allocation and a per-declaration clone.
    capture_jsdoc_raw: bool,
}

/// AST visitor for extracting documentation.
struct DocVisitor<'a> {
    source: &'a str,
    file_path: &'a str,
    include_private: bool,
    include_internal: bool,
    include_undocumented_declarations: bool,
    jsdoc_cache: FxHashMap<u32, ParsedJsdoc>,
    line_starts: Vec<usize>,
    items: Vec<DocItem>,
    type_alias_function_metadata: FxHashMap<String, FunctionTypeMetadata>,
    /// Track default export
    has_default_export: bool,
}

#[cfg(test)]
mod tests;
