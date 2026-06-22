use rustc_hash::FxHashSet;

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, Hover, HoverContents, InsertTextFormat,
    MarkupContent, MarkupKind, Position,
};

use crate::document::TextDocumentState;
use crate::frontmatter::utils::{contains_position, display_value};
use crate::frontmatter::{FrontmatterBlock, FrontmatterSchema};

pub fn completion_items(
    document: &TextDocumentState,
    position: Position,
    block: &FrontmatterBlock,
    schema: &FrontmatterSchema,
) -> Option<Vec<CompletionItem>> {
    if !contains_position(&block.block_range, position) {
        return None;
    }

    let line = crate::frontmatter::utils::strip_line_breaks(document.line_text(position.line));
    let line_start = document.line_start_offset(position.line as usize);
    let cursor_offset = document.position_to_offset(position);
    let cursor_in_line = cursor_offset.saturating_sub(line_start).min(line.len());
    let prefix = &line[..cursor_in_line];
    let colon_index = line.find(':');

    let schema = schema_for_position(document, position, block, schema).unwrap_or(schema);

    if colon_index.is_none() || cursor_in_line <= colon_index.unwrap_or(0) {
        return Some(property_items(document, position, block, schema));
    }

    value_items(document, position, prefix, line, schema)
}

pub fn hover(
    block: &FrontmatterBlock,
    position: Position,
    schema: &FrontmatterSchema,
) -> Option<Hover> {
    let key =
        block.top_level_keys.iter().find(|entry| contains_position(&entry.key_range, position))?;
    let property = schema.property(&key.name)?;
    let mut lines = vec![format!("**{}**", key.name), format!("Type: `{}`", property.kind_label())];

    if let Some(description) = &property.description {
        lines.extend([String::new(), description.clone()]);
    }
    if !property.enum_values.is_empty() {
        lines.extend([
            String::new(),
            format!(
                "Allowed values: {}",
                property.enum_values.iter().map(display_value).collect::<Vec<_>>().join(", ")
            ),
        ]);
    }
    if let Some(default) = &property.default {
        lines.extend([String::new(), format!("Default: `{}`", display_value(default))]);
    }

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: lines.join("\n"),
        }),
        range: Some(key.key_range),
    })
}

fn property_items(
    document: &TextDocumentState,
    position: Position,
    block: &FrontmatterBlock,
    schema: &FrontmatterSchema,
) -> Vec<CompletionItem> {
    let existing = sibling_keys(document, position, block);
    let replace = document.word_range_at(position, |ch| {
        ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '"'
    });

    schema
        .properties
        .iter()
        .map(|(name, property)| CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(property.kind_label()),
            documentation: property.description.as_ref().map(|description| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: description.clone(),
                })
            }),
            insert_text: Some(property_snippet(name, property)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(if existing.contains(name.as_str()) {
                format!("1{name}")
            } else {
                format!("0{name}")
            }),
            text_edit: Some(tower_lsp::lsp_types::CompletionTextEdit::Edit(
                tower_lsp::lsp_types::TextEdit {
                    range: replace,
                    new_text: property_snippet(name, property),
                },
            )),
            ..Default::default()
        })
        .collect()
}

fn schema_for_position<'a>(
    document: &TextDocumentState,
    position: Position,
    block: &FrontmatterBlock,
    schema: &'a FrontmatterSchema,
) -> Option<&'a FrontmatterSchema> {
    let current_line =
        crate::frontmatter::utils::strip_line_breaks(document.line_text(position.line));
    let current_indent = leading_indent(current_line);
    let mut stack: Vec<(usize, String)> = Vec::new();

    for line_index in block.content_range.start.line..position.line {
        let line = crate::frontmatter::utils::strip_line_breaks(document.line_text(line_index));
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let indent = leading_indent(line);
        let Some(key) = key_before_colon(line) else {
            continue;
        };
        while stack.last().is_some_and(|(existing_indent, _)| *existing_indent >= indent) {
            stack.pop();
        }
        stack.push((indent, key.to_string()));
    }

    while stack.last().is_some_and(|(existing_indent, _)| *existing_indent >= current_indent) {
        stack.pop();
    }

    let mut current = schema;
    for (_, key) in stack {
        current = current.property(&key)?;
    }
    Some(current)
}

