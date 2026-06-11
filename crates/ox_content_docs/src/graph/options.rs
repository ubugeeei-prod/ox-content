use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Entry point used to group generated API docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointSpec {
    /// Source file path.
    pub path: PathBuf,
    /// Public module name. Defaults to the file stem.
    pub name: Option<String>,
}

/// Export graph resolution options.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphOptions {
    /// Root used to resolve relative entry point paths.
    pub root: Option<PathBuf>,
    /// Optional TypeScript config for path alias resolution.
    pub tsconfig: Option<PathBuf>,
    /// External package documentation extraction options.
    #[serde(default)]
    pub external_docs: ExternalDocsOptions,
}

/// External package documentation extraction options.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalDocsOptions {
    /// Resolve external package re-exports into documentation entries.
    pub enabled: bool,
    /// Explicit package source entries used before package exports/types resolution.
    #[serde(default)]
    pub package_sources: Vec<ExternalPackageSource>,
}

/// Explicit source entry for an external package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalPackageSource {
    /// Package name or exact package subpath specifier.
    pub package: String,
    /// Source or declaration entry file.
    pub entry: PathBuf,
}

/// Options for extracting docs grouped by entry point.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryPointDocsOptions {
    /// Export graph options.
    pub graph: GraphOptions,
    /// Include `@private` docs.
    pub include_private: bool,
    /// Include `@internal` docs.
    pub include_internal: bool,
    /// Opt in to TSDoc-style type-parameter docs (`@typeParam` / `<T>` table).
    /// Off by default (JSDoc semantics).
    #[serde(default)]
    pub type_parameters: bool,
}
