use serde_json::Value;

use super::super::generate_docs_data_json;
use crate::model::{
    ApiDocEntry, ApiDocMember, ApiDocModule, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc,
    ApiTypeParamDoc,
};

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
