use ox_content_mdc_checker::{Component, Registry};
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range};

use crate::document::TextDocumentState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolAt<'a> {
    Component { name: &'a str, range: Range },
    Attribute { component: &'a str, name: &'a str, range: Range },
}

#[must_use]
pub fn symbol_at(document: &TextDocumentState, position: Position) -> Option<SymbolAt<'_>> {
    let source = document.text();
    let offset = document.position_to_offset(position);
    let tag_start = open_tag_start_before(source, offset)?;
    let bytes = source.as_bytes();
    let closing = bytes.get(tag_start + 1) == Some(&b'/');
    let name_start = tag_start + 1 + usize::from(closing);
    let name_end = scan_name(bytes, name_start);
    if name_end == name_start {
        return None;
    }
    let name = &source[name_start..name_end];
    if !name.starts_with(|ch: char| ch.is_ascii_uppercase()) {
        return None;
    }
    if contains_offset(name_start, name_end, offset) {
        return Some(SymbolAt::Component {
            name,
            range: document.range_from_offsets(name_start, name_end),
        });
    }
    if closing {
        return None;
    }
    attribute_at_offset(document, name, name_end, offset)
}

fn open_tag_start_before(source: &str, offset: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut cursor = offset.min(bytes.len());
    let lower_bound = cursor.saturating_sub(4096);
    while cursor > lower_bound {
        cursor -= 1;
        match bytes[cursor] {
            b'<' => return Some(cursor),
            b'>' => return None,
            _ => {}
        }
    }
    None
}

fn scan_name(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len()
        && (bytes[cursor].is_ascii_alphanumeric() || matches!(bytes[cursor], b'.' | b'_' | b'-'))
    {
        cursor += 1;
    }
    cursor
}

fn contains_offset(start: usize, end: usize, offset: usize) -> bool {
    offset >= start && offset <= end
}

fn attribute_at_offset<'a>(
    document: &'a TextDocumentState,
    component: &'a str,
    mut cursor: usize,
    offset: usize,
) -> Option<SymbolAt<'a>> {
    let source = document.text();
    let bytes = source.as_bytes();
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || matches!(bytes[cursor], b'>' | b'/') {
            return None;
        }
        let attr_start = cursor;
        while cursor < bytes.len() && is_attr_name_char(bytes[cursor]) {
            cursor += 1;
        }
        if cursor == attr_start {
            cursor += 1;
            continue;
        }
        let attr_end = cursor;
        if contains_offset(attr_start, attr_end, offset) {
            return Some(SymbolAt::Attribute {
                component,
                name: &source[attr_start..attr_end],
                range: document.range_from_offsets(attr_start, attr_end),
            });
        }
        cursor = skip_attribute_value(bytes, cursor);
    }
    None
}

fn is_attr_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-')
}

fn skip_attribute_value(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'=') {
        return cursor;
    }
    cursor += 1;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    match bytes.get(cursor).copied() {
        Some(b'"' | b'\'') => {
            let quote = bytes[cursor];
            cursor += 1;
            while cursor < bytes.len() {
                if bytes[cursor] == quote {
                    return cursor + 1;
                }
                cursor += 1;
            }
            cursor
        }
        Some(b'{') => {
            let mut depth = 1usize;
            cursor += 1;
            while cursor < bytes.len() {
                match bytes[cursor] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            return cursor + 1;
                        }
                    }
                    _ => {}
                }
                cursor += 1;
            }
            cursor
        }
        _ => {
            while cursor < bytes.len()
                && !bytes[cursor].is_ascii_whitespace()
                && bytes[cursor] != b'>'
            {
                cursor += 1;
            }
            cursor
        }
    }
}

#[must_use]
pub fn hover(symbol: &SymbolAt<'_>, registry: &Registry) -> Option<Hover> {
    let (value, range) = match symbol {
        SymbolAt::Component { name, range } => {
            let component = registry.get(name)?;
            (component_hover(name, component), *range)
        }
        SymbolAt::Attribute { component, name, range } => {
            let attribute = registry.get(component)?.attributes.get(*name)?;
            (attribute_hover(component, name, attribute), *range)
        }
    };
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value }),
        range: Some(range),
    })
}

fn component_hover(name: &str, component: &Component) -> String {
    let mut lines = vec![format!("**`<{name}>`**"), "MDC component".to_string()];
    if let Some(description) = &component.description {
        lines.extend([String::new(), description.clone()]);
    }
    if !component.attributes.is_empty() {
        lines.extend([String::new(), "Props:".to_string()]);
        for (name, attribute) in &component.attributes {
            let type_hint = attribute.type_hint.as_deref().unwrap_or("value");
            let required = if attribute.required { " required" } else { "" };
            lines.push(format!("- `{name}`: `{type_hint}`{required}"));
        }
    }
    lines.join("\n")
}

fn attribute_hover(
    component: &str,
    name: &str,
    attribute: &ox_content_mdc_checker::Attribute,
) -> String {
    let mut lines = vec![format!("**`{name}`**"), format!("MDC prop for `<{component}>`")];
    if let Some(type_hint) = &attribute.type_hint {
        lines.push(format!("Type: `{type_hint}`"));
    }
    if attribute.required {
        lines.push("Required.".to_string());
    }
    if let Some(description) = &attribute.description {
        lines.extend([String::new(), description.clone()]);
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests;
