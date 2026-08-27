//! The three numbering passes: sections, figures, and tables.
//!
//! Each walks the HTML once, assigns the next number to every element carrying
//! a trackable `id`, and writes the `data-ox-xref-*` attributes the reference
//! pass and the stylesheet read.

use super::types::{CrossReferenceDiagnostic, CrossReferenceEntry, FailureMode};
use rustc_hash::FxHashMap;

mod figures;
mod sections;
mod tables;

pub(super) use figures::annotate_figures_and_images;
pub(super) use sections::annotate_sections;
pub(super) use tables::{annotate_tables, apply_trailing_table_labels};

/// A target plus where it was found, so the caller can report in document order.
pub struct TrackedTarget {
    pub entry: CrossReferenceEntry,
    pub position: usize,
}

/// Everything the passes accumulate.
///
/// The vector keeps discovery order for the final sort; the index answers the
/// lookups. Every registration and every `@ref` in the document does a lookup,
/// so scanning the vector instead made the whole pass quadratic — measurably
/// so: a 100-section document took 88ms that way against 1.9ms for the
/// TypeScript this replaced.
pub struct Registry {
    pub targets: Vec<TrackedTarget>,
    index: FxHashMap<String, usize>,
    pub diagnostics: Vec<CrossReferenceDiagnostic>,
}

impl Registry {
    pub(crate) fn new() -> Self {
        Self { targets: Vec::new(), index: FxHashMap::default(), diagnostics: Vec::new() }
    }

    pub(crate) fn get(&self, id: &str) -> Option<&CrossReferenceEntry> {
        self.index.get(id).map(|slot| &self.targets[*slot].entry)
    }

    fn register(&mut self, policy: FailureMode, target: TrackedTarget) {
        if self.index.contains_key(&target.entry.id) {
            self.diagnostics.push(CrossReferenceDiagnostic {
                policy,
                message: format!("duplicate cross-reference target \"{}\"", target.entry.id),
            });
            return;
        }
        self.index.insert(target.entry.id.clone(), self.targets.len());
        self.targets.push(target);
    }
}
