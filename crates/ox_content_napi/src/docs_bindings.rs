// BTreeMap keeps JavaScript-facing docs output deterministic.
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use ox_content_docs::{
    build_export_graph, extract_docs_from_directories, extract_docs_from_entry_points,
    generate_docs_data_json, generate_markdown, generate_nav_code, generate_nav_metadata,
    generate_nav_metadata_from_docs_with_options, normalize_doc_items, write_docs_output,
    DocExtractor, DocsNavMetadataOptions,
};

use crate::{
    JsDocEntry, JsDocsMarkdownModule, JsDocsMarkdownOptions, JsDocsNavItem, JsDocsNavOptions,
    JsDocsOutputOptions, JsEntryPointDocsOptions, JsEntryPointSpec, JsEntrypointDocsModule,
    JsExportGraph, JsExtractedDocsModule, JsGraphOptions, JsSourceDocItem,
};

mod graph;
mod markdown;
mod nav;
mod normalized;
mod options;
mod paths;
mod source;

use graph::{map_docs_diagnostic, map_export_graph, map_public_export};
#[cfg(test)]
pub use markdown::convert_markdown_entry;
use markdown::{convert_markdown_module, map_api_doc_tag};
use nav::{convert_docs_nav_item, map_docs_nav_item};
pub use normalized::map_normalized_doc_entry;
use options::{
    convert_docs_output_options, convert_entrypoint_docs_options, convert_entrypoint_spec,
    convert_graph_options, convert_markdown_docs_options, parse_markdown_path_strategy,
    parse_single_entry_root,
};
use paths::path_to_string;
use source::map_doc_item;

/// Extracts documented declarations from a JavaScript/TypeScript file using Oxc.
#[napi]
pub fn extract_file_docs(
    file_path: String,
    include_private: Option<bool>,
    include_internal: Option<bool>,
) -> Result<Vec<JsSourceDocItem>> {
    let extractor = DocExtractor::with_visibility(
        include_private.unwrap_or(false),
        include_internal.unwrap_or(false),
    );
    let items = extractor
        .extract_file(Path::new(&file_path))
        .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(items.into_iter().map(map_doc_item).collect())
}

/// Extracts normalized documentation entries from a JavaScript/TypeScript file using Oxc.
#[napi(js_name = "extractFileDocEntries")]
pub fn extract_file_doc_entries(
    file_path: String,
    include_private: Option<bool>,
    include_internal: Option<bool>,
    type_parameters: Option<bool>,
) -> Result<Vec<JsDocEntry>> {
    let extractor = DocExtractor::with_visibility(
        include_private.unwrap_or(false),
        include_internal.unwrap_or(false),
    );
    let items = extractor
        .extract_file(Path::new(&file_path))
        .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(normalize_doc_items(items, type_parameters.unwrap_or(false))
        .into_iter()
        .map(map_normalized_doc_entry)
        .collect())
}

/// Generates sidebar navigation metadata from documentation file paths.
#[napi(js_name = "generateDocsNavMetadata")]
pub fn generate_docs_nav_metadata(
    files: Vec<String>,
    base_path: Option<String>,
) -> Vec<JsDocsNavItem> {
    generate_nav_metadata(&files, base_path.as_deref()).into_iter().map(map_docs_nav_item).collect()
}

/// Generates sidebar navigation metadata from extracted documentation modules.
///
/// Use this when the output `pathStrategy` is `"typedoc"` so that the navigation
/// tree mirrors the nested module/category/symbol pages.
#[napi(js_name = "generateDocsNavMetadataFromDocs")]
pub fn generate_docs_nav_metadata_from_docs_napi(
    docs: Vec<JsDocsMarkdownModule>,
    options: Option<JsDocsNavOptions>,
) -> Vec<JsDocsNavItem> {
    let options = options.unwrap_or_default();
    let strategy = parse_markdown_path_strategy(options.path_strategy.as_deref());
    let modules = docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>();
    generate_nav_metadata_from_docs_with_options(
        &modules,
        &DocsNavMetadataOptions {
            base_path: options.base_path.as_deref(),
            path_strategy: strategy,
            group_order: options.group_order.as_deref(),
            sort: options.sort.as_deref(),
            sort_entry_points: options.sort_entry_points.unwrap_or(true),
            kind_sort_order: options.kind_sort_order.as_deref(),
            single_entry_root: parse_single_entry_root(options.single_entry_root.as_deref()),
        },
    )
    .into_iter()
    .map(map_docs_nav_item)
    .collect()
}

/// Generates TypeScript source code for documentation navigation metadata.
#[napi(js_name = "generateDocsNavCode")]
pub fn generate_docs_nav_code(
    nav_items: Vec<JsDocsNavItem>,
    export_name: Option<String>,
) -> String {
    let nav_items = nav_items.into_iter().map(convert_docs_nav_item).collect::<Vec<_>>();
    generate_nav_code(&nav_items, export_name.as_deref())
}

