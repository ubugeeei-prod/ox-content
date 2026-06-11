//! Normalized documentation entries for JavaScript-facing generators.

// BTreeMap/BTreeSet keep generated tag and type-parameter output deterministic.
use phf::phf_set;

mod entry;
mod item;
mod kind;
mod member;
mod merge;
mod metadata;
mod model;
mod tags;

pub use entry::NormalizedDocEntry;
pub use kind::{NormalizedDocKind, NormalizedMemberKind};
pub use member::NormalizedMember;
pub use model::{
    NormalizedParamDoc, NormalizedReturnDoc, NormalizedThrowsDoc, NormalizedTypeParam,
};

use item::normalize_member;
use merge::{merge_extracted_params, merge_extracted_return};
use metadata::{build_type_parameters, normalize_doc_metadata};

use crate::extractor::DocItem;
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

#[cfg(test)]
mod tests;
