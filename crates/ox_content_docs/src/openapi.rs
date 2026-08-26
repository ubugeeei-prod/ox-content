//! Static OpenAPI reference generation.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use thiserror::Error;

use crate::nav::DocsNavItem;

mod inspect;
mod render;
mod routes;
#[cfg(test)]
mod tests;
mod value;

const SPEC_SLUG_PLACEHOLDER: &str = "{spec}";

/// One local OpenAPI document to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenApiSpecInput {
    /// Local JSON or YAML file path, resolved against `OpenApiDocsOptions::root`.
    pub path: PathBuf,
    /// Optional display name. Defaults to `info.title` or the file stem.
    pub name: Option<String>,
    /// Whether unresolved or remote `$ref` values fail generation.
    pub fail_on_unresolved_refs: bool,
}

impl Default for OpenApiSpecInput {
    fn default() -> Self {
        Self { path: PathBuf::new(), name: None, fail_on_unresolved_refs: true }
    }
}

/// Options shared by all OpenAPI inputs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenApiDocsOptions {
    /// Project root used for path-safety checks. Defaults to the current directory.
    pub root: Option<PathBuf>,
    /// Route prefix used by generated nav paths.
    pub base_path: Option<String>,
}

/// Generated OpenAPI Markdown pages and sidebar metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneratedOpenApiDocs {
    /// Markdown files keyed by path relative to the docs output directory.
    pub pages: BTreeMap<String, String>,
    /// Nav items that point at the generated files.
    pub nav_items: Vec<DocsNavItem>,
}

/// Error returned while generating OpenAPI docs.
#[derive(Debug, Error)]
pub enum OpenApiDocsError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parse error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML parse error.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// The input path escaped the configured root.
    #[error("OpenAPI path `{path}` is outside the configured root `{root}`")]
    UnsafePath { path: String, root: String },

    /// The document was not a supported OpenAPI document.
    #[error("Invalid OpenAPI document `{path}`: {message}")]
    InvalidSpec { path: String, message: String },

    /// The OpenAPI version is unsupported.
    #[error("Unsupported OpenAPI version `{version}` in `{path}`")]
    UnsupportedVersion { path: String, version: String },

    /// A `$ref` could not be resolved.
    #[error("Unresolved OpenAPI ref `{reference}` in `{path}`")]
    UnresolvedRef { path: String, reference: String },
}

/// Result type for OpenAPI docs generation.
pub type OpenApiDocsResult<T> = Result<T, OpenApiDocsError>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SpecDoc {
    pub title: String,
    pub description: String,
    pub version: String,
    pub slug: String,
    pub servers: Vec<String>,
    pub operations: Vec<OperationDoc>,
    pub schemas: Vec<SchemaDoc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct OperationDoc {
    pub method: String,
    pub path: String,
    pub title: String,
    pub file_name: String,
    pub summary: String,
    pub description: String,
    pub operation_id: Option<String>,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub parameters: Vec<ParameterDoc>,
    pub request_body: Option<RequestBodyDoc>,
    pub responses: Vec<ResponseDoc>,
    pub security: Vec<SecurityRequirementDoc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ParameterDoc {
    pub name: String,
    pub location: String,
    pub required: bool,
    pub schema: String,
    pub description: String,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RequestBodyDoc {
    pub required: bool,
    pub description: String,
    pub content: Vec<MediaTypeDoc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResponseDoc {
    pub status: String,
    pub description: String,
    pub content: Vec<MediaTypeDoc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct MediaTypeDoc {
    pub media_type: String,
    pub schema: String,
    pub examples: Vec<ExampleDoc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ExampleDoc {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SecurityRequirementDoc {
    pub name: String,
    pub kind: String,
    pub scopes: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SchemaDoc {
    pub name: String,
    pub schema: String,
    pub description: String,
    pub required: Vec<String>,
    pub properties: Vec<SchemaPropertyDoc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SchemaPropertyDoc {
    pub name: String,
    pub schema: String,
    pub required: bool,
    pub description: String,
}

/// Generates Markdown reference pages and nav metadata from local OpenAPI files.
pub fn generate_openapi_docs(
    inputs: &[OpenApiSpecInput],
    options: &OpenApiDocsOptions,
) -> OpenApiDocsResult<GeneratedOpenApiDocs> {
    let root = options.root.clone().unwrap_or(std::env::current_dir()?);
    let mut generated = GeneratedOpenApiDocs::default();
    let mut spec_slugs = BTreeSet::new();

    for input in inputs {
        let path = resolve_local_input_path(&root, &input.path)?;
        let value = parse_openapi_file(&path)?;
        let source_path = path.to_string_lossy().into_owned();
        let mut spec = value::build_spec_doc(&value, &source_path, input)?;
        spec.slug = inspect::unique_slug(&spec.slug, &mut spec_slugs);
        let rendered = render::render_spec_doc(&spec, options.base_path.as_deref());
        generated.pages.extend(rendered.pages);
        generated.nav_items.push(rendered.nav_item);
    }

    Ok(generated)
}

fn resolve_local_input_path(root: &Path, input_path: &Path) -> OpenApiDocsResult<PathBuf> {
    let root = root.canonicalize()?;
    let candidate =
        if input_path.is_absolute() { input_path.to_path_buf() } else { root.join(input_path) };
    let canonical = candidate.canonicalize()?;

    if !canonical.starts_with(&root) {
        return Err(OpenApiDocsError::UnsafePath {
            path: canonical.to_string_lossy().into_owned(),
            root: root.to_string_lossy().into_owned(),
        });
    }

    Ok(canonical)
}

fn parse_openapi_file(path: &Path) -> OpenApiDocsResult<Value> {
    let source = fs::read_to_string(path)?;
    if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
        return Ok(serde_json::from_str(&source)?);
    }
    Ok(serde_yaml::from_str(&source)?)
}
