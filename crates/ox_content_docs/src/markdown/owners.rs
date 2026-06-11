use rustc_hash::FxHashMap;

use crate::model::{ApiDocEntry, ApiDocModule};

use super::{file_name, module_file_name};

/// Resolves, for each distinct symbol, the single module that owns its canonical
/// TypeDoc per-symbol page. A symbol re-exported from several entry points
/// otherwise produces an identical page under each one; TypeDoc emits one page.
///
/// A symbol is keyed by `(name, defining_file)` so that two distinct symbols
/// sharing a name (different source files) keep separate pages. The owner is:
///
/// 1. the module whose own entry-point source (`source_path`) is the symbol's
///    defining file (i.e. the symbol is declared in that entry point), else
/// 2. the first module that exports it, in the same order pages are emitted.
pub struct CanonicalOwners {
    owners: FxHashMap<(String, String), String>,
}

impl CanonicalOwners {
    pub fn compute(docs: &[ApiDocModule]) -> Self {
        // Build the owner table in the same deterministic order that pages are
        // emitted (see `sort_extracted_docs`) so the fallback "first exporter"
        // rule agrees between the page generator and the nav generator,
        // regardless of the caller's input order.
        let mut order: Vec<&ApiDocModule> = docs.iter().collect();
        order.sort_by_cached_key(|module| {
            let name = file_name(&module.file);
            (name.to_lowercase(), name)
        });

        let mut owners: FxHashMap<(String, String), String> = FxHashMap::default();
        let mut fallback: FxHashMap<(String, String), String> = FxHashMap::default();
        for doc in order {
            let module_name = module_file_name(&doc.file);
            for entry in &doc.entries {
                let key = (entry.name.clone(), entry.file.clone());
                fallback.entry(key.clone()).or_insert_with(|| module_name.clone());
                // Rule 1: the defining module wins, if it is itself an entry point.
                if !entry.file.is_empty() && doc.source_path == entry.file {
                    owners.entry(key).or_insert_with(|| module_name.clone());
                }
            }
        }
        // Rule 2: symbols with no defining-module match fall back to the first
        // module that exported them.
        for (key, module_name) in fallback {
            owners.entry(key).or_insert(module_name);
        }

        Self { owners }
    }

    /// The module name owning `entry`'s canonical page, if known.
    pub(super) fn canonical_module(&self, entry: &ApiDocEntry) -> Option<&str> {
        self.owners.get(&(entry.name.clone(), entry.file.clone())).map(String::as_str)
    }

    /// True when `entry` should render its full page under `doc` (rather than be
    /// a re-export reference to another module's canonical page).
    pub fn is_canonical(&self, doc: &ApiDocModule, entry: &ApiDocEntry) -> bool {
        self.canonical_module(entry) == Some(module_file_name(&doc.file).as_str())
    }
}
