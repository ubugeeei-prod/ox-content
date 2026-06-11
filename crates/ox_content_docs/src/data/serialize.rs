use serde_json::{json, Map, Value};

use crate::model::{
    ApiDocEntry, ApiDocMember, ApiDocModule, ApiParamDoc, ApiReturnDoc, ApiThrowsDoc,
    ApiTypeParamDoc,
};

pub(super) fn module_to_json(module: &ApiDocModule) -> Value {
    let mut value = Map::new();
    value.insert("file".to_string(), json!(normalize_doc_file_path(&module.file)));
    value.insert("description".to_string(), json!(module.description));
    if !module.examples.is_empty() {
        value.insert("examples".to_string(), json!(module.examples));
    }
    if !module.tags.is_empty() {
        value.insert(
            "tags".to_string(),
            Value::Object(
                module.tags.iter().map(|tag| (tag.tag.clone(), json!(tag.value))).collect(),
            ),
        );
    }
    value.insert(
        "entries".to_string(),
        Value::Array(module.entries.iter().map(entry_to_json).collect()),
    );

    Value::Object(value)
}

fn entry_to_json(entry: &ApiDocEntry) -> Value {
    let mut value = Map::new();
    value.insert("name".to_string(), json!(entry.name));
    value.insert("kind".to_string(), json!(entry.kind));
    value.insert("description".to_string(), json!(entry.description));

    if !entry.type_parameters.is_empty() {
        value.insert(
            "typeParameters".to_string(),
            Value::Array(entry.type_parameters.iter().map(type_param_to_json).collect()),
        );
    }
    if !entry.params.is_empty() {
        value.insert(
            "params".to_string(),
            Value::Array(entry.params.iter().map(param_to_json).collect()),
        );
    }
    if let Some(returns) = &entry.returns {
        value.insert("returns".to_string(), return_to_json(returns));
    }
    if !entry.throws.is_empty() {
        value.insert(
            "throws".to_string(),
            Value::Array(entry.throws.iter().map(throws_to_json).collect()),
        );
    }
    if !entry.examples.is_empty() {
        value.insert("examples".to_string(), json!(entry.examples));
    }
    if !entry.tags.is_empty() {
        value.insert(
            "tags".to_string(),
            Value::Object(
                entry.tags.iter().map(|tag| (tag.tag.clone(), json!(tag.value))).collect(),
            ),
        );
    }
    if !entry.members.is_empty() {
        value.insert(
            "members".to_string(),
            Value::Array(entry.members.iter().map(member_to_json).collect()),
        );
    }
    if entry.private {
        value.insert("private".to_string(), json!(true));
    }

    // An empty `file` means the symbol has no source in the consumer's repo
    // (e.g. re-exported from an external package): omit the source location
    // entirely rather than leak an absolute local path.
    if !entry.file.is_empty() {
        value.insert("file".to_string(), json!(normalize_doc_file_path(&entry.file)));
        value.insert("line".to_string(), json!(entry.line));
        value.insert("endLine".to_string(), json!(entry.end_line));
    }
    if let Some(signature) = &entry.signature {
        value.insert("signature".to_string(), json!(signature));
    }
    if !entry.extends.is_empty() {
        value.insert("extends".to_string(), json!(entry.extends));
    }
    if !entry.implements.is_empty() {
        value.insert("implements".to_string(), json!(entry.implements));
    }

    Value::Object(value)
}

