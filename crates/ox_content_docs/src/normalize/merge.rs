use super::item::normalize_member;
use super::model::{NormalizedParamDoc, NormalizedReturnDoc};
use super::{PARAM_TAG_NAME, UNKNOWN_TYPE};
use crate::extractor::{DocItem, ParamDoc};

pub(super) fn merge_extracted_params(
    params: &mut Vec<NormalizedParamDoc>,
    extracted_params: Vec<ParamDoc>,
) {
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

pub(super) fn merge_extracted_return(
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

pub(super) fn merge_param(params: &mut Vec<NormalizedParamDoc>, next: NormalizedParamDoc) {
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

pub(super) fn merge_returns(returns: &mut Option<NormalizedReturnDoc>, next: NormalizedReturnDoc) {
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

fn is_placeholder_param(existing_params: &[NormalizedParamDoc], param: &ParamDoc) -> bool {
    !existing_params.is_empty()
        && param.name == PARAM_TAG_NAME
        && param.type_annotation.is_none()
        && param.description.is_none()
        && param.default_value.is_none()
}

fn param_names_match(left: &str, right: &str) -> bool {
    left == right || left.trim_end_matches('?') == right.trim_end_matches('?')
}
