use oxc_span::SourceType;

use super::super::*;
use crate::extractor::DocExtractor;

#[test]
fn interface_with_properties_emits_members() {
    let source = r"
/**
 * Runtime command.
 */
export interface Command {
    /** Command name. */
    readonly name: string;
    /** Positional arguments. */
    args?: string[];
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "command.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let members = &entries[0].members;

    assert_eq!(members.len(), 2);
    assert_eq!(members[0].name, "name");
    assert_eq!(members[0].kind, NormalizedMemberKind::Property);
    assert_eq!(members[0].type_annotation.as_deref(), Some("string"));
    assert_eq!(members[0].description, "Command name.");
    assert!(members[0].readonly);
    assert!(!members[0].optional);
    assert_eq!(members[1].name, "args");
    assert_eq!(members[1].type_annotation.as_deref(), Some("string[]"));
    assert!(members[1].optional);
}

#[test]
fn property_default_tags_are_normalized_to_member_defaults() {
    let source = r#"
/**
 * Runtime options.
 */
export interface Options {
    /**
     * Request timeout.
     * @default 5000
     */
    timeout?: number;
    /**
     * Retry mode.
     * @defaultValue "exponential"
     */
    retryMode?: "none" | "linear" | "exponential";
    /** HTTP options. */
    http: {
        /**
         * Request headers.
         * @defaultValue {}
         */
        headers: Record<string, string>;
    };
}
"#;

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "options.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let members = &entries[0].members;

    assert_eq!(members[0].name, "timeout");
    assert_eq!(members[0].default_value.as_deref(), Some("5000"));
    assert!(!members[0].tags.contains_key("default"));

    assert_eq!(members[1].name, "retryMode");
    assert_eq!(members[1].default_value.as_deref(), Some("\"exponential\""));
    assert!(!members[1].tags.contains_key("defaultValue"));

    assert_eq!(members[2].name, "http");
    assert_eq!(members[2].members[0].name, "headers");
    assert_eq!(members[2].members[0].default_value.as_deref(), Some("{}"));
    assert!(!members[2].members[0].tags.contains_key("defaultValue"));
}

#[test]
fn class_and_type_alias_property_default_tags_are_normalized() {
    let source = r"
/**
 * Runtime options.
 */
export class RuntimeOptions {
    /**
     * Enables cache.
     * @default true
     */
    cache?: boolean;
}

/**
 * Retry options.
 */
export type RetryOptions = {
    /**
     * Retry count.
     * @defaultValue 3
     */
    retries?: number;
};
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "options.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);

    let class = entries.iter().find(|entry| entry.name == "RuntimeOptions").unwrap();
    assert_eq!(class.members[0].name, "cache");
    assert_eq!(class.members[0].default_value.as_deref(), Some("true"));
    assert!(!class.members[0].tags.contains_key("default"));

    let alias = entries.iter().find(|entry| entry.name == "RetryOptions").unwrap();
    assert_eq!(alias.members[0].name, "retries");
    assert_eq!(alias.members[0].default_value.as_deref(), Some("3"));
    assert!(!alias.members[0].tags.contains_key("defaultValue"));
}

#[test]
fn interface_property_type_literal_members_are_normalized() {
    let source = r"
/**
 * Request options.
 */
export interface RequestOptions {
    /** HTTP options. */
    http: {
        /** Request timeout. */
        timeout?: number;
        /** Request headers. */
        headers: Record<string, string>;
    };
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "request.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let http = &entries[0].members[0];

    assert_eq!(http.name, "http");
    assert_eq!(http.kind, NormalizedMemberKind::Property);
    assert_eq!(http.description, "HTTP options.");
    assert_eq!(http.members.len(), 2);
    assert_eq!(http.members[0].name, "timeout");
    assert_eq!(http.members[0].description, "Request timeout.");
    assert_eq!(http.members[0].type_annotation.as_deref(), Some("number"));
    assert!(http.members[0].optional);
    assert_eq!(http.members[1].name, "headers");
    assert_eq!(http.members[1].type_annotation.as_deref(), Some("Record<string, string>"));
}

#[test]
fn normal_property_suppresses_description_only_returns_tag() {
    let source = r"
/**
 * Plugin context.
 */
export interface PluginContext {
    /**
     * Get the global options.
     *
     * @returns A map of global options.
     */
    readonly globalOptions: Map<string, ArgSchema>;
}
";

    let extractor = DocExtractor::new();
    let items = extractor.extract_source(source, "context.ts", SourceType::ts()).unwrap();
    let entries = normalize_doc_items(items, false);
    let member = &entries[0].members[0];

    assert_eq!(member.name, "globalOptions");
    assert_eq!(member.kind, NormalizedMemberKind::Property);
    assert_eq!(member.description, "Get the global options.");
    assert_eq!(member.type_annotation.as_deref(), Some("Map<string, ArgSchema>"));
    assert!(member.readonly);
    assert!(member.returns.is_none());
}
