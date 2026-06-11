//! Normalized documentation entries for JavaScript-facing generators.

// BTreeMap/BTreeSet keep generated tag and type-parameter output deterministic.
use std::collections::BTreeMap;

use phf::phf_set;

mod entry;
mod kind;
mod member;
mod model;

pub use entry::NormalizedDocEntry;
pub use kind::{NormalizedDocKind, NormalizedMemberKind};
pub use member::NormalizedMember;
pub use model::{
    NormalizedParamDoc, NormalizedReturnDoc, NormalizedThrowsDoc, NormalizedTypeParam,
};

use crate::extractor::{DocItem, DocTag, ParamDoc, TypeParamDoc};
#[allow(unused_imports)]
use crate::profile_span;

const UNKNOWN_TYPE: &str = "unknown";
const PARAM_TAG_NAME: &str = "param";
const EXAMPLE_TAG_NAME: &str = "example";
const PRIVATE_TAG_NAME: &str = "private";

static PARAM_TAG_NAMES: phf::Set<&'static str> = phf_set! {
    "param",
    "arg",
    "argument",
};

static RETURN_TAG_NAMES: phf::Set<&'static str> = phf_set! {
    "returns",
    "return",
};

static THROWS_TAG_NAMES: phf::Set<&'static str> = phf_set! {
    "throws",
    "exception",
};

static TYPE_PARAM_TAG_NAMES: phf::Set<&'static str> = phf_set! {
    "typeParam",
    "template",
};

/// Normalizes extracted documentation items into API reference entries.
#[must_use]
pub fn normalize_doc_items(items: Vec<DocItem>, type_parameters: bool) -> Vec<NormalizedDocEntry> {
    profile_span!("docs::normalize_items");
    items.into_iter().filter_map(|item| normalize_doc_item(item, type_parameters)).collect()
}

/// Normalizes a single extracted documentation item into an API reference entry.
///
/// `type_parameters` opts in to TSDoc-style type-parameter docs: when `true`,
/// `@typeParam` / `@template` tags are merged into structured type parameters and
/// removed from the generic tag map; when `false` they remain generic tags and
/// `type_parameters` stays empty (default JSDoc behavior).
#[must_use]
pub fn normalize_doc_item(item: DocItem, type_parameters: bool) -> Option<NormalizedDocEntry> {
    let kind = NormalizedDocKind::from_doc_item_kind(item.kind)?;

    let mut metadata = normalize_doc_metadata(&item.tags, type_parameters);
    merge_extracted_params(&mut metadata.params, item.params);
    merge_extracted_return(
        &mut metadata.returns,
        item.return_type,
        item.return_members,
        type_parameters,
    );
    let members = item
        .children
        .into_iter()
        .filter_map(|item| normalize_member(item, type_parameters))
        .collect();
    let type_parameters = if type_parameters {
        build_type_parameters(item.type_parameters, &metadata.type_param_descriptions)
    } else {
        Vec::new()
    };

    Some(NormalizedDocEntry {
        name: item.name,
        kind,
        description: item.doc.unwrap_or_default(),
        params: metadata.params,
        returns: metadata.returns,
        throws: metadata.throws,
        examples: metadata.examples,
        tags: metadata.tags,
        private: metadata.private,
        file: item.source_path,
        line: item.line,
        end_line: item.end_line,
        signature: item.signature,
        extends: item.extends,
        implements: item.implements,
        has_body: item.has_body,
        members,
        type_parameters,
    })
}

