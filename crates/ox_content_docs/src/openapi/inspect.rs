use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::string_builder::{join2, join3};

use super::{OpenApiDocsError, OpenApiDocsResult, OpenApiSpecInput};

pub(super) fn schema_label(
    root: &Value,
    schema: &Value,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<String> {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        resolve_ref(root, reference, source_path, input)?;
        return Ok(ref_name(reference));
    }
    let Some(object) = schema.as_object() else {
        return Ok("-".to_string());
    };
    if let Some(items) = object.get("items") {
        return Ok(join3("array<", &schema_label(root, items, source_path, input)?, ">"));
    }
    if let Some(values) = object.get("enum").and_then(Value::as_array) {
        let labels = values.iter().map(enum_value).collect::<Vec<_>>();
        return Ok(join2("enum: ", &labels.join(" | ")));
    }
    for key in ["oneOf", "anyOf", "allOf"] {
        if let Some(values) = object.get(key).and_then(Value::as_array) {
            let mut labels = Vec::new();
            for value in values {
                labels.push(schema_label(root, value, source_path, input)?);
            }
            return Ok(labels.join(if key == "allOf" { " & " } else { " | " }));
        }
    }
    let ty = string_field(object, "type").unwrap_or("object");
    Ok(object
        .get("format")
        .and_then(Value::as_str)
        .map_or_else(|| ty.to_string(), |format| join3(ty, " (", &join2(format, ")"))))
}

pub(super) fn resolve_node(
    root: &Value,
    value: &Value,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<Value> {
    match value.get("$ref").and_then(Value::as_str) {
        Some(reference) => resolve_ref(root, reference, source_path, input),
        None => Ok(value.clone()),
    }
}

pub(super) fn sorted_child_objects(value: Option<&Value>) -> Vec<(&str, &Map<String, Value>)> {
    let mut entries = value
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(Map::iter)
        .filter_map(|(key, value)| value.as_object().map(|object| (key.as_str(), object)))
        .collect::<Vec<_>>();
    entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
    entries
}

pub(super) fn sorted_child_values(value: Option<&Value>) -> Vec<(&str, &Value)> {
    let mut entries = value
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(Map::iter)
        .map(|(key, value)| (key.as_str(), value))
        .collect::<Vec<_>>();
    entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
    entries
}

pub(super) fn string_field<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

pub(super) fn object_string<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    object.get(key).and_then(Value::as_str)
}

pub(super) fn pointer_string(root: &Value, path: &[&str]) -> Option<String> {
    let mut current = root;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_str().map(ToOwned::to_owned)
}

pub(super) fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn server_urls(root: &Value) -> Vec<String> {
    root.get("servers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|server| server.get("url").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn fallback_title(source_path: &str) -> String {
    std::path::Path::new(source_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("OpenAPI")
        .to_string()
}

pub(super) fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() { "openapi".to_string() } else { trimmed.to_string() }
}

pub(super) fn unique_slug(base: &str, used: &mut BTreeSet<String>) -> String {
    let base = if base.is_empty() { "operation" } else { base };
    let mut slug = base.to_string();
    let mut suffix = 2;
    while used.contains(&slug) {
        slug = join3(base, "-", &suffix.to_string());
        suffix += 1;
    }
    used.insert(slug.clone());
    slug
}

pub(super) fn example_value(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

pub(super) fn invalid(path: &str, message: &str) -> OpenApiDocsError {
    OpenApiDocsError::InvalidSpec { path: path.to_string(), message: message.to_string() }
}

fn resolve_ref(
    root: &Value,
    reference: &str,
    source_path: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<Value> {
    if !reference.starts_with("#/") {
        return unresolved_ref(source_path, reference, input);
    }
    let pointer = decode_json_pointer(reference);
    match root.pointer(&pointer) {
        Some(value) => Ok(value.clone()),
        None => unresolved_ref(source_path, reference, input),
    }
}

fn unresolved_ref(
    source_path: &str,
    reference: &str,
    input: &OpenApiSpecInput,
) -> OpenApiDocsResult<Value> {
    if input.fail_on_unresolved_refs {
        return Err(OpenApiDocsError::UnresolvedRef {
            path: source_path.to_string(),
            reference: reference.to_string(),
        });
    }
    Ok(Value::Null)
}

fn decode_json_pointer(reference: &str) -> String {
    let mut decoded = String::from("/");
    decoded.push_str(
        &reference[2..]
            .split('/')
            .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
            .collect::<Vec<_>>()
            .join("/"),
    );
    decoded
}

fn ref_name(reference: &str) -> String {
    reference.rsplit('/').next().unwrap_or(reference).replace("~1", "/").replace("~0", "~")
}

fn enum_value(value: &Value) -> String {
    value.as_str().map_or_else(|| value.to_string(), ToOwned::to_owned)
}
