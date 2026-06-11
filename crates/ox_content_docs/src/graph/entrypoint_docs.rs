use std::path::PathBuf;

use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use super::build::build_export_graph_inner;
use super::docs::{
    docs_diagnostic, export_entrypoint_message, filtered_visibility_reason, is_dependency_source,
    normalized_entries_for_module, resolve_entrypoint_module_metadata,
};
use super::error::GraphError;
use super::model::{DocsDiagnosticCode, EntrypointDocsModule, ExportSource};
use super::options::{EntryPointDocsOptions, EntryPointSpec};
#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::{join2, StringBuilder};
use crate::{DocExtractor, NormalizedDocEntry};

/// Extracts normalized docs grouped by public entry points.
pub fn extract_docs_from_entry_points(
    entrypoints: &[EntryPointSpec],
    options: &EntryPointDocsOptions,
) -> Result<Vec<EntrypointDocsModule>, GraphError> {
    profile_span!("docs::extract_entry_points");
    // Build the export graph and extract docs from the same parse in one walk,
    // so each reachable module is parsed once here instead of again below.
    let (graph, mut walk_docs) = build_export_graph_inner(
        entrypoints,
        &options.graph,
        Some(DocExtractor::for_entrypoint_exports(
            options.include_private,
            options.include_internal,
        )),
    )?;
    let extractor =
        DocExtractor::for_entrypoint_exports(options.include_private, options.include_internal);
    let all_visibility_extractor = DocExtractor::for_entrypoint_exports(true, true);
    let mut docs_cache: FxHashMap<PathBuf, Vec<NormalizedDocEntry>> =
        FxHashMap::with_hasher(FxBuildHasher);
    let mut all_docs_cache: FxHashMap<PathBuf, Vec<NormalizedDocEntry>> =
        FxHashMap::with_hasher(FxBuildHasher);
    let mut modules = Vec::with_capacity(graph.entrypoints.len());

    for entrypoint in graph.entrypoints {
        let mut entries = Vec::new();
        let mut diagnostics = Vec::new();
        let mut seen = FxHashSet::default();

        for export in &entrypoint.exports {
            let (module, original_name) = match &export.source {
                ExportSource::Local { module, original_name }
                | ExportSource::External { module: Some(module), original_name, .. } => {
                    (module, original_name)
                }
                ExportSource::External { .. } => {
                    diagnostics.push(docs_diagnostic(
                        DocsDiagnosticCode::UnresolvedExternal,
                        &entrypoint.name,
                        export,
                        export_entrypoint_message(
                            &export.name,
                            &entrypoint.name,
                            " was not documented because its external source could not be resolved",
                        ),
                    ));
                    continue;
                }
            };
            if original_name == "*" {
                diagnostics.push(docs_diagnostic(
                    DocsDiagnosticCode::UnsupportedExport,
                    &entrypoint.name,
                    export,
                    export_entrypoint_message(
                        &export.name,
                        &entrypoint.name,
                        " was not documented because namespace exports are not emitted as docs entries",
                    ),
                ));
                continue;
            }

            let matched = {
                let module_entries = normalized_entries_for_module(
                    &mut docs_cache,
                    Some(&mut walk_docs),
                    &extractor,
                    module,
                    options.type_parameters,
                )?;
                let mut matched = false;
                for entry in module_entries.iter().filter(|entry| entry.name == *original_name) {
                    matched = true;
                    let mut key =
                        StringBuilder::with_capacity(export.name.len() + entry.file.len() + 2 + 20);
                    key.push_str(&export.name);
                    key.push_char('\0');
                    key.push_str(&entry.file);
                    key.push_char('\0');
                    key.push_usize(entry.line as usize);
                    let key = key.into_string();
                    if !seen.insert(key) {
                        continue;
                    }
                    let mut entry = entry.clone();
                    entry.name.clone_from(&export.name);
                    if is_dependency_source(module) {
                        // Source lives in an installed dependency (under a
                        // node_modules directory): drop the absolute path so we emit
                        // no "View source" link and never leak a local absolute path
                        // (matches TypeDoc's inlined external symbols). Workspace
                        // sources resolved inside the repo keep their path.
                        entry.file = String::new();
                    }
                    entries.push(entry);
                }
                matched
            };

            if matched {
                continue;
            }

            let all_module_entries = normalized_entries_for_module(
                &mut all_docs_cache,
                None,
                &all_visibility_extractor,
                module,
                options.type_parameters,
            )?;
            if let Some(hidden_entry) =
                all_module_entries.iter().find(|entry| entry.name == *original_name)
            {
                if let Some(reason) = filtered_visibility_reason(
                    hidden_entry,
                    options.include_private,
                    options.include_internal,
                ) {
                    let suffix = join2(" was excluded from docs because it is marked ", reason);
                    diagnostics.push(docs_diagnostic(
                        DocsDiagnosticCode::FilteredByVisibility,
                        &entrypoint.name,
                        export,
                        export_entrypoint_message(&export.name, &entrypoint.name, &suffix),
                    ));
                    continue;
                }
            }

            diagnostics.push(docs_diagnostic(
                DocsDiagnosticCode::MissingDeclaration,
                &entrypoint.name,
                export,
                export_entrypoint_message(
                    &export.name,
                    &entrypoint.name,
                    " was not documented because no matching declaration was extracted",
                ),
            ));
        }

        // The entry file's own module-level `@module` / leading JSDoc is emitted
        // by the extractor as a `Module`-kind entry but is never an export, so it
        // is dropped from `entries` above. Pull it out of the entry file's
        // normalized items and carry it as the module description.
        let module_metadata = resolve_entrypoint_module_metadata(
            &entrypoint.name,
            normalized_entries_for_module(
                &mut docs_cache,
                Some(&mut walk_docs),
                &extractor,
                &entrypoint.source_path,
                options.type_parameters,
            )?,
        );

        modules.push(EntrypointDocsModule {
            file: module_metadata.name.clone(),
            name: module_metadata.name,
            source_path: entrypoint.source_path,
            description: module_metadata.description,
            examples: module_metadata.examples,
            tags: module_metadata.tags,
            entries,
            exports: entrypoint.exports,
            diagnostics,
        });
    }

    Ok(modules)
}
