use oxc_span::SourceType;

use super::super::*;
use crate::extractor::DocExtractor;

#[test]
fn interface_with_method_signatures_emits_method_members() {
    let source = r"
/**
 * Runtime command.
 */
export interface Command {
    /**
     * Runs the command.
     * @param ctx - Runtime context
     * @returns Run result
     * @throws {RunError} When the command fails.
     */
    run(ctx: Context): Promise<void>;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "command.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let member = &entries[0].members[0];

    assert_eq!(member.name, "run");
    assert_eq!(member.kind, NormalizedMemberKind::Method);
    assert_eq!(member.signature.as_deref(), Some("run(ctx: Context): Promise<void>"));
    assert_eq!(member.params.len(), 1);
    assert_eq!(member.params[0].name, "ctx");
    assert_eq!(member.params[0].type_annotation, "Context");
    assert_eq!(member.params[0].description, "Runtime context");
    assert_eq!(
        member.returns,
        Some(NormalizedReturnDoc {
            type_annotation: "Promise<void>".to_string(),
            description: "Run result".to_string(),
            members: Vec::new()
        })
    );
    assert_eq!(
        member.throws,
        vec![NormalizedThrowsDoc {
            type_annotation: Some("RunError".to_string()),
            description: "When the command fails.".to_string(),
        }]
    );
    assert!(!member.tags.contains_key("throws"));
}

#[test]
fn index_signature_members_are_normalized_with_parameter_and_value_types() {
    let source = r"
/**
 * Value type.
 */
export interface ArgSchema {}

/**
 * Arguments.
 */
export interface Args {
    /** Argument schema by option name. */
    readonly [option: string]: ArgSchema;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "args.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let args = entries.iter().find(|entry| entry.name == "Args").unwrap();
    let member = &args.members[0];

    assert_eq!(member.name, "[option: string]");
    assert_eq!(member.kind, NormalizedMemberKind::IndexSignature);
    assert_eq!(member.signature.as_deref(), Some("readonly [option: string]: ArgSchema"));
    assert_eq!(member.type_annotation.as_deref(), Some("ArgSchema"));
    assert_eq!(member.params[0].name, "option");
    assert_eq!(member.params[0].type_annotation, "string");
    assert!(member.readonly);
    assert!(member.returns.is_none());
}

#[test]
fn class_emits_constructor_static_method_and_property_members() {
    let source = r"
/**
 * Registry.
 */
export class Registry {
    /** Creates a registry. */
    constructor(name: string) {}
    /** Default registry. */
    static defaultName: string;
    /** Registers a value. */
    register(value: string): void {}
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "registry.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let members = &entries[0].members;

    assert_eq!(
        members.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
        ["constructor", "defaultName", "register"]
    );
    assert_eq!(members[0].kind, NormalizedMemberKind::Constructor);
    assert_eq!(members[0].params[0].type_annotation, "string");
    assert_eq!(members[1].kind, NormalizedMemberKind::Property);
    assert!(members[1].r#static);
    assert_eq!(members[1].type_annotation.as_deref(), Some("string"));
    assert_eq!(members[2].kind, NormalizedMemberKind::Method);
}

#[test]
fn enum_emits_enum_members_in_declaration_order() {
    let source = r"
/**
 * Available modes.
 */
export enum Mode {
    Fast = 'fast',
    Slow = 'slow',
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "mode.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let members = &entries[0].members;

    assert_eq!(
        members.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
        ["Fast", "Slow"]
    );
    assert!(members.iter().all(|member| member.kind == NormalizedMemberKind::EnumMember));
    assert_eq!(members[0].type_annotation.as_deref(), Some("'fast'"));
}

#[test]
fn member_visibility_tags_are_filtered_by_extractor_options() {
    let source = r"
/**
 * Runtime command.
 */
export interface Command {
    /** Command name. */
    name: string;
    /**
     * Internal token.
     * @internal
     */
    token: string;
    /**
     * Private secret.
     * @private
     */
    secret: string;
}
";

    let public_items =
        DocExtractor::new().extract_source(source, "command.ts", SourceType::ts()).unwrap();
    let public_entries = normalize_doc_items(public_items, false);
    assert_eq!(
        public_entries[0].members.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
        ["name"]
    );

    let all_items = DocExtractor::with_visibility(true, true)
        .extract_source(source, "command.ts", SourceType::ts())
        .unwrap();
    let all_entries = normalize_doc_items(all_items, false);
    assert_eq!(
        all_entries[0].members.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
        ["name", "token", "secret"]
    );
    assert!(all_entries[0].members[2].private);
}
