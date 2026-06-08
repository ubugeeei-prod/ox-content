use super::*;

#[test]
fn normalized_doc_entry_maps_members_to_js_shape() {
    let entry = NormalizedDocEntry {
        name: "Command".to_string(),
        kind: NormalizedDocKind::Interface,
        description: "Runtime command.".to_string(),
        params: vec![],
        returns: None,
        examples: vec![],
        tags: BTreeMap::new(),
        private: false,
        file: "command.ts".to_string(),
        line: 1,
        end_line: 8,
        signature: Some("export interface Command".to_string()),
        extends: vec![],
        implements: vec![],
        has_body: false,
        members: vec![NormalizedMember {
            name: "name".to_string(),
            kind: NormalizedMemberKind::Property,
            description: "Command name.".to_string(),
            signature: None,
            type_annotation: Some("string".to_string()),
            default_value: Some("\"cli\"".to_string()),
            params: vec![],
            type_parameters: vec![NormalizedTypeParam {
                name: "T".to_string(),
                constraint: Some("Base".to_string()),
                default: Some("Default".to_string()),
                description: "Value type.".to_string(),
            }],
            returns: None,
            members: vec![NormalizedMember {
                name: "timeout".to_string(),
                kind: NormalizedMemberKind::Property,
                description: "Request timeout.".to_string(),
                signature: None,
                type_annotation: Some("number".to_string()),
                default_value: Some("5000".to_string()),
                params: vec![],
                type_parameters: vec![],
                returns: None,
                members: vec![],
                optional: true,
                readonly: false,
                r#static: false,
                private: false,
                tags: BTreeMap::new(),
                line: 5,
                end_line: 5,
            }],
            optional: true,
            readonly: true,
            r#static: false,
            private: false,
            tags: BTreeMap::new(),
            line: 4,
            end_line: 4,
        }],
        type_parameters: vec![],
    };

    let js_entry = map_normalized_doc_entry(entry);
    let member = &js_entry.members.as_ref().unwrap()[0];

    assert_eq!(member.name, "name");
    assert_eq!(member.kind, "property");
    assert_eq!(member.r#type.as_deref(), Some("string"));
    assert_eq!(member.r#default.as_deref(), Some("\"cli\""));
    let type_param = &member.type_parameters.as_ref().unwrap()[0];
    assert_eq!(type_param.name, "T");
    assert_eq!(type_param.constraint.as_deref(), Some("Base"));
    assert_eq!(type_param.r#default.as_deref(), Some("Default"));
    assert_eq!(type_param.description, "Value type.");
    assert_eq!(member.optional, Some(true));
    assert_eq!(member.readonly, Some(true));
    let nested_member = &member.members.as_ref().unwrap()[0];
    assert_eq!(nested_member.name, "timeout");
    assert_eq!(nested_member.kind, "property");
    assert_eq!(nested_member.r#type.as_deref(), Some("number"));
    assert_eq!(nested_member.r#default.as_deref(), Some("5000"));
    assert_eq!(nested_member.description, "Request timeout.");
    assert_eq!(nested_member.optional, Some(true));
}

#[test]
fn normalized_doc_entry_maps_index_signature_members_to_js_shape() {
    let entry = NormalizedDocEntry {
        name: "Args".to_string(),
        kind: NormalizedDocKind::Interface,
        description: "Arguments.".to_string(),
        params: vec![],
        returns: None,
        examples: vec![],
        tags: BTreeMap::new(),
        private: false,
        file: "args.ts".to_string(),
        line: 1,
        end_line: 5,
        signature: Some("export interface Args".to_string()),
        extends: vec![],
        implements: vec![],
        has_body: false,
        members: vec![NormalizedMember {
            name: "[option: string]".to_string(),
            kind: NormalizedMemberKind::IndexSignature,
            description: "Argument schema by option name.".to_string(),
            signature: Some("readonly [option: string]: ArgSchema".to_string()),
            type_annotation: Some("ArgSchema".to_string()),
            default_value: None,
            params: vec![ox_content_docs::NormalizedParamDoc {
                name: "option".to_string(),
                type_annotation: "string".to_string(),
                description: String::new(),
                optional: false,
                default_value: None,
            }],
            type_parameters: vec![],
            returns: None,
            members: vec![],
            optional: false,
            readonly: true,
            r#static: false,
            private: false,
            tags: BTreeMap::new(),
            line: 4,
            end_line: 4,
        }],
        type_parameters: vec![],
    };

    let js_entry = map_normalized_doc_entry(entry);
    let member = &js_entry.members.as_ref().unwrap()[0];

    assert_eq!(member.name, "[option: string]");
    assert_eq!(member.kind, "indexSignature");
    assert_eq!(member.signature.as_deref(), Some("readonly [option: string]: ArgSchema"));
    assert_eq!(member.r#type.as_deref(), Some("ArgSchema"));
    assert_eq!(member.params.as_ref().unwrap()[0].name, "option");
    assert_eq!(member.params.as_ref().unwrap()[0].r#type, "string");
    assert_eq!(member.readonly, Some(true));
}

#[test]
fn normalized_doc_entry_maps_heritage_to_js_shape() {
    let entry = NormalizedDocEntry {
        name: "DefaultTranslation".to_string(),
        kind: NormalizedDocKind::Class,
        description: "Default adapter.".to_string(),
        params: vec![],
        returns: None,
        examples: vec![],
        tags: BTreeMap::new(),
        private: false,
        file: "adapter.ts".to_string(),
        line: 1,
        end_line: 10,
        signature: Some("class DefaultTranslation implements TranslationAdapter".to_string()),
        extends: vec!["BaseTranslation".to_string()],
        implements: vec!["TranslationAdapter".to_string()],
        has_body: false,
        members: vec![],
        type_parameters: vec![],
    };

    let js_entry = map_normalized_doc_entry(entry);

    assert_eq!(js_entry.extends, Some(vec!["BaseTranslation".to_string()]));
    assert_eq!(js_entry.implements, Some(vec!["TranslationAdapter".to_string()]));
}

#[test]
fn normalized_doc_entry_maps_return_members_to_js_shape() {
    let entry = NormalizedDocEntry {
        name: "resolveArgs".to_string(),
        kind: NormalizedDocKind::Function,
        description: "Resolve.".to_string(),
        params: vec![],
        returns: Some(NormalizedReturnDoc {
            type_annotation: "object".to_string(),
            description: "Resolved args.".to_string(),
            members: vec![NormalizedMember {
                name: "values".to_string(),
                kind: NormalizedMemberKind::Property,
                description: String::new(),
                signature: None,
                type_annotation: Some("ArgValues<A>".to_string()),
                default_value: None,
                params: vec![],
                type_parameters: vec![],
                returns: None,
                members: vec![],
                optional: false,
                readonly: false,
                r#static: false,
                private: false,
                tags: BTreeMap::new(),
                line: 3,
                end_line: 3,
            }],
        }),
        examples: vec![],
        tags: BTreeMap::new(),
        private: false,
        file: "resolver.ts".to_string(),
        line: 1,
        end_line: 8,
        signature: Some("export function resolveArgs(): object".to_string()),
        extends: vec![],
        implements: vec![],
        has_body: false,
        members: vec![],
        type_parameters: vec![],
    };

    let js_entry = map_normalized_doc_entry(entry);
    let returns = js_entry.returns.as_ref().unwrap();
    let member = &returns.members.as_ref().unwrap()[0];

    assert_eq!(returns.r#type, "object");
    assert_eq!(member.name, "values");
    assert_eq!(member.r#type.as_deref(), Some("ArgValues<A>"));
}
