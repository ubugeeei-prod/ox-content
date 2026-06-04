//! Normalized documentation entries for JavaScript-facing generators.

use std::collections::BTreeMap;

use phf::phf_set;
use serde::{Deserialize, Serialize};

use crate::extractor::{DocItem, DocItemKind, DocTag, ParamDoc, TypeParamDoc};
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

static TYPE_PARAM_TAG_NAMES: phf::Set<&'static str> = phf_set! {
    "typeParam",
    "template",
};

/// Documentation item kind supported by the generated API reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NormalizedDocKind {
    /// Function declaration or function-valued variable.
    Function,
    /// Class declaration.
    Class,
    /// TypeScript interface declaration.
    Interface,
    /// Type alias.
    Type,
    /// Enum declaration.
    Enum,
    /// Variable declaration.
    Variable,
    /// Module or namespace.
    Module,
}

impl NormalizedDocKind {
    /// Converts extractor-level kinds into public documentation kinds.
    #[must_use]
    pub fn from_doc_item_kind(kind: DocItemKind) -> Option<Self> {
        match kind {
            DocItemKind::Function => Some(Self::Function),
            DocItemKind::Class => Some(Self::Class),
            DocItemKind::Interface => Some(Self::Interface),
            DocItemKind::Type => Some(Self::Type),
            DocItemKind::Enum => Some(Self::Enum),
            DocItemKind::Variable => Some(Self::Variable),
            DocItemKind::Module => Some(Self::Module),
            DocItemKind::Method
            | DocItemKind::Property
            | DocItemKind::Constructor
            | DocItemKind::Getter
            | DocItemKind::Setter
            | DocItemKind::EnumMember => None,
        }
    }

    /// Returns the JavaScript-facing string for this kind.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::Type => "type",
            Self::Enum => "enum",
            Self::Variable => "variable",
            Self::Module => "module",
        }
    }
}

/// Documentation item kind supported for class/interface/type/enum members.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NormalizedMemberKind {
    /// Object or class property.
    Property,
    /// Method signature.
    Method,
    /// Class constructor.
    Constructor,
    /// Getter accessor.
    Getter,
    /// Setter accessor.
    Setter,
    /// Enum member.
    #[serde(rename = "enumMember")]
    EnumMember,
}

impl NormalizedMemberKind {
    /// Converts extractor-level member kinds into public member kinds.
    #[must_use]
    pub fn from_doc_item_kind(kind: DocItemKind) -> Option<Self> {
        match kind {
            DocItemKind::Property => Some(Self::Property),
            DocItemKind::Method => Some(Self::Method),
            DocItemKind::Constructor => Some(Self::Constructor),
            DocItemKind::Getter => Some(Self::Getter),
            DocItemKind::Setter => Some(Self::Setter),
            DocItemKind::EnumMember => Some(Self::EnumMember),
            DocItemKind::Module
            | DocItemKind::Function
            | DocItemKind::Class
            | DocItemKind::Interface
            | DocItemKind::Type
            | DocItemKind::Enum
            | DocItemKind::Variable => None,
        }
    }

    /// Returns the JavaScript-facing string for this kind.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Property => "property",
            Self::Method => "method",
            Self::Constructor => "constructor",
            Self::Getter => "getter",
            Self::Setter => "setter",
            Self::EnumMember => "enumMember",
        }
    }
}

/// Normalized parameter documentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedParamDoc {
    /// Parameter name.
    pub name: String,
    /// Parameter type, or `unknown` if it cannot be inferred.
    pub type_annotation: String,
    /// Parameter description.
    pub description: String,
    /// Whether the parameter is optional.
    pub optional: bool,
    /// Default value if specified.
    pub default_value: Option<String>,
}

/// Normalized return documentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedReturnDoc {
    /// Return type, or `unknown` if it cannot be inferred.
    pub type_annotation: String,
    /// Return value description.
    pub description: String,
    /// Members of an inline object literal return type.
    #[serde(default)]
    pub members: Vec<NormalizedMember>,
}

/// Normalized type parameter documentation (`<T extends C = D>`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedTypeParam {
    /// Type parameter name (e.g. `T`).
    pub name: String,
    /// Constraint after `extends`, when present.
    pub constraint: Option<String>,
    /// Default type after `=`, when present.
    pub default: Option<String>,
    /// Description merged from a `@typeParam` / `@template` tag.
    pub description: String,
}

