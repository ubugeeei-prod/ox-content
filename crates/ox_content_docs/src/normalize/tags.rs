use super::model::{NormalizedParamDoc, NormalizedReturnDoc, NormalizedThrowsDoc};
use super::UNKNOWN_TYPE;
use crate::extractor::DocTag;

pub(super) fn normalized_param_from_tag(tag: &DocTag) -> Option<NormalizedParamDoc> {
    let name = tag.name.as_ref()?.trim().to_string();
    (!name.is_empty()).then(|| NormalizedParamDoc {
        name,
        type_annotation: tag.type_annotation.clone().unwrap_or_else(|| UNKNOWN_TYPE.to_string()),
        description: tag.description.clone().unwrap_or_default(),
        optional: tag.optional.unwrap_or(false),
        default_value: tag.default_value.clone(),
    })
}

pub(super) fn normalized_return_from_tag(tag: &DocTag) -> NormalizedReturnDoc {
    NormalizedReturnDoc {
        type_annotation: tag.type_annotation.clone().unwrap_or_else(|| UNKNOWN_TYPE.to_string()),
        description: tag.description.clone().unwrap_or_default(),
        members: Vec::new(),
    }
}

pub(super) fn normalized_throws_from_tag(tag: &DocTag) -> Option<NormalizedThrowsDoc> {
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