fn member_to_json(member: &ApiDocMember) -> Value {
    let mut value = Map::new();
    value.insert("name".to_string(), json!(member.name));
    value.insert("kind".to_string(), json!(member.kind));
    value.insert("description".to_string(), json!(member.description));
    if let Some(signature) = &member.signature {
        value.insert("signature".to_string(), json!(signature));
    }
    if let Some(type_annotation) = &member.type_annotation {
        value.insert("type".to_string(), json!(type_annotation));
    }
    if let Some(default_value) = &member.default_value {
        value.insert("default".to_string(), json!(default_value));
    }
    if !member.params.is_empty() {
        value.insert(
            "params".to_string(),
            Value::Array(member.params.iter().map(param_to_json).collect()),
        );
    }
    if !member.type_parameters.is_empty() {
        value.insert(
            "typeParameters".to_string(),
            Value::Array(member.type_parameters.iter().map(type_param_to_json).collect()),
        );
    }
    if let Some(returns) = &member.returns {
        value.insert("returns".to_string(), return_to_json(returns));
    }
    if !member.throws.is_empty() {
        value.insert(
            "throws".to_string(),
            Value::Array(member.throws.iter().map(throws_to_json).collect()),
        );
    }
    if !member.members.is_empty() {
        value.insert(
            "members".to_string(),
            Value::Array(member.members.iter().map(member_to_json).collect()),
        );
    }
    if member.optional {
        value.insert("optional".to_string(), json!(true));
    }
    if member.readonly {
        value.insert("readonly".to_string(), json!(true));
    }
    if member.r#static {
        value.insert("static".to_string(), json!(true));
    }
    if member.private {
        value.insert("private".to_string(), json!(true));
    }
    if !member.tags.is_empty() {
        value.insert(
            "tags".to_string(),
            Value::Object(
                member.tags.iter().map(|tag| (tag.tag.clone(), json!(tag.value))).collect(),
            ),
        );
    }
    if !member.implementation_of.is_empty() {
        value.insert("implementationOf".to_string(), json!(member.implementation_of));
    }
    value.insert("line".to_string(), json!(member.line));
    value.insert("endLine".to_string(), json!(member.end_line));

    Value::Object(value)
}

fn param_to_json(param: &ApiParamDoc) -> Value {
    let mut value = Map::new();
    value.insert("name".to_string(), json!(param.name));
    value.insert("type".to_string(), json!(param.type_annotation));
    value.insert("description".to_string(), json!(param.description));
    if param.optional {
        value.insert("optional".to_string(), json!(true));
    }
    if let Some(default_value) = &param.default_value {
        value.insert("default".to_string(), json!(default_value));
    }
    Value::Object(value)
}

fn return_to_json(return_doc: &ApiReturnDoc) -> Value {
    let mut value = Map::new();
    value.insert("type".to_string(), json!(return_doc.type_annotation));
    value.insert("description".to_string(), json!(return_doc.description));
    if !return_doc.members.is_empty() {
        value.insert(
            "members".to_string(),
            Value::Array(return_doc.members.iter().map(member_to_json).collect()),
        );
    }
    Value::Object(value)
}

fn throws_to_json(throws: &ApiThrowsDoc) -> Value {
    let mut value = Map::new();
    if let Some(type_annotation) = &throws.type_annotation {
        value.insert("type".to_string(), json!(type_annotation));
    }
    value.insert("description".to_string(), json!(throws.description));
    Value::Object(value)
}

fn type_param_to_json(type_param: &ApiTypeParamDoc) -> Value {
    let mut value = Map::new();
    value.insert("name".to_string(), json!(type_param.name));
    if let Some(constraint) = &type_param.constraint {
        value.insert("constraint".to_string(), json!(constraint));
    }
    if let Some(default) = &type_param.default {
        value.insert("default".to_string(), json!(default));
    }
    if !type_param.description.is_empty() {
        value.insert("description".to_string(), json!(type_param.description));
    }
    Value::Object(value)
}

fn normalize_doc_file_path(file_path: &str) -> String {
    let normalized = file_path.replace('\\', "/");
    for prefix in ["npm/", "packages/", "crates/", "src/"] {
        if let Some(index) = normalized.find(prefix) {
            if index == 0 || normalized.as_bytes().get(index - 1) == Some(&b'/') {
                return normalized[index..].to_string();
            }
        }
    }

    normalized.trim_start_matches('/').to_string()
}
