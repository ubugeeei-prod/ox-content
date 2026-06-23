use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use ox_content_transform::transformer::MarkdownTransformer;
use serde_json::{json, Map, Value};

use crate::JsTransformOptions;

mod entry_metadata;
mod path_match;

use entry_metadata::{
    collection_path, extract_first_heading, first_h1_from_toc, format_title_from_path,
    markdown_extension, normalize_markdown_extensions, string_field, toc_entry_to_value,
};
use path_match::{natural_compare, normalize_source_patterns, path_to_slash};

#[napi(object)]
#[derive(Clone)]
pub struct JsCollectionDefinition {
    /// Collection name.
    pub name: String,

    /// Glob-like source patterns relative to `srcDir`.
    pub source: Vec<String>,

    /// Extra payload fields: `body`, `html`, and/or `toc`.
    pub include: Vec<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct JsBuildCollectionManifestOptions {
    /// Source directory containing Markdown files.
    pub src_dir: String,

    /// Markdown extensions to collect.
    pub extensions: Vec<String>,

    /// Parse YAML frontmatter before building entries.
    pub frontmatter: Option<bool>,

    /// Collection definitions.
    pub collections: Vec<JsCollectionDefinition>,

    /// Rust Markdown transform options used only when a collection includes `html` or `toc`.
    pub transform_options: Option<JsTransformOptions>,
}

#[derive(Clone)]
struct SourceFile {
    path: PathBuf,
    path_key: String,
    relative_path: String,
    extension: String,
}

#[derive(Clone)]
struct PreparedFile {
    content: String,
    frontmatter: Map<String, Value>,
}

struct TransformedFile {
    html: String,
    toc: Vec<Value>,
}

struct BuildEntryContext<'a> {
    extensions: &'a [String],
    include: &'a IncludeSet,
    prepare_transformer: &'a MarkdownTransformer,
    base_transform_options: &'a JsTransformOptions,
    prepared_cache: &'a mut HashMap<String, PreparedFile>,
    transformed_cache: &'a mut HashMap<String, TransformedFile>,
}

#[derive(Default)]
struct IncludeSet {
    body: bool,
    html: bool,
    toc: bool,
}

/// Builds a Markdown collection manifest directly from files on the Rust side.
///
/// File discovery, pattern filtering, frontmatter preparation, route metadata,
/// and optional raw Rust Markdown transforms happen in one native call to avoid
/// per-file JS/NAPI round trips on large content directories.
#[napi(js_name = "buildCollectionManifest")]
pub fn build_collection_manifest(options: JsBuildCollectionManifestOptions) -> Result<String> {
    let src_dir = PathBuf::from(&options.src_dir);
    let extensions = normalize_markdown_extensions(&options.extensions);
    let files =
        if src_dir.is_dir() { collect_source_files(&src_dir, &extensions) } else { Vec::new() };
    let frontmatter = options.frontmatter.unwrap_or(true);
    let prepare_transformer = MarkdownTransformer::with_frontmatter(frontmatter);
    let mut prepared_cache = HashMap::new();
    let mut transformed_cache = HashMap::new();
    let base_transform_options = options.transform_options.unwrap_or_default();

    let mut collections = Map::new();
    for collection in options.collections {
        let patterns = normalize_source_patterns(&collection.source);
        let include = IncludeSet::from_fields(&collection.include);
        let mut entries = Vec::new();
        let mut context = BuildEntryContext {
            extensions: &extensions,
            include: &include,
            prepare_transformer: &prepare_transformer,
            base_transform_options: &base_transform_options,
            prepared_cache: &mut prepared_cache,
            transformed_cache: &mut transformed_cache,
        };

        for file in &files {
            if !patterns.iter().any(|pattern| pattern.matches(&file.relative_path)) {
                continue;
            }

            let entry = build_entry(file, &collection.name, &mut context)?;
            entries.push(entry);
        }

        collections.insert(collection.name, Value::Array(entries));
    }

    serde_json::to_string(&json!({ "collections": collections }))
        .map_err(|error| Error::from_reason(error.to_string()))
}

