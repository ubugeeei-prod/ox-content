use std::path::PathBuf;

use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use super::builder::GraphBuilder;
use super::error::GraphError;
use super::model::{EntrypointModule, ExportGraph};
use super::options::{EntryPointSpec, GraphOptions};
use super::resolver::ModuleResolver;
use super::util::{graph_root, module_name_from_path};
#[allow(unused_imports)]
use crate::profile_span;
use crate::{DocExtractor, DocItem};

/// Builds an export graph from entry points.
///
/// Local re-exports are followed recursively. External package re-exports are
/// preserved as metadata and are not resolved into declaration docs.
pub fn build_export_graph(
    entrypoints: &[EntryPointSpec],
    options: &GraphOptions,
) -> Result<ExportGraph, GraphError> {
    Ok(build_export_graph_inner(entrypoints, options, None)?.0)
}

/// Shared implementation behind [`build_export_graph`] and
/// [`super::extract_docs_from_entry_points`].
///
/// When `doc_extractor` is `Some`, doc items are extracted from each module's
/// already-parsed AST during the walk and returned in the second tuple element,
/// keyed by normalized path, so the extraction phase can reuse them instead of
/// re-parsing every module.
pub(super) fn build_export_graph_inner(
    entrypoints: &[EntryPointSpec],
    options: &GraphOptions,
    doc_extractor: Option<DocExtractor>,
) -> Result<(ExportGraph, FxHashMap<PathBuf, Vec<DocItem>>), GraphError> {
    profile_span!("docs::build_export_graph");
    let root = graph_root(options);
    let resolver = ModuleResolver::new(&root, options);
    let mut builder = GraphBuilder {
        root,
        resolver,
        modules: FxHashMap::with_hasher(FxBuildHasher),
        active: FxHashSet::default(),
        doc_extractor,
        docs: FxHashMap::with_hasher(FxBuildHasher),
    };

    let mut graph_entrypoints = Vec::with_capacity(entrypoints.len());
    for entrypoint in entrypoints {
        let source_path = builder.entrypoint_path(&entrypoint.path)?;
        let name = entrypoint.name.clone().unwrap_or_else(|| module_name_from_path(&source_path));
        let exports = builder.collect_module_exports(&source_path)?;
        graph_entrypoints.push(EntrypointModule { name, source_path, exports });
    }

    Ok((ExportGraph { entrypoints: graph_entrypoints, modules: builder.modules }, builder.docs))
}
