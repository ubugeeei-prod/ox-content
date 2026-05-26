//! MDC-specific LSP features (component name + attribute completion,
//! hover). Backed by `ox_content_mdc_checker::Registry`.

use ox_content_mdc_checker::{Component, Registry};
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, MarkupContent, MarkupKind,
};

/// Position of the cursor in an in-flight MDC tag, derived purely
/// from the line text before it. The detection is textual so it
/// keeps working while the user is typing and the surrounding tag
/// is not yet syntactically valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionSite<'a> {
    /// `<Foo|`. Suggest component names whose start matches `prefix`.
    ComponentName { prefix: &'a str },
    /// `<Foo a|`. Suggest attributes whose start matches `prefix`,
    /// scoped to `component`. The current attribute prefix may be
    /// empty when the cursor is right after whitespace.
    AttributeName { component: &'a str, prefix: &'a str },
}

/// Inspect the chunk of the current line that sits before the cursor
/// and decide whether MDC completion applies. Returns `None` when the
/// cursor is outside any MDC tag.
#[must_use]
pub fn detect_site(line_prefix: &str) -> Option<CompletionSite<'_>> {
    // Walk back to the last `<` that isn't followed by `/` (closing
    // tag completion would be just `</TagName>` which we leave to
    // the user — it auto-completes from the matching opener).
    let bytes = line_prefix.as_bytes();
    let lt = line_prefix.rfind('<')?;
    if lt + 1 < bytes.len() && bytes[lt + 1] == b'/' {
        return None;
    }
    // Anything beyond the next `>` means we are past the tag.
    if line_prefix[lt..].contains('>') {
        return None;
    }
    let inside = &line_prefix[lt + 1..];

    // Component name is the first identifier-like run in `inside`.
    // Bail out before that if the user opened `<` with non-alpha
    // (would be a lone `<` for typography or an HTML comment).
    let first = inside.chars().next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    // MDC components start with an uppercase letter, matching the
    // checker's heuristic in `lib.rs::check`. Lowercase tags fall
    // through to the HTML completion path elsewhere.
    if !first.is_ascii_uppercase() {
        return None;
    }

    let name_end = inside
        .char_indices()
        .find(|(_, ch)| !is_tag_name_char(*ch))
        .map_or(inside.len(), |(idx, _)| idx);

    if name_end == inside.len() {
        // Still typing the component name (no whitespace yet).
        return Some(CompletionSite::ComponentName { prefix: inside });
    }

    let component_name = &inside[..name_end];
    let after_name = &inside[name_end..];

    // Past the name. We only complete attribute names when the cursor
    // sits in a "fresh slot": after whitespace and before any `=`.
    // `<Foo bar=` and `<Foo bar="..."` should not surface suggestions.
    let mut attr_prefix_start = None;
    let mut equal_seen = false;
    let mut in_quote: Option<u8> = None;

    for (idx, byte) in after_name.bytes().enumerate() {
        if let Some(quote) = in_quote {
            if byte == quote {
                in_quote = None;
                equal_seen = false;
                attr_prefix_start = None;
            }
            continue;
        }
        match byte {
            b'"' | b'\'' => in_quote = Some(byte),
            b'=' => equal_seen = true,
            b' ' | b'\t' => {
                if equal_seen {
                    // Whitespace inside an attribute value is unusual
                    // but harmless; treat it like we're between props.
                    equal_seen = false;
                }
                attr_prefix_start = Some(idx + 1);
            }
            _ if attr_prefix_start.is_none() => {
                // First non-space char after the name; we're in the
                // middle of typing an attribute prefix.
                attr_prefix_start = Some(idx);
            }
            _ => {}
        }
    }

    if equal_seen || in_quote.is_some() {
        return None;
    }

    let prefix_start = attr_prefix_start.unwrap_or(after_name.len());
    Some(CompletionSite::AttributeName {
        component: component_name,
        prefix: &after_name[prefix_start..],
    })
}

fn is_tag_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-')
}

#[must_use]
pub fn completion_items(site: &CompletionSite<'_>, registry: &Registry) -> Vec<CompletionItem> {
    match site {
        CompletionSite::ComponentName { prefix } => registry
            .complete_components(prefix)
            .map(|(name, component)| component_item(name, component))
            .collect(),
        CompletionSite::AttributeName { component, prefix } => registry
            .complete_attributes(component, prefix)
            .map(|(name, attribute)| attribute_item(name, attribute))
            .collect(),
    }
}

fn component_item(name: &str, component: &Component) -> CompletionItem {
    CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some("MDC component".to_string()),
        documentation: component.description.as_ref().map(|description| markdown_doc(description)),
        ..Default::default()
    }
}

fn attribute_item(name: &str, attribute: &ox_content_mdc_checker::Attribute) -> CompletionItem {
    let detail = match attribute.type_hint.as_deref() {
        Some(type_hint) => format!("MDC prop: {type_hint}"),
        None => "MDC prop".to_string(),
    };
    CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some(detail),
        documentation: attribute.description.as_ref().map(|description| markdown_doc(description)),
        insert_text: Some(format!("{name}=\"$0\"")),
        insert_text_format: Some(tower_lsp::lsp_types::InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}

fn markdown_doc(value: &str) -> Documentation {
    Documentation::MarkupContent(MarkupContent {
        kind: MarkupKind::Markdown,
        value: value.to_string(),
    })
}

#[cfg(test)]
mod tests;
