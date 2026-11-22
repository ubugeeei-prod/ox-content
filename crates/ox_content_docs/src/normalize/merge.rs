use super::item::normalize_member;
use super::model::{NormalizedParamDoc, NormalizedReturnDoc};
use super::{NormalizedMember, NormalizedMemberKind, PARAM_TAG_NAME, UNKNOWN_TYPE};
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

/// Moves the declaration's `@param` and `@returns` docs to its only call
/// signature when that signature has no docs of its own, as TypeDoc does. The
/// parameters are then listed once, on the signature.
///
/// The caller passes only docs that come from the declaration's tags: params or
/// a return extracted from a function type belong to that type, not to the
/// signature. Only the descriptions (and a default value) move; the signature
/// keeps its own types. A signature without a return of its own takes the
/// declared return whole, with the type from `@returns {T}` when it is written.
pub(super) fn move_declaration_docs_to_call_signature(
    members: &mut [NormalizedMember],
    params: &mut Vec<NormalizedParamDoc>,
    returns: &mut Option<NormalizedReturnDoc>,
) {
    if params.is_empty() && returns.is_none() {
        return;
    }
    let mut call_signatures =
        members.iter_mut().filter(|member| member.kind == NormalizedMemberKind::CallSignature);
    let (Some(signature), None) = (call_signatures.next(), call_signatures.next()) else {
        return;
    };
    let every_param_matches = params
        .iter()
        .all(|param| signature.params.iter().any(|own| param_names_match(&own.name, &param.name)));
    if has_own_docs(signature) || !every_param_matches {
        return;
    }

    for param in params.drain(..) {
        if let Some(own) =
            signature.params.iter_mut().find(|own| param_names_match(&own.name, &param.name))
        {
            own.description = param.description;
            if own.default_value.is_none() {
                own.default_value = param.default_value;
            }
        }
    }
    if let Some(declared) = returns.take() {
        match &mut signature.returns {
            Some(own) => own.description = declared.description,
            None => signature.returns = Some(declared),
        }
    }
}

/// Whether the member has a doc comment of its own: a description, tags, throws
/// or a described parameter, type parameter or return.
fn has_own_docs(member: &NormalizedMember) -> bool {
    !member.description.is_empty()
        || !member.tags.is_empty()
        || !member.throws.is_empty()
        || !member.examples.is_empty()
        || member.params.iter().any(|param| !param.description.is_empty())
        || member.type_parameters.iter().any(|param| !param.description.is_empty())
        || member.returns.as_ref().is_some_and(|returns| !returns.description.is_empty())
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
