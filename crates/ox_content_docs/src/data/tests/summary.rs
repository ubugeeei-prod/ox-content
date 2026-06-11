use serde_json::{json, Value};

use super::super::generate_docs_data_json;
use crate::model::{
    ApiDocEntry, ApiDocModule, ApiDocTag, ApiParamDoc, ApiThrowsDoc, ApiTypeParamDoc,
};

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
    assert_eq!(value["modules"][0]["tags"]["experimental"], "This entry point is experimental.");
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
