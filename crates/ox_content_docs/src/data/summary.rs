use serde_json::{json, Map, Value};

use crate::model::ApiDocModule;

const DOC_KIND_ORDER: [&str; 7] =
    ["function", "class", "interface", "type", "enum", "variable", "module"];

#[derive(Default)]
struct EntryStats {
    entries: u32,
    members: u32,
    params: u32,
    returns: u32,
    examples: u32,
    deprecated: u32,
    by_kind: [u32; DOC_KIND_ORDER.len()],
}

pub(super) fn build_docs_summary(docs: &[ApiDocModule]) -> Value {
    let mut stats = EntryStats::default();

    for module in docs {
        stats.examples += module.examples.len() as u32;
        for entry in &module.entries {
            stats.entries += 1;
            if let Some(index) = doc_kind_index(&entry.kind) {
                stats.by_kind[index] += 1;
            }
            stats.members += entry.members.len() as u32;
            stats.params += entry.params.len() as u32;
            stats.returns += u32::from(entry.returns.is_some());
            stats.examples += entry.examples.len() as u32;
            stats.deprecated += u32::from(entry.tags.iter().any(|tag| tag.tag == "deprecated"));
        }
    }

    let mut by_kind = Map::new();
    for (index, kind) in DOC_KIND_ORDER.iter().enumerate() {
        let count = stats.by_kind[index];
        if count > 0 {
            by_kind.insert((*kind).to_string(), json!(count));
        }
    }

    json!({
        "modules": docs.len(),
        "entries": stats.entries,
        "byKind": by_kind,
        "members": stats.members,
        "params": stats.params,
        "returns": stats.returns,
        "examples": stats.examples,
        "deprecated": stats.deprecated,
    })
}

fn doc_kind_index(kind: &str) -> Option<usize> {
    match kind {
        "function" => Some(0),
        "class" => Some(1),
        "interface" => Some(2),
        "type" => Some(3),
        "enum" => Some(4),
        "variable" => Some(5),
        "module" => Some(6),
        _ => None,
    }
}
