//! Normalized documentation entries for JavaScript-facing generators.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::extractor::{DocItem, DocItemKind, DocTag, ParamDoc};

const UNKNOWN_TYPE: &str = "unknown";
const PARAM_TAG_NAMES: [&str; 3] = ["param", "arg", "argument"];
const RETURN_TAG_NAMES: [&str; 2] = ["returns", "return"];
const EXAMPLE_TAG_NAME: &str = "example";
const PRIVATE_TAG_NAME: &str = "private";

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
    /// Type alias or enum.
    Type,
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
            DocItemKind::Type | DocItemKind::Enum => Some(Self::Type),
            DocItemKind::Variable => Some(Self::Variable),
            DocItemKind::Module => Some(Self::Module),
            DocItemKind::Method
            | DocItemKind::Property
            | DocItemKind::Constructor
            | DocItemKind::Getter
            | DocItemKind::Setter => None,
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
            Self::Variable => "variable",
            Self::Module => "module",
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
}

/// Normalizes extracted documentation items into API reference entries.
#[must_use]
pub fn normalize_doc_items(items: Vec<DocItem>) -> Vec<NormalizedDocEntry> {
    items.into_iter().filter_map(normalize_doc_item).collect()
}

/// Normalizes a single extracted documentation item into an API reference entry.
#[must_use]
pub fn normalize_doc_item(item: DocItem) -> Option<NormalizedDocEntry> {
    let kind = NormalizedDocKind::from_doc_item_kind(item.kind)?;

    let mut params = Vec::new();
    let mut returns = None;
    let mut examples = Vec::new();
    let mut tags = BTreeMap::new();
    let mut is_private = false;

    for tag in &item.tags {
        match tag.tag.as_str() {
            tag_name if PARAM_TAG_NAMES.contains(&tag_name) => {
                if let Some(param) = normalized_param_from_tag(tag) {
                    merge_param(&mut params, param);
                }
            }
            tag_name if RETURN_TAG_NAMES.contains(&tag_name) => {
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
                is_private = true;
            }
            tag_name => {
                tags.entry(tag_name.to_string()).or_insert_with(|| tag.value.clone());
            }
        }
    }

    for param in item.params {
        if is_placeholder_param(&params, &param) {
            continue;
        }

        merge_param(
            &mut params,
            NormalizedParamDoc {
                name: param.name,
                type_annotation: param.type_annotation.unwrap_or_else(|| UNKNOWN_TYPE.to_string()),
                description: param.description.unwrap_or_default(),
                optional: param.optional,
                default_value: param.default_value,
            },
        );
    }

    if let Some(return_type) = item.return_type {
        match &mut returns {
            Some(current) => current.type_annotation = return_type,
            None => {
                returns = Some(NormalizedReturnDoc {
                    type_annotation: return_type,
                    description: String::new(),
                });
            }
        }
    }

    Some(NormalizedDocEntry {
        name: item.name,
        kind,
        description: item.doc.unwrap_or_default(),
        params,
        returns,
        examples,
        tags,
        private: is_private,
        file: item.source_path,
        line: item.line,
        end_line: item.end_line,
        signature: item.signature,
    })
}

fn is_placeholder_param(existing_params: &[NormalizedParamDoc], param: &ParamDoc) -> bool {
    !existing_params.is_empty()
        && param.name == PARAM_TAG_NAMES[0]
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
        let entries = normalize_doc_items(items);

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
                description: "Formatted label".to_string()
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
        let entries = normalize_doc_items(items);

        assert_eq!(entries.len(), 1);
        assert!(entries[0].private);
    }

    #[test]
    fn maps_enums_to_type_entries() {
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
        let entries = normalize_doc_items(items);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, NormalizedDocKind::Type);
    }
}
