//! Inline table-of-contents discovery.
//!
//! A standalone `[[toc]]` paragraph is rendered as a flat navigation list. This module
//! pre-collects headings only when such a marker exists, preserving duplicate-heading
//! ID behavior while avoiding allocation in documents without TOC markers.

use rustc_hash::FxHashMap;

use ox_content_ast::{Document, ListItem, Node, Paragraph};

use super::heading::{collect_heading_text, slugify_heading};

#[derive(Debug, Clone)]
pub(super) struct InlineTocEntry {
    pub(super) depth: u8,
    pub(super) text: String,
    pub(super) id: String,
}

pub(super) fn collect_inline_toc_entries(
    document: &Document<'_>,
    max_depth: u8,
    entries: &mut Vec<InlineTocEntry>,
) {
    let mut counts = FxHashMap::default();

    for node in &document.children {
        collect_inline_toc_node(node, max_depth, &mut counts, entries);
    }
}

/// Cheap, allocation-free scan for a standalone `[[toc]]` paragraph. The
/// directive is recognized only when the paragraph contains a single text
/// node whose trimmed value is exactly `[[toc]]` — which is also the only
/// form `visit_paragraph` will render as a TOC.
pub(super) fn document_has_toc_marker(document: &Document<'_>) -> bool {
    document.children.iter().any(node_has_toc_marker)
}

fn node_has_toc_marker(node: &Node<'_>) -> bool {
    match node {
        Node::Paragraph(p) => is_toc_marker_paragraph(p),
        Node::BlockQuote(bq) => bq.children.iter().any(node_has_toc_marker),
        Node::List(list) => list.children.iter().any(list_item_has_toc_marker),
        Node::ListItem(item) => list_item_has_toc_marker(item),
        Node::FootnoteDefinition(def) => def.children.iter().any(node_has_toc_marker),
        _ => false,
    }
}

fn list_item_has_toc_marker(item: &ListItem<'_>) -> bool {
    item.children.iter().any(node_has_toc_marker)
}

pub(super) fn is_toc_marker_paragraph(paragraph: &Paragraph<'_>) -> bool {
    // Equivalent to the prior
    // `collect_text_nodes_only(...).is_some_and(|t| t.trim() == "[[toc]]")`
    // check, but allocation-free: bails on the first non-Text child and
    // matches the marker byte-by-byte against the concatenated text. Note
    // that the inline parser emits the literal "[[toc]]" as three Text
    // nodes (`[`, `[`, `toc]]`) because the bracket-as-link path fails
    // open — so a "single Text child only" shortcut would miss it.
    const MARKER: &[u8] = b"[[toc]]";
    let mut matched = 0usize;
    let mut after_marker_ws = false;

    for child in &paragraph.children {
        let Node::Text(text) = child else {
            return false;
        };
        for &byte in text.value.as_bytes() {
            let is_ws = matches!(byte, b' ' | b'\t' | b'\n' | b'\r');
            if is_ws {
                if matched > 0 {
                    after_marker_ws = true;
                }
                continue;
            }
            if after_marker_ws || matched == MARKER.len() || byte != MARKER[matched] {
                return false;
            }
            matched += 1;
        }
    }

    matched == MARKER.len()
}

fn collect_inline_toc_node(
    node: &Node<'_>,
    max_depth: u8,
    counts: &mut FxHashMap<String, usize>,
    entries: &mut Vec<InlineTocEntry>,
) {
    use std::fmt::Write as _;

    match node {
        Node::Heading(heading) => {
            let include_heading = heading.depth <= max_depth;
            let text = collect_heading_text(&heading.children);
            let mut slug = slugify_heading(&text);
            let id = if let Some(count) = counts.get_mut(slug.as_str()) {
                let suffix = *count;
                *count += 1;
                if include_heading {
                    let _ = write!(slug, "-{suffix}");
                    Some(slug)
                } else {
                    None
                }
            } else if include_heading {
                counts.insert(slug.clone(), 1);
                Some(slug)
            } else {
                counts.insert(slug, 1);
                None
            };

            if let Some(id) = id {
                entries.push(InlineTocEntry { depth: heading.depth, text, id });
            }
        }
        Node::BlockQuote(block_quote) => {
            for child in &block_quote.children {
                collect_inline_toc_node(child, max_depth, counts, entries);
            }
        }
        Node::List(list) => {
            for item in &list.children {
                for child in &item.children {
                    collect_inline_toc_node(child, max_depth, counts, entries);
                }
            }
        }
        Node::ListItem(item) => {
            for child in &item.children {
                collect_inline_toc_node(child, max_depth, counts, entries);
            }
        }
        Node::FootnoteDefinition(definition) => {
            for child in &definition.children {
                collect_inline_toc_node(child, max_depth, counts, entries);
            }
        }
        _ => {}
    }
}
