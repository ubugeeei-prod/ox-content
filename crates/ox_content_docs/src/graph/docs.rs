use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;

use super::{
    normalize_existing_path, DocsDiagnostic, DocsDiagnosticCode, GraphError, PublicExport,
};
use crate::string_builder::StringBuilder;
use crate::{
    normalize_doc_items, ApiDocTag, DocExtractor, DocItem, NormalizedDocEntry, NormalizedDocKind,
};

pub(super) struct EntrypointModuleMetadata {
    pub(super) name: String,
    pub(super) description: String,
    pub(super) examples: Vec<String>,
    pub(super) tags: Vec<ApiDocTag>,
}

pub(super) fn resolve_entrypoint_module_metadata(
    entrypoint_name: &str,
    entries: &[NormalizedDocEntry],
) -> EntrypointModuleMetadata {
    let module_entry = entries.iter().find(|entry| entry.kind == NormalizedDocKind::Module);
    let explicit_module_name =
        module_entry.and_then(|entry| explicit_module_name_from_tags(&entry.tags));

    EntrypointModuleMetadata {
        name: explicit_module_name.unwrap_or(entrypoint_name).to_string(),
        description: module_entry.map(|entry| entry.description.clone()).unwrap_or_default(),
        examples: module_entry.map(|entry| entry.examples.clone()).unwrap_or_default(),
        tags: module_entry.map(module_tags_from_normalized_entry).unwrap_or_default(),
    }
}

fn module_tags_from_normalized_entry(entry: &NormalizedDocEntry) -> Vec<ApiDocTag> {
    entry
        .tags
        .iter()
        .filter(|(tag, _)| tag.as_str() != "module")
        .map(|(tag, value)| ApiDocTag { tag: tag.clone(), value: value.clone() })
        .collect()
}

fn explicit_module_name_from_tags(
    // Normalized doc tags are ordered for deterministic generated output.
    tags: &std::collections::BTreeMap<String, String>,
) -> Option<&str> {
    tags.get("module")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .and_then(|value| value.split_whitespace().next())
        .filter(|value| !value.is_empty())
}

/// Returns true when a resolved module path is an installed dependency, i.e. it
/// lives under a `node_modules` directory. Such sources are not in the consumer's
/// repository, so generated docs must not link to them or leak their absolute
/// local path. Workspace sources resolved inside the repo return false and keep
/// their source location.
pub(super) fn is_dependency_source(module: &Path) -> bool {
    module.components().any(|component| component.as_os_str() == "node_modules")
}

pub(super) fn normalized_entries_for_module<'a>(
    docs_cache: &'a mut FxHashMap<PathBuf, Vec<NormalizedDocEntry>>,
    walk_docs: Option<&mut FxHashMap<PathBuf, Vec<DocItem>>>,
    extractor: &DocExtractor,
    module: &PathBuf,
    type_parameters: bool,
) -> Result<&'a [NormalizedDocEntry], GraphError> {
    // Dependency graph construction can revisit the same module through many
    // re-export edges. Cache normalized entries per resolved path so each file
    // is parsed and normalized once, then return borrowed slices for all later
    // graph edges. The explicit insert-before-get shape keeps the returned
    // borrow simple while avoiding repeated extractor work.
    if !docs_cache.contains_key(module) {
        // Take the doc items the export-graph walk already extracted from this
        // module's AST when available - `remove` so they move into the cache
        // rather than being cloned (each module is normalized once). Only parse
        // the file again on a miss (the all-visibility fallback, or a module the
        // walk didn't visit).
        let items = match walk_docs.and_then(|docs| docs.remove(&normalize_existing_path(module))) {
            Some(items) => items,
            None => extractor
                .extract_file(module)
                .map_err(|source| GraphError::Extract { path: module.clone(), source })?,
        };
        docs_cache.insert(module.clone(), normalize_doc_items(items, type_parameters));
    }

    Ok(docs_cache.get(module).expect("normalized docs cache entry").as_slice())
}

pub(super) fn filtered_visibility_reason(
    entry: &NormalizedDocEntry,
    include_private: bool,
    include_internal: bool,
) -> Option<&'static str> {
    if !include_private && entry.private {
        return Some("@private");
    }
    if !include_internal && entry.tags.contains_key("internal") {
        return Some("@internal");
    }
    None
}

pub(super) fn docs_diagnostic(
    code: DocsDiagnosticCode,
    entrypoint: &str,
    export: &PublicExport,
    message: String,
) -> DocsDiagnostic {
    DocsDiagnostic {
        code,
        entrypoint: entrypoint.to_string(),
        export_name: export.name.clone(),
        export_kind: export.kind,
        source: export.source.clone(),
        message,
    }
}

pub(super) fn export_entrypoint_message(
    export_name: &str,
    entrypoint_name: &str,
    suffix: &str,
) -> String {
    let mut message =
        StringBuilder::with_capacity(export_name.len() + entrypoint_name.len() + suffix.len() + 27);
    message.push_str("export \"");
    message.push_str(export_name);
    message.push_str("\" from entrypoint \"");
    message.push_str(entrypoint_name);
    message.push_char('"');
    message.push_str(suffix);
    message.into_string()
}
