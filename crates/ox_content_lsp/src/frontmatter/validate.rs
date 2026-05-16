use serde_json::Value;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

use crate::frontmatter::utils::{
    display_value, effective_type, matches_type, range_for_named_key, range_for_path, value_kind,
};
use crate::frontmatter::{FrontmatterBlock, FrontmatterSchema};

pub fn validate_frontmatter(
    block: &FrontmatterBlock,
    schema: &FrontmatterSchema,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let Some(value) = &block.value else {
        return diagnostics;
    };

    validate_value(schema, value, &mut Vec::new(), block, &mut diagnostics);
    diagnostics
}

fn validate_value(
    schema: &FrontmatterSchema,
    value: &Value,
    path: &mut Vec<String>,
    block: &FrontmatterBlock,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(type_name) = effective_type(schema) {
        if !matches_type(type_name, value) {
            diagnostics.push(Diagnostic {
                range: range_for_path(block, path),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("ox-content".to_string()),
                message: format!("Expected `{type_name}` but found `{}`", value_kind(value)),
                ..Default::default()
            });
            return;
        }
    }

    validate_enum(schema, value, path, block, diagnostics);
    validate_object(schema, value, path, block, diagnostics);
    validate_array(schema, value, path, block, diagnostics);
}

fn validate_enum(
    schema: &FrontmatterSchema,
    value: &Value,
    path: &[String],
    block: &FrontmatterBlock,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if schema.enum_values.is_empty()
        || schema.enum_values.iter().any(|candidate| candidate == value)
    {
        return;
    }

    diagnostics.push(Diagnostic {
        range: range_for_path(block, path),
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some("ox-content".to_string()),
        message: format!(
            "Expected one of {}",
            schema.enum_values.iter().map(display_value).collect::<Vec<_>>().join(", ")
        ),
        ..Default::default()
    });
}

fn validate_object(
    schema: &FrontmatterSchema,
    value: &Value,
    path: &mut Vec<String>,
    block: &FrontmatterBlock,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Value::Object(map) = value else {
        return;
    };

    for required in &schema.required {
        if !map.contains_key(required) {
            diagnostics.push(Diagnostic {
                range: block.content_range,
                severity: Some(DiagnosticSeverity::WARNING),
                source: Some("ox-content".to_string()),
                message: format!("Missing required frontmatter field `{required}`"),
                ..Default::default()
            });
        }
    }

    for (key, child) in map {
        if let Some(child_schema) = schema.properties.get(key) {
            path.push(key.clone());
            validate_value(child_schema, child, path, block, diagnostics);
            path.pop();
        } else if schema.additional_properties == Some(false) {
            diagnostics.push(Diagnostic {
                range: range_for_named_key(block, key),
                severity: Some(DiagnosticSeverity::WARNING),
                source: Some("ox-content".to_string()),
                message: format!("Unknown frontmatter field `{key}`"),
                ..Default::default()
            });
        }
    }
}

fn validate_array(
    schema: &FrontmatterSchema,
    value: &Value,
    path: &mut Vec<String>,
    block: &FrontmatterBlock,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let (Some(item_schema), Value::Array(items)) = (&schema.items, value) else {
        return;
    };

    for item in items {
        validate_value(item_schema, item, path, block, diagnostics);
    }
}
