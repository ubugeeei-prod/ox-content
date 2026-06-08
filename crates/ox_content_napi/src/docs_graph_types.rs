use napi_derive::napi;

use crate::{JsDocEntry, JsDocsMarkdownTag};

/// Entry point used to group generated API docs.
#[napi(object)]
#[derive(Clone)]
pub struct JsEntryPointSpec {
    pub path: String,
    pub name: Option<String>,
}

/// Export graph resolution options.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsGraphOptions {
    pub root: Option<String>,
    pub tsconfig: Option<String>,
    pub external_docs: Option<bool>,
    pub external_package_sources: Option<Vec<JsExternalPackageSource>>,
}

/// Options for extracting docs grouped by entry point.
#[napi(object)]
#[derive(Clone, Default)]
pub struct JsEntryPointDocsOptions {
    pub root: Option<String>,
    pub tsconfig: Option<String>,
    pub private: Option<bool>,
    pub internal: Option<bool>,
    pub external_docs: Option<bool>,
    pub external_package_sources: Option<Vec<JsExternalPackageSource>>,
    /// Opt in to TSDoc-style type-parameter docs (`@typeParam` / `<T>` table).
    /// Off by default.
    pub type_parameters: Option<bool>,
}

/// Explicit source entry for an external package.
#[napi(object)]
#[derive(Clone)]
pub struct JsExternalPackageSource {
    pub package: String,
    pub entry: String,
}

/// Export source metadata.
#[napi(object)]
#[derive(Clone)]
pub struct JsExportSource {
    pub kind: String,
    pub module: Option<String>,
    pub package: Option<String>,
    pub specifier: Option<String>,
    pub original_name: String,
    pub type_only: bool,
}

/// Public export metadata.
#[napi(object)]
#[derive(Clone)]
pub struct JsPublicExport {
    pub name: String,
    pub kind: String,
    pub source: JsExportSource,
}

/// Public entry point module.
#[napi(object)]
#[derive(Clone)]
pub struct JsEntrypointModule {
    pub name: String,
    pub source_path: String,
    pub exports: Vec<JsPublicExport>,
}

/// Resolved source module.
#[napi(object)]
#[derive(Clone)]
pub struct JsResolvedModule {
    pub path: String,
    pub exports: Vec<JsPublicExport>,
}

/// Resolved export graph.
#[napi(object)]
#[derive(Clone)]
pub struct JsExportGraph {
    pub entrypoints: Vec<JsEntrypointModule>,
    pub modules: Vec<JsResolvedModule>,
}

/// Docs grouped by a public entry point.
#[napi(object)]
#[derive(Clone)]
pub struct JsEntrypointDocsModule {
    pub name: String,
    pub file: String,
    pub source_path: String,
    /// Module-level description from the entry file's `@module` / leading JSDoc.
    pub description: String,
    /// Module-level example blocks from the entry file's `@module` / leading JSDoc.
    pub examples: Vec<String>,
    /// Module-level custom JSDoc tags.
    pub tags: Vec<JsDocsMarkdownTag>,
    pub entries: Vec<JsDocEntry>,
    pub exports: Vec<JsPublicExport>,
    pub diagnostics: Vec<JsDocsDiagnostic>,
}

/// Diagnostic for an entry point export during docs extraction.
#[napi(object)]
#[derive(Clone)]
pub struct JsDocsDiagnostic {
    pub code: String,
    pub entrypoint: String,
    pub export_name: String,
    pub export_kind: String,
    pub source: JsExportSource,
    pub message: String,
}
