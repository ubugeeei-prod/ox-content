use oxc_span::SourceType;

use super::super::*;
use crate::extractor::DocExtractor;

#[test]
fn destructured_parameter_merges_jsdoc_name_without_unknown_duplicate() {
    let source = r"
/**
 * Resolve command line arguments.
 *
 * @param args - Argument schema.
 * @param tokens - Parsed tokens.
 * @param resolveArgs - Resolve options.
 */
export declare function resolveArgs<A extends Args>(
    args: A,
    tokens: ArgToken[],
    { shortGrouping, skipPositional, toKebab }?: ResolveArgs
): void;
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "resolver.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let entry = entries.iter().find(|entry| entry.name == "resolveArgs").unwrap();

    assert_eq!(
        entry.params.iter().map(|param| param.name.as_str()).collect::<Vec<_>>(),
        ["args", "tokens", "resolveArgs"]
    );
    assert_eq!(entry.params[2].type_annotation, "ResolveArgs");
    assert_eq!(entry.params[2].description, "Resolve options.");
    assert!(entry.params[2].optional);
}

#[test]
fn function_valued_property_merges_extracted_types_with_description_only_tags() {
    let source = r"
/**
 * Argument schema.
 */
export interface ArgSchema {
    /**
     * Parses a raw value.
     * @param value - Raw string value from command line.
     * @returns Parsed value.
     */
    parse?: (value: string) => any;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "schema.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let member = &entries[0].members[0];

    assert_eq!(member.name, "parse");
    assert_eq!(member.kind, NormalizedMemberKind::Property);
    assert_eq!(member.type_annotation.as_deref(), Some("(value: string) => any"));
    assert_eq!(member.params.len(), 1);
    assert_eq!(member.params[0].name, "value");
    assert_eq!(member.params[0].type_annotation, "string");
    assert_eq!(member.params[0].description, "Raw string value from command line.");
    assert_eq!(
        member.returns,
        Some(NormalizedReturnDoc {
            type_annotation: "any".to_string(),
            description: "Parsed value.".to_string(),
            members: Vec::new()
        })
    );
}

#[test]
fn function_return_type_literal_members_are_normalized() {
    let source = r"
/**
 * Resolve arguments.
 * @returns Resolved args.
 */
export function resolveArgs<A extends Args>(): {
    values: ArgValues<A>;
    positionals: string[];
    error: AggregateError | undefined;
} {
    return {} as any;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "resolver.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let returns = entries[0].returns.as_ref().unwrap();

    assert_eq!(returns.type_annotation, "object");
    assert_eq!(returns.description, "Resolved args.");
    assert_eq!(
        returns.members.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
        ["values", "positionals", "error"]
    );
    assert_eq!(returns.members[0].type_annotation.as_deref(), Some("ArgValues<A>"));
    assert_eq!(returns.members[2].type_annotation.as_deref(), Some("AggregateError | undefined"));
}
