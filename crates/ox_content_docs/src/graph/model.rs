use std::path::PathBuf;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{ApiDocTag, NormalizedDocEntry};

/// Resolved export graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportGraph {
    /// Public entry point modules.
    pub entrypoints: Vec<EntrypointModule>,
    /// Resolved modules keyed by absolute source path.
    pub modules: FxHashMap<PathBuf, ResolvedModule>,
}

/// Public entry point module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntrypointModule {
    /// Public module name.
    pub name: String,
    /// Source file path.
    pub source_path: PathBuf,
    /// Public exports reachable from this entry point.
    pub exports: Vec<PublicExport>,
}

/// Resolved source module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedModule {
    /// Source file path.
    pub path: PathBuf,
    /// Exports declared or re-exported by this module.
    pub exports: Vec<PublicExport>,
}

/// Public export metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicExport {
    /// Public export name after aliasing.
    pub name: String,
    /// Export kind.
    pub kind: ExportKind,
    /// Export source.
    pub source: ExportSource,
}

/// Export kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExportKind {
    /// Runtime value export.
    Value,
    /// Type-only export.
    Type,
    /// Export available as both value and type.
    ValueAndType,
    /// Namespace export.
    Namespace,
    /// Default export.
    Default,
}

/// Export source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ExportSource {
    /// Export resolved to a local source file.
    Local {
        /// Source module path.
        module: PathBuf,
        /// Original exported name in the source module.
        original_name: String,
    },
    /// Export from an external package.
    External {
        /// Package name.
        package: String,
        /// Original module specifier.
        #[serde(default, skip_serializing_if = "String::is_empty")]
        specifier: String,
        /// Resolved source or declaration module path, when available.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        module: Option<PathBuf>,
        /// Original exported name.
        original_name: String,
        /// Whether the export is type-only.
        type_only: bool,
    },
}

/// Docs grouped by a public entry point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntrypointDocsModule {
    /// Public module name.
    pub name: String,
    /// Virtual docs module file used by Markdown generation.
    pub file: String,
    /// Source file path.
    pub source_path: PathBuf,
    /// Module-level description from the entry file's `@module` block or leading
    /// file comment. Empty when the entry file has no module-level JSDoc.
    #[serde(default)]
    pub description: String,
    /// Module-level example blocks from the entry file's `@module` block or
    /// leading file comment.
    #[serde(default)]
    pub examples: Vec<String>,
    /// Module-level custom JSDoc tags.
    #[serde(default)]
    pub tags: Vec<ApiDocTag>,
    /// Normalized docs entries for reachable exports.
    pub entries: Vec<NormalizedDocEntry>,
    /// Public export metadata, including external re-exports.
    pub exports: Vec<PublicExport>,
    /// Diagnostics for exports that could not be emitted as docs entries.
    #[serde(default)]
    pub diagnostics: Vec<DocsDiagnostic>,
}

/// Diagnostic for an entry point export during docs extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocsDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: DocsDiagnosticCode,
    /// Public entry point name.
    pub entrypoint: String,
    /// Public export name.
    pub export_name: String,
    /// Public export kind.
    pub export_kind: ExportKind,
    /// Export source metadata.
    pub source: ExportSource,
    /// Human-readable diagnostic message.
    pub message: String,
}

/// Diagnostic code for entry point docs extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocsDiagnosticCode {
    /// Export was intentionally filtered by visibility options.
    FilteredByVisibility,
    /// Graph export could not be matched to a declaration.
    MissingDeclaration,
    /// Export kind is not emitted as a docs entry.
    UnsupportedExport,
    /// External export could not be resolved to a source or declaration module.
    UnresolvedExternal,
}
