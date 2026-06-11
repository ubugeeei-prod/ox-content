use std::collections::BTreeMap;

use super::kind::NormalizedMemberKind;
use super::member::NormalizedMember;
use super::merge::{merge_extracted_params, merge_extracted_return};
use super::metadata::{build_type_parameters, normalize_doc_metadata};
use crate::extractor::{DocItem, DocTag};

pub(super) fn normalize_member(item: DocItem, type_parameters: bool) -> Option<NormalizedMember> {
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