/// Collects source files for generated API documentation.
#[napi(js_name = "collectDocsSourceFiles")]
pub fn collect_docs_source_files(
    src_dir: String,
    include: Vec<String>,
    exclude: Vec<String>,
) -> Vec<String> {
    ox_content_docs::collect_source_files(&src_dir, &include, &exclude)
}

/// Extracts normalized documentation entries from source directories using Oxc.
#[napi(js_name = "extractDocsFromDirectories")]
pub fn extract_docs_from_directories_napi(
    src_dirs: Vec<String>,
    include: Vec<String>,
    exclude: Vec<String>,
    include_private: Option<bool>,
    include_internal: Option<bool>,
    type_parameters: Option<bool>,
) -> Result<Vec<JsExtractedDocsModule>> {
    let modules = extract_docs_from_directories(
        &src_dirs,
        &include,
        &exclude,
        include_private.unwrap_or(false),
        include_internal.unwrap_or(false),
        type_parameters.unwrap_or(false),
    )
    .map_err(|err| Error::from_reason(err.to_string()))?;

    Ok(modules
        .into_iter()
        .map(|module| JsExtractedDocsModule {
            file: module.file,
            entries: module.entries.into_iter().map(map_normalized_doc_entry).collect(),
        })
        .collect())
}

/// Builds the public API export graph from entry points.
#[napi(js_name = "buildExportGraph")]
pub fn build_export_graph_napi(
    entry_points: Vec<JsEntryPointSpec>,
    options: Option<JsGraphOptions>,
) -> Result<JsExportGraph> {
    let entry_points = entry_points.into_iter().map(convert_entrypoint_spec).collect::<Vec<_>>();
    let graph = build_export_graph(&entry_points, &convert_graph_options(options))
        .map_err(|error| Error::from_reason(error.to_string()))?;
    Ok(map_export_graph(graph))
}

/// Extracts generated API docs grouped by public entry points.
#[napi(js_name = "extractDocsFromEntryPoints")]
pub fn extract_docs_from_entry_points_napi(
    entry_points: Vec<JsEntryPointSpec>,
    options: Option<JsEntryPointDocsOptions>,
) -> Result<Vec<JsEntrypointDocsModule>> {
    let entry_points = entry_points.into_iter().map(convert_entrypoint_spec).collect::<Vec<_>>();
    let modules =
        extract_docs_from_entry_points(&entry_points, &convert_entrypoint_docs_options(options))
            .map_err(|error| Error::from_reason(error.to_string()))?;

    Ok(modules
        .into_iter()
        .map(|module| JsEntrypointDocsModule {
            name: module.name,
            file: module.file,
            source_path: path_to_string(&module.source_path),
            description: module.description,
            examples: module.examples,
            tags: module.tags.into_iter().map(map_api_doc_tag).collect(),
            entries: module.entries.into_iter().map(map_normalized_doc_entry).collect(),
            exports: module.exports.into_iter().map(map_public_export).collect(),
            diagnostics: module.diagnostics.into_iter().map(map_docs_diagnostic).collect(),
        })
        .collect())
}

/// Generates Markdown API reference pages from extracted documentation entries.
#[napi(js_name = "generateDocsMarkdown")]
#[allow(clippy::disallowed_types)]
pub fn generate_docs_markdown(
    docs: Vec<JsDocsMarkdownModule>,
    options: Option<JsDocsMarkdownOptions>,
) -> HashMap<String, String> {
    let options = convert_markdown_docs_options(options);
    generate_markdown(&docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>(), &options)
        .into_iter()
        .collect()
}

/// Generates the machine-readable docs data JSON payload.
#[napi(js_name = "generateDocsDataJson")]
pub fn generate_docs_data_json_napi(
    docs: Vec<JsDocsMarkdownModule>,
    generated_at: String,
) -> Result<String> {
    generate_docs_data_json(
        &docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>(),
        &generated_at,
    )
    .map_err(|error| Error::from_reason(error.to_string()))
}

/// Writes generated API documentation files and native sidecars.
#[napi(js_name = "writeGeneratedDocs")]
#[allow(clippy::disallowed_types, clippy::implicit_hasher)]
pub fn write_generated_docs(
    docs: HashMap<String, String>,
    out_dir: String,
    extracted_docs: Option<Vec<JsDocsMarkdownModule>>,
    options: Option<JsDocsOutputOptions>,
) -> Result<()> {
    let docs = docs.into_iter().collect::<BTreeMap<_, _>>();
    let extracted_docs = extracted_docs
        .map(|docs| docs.into_iter().map(convert_markdown_module).collect::<Vec<_>>());
    let options = convert_docs_output_options(options);

    write_docs_output(&docs, Path::new(&out_dir), extracted_docs.as_deref(), &options)
        .map_err(|error| Error::from_reason(error.to_string()))
}