fn normalize_member(item: DocItem, type_parameters: bool) -> Option<NormalizedMember> {
    let include_type_parameters = type_parameters;
    let kind = NormalizedMemberKind::from_doc_item_kind(item.kind)?;
    let mut metadata = normalize_doc_metadata(&item.tags, type_parameters);
    let default_value = member_default_value_from_tags(&item.tags);
    if default_value.is_some() {
        remove_member_default_tags(&mut metadata.tags);
    }
    let has_extracted_params = !item.params.is_empty();
    let has_extracted_return = item.return_type.is_some() || !item.return_members.is_empty();
    let has_callable_shape = matches!(
        kind,
        NormalizedMemberKind::Method
            | NormalizedMemberKind::Constructor
            | NormalizedMemberKind::Getter
            | NormalizedMemberKind::Setter
    ) || has_extracted_params
        || has_extracted_return;
    merge_extracted_params(&mut metadata.params, item.params);
    let mut return_type = item.return_type;

    if has_callable_shape && kind != NormalizedMemberKind::IndexSignature {
        merge_extracted_return(
            &mut metadata.returns,
            return_type.take(),
            item.return_members,
            type_parameters,
        );
    } else {
        metadata.returns = None;
    }
    let type_parameters = if include_type_parameters {
        build_type_parameters(item.type_parameters, &metadata.type_param_descriptions)
    } else {
        Vec::new()
    };
    let members = item
        .children
        .into_iter()
        .filter_map(|item| normalize_member(item, include_type_parameters))
        .collect();

    let (signature, type_annotation) = match kind {
        NormalizedMemberKind::Property | NormalizedMemberKind::EnumMember => (None, item.signature),
        NormalizedMemberKind::IndexSignature => (item.signature, return_type),
        NormalizedMemberKind::Method
        | NormalizedMemberKind::Constructor
        | NormalizedMemberKind::Getter
        | NormalizedMemberKind::Setter => (item.signature, None),
    };

    Some(NormalizedMember {
        name: item.name,
        kind,
        description: item.doc.unwrap_or_default(),
        signature,
        type_annotation,
        default_value,
        params: metadata.params,
        type_parameters,
        returns: metadata.returns,
        throws: metadata.throws,
        members,
        optional: item.optional,
        readonly: item.readonly,
        r#static: item.r#static,
        private: metadata.private,
        tags: metadata.tags,
        line: item.line,
        end_line: item.end_line,
    })
}

fn member_default_value_from_tags(tags: &[DocTag]) -> Option<String> {
    for tag in tags {
        if !matches!(tag.tag.as_str(), "default" | "defaultValue" | "defaultvalue") {
            continue;
        }

        let value = tag.value.trim();
        let value = if value.is_empty() {
            tag.default_value.as_deref().unwrap_or("").trim()
        } else {
            value
        };

        if !value.is_empty() {
            return Some(value.to_string());
        }
    }

    None
}

fn remove_member_default_tags(tags: &mut BTreeMap<String, String>) {
    tags.remove("default");
    tags.remove("defaultValue");
    tags.remove("defaultvalue");
}

struct NormalizedDocMetadata {
    params: Vec<NormalizedParamDoc>,
    returns: Option<NormalizedReturnDoc>,
    throws: Vec<NormalizedThrowsDoc>,
    examples: Vec<String>,
    tags: BTreeMap<String, String>,
    type_param_descriptions: BTreeMap<String, String>,
    private: bool,
}

fn normalize_doc_metadata(tags: &[DocTag], type_parameters: bool) -> NormalizedDocMetadata {
    // Normalize tags in one pass. The PHF sets keep alias checks (`@arg`,
    // `@returns`, `@template`, etc.) allocation-free and avoid re-parsing the
    // same tag list for params, returns, examples, privacy, and generic tags.
    let mut params = Vec::new();
    let mut returns = None;
    let mut throws = Vec::new();
    let mut examples = Vec::new();
    let mut normalized_tags = BTreeMap::new();
    let mut type_param_descriptions = BTreeMap::new();
    let mut private = false;

    for tag in tags {
        match tag.tag.as_str() {
            tag_name if PARAM_TAG_NAMES.contains(tag_name) => {
                if let Some(param) = normalized_param_from_tag(tag) {
                    merge_param(&mut params, param);
                }
            }
            tag_name if RETURN_TAG_NAMES.contains(tag_name) => {
                let parsed_returns = normalized_return_from_tag(tag);
                merge_returns(&mut returns, parsed_returns);
            }
            tag_name if THROWS_TAG_NAMES.contains(tag_name) => {
                if let Some(parsed_throws) = normalized_throws_from_tag(tag) {
                    throws.push(parsed_throws);
                }
            }
            EXAMPLE_TAG_NAME => {
                let example = tag.value.trim();
                if !example.is_empty() && !examples.iter().any(|existing| existing == example) {
                    examples.push(example.to_string());
                }
            }
            PRIVATE_TAG_NAME => {
                private = true;
            }
            // TSDoc `@typeParam` / `@template`: only handled specially when opted
            // in. Otherwise it falls through to the generic tag map (JSDoc default).
            tag_name if type_parameters && TYPE_PARAM_TAG_NAMES.contains(tag_name) => {
                if let Some((name, description)) = parse_type_param_tag(tag) {
                    type_param_descriptions.entry(name).or_insert(description);
                }
            }
            tag_name => {
                normalized_tags.entry(tag_name.to_string()).or_insert_with(|| tag.value.clone());
            }
        }
    }

    NormalizedDocMetadata {
        params,
        returns,
        throws,
        examples,
        tags: normalized_tags,
        type_param_descriptions,
        private,
    }
}

