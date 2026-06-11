use std::rc::Rc;

use rustc_hash::FxHashMap;

use super::links::SymbolLocation;
use super::options::{MarkdownDocsOptions, MarkdownPathStrategy};
use super::paths::{entry_anchor, member_anchor, module_file_name};
use super::typedoc::{plural_kind_file_name, typedoc_entry_file_name};
use super::{member_symbol_name, CanonicalOwners};
use crate::model::ApiDocModule;
#[allow(unused_imports)]
use crate::profile_span;

pub(super) fn build_symbol_map(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
) -> FxHashMap<String, Vec<SymbolLocation>> {
    profile_span!("docs::build_symbol_map");
    let mut map = FxHashMap::default();
    // In the TypeDoc strategy a re-exported symbol has a single canonical page;
    // resolve every reference to that owner module so cross-links never point at
    // a duplicate page that is no longer emitted.
    let canonical = (options.group_by == "file"
        && options.path_strategy == MarkdownPathStrategy::TypeDoc)
        .then(|| CanonicalOwners::compute(docs));

    for doc in docs {
        // Interned once per module and shared by every entry + member below.
        let module_name: Rc<str> = Rc::from(module_file_name(&doc.file));
        for entry in &doc.entries {
            let (file_name, anchor): (Rc<str>, Option<String>) =
                match (options.group_by.as_str(), options.path_strategy) {
                    ("file", MarkdownPathStrategy::TypeDoc) => {
                        let owner_module = canonical
                            .as_ref()
                            .and_then(|owners| owners.canonical_module(entry))
                            .unwrap_or(&module_name);
                        (Rc::from(typedoc_entry_file_name(owner_module, entry)), None)
                    }
                    ("category", _) => (
                        Rc::from(plural_kind_file_name(&entry.kind)),
                        Some(entry_anchor(&entry.name)),
                    ),
                    _ => (Rc::clone(&module_name), Some(entry_anchor(&entry.name))),
                };
            insert_symbol_location(
                &mut map,
                entry.name.clone(),
                SymbolLocation {
                    module_name: Rc::clone(&module_name),
                    file_name: Rc::clone(&file_name),
                    anchor,
                },
            );
            for member in &entry.members {
                insert_symbol_location(
                    &mut map,
                    member_symbol_name(&entry.name, &member.name),
                    SymbolLocation {
                        module_name: Rc::clone(&module_name),
                        file_name: Rc::clone(&file_name),
                        anchor: Some(member_anchor(&entry.name, member, options.path_strategy)),
                    },
                );
            }
        }
    }

    map
}

fn insert_symbol_location(
    map: &mut FxHashMap<String, Vec<SymbolLocation>>,
    symbol_name: String,
    location: SymbolLocation,
) {
    map.entry(symbol_name).or_default().push(location);
}