fn build_entry(
    file: &SourceFile,
    collection_name: &str,
    context: &mut BuildEntryContext<'_>,
) -> Result<Value> {
    let prepared = get_prepared_file(file, context.prepare_transformer, context.prepared_cache)?;
    let route_path = collection_path(&file.relative_path, context.extensions);
    let stem = route_path.strip_prefix('/').unwrap_or(&route_path).to_string();
    let transformed = if context.include.html || context.include.toc {
        Some(get_transformed_file(file, context.base_transform_options, context.transformed_cache)?)
    } else {
        None
    };
    let title = string_field(&prepared.frontmatter, "title")
        .or_else(|| first_h1_from_toc(transformed.as_ref().map(|item| item.toc.as_slice())))
        .or_else(|| extract_first_heading(&prepared.content))
        .unwrap_or_else(|| format_title_from_path(if stem.is_empty() { "index" } else { &stem }));

    let mut entry = prepared.frontmatter.clone();
    entry.insert("id".to_string(), json!(stem));
    entry.insert("collection".to_string(), json!(collection_name));
    entry.insert("path".to_string(), json!(route_path));
    entry.insert("stem".to_string(), json!(stem));
    entry.insert("source".to_string(), json!(file.relative_path));
    entry.insert("extension".to_string(), json!(file.extension));
    entry.insert("title".to_string(), json!(title));
    if let Some(description) = string_field(&prepared.frontmatter, "description") {
        entry.insert("description".to_string(), json!(description));
    }
    entry.insert("frontmatter".to_string(), Value::Object(prepared.frontmatter.clone()));

    if context.include.body {
        entry.insert("body".to_string(), json!(prepared.content));
    }
    if let Some(transformed) = transformed {
        if context.include.html {
            entry.insert("html".to_string(), json!(transformed.html));
        }
        if context.include.toc {
            entry.insert("toc".to_string(), Value::Array(transformed.toc.clone()));
        }
    }

    Ok(Value::Object(entry))
}

fn get_prepared_file(
    file: &SourceFile,
    transformer: &MarkdownTransformer,
    cache: &mut HashMap<String, PreparedFile>,
) -> Result<PreparedFile> {
    if let Some(prepared) = cache.get(&file.path_key) {
        return Ok(prepared.clone());
    }

    let source = fs::read_to_string(&file.path).map_err(|error| {
        Error::from_reason(format!("failed to read {}: {error}", file.path_key))
    })?;
    let prepared = transformer.prepare_source(&source);
    let prepared = PreparedFile {
        content: prepared.content,
        frontmatter: prepared.frontmatter.into_iter().collect(),
    };
    cache.insert(file.path_key.clone(), prepared.clone());
    Ok(prepared)
}

fn get_transformed_file<'a>(
    file: &SourceFile,
    base_options: &JsTransformOptions,
    cache: &'a mut HashMap<String, TransformedFile>,
) -> Result<&'a TransformedFile> {
    if !cache.contains_key(&file.path_key) {
        let source = fs::read_to_string(&file.path).map_err(|error| {
            Error::from_reason(format!("failed to read {}: {error}", file.path_key))
        })?;
        let mut options = base_options.clone();
        options.source_path = Some(file.path_key.clone());
        let core_options = options.into();
        let result = MarkdownTransformer::from_options(&core_options).transform(&source);
        let transformed = TransformedFile {
            html: result.html,
            toc: result.toc.into_iter().map(toc_entry_to_value).collect(),
        };
        cache.insert(file.path_key.clone(), transformed);
    }

    Ok(cache.get(&file.path_key).expect("transformed file should be cached"))
}

fn collect_source_files(src_dir: &Path, extensions: &[String]) -> Vec<SourceFile> {
    let mut files = Vec::new();
    collect_source_files_inner(src_dir, src_dir, extensions, &mut files);
    files.sort_by(|left, right| natural_compare(&left.relative_path, &right.relative_path));
    files
}

fn collect_source_files_inner(
    root: &Path,
    dir: &Path,
    extensions: &[String],
    files: &mut Vec<SourceFile>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if file_type.is_dir() {
            if name.starts_with('.') || name == "node_modules" {
                continue;
            }
            collect_source_files_inner(root, &path, extensions, files);
        } else if file_type.is_file() && !name.starts_with('.') {
            if let Some(extension) = markdown_extension(&path_to_slash(&path), extensions) {
                let relative = path.strip_prefix(root).unwrap_or(&path);
                files.push(SourceFile {
                    path_key: path_to_slash(&path),
                    relative_path: path_to_slash(relative),
                    path,
                    extension,
                });
            }
        }
    }
}

impl IncludeSet {
    fn from_fields(fields: &[String]) -> Self {
        Self {
            body: fields.iter().any(|field| field == "body"),
            html: fields.iter().any(|field| field == "html"),
            toc: fields.iter().any(|field| field == "toc"),
        }
    }
}