fn sibling_keys(
    document: &TextDocumentState,
    position: Position,
    block: &FrontmatterBlock,
) -> FxHashSet<String> {
    let current_line =
        crate::frontmatter::utils::strip_line_breaks(document.line_text(position.line));
    let current_indent = leading_indent(current_line);
    let mut keys = FxHashSet::default();

    for line_index in block.content_range.start.line..=block.content_range.end.line {
        if line_index == position.line {
            continue;
        }
        let line = crate::frontmatter::utils::strip_line_breaks(document.line_text(line_index));
        if leading_indent(line) == current_indent {
            if let Some(key) = key_before_colon(line) {
                keys.insert(key.to_string());
            }
        }
    }
    keys
}

fn leading_indent(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ' || *ch == '\t').count()
}

fn key_before_colon(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let colon = trimmed.find(':')?;
    let key = trimmed[..colon].trim().trim_matches('"').trim_matches('\'');
    (!key.is_empty()).then_some(key)
}

fn value_items(
    document: &TextDocumentState,
    position: Position,
    prefix: &str,
    line: &str,
    schema: &FrontmatterSchema,
) -> Option<Vec<CompletionItem>> {
    let key = line[..line.find(':')?].trim().trim_matches('"').trim_matches('\'');
    let property = schema.property(key)?;
    let replace = document.word_range_at(position, |ch| {
        ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '"' || ch == '\''
    });
    let mut items = enum_or_default_items(property, replace);

    if items.is_empty() && prefix.trim_end().ends_with(':') {
        items.push(CompletionItem {
            label: property.kind_label(),
            kind: Some(CompletionItemKind::VALUE),
            insert_text: Some(default_value_snippet(property)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: property.description.clone(),
            ..Default::default()
        });
    }

    Some(items)
}

fn enum_or_default_items(
    property: &FrontmatterSchema,
    range: tower_lsp::lsp_types::Range,
) -> Vec<CompletionItem> {
    let mut items = property
        .enum_values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            completion_value(display_value(value), property.kind_label(), range, index)
        })
        .collect::<Vec<_>>();

    if items.is_empty() && property.type_name.as_deref() == Some("boolean") {
        items.extend(["true", "false"].into_iter().enumerate().map(|(index, value)| {
            completion_value(value.to_string(), "boolean".to_string(), range, index)
        }));
    } else if items.is_empty() {
        if let Some(default) = &property.default {
            items.push(CompletionItem {
                label: display_value(default),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("default".to_string()),
                insert_text: Some(display_value(default)),
                text_edit: Some(tower_lsp::lsp_types::CompletionTextEdit::Edit(
                    tower_lsp::lsp_types::TextEdit { range, new_text: display_value(default) },
                )),
                ..Default::default()
            });
        }
    }

    items
}

fn completion_value(
    text: String,
    detail: String,
    range: tower_lsp::lsp_types::Range,
    index: usize,
) -> CompletionItem {
    CompletionItem {
        label: text.clone(),
        kind: Some(CompletionItemKind::VALUE),
        detail: Some(detail),
        insert_text: Some(text.clone()),
        sort_text: Some(format!("0{index:02}")),
        text_edit: Some(tower_lsp::lsp_types::CompletionTextEdit::Edit(
            tower_lsp::lsp_types::TextEdit { range, new_text: text },
        )),
        ..Default::default()
    }
}

fn property_snippet(name: &str, property: &FrontmatterSchema) -> String {
    format!("{name}: {}", default_value_snippet(property))
}

#[allow(clippy::literal_string_with_formatting_args)]
fn default_value_snippet(property: &FrontmatterSchema) -> String {
    if let Some(default) = &property.default {
        return display_value(default);
    }
    if let Some(first) = property.enum_values.first() {
        return display_value(first);
    }

    match property.type_name.as_deref() {
        Some("boolean") => "${1:true}".to_string(),
        Some("number") | Some("integer") => "${1:0}".to_string(),
        Some("array") => "[${1}]".to_string(),
        Some("object") => "{ ${1:key}: ${2:value} }".to_string(),
        _ => "\"${1:value}\"".to_string(),
    }
}