/// Parses a `@typeParam` / `@template` tag into `(name, description)`.
/// Prefers the structured `name`/`description` from the JSDoc parser; otherwise
/// splits the raw value as `"<name>[ - ]<description>"`.
fn parse_type_param_tag(tag: &DocTag) -> Option<(String, String)> {
    if let Some(name) = tag.name.as_ref().map(|name| name.trim()).filter(|name| !name.is_empty()) {
        let description = tag.description.clone().unwrap_or_default().trim().to_string();
        return Some((name.to_string(), description));
    }

    let value = tag.value.trim();
    if value.is_empty() {
        return None;
    }
    let mut parts = value.splitn(2, char::is_whitespace);
    let name = parts.next()?.trim();
    if name.is_empty() {
        return None;
    }
    let description = parts.next().unwrap_or("").trim().trim_start_matches('-').trim().to_string();
    Some((name.to_string(), description))
}

/// Merges `@typeParam` descriptions into the AST-derived type parameters by name.
/// Descriptions with no matching declaration parameter are appended as
/// name+description-only entries so they are not lost.
fn build_type_parameters(
    ast: Vec<TypeParamDoc>,
    descriptions: &BTreeMap<String, String>,
) -> Vec<NormalizedTypeParam> {
    let mut used = std::collections::BTreeSet::new();
    let mut result: Vec<NormalizedTypeParam> = ast
        .into_iter()
        .map(|param| {
            used.insert(param.name.clone());
            NormalizedTypeParam {
                description: descriptions.get(&param.name).cloned().unwrap_or_default(),
                name: param.name,
                constraint: param.constraint,
                default: param.default,
            }
        })
        .collect();

    for (name, description) in descriptions {
        if !used.contains(name) {
            result.push(NormalizedTypeParam {
                name: name.clone(),
                constraint: None,
                default: None,
                description: description.clone(),
            });
        }
    }

    result
}

fn merge_extracted_params(params: &mut Vec<NormalizedParamDoc>, extracted_params: Vec<ParamDoc>) {
    for param in extracted_params {
        if is_placeholder_param(params, &param) {
            continue;
        }

        merge_param(
            params,
            NormalizedParamDoc {
                name: param.name,
                type_annotation: param.type_annotation.unwrap_or_else(|| UNKNOWN_TYPE.to_string()),
                description: param.description.unwrap_or_default(),
                optional: param.optional,
                default_value: param.default_value,
            },
        );
    }
}

fn merge_extracted_return(
    returns: &mut Option<NormalizedReturnDoc>,
    return_type: Option<String>,
    return_members: Vec<DocItem>,
    type_parameters: bool,
) {
    let members = return_members
        .into_iter()
        .filter_map(|item| normalize_member(item, type_parameters))
        .collect::<Vec<_>>();
    if return_type.is_none() && members.is_empty() {
        return;
    }

    match returns {
        Some(current) => {
            if let Some(return_type) = return_type {
                current.type_annotation = return_type;
            }
            if !members.is_empty() {
                current.members = members;
            }
        }
        None => {
            *returns = Some(NormalizedReturnDoc {
                type_annotation: return_type.unwrap_or_else(|| UNKNOWN_TYPE.to_string()),
                description: String::new(),
                members,
            });
        }
    }
}

