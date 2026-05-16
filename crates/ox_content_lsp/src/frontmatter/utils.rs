use serde_json::Value;
use tower_lsp::lsp_types::{Position, Range};

use crate::document::TextDocumentState;
use crate::frontmatter::{FrontmatterBlock, FrontmatterSchema};

pub fn contains_position(range: &Range, position: Position) -> bool {
    (position.line > range.start.line
        || (position.line == range.start.line && position.character >= range.start.character))
        && (position.line < range.end.line
            || (position.line == range.end.line && position.character <= range.end.character))
}

pub fn strip_line_breaks(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}

pub fn yaml_error_range(
    document: &TextDocumentState,
    content_start_offset: usize,
    raw: &str,
    error: &serde_yaml::Error,
) -> Range {
    let Some(location) = error.location() else {
        return document.range_from_offsets(content_start_offset, content_start_offset);
    };

    let raw_offset = raw_offset_from_line_col(raw, location.line(), location.column());
    let start = content_start_offset + raw_offset;
    document.range_from_offsets(start, (start + 1).min(document.text().len()))
}

fn raw_offset_from_line_col(raw: &str, line: usize, column: usize) -> usize {
    let mut current_line = 1usize;
    let mut current_column = 1usize;
    let mut offset = 0usize;

    for ch in raw.chars() {
        if current_line == line && current_column == column {
            return offset;
        }

        offset += ch.len_utf8();
        if ch == '\n' {
            current_line += 1;
            current_column = 1;
        } else {
            current_column += 1;
        }
    }

    offset
}

pub fn display_value(value: &Value) -> String {
    match value {
        Value::String(value) => format!("\"{value}\""),
        _ => value.to_string(),
    }
}

pub fn effective_type(schema: &FrontmatterSchema) -> Option<&str> {
    schema.type_name.as_deref().or({
        if schema.properties.is_empty() {
            None
        } else {
            Some("object")
        }
    })
}

pub fn matches_type(type_name: &str, value: &Value) -> bool {
    match type_name {
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "null" => value.is_null(),
        _ => true,
    }
}

pub fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(number) if number.is_i64() || number.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub fn range_for_path(block: &FrontmatterBlock, path: &[String]) -> Range {
    path.first().map_or(block.content_range, |name| range_for_named_key(block, name))
}

pub fn range_for_named_key(block: &FrontmatterBlock, name: &str) -> Range {
    block
        .top_level_keys
        .iter()
        .find(|entry| entry.name == name)
        .map_or(block.content_range, |entry| entry.key_range)
}
