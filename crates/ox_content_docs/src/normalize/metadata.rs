use std::collections::BTreeMap;

use super::merge::{merge_param, merge_returns};
use super::model::{
    NormalizedParamDoc, NormalizedReturnDoc, NormalizedThrowsDoc, NormalizedTypeParam,
};
use super::tags::{
    normalized_param_from_tag, normalized_return_from_tag, normalized_throws_from_tag,
};
use super::{
    EXAMPLE_TAG_NAME, PARAM_TAG_NAMES, PRIVATE_TAG_NAME, RETURN_TAG_NAMES, THROWS_TAG_NAMES,
    TYPE_PARAM_TAG_NAMES,
};
use crate::extractor::{DocTag, TypeParamDoc};

pub(super) struct NormalizedDocMetadata {
    pub(super) params: Vec<NormalizedParamDoc>,
    pub(super) returns: Option<NormalizedReturnDoc>,
    pub(super) throws: Vec<NormalizedThrowsDoc>,
    pub(super) examples: Vec<String>,
    pub(super) tags: BTreeMap<String, String>,
    pub(super) type_param_descriptions: BTreeMap<String, String>,
    pub(super) private: bool,
}

pub(super) fn normalize_doc_metadata(
    tags: &[DocTag],
    type_parameters: bool,
) -> NormalizedDocMetadata {
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

/// Merges `@typeParam` descriptions into the AST-derived type parameters by name.
/// Descriptions with no matching declaration parameter are appended as
/// name+description-only entries so they are not lost.
pub(super) fn build_type_parameters(
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
