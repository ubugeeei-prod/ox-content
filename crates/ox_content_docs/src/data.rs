use serde_json::{json, Map, Value};

use crate::model::{
    ApiDocEntry, ApiDocMember, ApiDocModule, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc,
    ApiTypeParamDoc,
};
#[allow(unused_imports)]
use crate::profile_span;

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

/// Generates the machine-readable docs data JSON payload.
///
/// The returned JSON is pretty-formatted and preserves the shape consumed by
/// TypeScript docs tooling.
pub fn generate_docs_data_json(
    docs: &[ApiDocModule],
    generated_at: &str,
) -> serde_json::Result<String> {
    profile_span!("docs::generate_json");
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

fn module_to_json(module: &ApiDocModule) -> Value {
    let mut value = Map::new();
    value.insert("file".to_string(), json!(normalize_doc_file_path(&module.file)));
    value.insert("description".to_string(), json!(module.description));
    if !module.examples.is_empty() {
        value.insert("examples".to_string(), json!(module.examples));
    }
    if !module.tags.is_empty() {
        value.insert(
            "tags".to_string(),
            Value::Object(
                module.tags.iter().map(|tag| (tag.tag.clone(), json!(tag.value))).collect(),
            ),
        );
    }
    value.insert(
        "entries".to_string(),
        Value::Array(module.entries.iter().map(entry_to_json).collect()),
    );

    Value::Object(value)
}

fn entry_to_json(entry: &ApiDocEntry) -> Value {
    let mut value = Map::new();
    value.insert("name".to_string(), json!(entry.name));
    value.insert("kind".to_string(), json!(entry.kind));
    value.insert("description".to_string(), json!(entry.description));

    if !entry.type_parameters.is_empty() {
        value.insert(
            "typeParameters".to_string(),
            Value::Array(entry.type_parameters.iter().map(type_param_to_json).collect()),
        );
    }
    if !entry.params.is_empty() {
        value.insert(
            "params".to_string(),
            Value::Array(entry.params.iter().map(param_to_json).collect()),
        );
    }
    if let Some(returns) = &entry.returns {
        value.insert("returns".to_string(), return_to_json(returns));
    }
    if !entry.throws.is_empty() {
        value.insert(
            "throws".to_string(),
            Value::Array(entry.throws.iter().map(throws_to_json).collect()),
        );
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
    if !entry.extends.is_empty() {
        value.insert("extends".to_string(), json!(entry.extends));
    }
    if !entry.implements.is_empty() {
        value.insert("implements".to_string(), json!(entry.implements));
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
    if let Some(default_value) = &member.default_value {
        value.insert("default".to_string(), json!(default_value));
    }
    if !member.params.is_empty() {
        value.insert(
            "params".to_string(),
            Value::Array(member.params.iter().map(param_to_json).collect()),
        );
    }
    if !member.type_parameters.is_empty() {
        value.insert(
            "typeParameters".to_string(),
            Value::Array(member.type_parameters.iter().map(type_param_to_json).collect()),
        );
    }
    if let Some(returns) = &member.returns {
        value.insert("returns".to_string(), return_to_json(returns));
    }
    if !member.throws.is_empty() {
        value.insert(
            "throws".to_string(),
            Value::Array(member.throws.iter().map(throws_to_json).collect()),
        );
    }
    if !member.members.is_empty() {
        value.insert(
            "members".to_string(),
            Value::Array(member.members.iter().map(member_to_json).collect()),
        );
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
    if !member.implementation_of.is_empty() {
        value.insert("implementationOf".to_string(), json!(member.implementation_of));
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
    let mut value = Map::new();
    value.insert("type".to_string(), json!(return_doc.type_annotation));
    value.insert("description".to_string(), json!(return_doc.description));
    if !return_doc.members.is_empty() {
        value.insert(
            "members".to_string(),
            Value::Array(return_doc.members.iter().map(member_to_json).collect()),
        );
    }
    Value::Object(value)
}

fn throws_to_json(throws: &ApiThrowsDoc) -> Value {
    let mut value = Map::new();
    if let Some(type_annotation) = &throws.type_annotation {
        value.insert("type".to_string(), json!(type_annotation));
    }
    value.insert("description".to_string(), json!(throws.description));
    Value::Object(value)
}

fn type_param_to_json(type_param: &ApiTypeParamDoc) -> Value {
    let mut value = Map::new();
    value.insert("name".to_string(), json!(type_param.name));
    if let Some(constraint) = &type_param.constraint {
        value.insert("constraint".to_string(), json!(constraint));
    }
    if let Some(default) = &type_param.default {
        value.insert("default".to_string(), json!(default));
    }
    if !type_param.description.is_empty() {
        value.insert("description".to_string(), json!(type_param.description));
    }
    Value::Object(value)
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
    use crate::model::{ApiDocMember, ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc};

    #[test]
    fn generated_docs_data_counts_and_normalizes_paths() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/math.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "clamp".to_string(),
                kind: "function".to_string(),
                description: "Clamps a number.".to_string(),
                params: vec![ApiParamDoc {
                    name: "value".to_string(),
                    type_annotation: "number".to_string(),
                    description: "Input.".to_string(),
                    ..ApiParamDoc::default()
                }],
                throws: vec![ApiThrowsDoc {
                    type_annotation: Some("RangeError".to_string()),
                    description: "When value is out of range.".to_string(),
                }],
                tags: vec![ApiDocTag { tag: "deprecated".to_string(), ..Default::default() }],
                file: "/repo/src/math.ts".to_string(),
                line: 10,
                end_line: 10,
                signature: Some("export function clamp(value: number): number".to_string()),
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
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
        assert_eq!(
            value["modules"][0]["entries"][0]["throws"][0],
            json!({
                "type": "RangeError",
                "description": "When value is out of range."
            })
        );
    }

    #[test]
    fn generated_docs_data_carries_module_metadata() {
        let docs = vec![ApiDocModule {
            description: "The entry for gunshi context.".to_string(),
            file: "/repo/src/context.ts".to_string(),
            examples: vec!["```ts\ncreateCommandContext()\n```".to_string()],
            tags: vec![ApiDocTag {
                tag: "experimental".to_string(),
                value: "This entry point is experimental.".to_string(),
            }],
            ..ApiDocModule::default()
        }];

        let json = generate_docs_data_json(&docs, "2026-05-31T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["summary"]["examples"], 1);
        assert_eq!(value["modules"][0]["description"], "The entry for gunshi context.");
        assert_eq!(value["modules"][0]["examples"][0], "```ts\ncreateCommandContext()\n```");
        assert_eq!(
            value["modules"][0]["tags"]["experimental"],
            "This entry point is experimental."
        );
    }

    #[test]
    fn entry_without_file_omits_source_location() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/combinators.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "Combinator".to_string(),
                kind: "type".to_string(),
                description: "A combinator.".to_string(),
                // External-package source: no in-repo location.
                file: String::new(),
                line: 15,
                end_line: 23,
                signature: Some("type Combinator = unknown".to_string()),
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
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

    #[test]
    fn entry_type_parameters_serialize_to_json() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/make.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "make".to_string(),
                kind: "function".to_string(),
                description: "Make.".to_string(),
                file: "/repo/src/make.ts".to_string(),
                type_parameters: vec![
                    ApiTypeParamDoc {
                        name: "G".to_string(),
                        constraint: Some("Base".to_string()),
                        default: Some("Default".to_string()),
                        ..ApiTypeParamDoc::default()
                    },
                    ApiTypeParamDoc {
                        name: "T".to_string(),
                        description: "Value.".to_string(),
                        ..ApiTypeParamDoc::default()
                    },
                ],
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
        }];

        let json = generate_docs_data_json(&docs, "2026-05-31T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let type_params = &value["modules"][0]["entries"][0]["typeParameters"];

        assert_eq!(type_params[0]["name"], "G");
        assert_eq!(type_params[0]["constraint"], "Base");
        assert_eq!(type_params[0]["default"], "Default");
        assert!(type_params[0].get("description").is_none());
        assert_eq!(type_params[1]["name"], "T");
        assert_eq!(type_params[1]["description"], "Value.");
    }

    #[test]
    fn return_members_serialize_to_json() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/resolver.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "resolveArgs".to_string(),
                kind: "function".to_string(),
                description: "Resolve.".to_string(),
                returns: Some(ApiReturnDoc {
                    type_annotation: "object".to_string(),
                    description: "Resolved args.".to_string(),
                    members: vec![ApiDocMember {
                        name: "values".to_string(),
                        kind: "property".to_string(),
                        type_annotation: Some("ArgValues<A>".to_string()),
                        default_value: Some("{}".to_string()),
                        line: 3,
                        end_line: 3,
                        ..ApiDocMember::default()
                    }],
                }),
                file: "/repo/src/resolver.ts".to_string(),
                end_line: 6,
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
        }];

        let json = generate_docs_data_json(&docs, "2026-05-31T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let returns = &value["modules"][0]["entries"][0]["returns"];

        assert_eq!(returns["type"], "object");
        assert_eq!(returns["members"][0]["name"], "values");
        assert_eq!(returns["members"][0]["type"], "ArgValues<A>");
        assert_eq!(returns["members"][0]["default"], "{}");
    }

    #[test]
    fn property_members_serialize_to_json() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/options.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "Options".to_string(),
                kind: "interface".to_string(),
                description: "Request options.".to_string(),
                file: "/repo/src/options.ts".to_string(),
                end_line: 8,
                signature: Some("export interface Options".to_string()),
                members: vec![ApiDocMember {
                    name: "http".to_string(),
                    kind: "property".to_string(),
                    description: "HTTP options.".to_string(),
                    type_annotation: Some("{ timeout?: number }".to_string()),
                    members: vec![ApiDocMember {
                        name: "timeout".to_string(),
                        kind: "property".to_string(),
                        description: "Request timeout.".to_string(),
                        type_annotation: Some("number".to_string()),
                        optional: true,
                        line: 4,
                        end_line: 4,
                        ..ApiDocMember::default()
                    }],
                    line: 3,
                    end_line: 6,
                    ..ApiDocMember::default()
                }],
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
        }];

        let json = generate_docs_data_json(&docs, "2026-05-31T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let nested_member = &value["modules"][0]["entries"][0]["members"][0]["members"][0];

        assert_eq!(nested_member["name"], "timeout");
        assert_eq!(nested_member["kind"], "property");
        assert_eq!(nested_member["description"], "Request timeout.");
        assert_eq!(nested_member["type"], "number");
        assert_eq!(nested_member["optional"], true);
    }

    #[test]
    fn index_signature_members_serialize_to_json() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/args.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "Args".to_string(),
                kind: "interface".to_string(),
                description: "Arguments.".to_string(),
                file: "/repo/src/args.ts".to_string(),
                end_line: 5,
                signature: Some("export interface Args".to_string()),
                members: vec![ApiDocMember {
                    name: "[option: string]".to_string(),
                    kind: "indexSignature".to_string(),
                    description: "Argument schema by option name.".to_string(),
                    signature: Some("readonly [option: string]: ArgSchema".to_string()),
                    type_annotation: Some("ArgSchema".to_string()),
                    params: vec![ApiParamDoc {
                        name: "option".to_string(),
                        type_annotation: "string".to_string(),
                        ..ApiParamDoc::default()
                    }],
                    readonly: true,
                    line: 4,
                    end_line: 4,
                    ..ApiDocMember::default()
                }],
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
        }];

        let json = generate_docs_data_json(&docs, "2026-05-31T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let member = &value["modules"][0]["entries"][0]["members"][0];

        assert_eq!(member["name"], "[option: string]");
        assert_eq!(member["kind"], "indexSignature");
        assert_eq!(member["signature"], "readonly [option: string]: ArgSchema");
        assert_eq!(member["type"], "ArgSchema");
        assert_eq!(member["params"][0]["name"], "option");
        assert_eq!(member["params"][0]["type"], "string");
        assert_eq!(member["readonly"], true);
    }

    #[test]
    fn function_valued_property_metadata_serializes_to_json() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/schema.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "ArgSchema".to_string(),
                kind: "interface".to_string(),
                description: "Argument schema.".to_string(),
                file: "/repo/src/schema.ts".to_string(),
                end_line: 10,
                signature: Some("export interface ArgSchema".to_string()),
                members: vec![ApiDocMember {
                    name: "parse".to_string(),
                    kind: "property".to_string(),
                    description: "Parses a raw value.".to_string(),
                    type_annotation: Some("(value: string) => any".to_string()),
                    params: vec![ApiParamDoc {
                        name: "value".to_string(),
                        type_annotation: "string".to_string(),
                        description: "Raw string value from command line.".to_string(),
                        ..ApiParamDoc::default()
                    }],
                    type_parameters: vec![ApiTypeParamDoc {
                        name: "T".to_string(),
                        constraint: Some("Base".to_string()),
                        default: Some("Default".to_string()),
                        description: "Parsed value type.".to_string(),
                    }],
                    returns: Some(ApiReturnDoc {
                        type_annotation: "any".to_string(),
                        description: "Parsed value.".to_string(),
                        ..ApiReturnDoc::default()
                    }),
                    throws: vec![ApiThrowsDoc {
                        type_annotation: Some("ParseError".to_string()),
                        description: "When the raw value cannot be parsed.".to_string(),
                    }],
                    optional: true,
                    line: 5,
                    end_line: 10,
                    ..ApiDocMember::default()
                }],
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
        }];

        let json = generate_docs_data_json(&docs, "2026-05-31T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let member = &value["modules"][0]["entries"][0]["members"][0];

        assert_eq!(member["name"], "parse");
        assert_eq!(member["kind"], "property");
        assert_eq!(member["type"], "(value: string) => any");
        assert_eq!(member["params"][0]["name"], "value");
        assert_eq!(member["params"][0]["type"], "string");
        assert_eq!(member["typeParameters"][0]["name"], "T");
        assert_eq!(member["typeParameters"][0]["constraint"], "Base");
        assert_eq!(member["typeParameters"][0]["default"], "Default");
        assert_eq!(member["typeParameters"][0]["description"], "Parsed value type.");
        assert_eq!(member["returns"]["type"], "any");
        assert_eq!(member["returns"]["description"], "Parsed value.");
        assert_eq!(member["throws"][0]["type"], "ParseError");
        assert_eq!(member["throws"][0]["description"], "When the raw value cannot be parsed.");
        assert_eq!(member["optional"], true);
    }

    #[test]
    fn function_type_alias_metadata_serializes_to_json() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/plugin.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "CommandRunner".to_string(),
                kind: "type".to_string(),
                description: "Command runner type.".to_string(),
                params: vec![ApiParamDoc {
                    name: "ctx".to_string(),
                    type_annotation: "Readonly<CommandContext<G>>".to_string(),
                    ..ApiParamDoc::default()
                }],
                returns: Some(ApiReturnDoc {
                    type_annotation: "Awaitable<string | void>".to_string(),
                    ..ApiReturnDoc::default()
                }),
                file: "/repo/src/plugin.ts".to_string(),
                signature: Some(
                    "export type CommandRunner<G> = (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>".to_string(),
                ),
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
        }];

        let json = generate_docs_data_json(&docs, "2026-06-05T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let entry = &value["modules"][0]["entries"][0];

        assert_eq!(entry["kind"], "type");
        assert_eq!(entry["name"], "CommandRunner");
        assert_eq!(entry["params"][0]["name"], "ctx");
        assert_eq!(entry["params"][0]["type"], "Readonly<CommandContext<G>>");
        assert_eq!(entry["returns"]["type"], "Awaitable<string | void>");
    }

    #[test]
    fn function_type_alias_return_without_description_serializes_to_json() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/plugin.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "OnPluginExtension".to_string(),
                kind: "type".to_string(),
                description: "Plugin extension hook.".to_string(),
                params: vec![
                    ApiParamDoc {
                        name: "ctx".to_string(),
                        type_annotation: "Readonly<CommandContext<G>>".to_string(),
                        description: "The command context.".to_string(),
                        ..ApiParamDoc::default()
                    },
                    ApiParamDoc {
                        name: "cmd".to_string(),
                        type_annotation: "Readonly<Command<G>>".to_string(),
                        description: "The command.".to_string(),
                        ..ApiParamDoc::default()
                    },
                ],
                returns: Some(ApiReturnDoc {
                    type_annotation: "Awaitable<void>".to_string(),
                    ..ApiReturnDoc::default()
                }),
                file: "/repo/src/plugin.ts".to_string(),
                end_line: 5,
                signature: Some(
                    "export type OnPluginExtension<G> = (ctx: Readonly<CommandContext<G>>, cmd: Readonly<Command<G>>) => Awaitable<void>"
                        .to_string(),
                ),
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
        }];

        let json = generate_docs_data_json(&docs, "2026-06-05T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let entry = &value["modules"][0]["entries"][0];

        assert_eq!(entry["name"], "OnPluginExtension");
        assert_eq!(entry["params"][0]["type"], "Readonly<CommandContext<G>>");
        assert_eq!(entry["params"][1]["type"], "Readonly<Command<G>>");
        assert_eq!(entry["returns"]["type"], "Awaitable<void>");
        assert_eq!(entry["returns"]["description"], "");
    }

    #[test]
    fn heritage_and_implementation_metadata_serialize_to_json() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/adapter.ts".to_string(),
            entries: vec![ApiDocEntry {
                name: "DefaultTranslation".to_string(),
                kind: "class".to_string(),
                description: "Default adapter.".to_string(),
                file: "/repo/src/adapter.ts".to_string(),
                end_line: 10,
                signature: Some(
                    "class DefaultTranslation implements TranslationAdapter".to_string(),
                ),
                extends: vec!["BaseTranslation".to_string()],
                implements: vec!["TranslationAdapter".to_string()],
                members: vec![ApiDocMember {
                    name: "getResource".to_string(),
                    kind: "method".to_string(),
                    description: "Gets a locale resource.".to_string(),
                    signature: Some(
                        "getResource(locale: string): Record<string, string> | undefined"
                            .to_string(),
                    ),
                    implementation_of: vec!["TranslationAdapter.getResource".to_string()],
                    line: 5,
                    end_line: 8,
                    ..ApiDocMember::default()
                }],
                ..ApiDocEntry::default()
            }],
            ..ApiDocModule::default()
        }];

        let json = generate_docs_data_json(&docs, "2026-06-05T00:00:00.000Z").unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        let entry = &value["modules"][0]["entries"][0];

        assert_eq!(entry["extends"][0], "BaseTranslation");
        assert_eq!(entry["implements"][0], "TranslationAdapter");
        assert_eq!(entry["members"][0]["implementationOf"][0], "TranslationAdapter.getResource");
    }
}