/// Normalized documentation for a member of a class/interface/type/enum entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedMember {
    /// Member name.
    pub name: String,
    /// Member kind.
    pub kind: NormalizedMemberKind,
    /// Human-readable description.
    pub description: String,
    /// Full member signature, for callable members.
    pub signature: Option<String>,
    /// Property or enum member type/value annotation.
    pub type_annotation: Option<String>,
    /// Parameters, if any.
    #[serde(default)]
    pub params: Vec<NormalizedParamDoc>,
    /// Return documentation, if any.
    pub returns: Option<NormalizedReturnDoc>,
    /// Whether the member is optional.
    #[serde(default)]
    pub optional: bool,
    /// Whether the member is readonly.
    #[serde(default)]
    pub readonly: bool,
    /// Whether the member is static.
    #[serde(default)]
    pub r#static: bool,
    /// Whether the member is marked private.
    #[serde(default)]
    pub private: bool,
    /// Custom JSDoc tags.
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
    /// Declaration start line.
    pub line: u32,
    /// Declaration end line.
    pub end_line: u32,
}

/// Normalized documentation entry consumed by generated API docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedDocEntry {
    /// Entry name.
    pub name: String,
    /// Entry kind.
    pub kind: NormalizedDocKind,
    /// Human-readable description.
    pub description: String,
    /// Parameters, if any.
    pub params: Vec<NormalizedParamDoc>,
    /// Return documentation, if any.
    pub returns: Option<NormalizedReturnDoc>,
    /// Example blocks.
    pub examples: Vec<String>,
    /// Custom JSDoc tags.
    pub tags: BTreeMap<String, String>,
    /// Whether the entry is marked private.
    pub private: bool,
    /// Source file path.
    pub file: String,
    /// Declaration start line.
    pub line: u32,
    /// Declaration end line.
    pub end_line: u32,
    /// Signature text.
    pub signature: Option<String>,
    /// Whether a function declaration carries an implementation body (`false` for
    /// overload signatures and ambient declarations). Used to hide the
    /// implementation signature when grouping overloads on TypeDoc symbol pages.
    #[serde(default)]
    pub has_body: bool,
    /// Members belonging to class/interface/type/enum entries.
    #[serde(default)]
    pub members: Vec<NormalizedMember>,
    /// Declaration type parameters. Populated only when type-parameter docs are
    /// enabled (opt-in); empty otherwise.
    #[serde(default)]
    pub type_parameters: Vec<NormalizedTypeParam>,
}

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
    merge_extracted_return(&mut metadata.returns, item.return_type, item.return_members);
    let members = item.children.into_iter().filter_map(normalize_member).collect();
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
        examples: metadata.examples,
        tags: metadata.tags,
        private: metadata.private,
        file: item.source_path,
        line: item.line,
        end_line: item.end_line,
        signature: item.signature,
        has_body: item.has_body,
        members,
        type_parameters,
    })
}

fn normalize_member(item: DocItem) -> Option<NormalizedMember> {
    let kind = NormalizedMemberKind::from_doc_item_kind(item.kind)?;
    // Member-level type parameters are out of scope; keep generic-tag behavior.
    let mut metadata = normalize_doc_metadata(&item.tags, false);
    merge_extracted_params(&mut metadata.params, item.params);
    merge_extracted_return(&mut metadata.returns, item.return_type, item.return_members);

    let (signature, type_annotation) = match kind {
        NormalizedMemberKind::Property | NormalizedMemberKind::EnumMember => (None, item.signature),
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
        params: metadata.params,
        returns: metadata.returns,
        optional: item.optional,
        readonly: item.readonly,
        r#static: item.r#static,
        private: metadata.private,
        tags: metadata.tags,
        line: item.line,
        end_line: item.end_line,
    })
}

struct NormalizedDocMetadata {
    params: Vec<NormalizedParamDoc>,
    returns: Option<NormalizedReturnDoc>,
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
) {
    let members = return_members.into_iter().filter_map(normalize_member).collect::<Vec<_>>();
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
    let Some(existing) = params.iter_mut().find(|param| param.name == next.name) else {
        params.push(next);
        return;
    };

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
        assert_eq!(entry.examples, vec!["label(\"hello\", 3)"]);
        assert_eq!(entry.tags.get("since").map(String::as_str), Some("1.2.3"));
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
}
