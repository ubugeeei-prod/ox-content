use std::path::{Path, PathBuf};

use rustc_hash::FxHashMap;

use super::util::normalize_existing_path;
use super::{DocsDiagnostic, DocsDiagnosticCode, GraphError, PublicExport};
use crate::string_builder::StringBuilder;
use crate::{
    ApiDocTag, DocExtractor, DocItem, NormalizedDocEntry, NormalizedDocKind, normalize_doc_items,
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
    // graph edges. `contains_key` then `insert` avoids a get-after-insert expect
    // and keeps the returned borrow simple.
    if !docs_cache.contains_key(module) {
        // Take the doc items the export-graph walk already extracted from this
        // module's AST when available - `remove` so they move into the cache
        // rather than being cloned (each module is normalized once). Only parse
        // the file again on a miss (the all-visibility fallback, or a module the
        // walk didn't visit). Extraction failures stay typed.
        let items = match walk_docs.and_then(|docs| docs.remove(&normalize_existing_path(module))) {
            Some(items) => items,
            None => extractor
                .extract_file(module)
                .map_err(|source| GraphError::Extract { path: module.clone(), source })?,
        };
        docs_cache.insert(module.clone(), normalize_doc_items(items, type_parameters));
    }

    // `insert` above is the only writer and this crate is single-threaded, so a
    // miss here would be a HashMap bug. Return an empty slice instead of aborting
    // so unexpected cache state degrades to missing-declaration diagnostics.
    Ok(docs_cache.get(module).map_or(&[], Vec::as_slice))
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use rustc_hash::FxHashMap;

    use super::*;
    use crate::DocExtractor;

    fn temp_module(name: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("ox-content-docs-cache-{nanos}-{seq}"));
        fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn missing_module_returns_extract_error() {
        let mut cache = FxHashMap::default();
        let extractor = DocExtractor::new();
        let missing = PathBuf::from("/definitely/does-not-exist-853.ts");
        let error = normalized_entries_for_module(&mut cache, None, &extractor, &missing, false)
            .unwrap_err();
        assert!(matches!(error, GraphError::Extract { path, .. } if path == missing));
        assert!(cache.is_empty());
    }

    #[test]
    fn unsupported_and_invalid_utf8_modules_return_extract_errors() {
        let extractor = DocExtractor::new();

        let readme = temp_module("README.md");
        fs::write(&readme, "# not a module\n").unwrap();
        let mut cache = FxHashMap::default();
        let error = normalized_entries_for_module(&mut cache, None, &extractor, &readme, false)
            .unwrap_err();
        assert!(matches!(error, GraphError::Extract { .. }), "{error}");

        let binary = temp_module("hostile.ts");
        fs::write(&binary, [0xff, 0xfe, 0x00, 0x01]).unwrap();
        let mut cache = FxHashMap::default();
        let error = normalized_entries_for_module(&mut cache, None, &extractor, &binary, false)
            .unwrap_err();
        assert!(matches!(error, GraphError::Extract { .. }), "{error}");
    }

    #[test]
    fn caches_normalized_entries_and_reuses_walk_docs() {
        let path = temp_module("math.ts");
        fs::write(&path, "/** Adds two numbers. */\nexport function add(a: number, b: number): number {\n  return a + b;\n}\n")
            .unwrap();
        let extractor = DocExtractor::new();
        let mut cache = FxHashMap::default();

        let first =
            normalized_entries_for_module(&mut cache, None, &extractor, &path, false).unwrap();
        assert!(first.iter().any(|entry| entry.name == "add"));
        let first_len = first.len();

        let second =
            normalized_entries_for_module(&mut cache, None, &extractor, &path, false).unwrap();
        assert_eq!(second.len(), first_len);

        let items = extractor.extract_file(&path).unwrap();
        let mut walk_docs = FxHashMap::default();
        walk_docs.insert(normalize_existing_path(&path), items);
        // Corrupt the file so a cache miss would surface as Extract, not success.
        fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();

        let mut walk_cache = FxHashMap::default();
        let walked = normalized_entries_for_module(
            &mut walk_cache,
            Some(&mut walk_docs),
            &extractor,
            &path,
            false,
        )
        .unwrap();
        assert!(walked.iter().any(|entry| entry.name == "add"));
        assert!(walk_docs.is_empty());
    }
}
