use serde_json::{json, Map, Value};

use crate::model::{ApiDocEntry, ApiDocMember, ApiDocModule, ApiParamDoc, ApiReturnDoc};

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
    by_kind: Map<String, Value>,
}

/// Generates the machine-readable docs data JSON payload.
///
/// The returned JSON is pretty-formatted and preserves the shape consumed by
/// TypeScript docs tooling.
pub fn generate_docs_data_json(
    docs: &[ApiDocModule],
    generated_at: &str,
) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&json!({
        "version": 1,
        "generatedAt": generated_at,
        "summary": build_docs_summary(docs),
        "modules": docs.iter().map(module_to_json).collect::<Vec<_>>(),
    }))
}

fn build_docs_summary(docs: &[ApiDocModule]) -> Value {
    let mut stats = EntryStats::default();

    for module in docs {
        for entry in &module.entries {
            stats.entries += 1;
            let count = stats.by_kind.get(&entry.kind).and_then(Value::as_u64).unwrap_or(0) + 1;
            stats.by_kind.insert(entry.kind.clone(), json!(count));
            stats.members += entry.members.len() as u32;
            stats.params += entry.params.len() as u32;
            stats.returns += u32::from(entry.returns.is_some());
            stats.examples += entry.examples.len() as u32;
            stats.deprecated += u32::from(entry.tags.iter().any(|tag| tag.tag == "deprecated"));
        }
    }

    let mut by_kind = Map::new();
    for kind in DOC_KIND_ORDER {
        if let Some(count) = stats.by_kind.get(kind) {
            by_kind.insert(kind.to_string(), count.clone());
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

fn module_to_json(module: &ApiDocModule) -> Value {
    json!({
        "file": normalize_doc_file_path(&module.file),
        "description": module.description,
        "entries": module.entries.iter().map(entry_to_json).collect::<Vec<_>>(),
    })
}

fn entry_to_json(entry: &ApiDocEntry) -> Value {
    let mut value = Map::new();
    value.insert("name".to_string(), json!(entry.name));
    value.insert("kind".to_string(), json!(entry.kind));
    value.insert("description".to_string(), json!(entry.description));

    if !entry.params.is_empty() {
        value.insert(
            "params".to_string(),
            Value::Array(entry.params.iter().map(param_to_json).collect()),
        );
    }
    if let Some(returns) = &entry.returns {
        value.insert("returns".to_string(), return_to_json(returns));
    }
    if !entry.examples.is_empty() {
        value.insert("examples".to_string(), json!(entry.examples));
    }
    if !entry.tags.is_empty() {
        value.insert(
            "tags".to_string(),
            Value::Object(
                entry.tags.iter().map(|tag| (tag.tag.clone(), json!(tag.value))).collect(),
            ),
        );
    }
    if !entry.members.is_empty() {
        value.insert(
            "members".to_string(),
            Value::Array(entry.members.iter().map(member_to_json).collect()),
        );
    }
    if entry.private {
        value.insert("private".to_string(), json!(true));
    }

    // An empty `file` means the symbol has no source in the consumer's repo
    // (e.g. re-exported from an external package): omit the source location
    // entirely rather than leak an absolute local path.
    if !entry.file.is_empty() {
        value.insert("file".to_string(), json!(normalize_doc_file_path(&entry.file)));
        value.insert("line".to_string(), json!(entry.line));
        value.insert("endLine".to_string(), json!(entry.end_line));
    }
    if let Some(signature) = &entry.signature {
        value.insert("signature".to_string(), json!(signature));
    }

    Value::Object(value)
}

fn member_to_json(member: &ApiDocMember) -> Value {
    let mut value = Map::new();
    value.insert("name".to_string(), json!(member.name));
    value.insert("kind".to_string(), json!(member.kind));
    value.insert("description".to_string(), json!(member.description));
    if let Some(signature) = &member.signature {
        value.insert("signature".to_string(), json!(signature));
    }
    if let Some(type_annotation) = &member.type_annotation {
        value.insert("type".to_string(), json!(type_annotation));
    }
    if !member.params.is_empty() {
        value.insert(
            "params".to_string(),
            Value::Array(member.params.iter().map(param_to_json).collect()),
        );
    }
    if let Some(returns) = &member.returns {
        value.insert("returns".to_string(), return_to_json(returns));
    }
    if member.optional {
        value.insert("optional".to_string(), json!(true));
    }
    if member.readonly {
        value.insert("readonly".to_string(), json!(true));
    }
    if member.r#static {
        value.insert("static".to_string(), json!(true));
    }
    if member.private {
        value.insert("private".to_string(), json!(true));
    }
    if !member.tags.is_empty() {
        value.insert(
            "tags".to_string(),
            Value::Object(
                member.tags.iter().map(|tag| (tag.tag.clone(), json!(tag.value))).collect(),
            ),
        );
    }
    value.insert("line".to_string(), json!(member.line));
    value.insert("endLine".to_string(), json!(member.end_line));

    Value::Object(value)
}

fn param_to_json(param: &ApiParamDoc) -> Value {
    let mut value = Map::new();
    value.insert("name".to_string(), json!(param.name));
    value.insert("type".to_string(), json!(param.type_annotation));
    value.insert("description".to_string(), json!(param.description));
    if param.optional {
        value.insert("optional".to_string(), json!(true));
    }
    if let Some(default_value) = &param.default_value {
        value.insert("default".to_string(), json!(default_value));
    }
    Value::Object(value)
}

fn return_to_json(return_doc: &ApiReturnDoc) -> Value {
    json!({
        "type": return_doc.type_annotation,
        "description": return_doc.description,
    })
}

fn normalize_doc_file_path(file_path: &str) -> String {
    let normalized = file_path.replace('\\', "/");
    for prefix in ["npm/", "packages/", "crates/", "src/"] {
        if let Some(index) = normalized.find(prefix) {
            if index == 0 || normalized.as_bytes().get(index - 1) == Some(&b'/') {
                return normalized[index..].to_string();
            }
        }
    }

    normalized.trim_start_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ApiDocTag, ApiParamDoc};

    #[test]
    fn generated_docs_data_counts_and_normalizes_paths() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "/repo/src/math.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "clamp".to_string(),
                kind: "function".to_string(),
                description: "Clamps a number.".to_string(),
                params: vec![ApiParamDoc {
                    name: "value".to_string(),
                    type_annotation: "number".to_string(),
                    description: "Input.".to_string(),
                    optional: false,
                    default_value: None,
                }],
                returns: None,
                examples: vec![],
                tags: vec![ApiDocTag { tag: "deprecated".to_string(), value: String::new() }],
                private: false,
                file: "/repo/src/math.ts".to_string(),
                line: 10,
                end_line: 10,
                signature: Some("export function clamp(value: number): number".to_string()),
                members: vec![],
            }],
        }];

        let json = generate_docs_data_json(&docs, "2026-05-22T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["summary"]["modules"], 1);
        assert_eq!(value["summary"]["byKind"]["function"], 1);
        assert_eq!(value["summary"]["members"], 0);
        assert_eq!(value["summary"]["params"], 1);
        assert_eq!(value["summary"]["deprecated"], 1);
        assert_eq!(value["modules"][0]["file"], "src/math.ts");
        assert_eq!(value["modules"][0]["entries"][0]["file"], "src/math.ts");
        assert_eq!(value["modules"][0]["entries"][0]["endLine"], 10);
    }

    #[test]
    fn generated_docs_data_carries_module_description() {
        let docs = vec![ApiDocModule {
            description: "The entry for gunshi context.".to_string(),
            file: "/repo/src/context.ts".to_string(),
            entries: vec![],
        }];

        let json = generate_docs_data_json(&docs, "2026-05-31T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["modules"][0]["description"], "The entry for gunshi context.");
    }

    #[test]
    fn entry_without_file_omits_source_location() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "/repo/src/combinators.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "Combinator".to_string(),
                kind: "type".to_string(),
                description: "A combinator.".to_string(),
                params: vec![],
                returns: None,
                examples: vec![],
                tags: vec![],
                private: false,
                // External-package source: no in-repo location.
                file: String::new(),
                line: 15,
                end_line: 23,
                signature: Some("type Combinator = unknown".to_string()),
                members: vec![],
            }],
        }];

        let json = generate_docs_data_json(&docs, "2026-05-31T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let entry = &value["modules"][0]["entries"][0];

        assert_eq!(entry["name"], "Combinator");
        assert_eq!(entry["signature"], "type Combinator = unknown");
        // No source location keys, so no absolute local path can leak.
        assert!(entry.get("file").is_none());
        assert!(entry.get("line").is_none());
        assert!(entry.get("endLine").is_none());
    }
}