fn is_placeholder_param(existing_params: &[NormalizedParamDoc], param: &ParamDoc) -> bool {
    !existing_params.is_empty()
        && param.name == PARAM_TAG_NAME
        && param.type_annotation.is_none()
        && param.description.is_none()
        && param.default_value.is_none()
}

fn merge_param(params: &mut Vec<NormalizedParamDoc>, next: NormalizedParamDoc) {
    let Some(existing) = params.iter_mut().find(|param| param_names_match(&param.name, &next.name))
    else {
        params.push(next);
        return;
    };

    if existing.name != next.name && next.name.ends_with('?') {
        existing.name.clone_from(&next.name);
    }
    if existing.type_annotation == UNKNOWN_TYPE || next.type_annotation != UNKNOWN_TYPE {
        existing.type_annotation = next.type_annotation;
    }
    if !next.description.is_empty() {
        existing.description = next.description;
    }
    if next.optional {
        existing.optional = true;
    }
    if next.default_value.is_some() {
        existing.default_value = next.default_value;
    }
}

fn param_names_match(left: &str, right: &str) -> bool {
    left == right || left.trim_end_matches('?') == right.trim_end_matches('?')
}

fn merge_returns(returns: &mut Option<NormalizedReturnDoc>, next: NormalizedReturnDoc) {
    let Some(existing) = returns else {
        *returns = Some(next);
        return;
    };

    if existing.type_annotation == UNKNOWN_TYPE {
        existing.type_annotation = next.type_annotation;
    }
    if existing.description.is_empty() {
        existing.description = next.description;
    }
    if existing.members.is_empty() {
        existing.members = next.members;
    }
}

fn normalized_param_from_tag(tag: &DocTag) -> Option<NormalizedParamDoc> {
    let name = tag.name.as_ref()?.trim().to_string();
    (!name.is_empty()).then(|| NormalizedParamDoc {
        name,
        type_annotation: tag.type_annotation.clone().unwrap_or_else(|| UNKNOWN_TYPE.to_string()),
        description: tag.description.clone().unwrap_or_default(),
        optional: tag.optional.unwrap_or(false),
        default_value: tag.default_value.clone(),
    })
}

fn normalized_return_from_tag(tag: &DocTag) -> NormalizedReturnDoc {
    NormalizedReturnDoc {
        type_annotation: tag.type_annotation.clone().unwrap_or_else(|| UNKNOWN_TYPE.to_string()),
        description: tag.description.clone().unwrap_or_default(),
        members: Vec::new(),
    }
}

fn normalized_throws_from_tag(tag: &DocTag) -> Option<NormalizedThrowsDoc> {
    let type_annotation = tag
        .type_annotation
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let description = throws_description_from_tag(tag);
    if !description.is_empty() {
        return Some(NormalizedThrowsDoc { type_annotation, description });
    }

    let value = tag.value.trim();
    if value.is_empty() {
        return type_annotation.map(|type_annotation| NormalizedThrowsDoc {
            type_annotation: Some(type_annotation),
            description: String::new(),
        });
    }

    if type_annotation.is_some() {
        return Some(NormalizedThrowsDoc { type_annotation, description: value.to_string() });
    }

    if let Some((type_annotation, description)) = parse_throws_tag_value(value) {
        return Some(NormalizedThrowsDoc { type_annotation: Some(type_annotation), description });
    }

    Some(NormalizedThrowsDoc { type_annotation: None, description: value.to_string() })
}

fn throws_description_from_tag(tag: &DocTag) -> String {
    let name = tag.name.as_ref().map(|value| value.trim()).filter(|value| !value.is_empty());
    let description =
        tag.description.as_ref().map(|value| value.trim()).filter(|value| !value.is_empty());
    match (name, description) {
        (Some(name), Some(description)) => {
            let mut value = String::with_capacity(name.len() + description.len() + 1);
            value.push_str(name);
            value.push(' ');
            value.push_str(description);
            value
        }
        (Some(name), None) => name.to_string(),
        (None, Some(description)) => description.to_string(),
        (None, None) => String::new(),
    }
}

