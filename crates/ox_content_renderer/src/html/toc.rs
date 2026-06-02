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

/// Cheap document-level facts needed at `HtmlRenderer::render` entry.
///
/// Rendering already scans for `[[toc]]` before deciding whether to collect
/// TOC entries. Counting headings in the same traversal lets the renderer
/// reserve the heading-id map once, avoiding incremental hash-map growth
/// while preserving the lazy TOC behavior for documents without a marker.
pub(super) struct DocumentRenderScan {
    pub(super) has_toc_marker: bool,
    pub(super) heading_count: usize,
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

/// Cheap, allocation-free scan for renderer setup.
///
/// The TOC directive is recognized only when a paragraph's text content trims
/// to exactly `[[toc]]`, which is also the only form `visit_paragraph` will
/// render as a TOC.
pub(super) fn scan_document_for_render(document: &Document<'_>) -> DocumentRenderScan {
    let mut scan = DocumentRenderScan { has_toc_marker: false, heading_count: 0 };
    for node in &document.children {
        scan_node_for_render(node, &mut scan);
    }
    scan
}

fn scan_node_for_render(node: &Node<'_>, scan: &mut DocumentRenderScan) {
    // This traversal intentionally collects only facts that are free to derive:
    // "does any paragraph equal the TOC marker?" and "how many headings exist?".
    // It does not slugify headings or collect text. That keeps the no-TOC
    // render path allocation-free while still giving `HtmlRenderer::render`
    // enough information to reserve the heading-id map up front.
    match node {
        Node::Heading(_) => scan.heading_count += 1,
        Node::Paragraph(p) if !scan.has_toc_marker && is_toc_marker_paragraph(p) => {
            scan.has_toc_marker = true;
        }
        Node::Paragraph(_) => {}
        Node::BlockQuote(bq) => {
            for child in &bq.children {
                scan_node_for_render(child, scan);
            }
        }
        Node::List(list) => {
            for item in &list.children {
                scan_list_item_for_render(item, scan);
            }
        }
        Node::ListItem(item) => scan_list_item_for_render(item, scan),
        Node::FootnoteDefinition(def) => {
            for child in &def.children {
                scan_node_for_render(child, scan);
            }
        }
        _ => {}
    }
}

fn scan_list_item_for_render(item: &ListItem<'_>, scan: &mut DocumentRenderScan) {
    for child in &item.children {
        scan_node_for_render(child, scan);
    }
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
