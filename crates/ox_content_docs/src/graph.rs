//! Public API export graph extraction for generated documentation.

mod build;
mod builder;
mod builder_exports;
mod docs;
mod entrypoint_docs;
mod error;
mod export_sources;
mod exports;
mod model;
mod options;
mod resolver;
mod util;

pub use build::build_export_graph;
pub use entrypoint_docs::extract_docs_from_entry_points;
pub use error::GraphError;
pub use model::{
    DocsDiagnostic, DocsDiagnosticCode, EntrypointDocsModule, EntrypointModule, ExportGraph,
    ExportKind, ExportSource, PublicExport, ResolvedModule,
};
pub use options::{
    EntryPointDocsOptions, EntryPointSpec, ExternalDocsOptions, ExternalPackageSource, GraphOptions,
};

#[cfg(test)]
use crate::NormalizedDocKind;

#[cfg(test)]
mod tests;
