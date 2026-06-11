use serde_json::Value;

use super::super::generate_docs_data_json;
use crate::model::{ApiDocEntry, ApiDocMember, ApiDocModule, ApiParamDoc, ApiReturnDoc};

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
                "export type CommandRunner<G> = (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>"
                    .to_string(),
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
            signature: Some("class DefaultTranslation implements TranslationAdapter".to_string()),
            extends: vec!["BaseTranslation".to_string()],
            implements: vec!["TranslationAdapter".to_string()],
            members: vec![ApiDocMember {
                name: "getResource".to_string(),
                kind: "method".to_string(),
                description: "Gets a locale resource.".to_string(),
                signature: Some(
                    "getResource(locale: string): Record<string, string> | undefined".to_string(),
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