fn parse_throws_tag_value(value: &str) -> Option<(String, String)> {
    let rest = value.strip_prefix('{')?;
    let end = rest.find('}')?;
    let type_annotation = rest[..end].trim();
    if type_annotation.is_empty() {
        return None;
    }
    let description = rest[end + 1..].trim().trim_start_matches('-').trim().to_string();
    Some((type_annotation.to_string(), description))
}

#[cfg(test)]
mod tests {
    use oxc_span::SourceType;

    use super::*;
    use crate::extractor::DocExtractor;

    #[test]
    fn normalizes_jsdoc_types_and_custom_tags() {
        let source = r#"
/**
 * Creates a user-facing label.
 *
 * @param {string} value - The label source
 * @param {number} [maxLength=20] - Maximum length before truncation
 * @returns {string} Formatted label
 * @throws {RangeError} When maxLength is negative.
 * @exception {TypeError} When value is not a string.
 * @example
 * label("hello", 3)
 * @since 1.2.3
 */
export function label(value, maxLength = 20) {
    return value.slice(0, maxLength);
}
"#;

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "labels.js", SourceType::mjs()).unwrap();
        let entries = normalize_doc_items(items, false);

        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert_eq!(entry.name, "label");
        assert_eq!(entry.kind, NormalizedDocKind::Function);
        assert_eq!(entry.description, "Creates a user-facing label.");
        assert!(!entry.private);
        assert_eq!(entry.params.len(), 2);
        assert_eq!(entry.params[0].type_annotation, "string");
        assert_eq!(entry.params[0].description, "The label source");
        assert_eq!(entry.params[1].type_annotation, "number");
        assert!(entry.params[1].optional);
        assert_eq!(entry.params[1].default_value.as_deref(), Some("20"));
        assert_eq!(entry.params[1].description, "Maximum length before truncation");
        assert_eq!(
            entry.returns,
            Some(NormalizedReturnDoc {
                type_annotation: "string".to_string(),
                description: "Formatted label".to_string(),
                members: Vec::new()
            })
        );
        assert_eq!(
            entry.throws,
            vec![
                NormalizedThrowsDoc {
                    type_annotation: Some("RangeError".to_string()),
                    description: "When maxLength is negative.".to_string(),
                },
                NormalizedThrowsDoc {
                    type_annotation: Some("TypeError".to_string()),
                    description: "When value is not a string.".to_string(),
                }
            ]
        );
        assert_eq!(entry.examples, vec!["label(\"hello\", 3)"]);
        assert_eq!(entry.tags.get("since").map(String::as_str), Some("1.2.3"));
        assert!(!entry.tags.contains_key("throws"));
        assert!(!entry.tags.contains_key("exception"));
    }

    #[test]
    fn preserves_private_flag_when_private_items_are_included() {
        let source = r"
/**
 * Internal helper.
 * @private
 */
export function internalHelper(): void {}
";

        let extractor = DocExtractor::with_private(true);
        let items = extractor.extract_source(source, "internal.ts", SourceType::ts()).unwrap();
        let entries = normalize_doc_items(items, false);

        assert_eq!(entries.len(), 1);
        assert!(entries[0].private);
    }

    #[test]
    fn preserves_enum_kind_in_normalized_entries() {
        let source = r"
/**
 * Available modes.
 */
export enum Mode {
    Fast,
    Slow,
}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "mode.ts", SourceType::ts()).unwrap();
        let entries = normalize_doc_items(items, false);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, NormalizedDocKind::Enum);
    }

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
    fn preserves_heritage_fields_in_normalized_entries() {
        let source = r"
/**
 * Base adapter.
 */
export interface BaseAdapter {}

/**
 * Runtime adapter.
 */
export interface TranslationAdapter extends BaseAdapter {}

/**
 * Default runtime adapter.
 */
export class DefaultTranslation implements TranslationAdapter {}
";

        let extractor = DocExtractor::new();
        let items = extractor.extract_source(source, "adapter.ts", SourceType::ts()).unwrap();
        let entries = normalize_doc_items(items, false);
        let adapter = entries.iter().find(|entry| entry.name == "TranslationAdapter").unwrap();
        let implementation =
            entries.iter().find(|entry| entry.name == "DefaultTranslation").unwrap();

        assert_eq!(adapter.extends, vec!["BaseAdapter"]);
        assert_eq!(implementation.implements, vec!["TranslationAdapter"]);
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
        assert_eq!(
            returns.members[2].type_annotation.as_deref(),
            Some("AggregateError | undefined")
        );
    }

    #[test]
    fn function_type_alias_metadata_keeps_extracted_types_and_jsdoc_descriptions() {
        let source = r"
/**
 * Run a command.
 * @param ctx - Command execution context.
 * @returns CLI output result.
 */
export type CommandRunner<G> = (ctx: Readonly<CommandContext<G>>) => Awaitable<string | void>;
";

        let extractor = DocExtractor::new();
        let entries = normalize_doc_items(
            extractor.extract_source(source, "runner.ts", SourceType::ts()).unwrap(),
            false,
        );
        let alias = entries.iter().find(|entry| entry.name == "CommandRunner").unwrap();

        assert_eq!(alias.params.len(), 1);
        assert_eq!(alias.params[0].name, "ctx");
        assert_eq!(alias.params[0].type_annotation, "Readonly<CommandContext<G>>");
        assert_eq!(alias.params[0].description, "Command execution context.");
        let returns = alias.returns.as_ref().unwrap();
        assert_eq!(returns.type_annotation, "Awaitable<string | void>");
        assert_eq!(returns.description, "CLI output result.");
    }

    #[test]
    fn function_type_alias_without_jsdoc_tags_still_has_type_information() {
        let source = r"
/**
 * Plugin function.
 */
export type PluginFunction<G> = (ctx: Readonly<PluginContext<G>>) => Awaitable<void>;
";

        let extractor = DocExtractor::new();
        let entries = normalize_doc_items(
            extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap(),
            false,
        );
        let alias = entries.iter().find(|entry| entry.name == "PluginFunction").unwrap();

        assert_eq!(alias.params.len(), 1);
        assert_eq!(alias.params[0].name, "ctx");
        assert_eq!(alias.params[0].type_annotation, "Readonly<PluginContext<G>>");
        assert_eq!(alias.params[0].description, "");
        assert_eq!(alias.returns.as_ref().unwrap().type_annotation, "Awaitable<void>");
    }

    #[test]
    fn intersection_type_alias_merges_callable_reference_metadata() {
        let source = r"
/**
 * Plugin function.
 */
export type PluginFunction<G> = (ctx: Readonly<PluginContext<G>>) => Awaitable<void>;

/**
 * Plugin.
 * @param ctx - Plugin context.
 * @returns Plugin setup result.
 */
export type Plugin<E> = PluginFunction & {
    id: string;
    name?: string;
};
";

        let extractor = DocExtractor::new();
        let entries = normalize_doc_items(
            extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap(),
            false,
        );
        let alias = entries.iter().find(|entry| entry.name == "Plugin").unwrap();

        assert_eq!(alias.params.len(), 1);
        assert_eq!(alias.params[0].name, "ctx");
        assert_eq!(alias.params[0].type_annotation, "Readonly<PluginContext<G>>");
        assert_eq!(alias.params[0].description, "Plugin context.");
        let returns = alias.returns.as_ref().unwrap();
        assert_eq!(returns.type_annotation, "Awaitable<void>");
        assert_eq!(returns.description, "Plugin setup result.");
        assert_eq!(
            alias.members.iter().map(|member| member.name.as_str()).collect::<Vec<_>>(),
            ["id", "name"]
        );
        assert_eq!(alias.members[0].type_annotation.as_deref(), Some("string"));
        assert!(alias.members[1].optional);
    }

    #[test]
    fn function_type_alias_without_returns_tag_still_normalizes_return_section() {
        let source = r"
/**
 * Plugin extension hook.
 *
 * @param ctx - The command context.
 * @param cmd - The command.
 */
export type OnPluginExtension<G> = (
    ctx: Readonly<CommandContext<G>>,
    cmd: Readonly<Command<G>>
) => Awaitable<void>;
";

        let extractor = DocExtractor::new();
        let entries = normalize_doc_items(
            extractor.extract_source(source, "plugin.ts", SourceType::ts()).unwrap(),
            false,
        );
        let alias = entries.iter().find(|entry| entry.name == "OnPluginExtension").unwrap();

        assert_eq!(alias.params.len(), 2);
        assert_eq!(alias.params[0].name, "ctx");
        assert_eq!(alias.params[0].type_annotation, "Readonly<CommandContext<G>>");
        assert_eq!(alias.params[0].description, "The command context.");
        assert_eq!(alias.params[1].name, "cmd");
        assert_eq!(alias.params[1].type_annotation, "Readonly<Command<G>>");
        assert_eq!(alias.params[1].description, "The command.");
        let returns = alias.returns.as_ref().unwrap();
        assert_eq!(returns.type_annotation, "Awaitable<void>");
        assert_eq!(returns.description, "");
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

    #[test]
    fn type_parameters_opt_in_merges_typeparam_and_excludes_tag() {
        let source = r"
/**
 * A combinator.
 * @typeParam T - The parsed value type.
 * @experimental
 */
export type Combinator<T> = { parse: (value: string) => T };
";
        let items =
            DocExtractor::new().extract_source(source, "src/c.ts", SourceType::ts()).unwrap();

        // Opted out (default): `@typeParam` stays a generic tag, no type parameters.
        let off = normalize_doc_items(items.clone(), false);
        let off = off.iter().find(|entry| entry.name == "Combinator").unwrap();
        assert!(off.type_parameters.is_empty());
        assert_eq!(
            off.tags.get("typeParam").map(String::as_str),
            Some("T - The parsed value type.")
        );

        // Opted in: structured type parameter with merged description; tag removed.
        let on = normalize_doc_items(items, true);
        let on = on.iter().find(|entry| entry.name == "Combinator").unwrap();
        assert_eq!(on.type_parameters.len(), 1);
        assert_eq!(on.type_parameters[0].name, "T");
        assert_eq!(on.type_parameters[0].description, "The parsed value type.");
        assert!(!on.tags.contains_key("typeParam"));
        assert!(on.tags.contains_key("experimental"));
    }

    #[test]
    fn member_type_parameters_opt_in_merges_typeparam_and_excludes_tag() {
        let source = r"
/** Plugin context. */
export interface PluginContext<G> {
  /**
   * Decorate the command.
   * @typeParam L - Extension context.
   * @experimental
   */
  decorateCommand<L extends Record<string, unknown> = DefaultExtensions>(
    decorator: (value: L) => void
  ): void;
}
";
        let items =
            DocExtractor::new().extract_source(source, "src/context.ts", SourceType::ts()).unwrap();

        let off = normalize_doc_items(items.clone(), false);
        let off_member =
            off[0].members.iter().find(|member| member.name == "decorateCommand").unwrap();
        assert!(off_member.type_parameters.is_empty());
        assert_eq!(
            off_member.tags.get("typeParam").map(String::as_str),
            Some("L - Extension context.")
        );

        let on = normalize_doc_items(items, true);
        let on_member =
            on[0].members.iter().find(|member| member.name == "decorateCommand").unwrap();
        assert_eq!(on_member.type_parameters.len(), 1);
        assert_eq!(on_member.type_parameters[0].name, "L");
        assert_eq!(
            on_member.type_parameters[0].constraint.as_deref(),
            Some("Record<string, unknown>")
        );
        assert_eq!(on_member.type_parameters[0].default.as_deref(), Some("DefaultExtensions"));
        assert_eq!(on_member.type_parameters[0].description, "Extension context.");
        assert!(!on_member.tags.contains_key("typeParam"));
        assert!(on_member.tags.contains_key("experimental"));
    }
}
