//! Heading text extraction and slug generation.
//!
//! Heading IDs and inline TOCs must agree on the same slug rules. This module owns the
//! shared text collector and slugifier so both code paths reuse the same Unicode-aware
//! normalization behavior.

use ox_content_ast::{Link, Node};

/// Class name on the opt-in heading permalink control.
///
/// Headings that already contain an `<a class="header-anchor">` or a `#`
/// link to the generated id do not receive a second marker.
pub const HEADING_PERMALINK_CLASS: &str = "header-anchor";

pub(super) fn collect_heading_text(nodes: &[Node<'_>]) -> String {
    let mut text = String::new();
    collect_heading_text_into(nodes, &mut text);
    text
}

pub(super) fn collect_heading_text_into(nodes: &[Node<'_>], text: &mut String) {
    crate::profile_span_detail!("renderer::collect_heading_text");
    for node in nodes {
        collect_node_text(node, text);
    }
}

fn collect_node_text(node: &Node<'_>, text: &mut String) {
    match node {
        Node::Text(value) => text.push_str(value.value),
        Node::InlineCode(value) => text.push_str(value.value),
        Node::Emphasis(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Strong(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Delete(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        Node::Link(value) => {
            for child in &value.children {
                collect_node_text(child, text);
            }
        }
        _ => {}
    }
}

/// Returns the canonical fragment identifier used for a rendered heading.
#[must_use]
pub fn slugify_heading(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    slugify_heading_into(text, &mut out);
    out
}

/// Slugify `text` into `out`.
///
/// `out` is **not** cleared by this function. Renderers keep a long-lived
/// scratch buffer for heading IDs, clear it at the call site, and pass it back
/// here on every heading. That avoids allocating one temporary slug string per
/// heading while still leaving ownership decisions, such as cloning the final
/// unique id into a hash map, with the caller.
pub(super) fn slugify_heading_into(text: &str, out: &mut String) {
    crate::profile_span_detail!("renderer::slugify");
    // Single-pass slugify. The hot path is the all-ASCII byte loop: no UTF-8
    // decode and no `char::to_lowercase` iterator allocation per character.
    // We switch to the Unicode-aware char iterator only for contiguous
    // non-ASCII runs, preserving Japanese and other non-Latin heading text
    // without slowing down the common ASCII API-doc heading.
    let bytes = text.as_bytes();
    out.reserve(text.len());
    let start_len = out.len();
    let mut last_was_separator = true;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b < 0x80 {
            if b.is_ascii_alphanumeric() {
                // Lowercase ASCII letters with a branchless add.
                let lower = if b.is_ascii_uppercase() { b + 32 } else { b };
                out.push(lower as char);
                last_was_separator = false;
            } else if !last_was_separator {
                out.push('-');
                last_was_separator = true;
            }
            i += 1;
        } else {
            // Find the next ASCII boundary and process the multi-byte run
            // through the char iterator (handles Unicode case folding /
            // alphanumeric classification correctly).
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] >= 0x80 {
                j += 1;
            }
            for ch in text[i..j].chars() {
                for lower in ch.to_lowercase() {
                    if lower.is_alphanumeric() {
                        out.push(lower);
                        last_was_separator = false;
                    } else if !last_was_separator {
                        out.push('-');
                        last_was_separator = true;
                    }
                }
            }
            i = j;
        }
    }

    while out.len() > start_len && out.ends_with('-') {
        out.pop();
    }

    if out.len() == start_len {
        out.push_str("section");
    }
}

pub(super) fn heading_has_permalink_marker(nodes: &[Node<'_>], id: &str) -> bool {
    nodes.iter().any(|node| node_has_permalink_marker(node, id))
}

fn node_has_permalink_marker(node: &Node<'_>, id: &str) -> bool {
    match node {
        Node::Link(link) => {
            is_hash_permalink_link(link, id) || heading_has_permalink_marker(&link.children, id)
        }
        Node::Html(html) => html_has_header_anchor(html.value),
        Node::Emphasis(value) => heading_has_permalink_marker(&value.children, id),
        Node::Strong(value) => heading_has_permalink_marker(&value.children, id),
        Node::Delete(value) => heading_has_permalink_marker(&value.children, id),
        _ => false,
    }
}

fn is_hash_permalink_link(link: &Link<'_>, id: &str) -> bool {
    let url = link.url;
    if url.len() != id.len() + 1 || !url.starts_with('#') || &url[1..] != id {
        return false;
    }
    collect_heading_text(&link.children) == "#"
}

fn html_has_header_anchor(value: &str) -> bool {
    let mut rest = value;
    while let Some(start) = rest.find("<a") {
        let tag = &rest[start..];
        let Some(end) = tag.find('>') else {
            break;
        };
        if class_attr_contains(&tag[..end], HEADING_PERMALINK_CLASS) {
            return true;
        }
        rest = &tag[end + 1..];
    }
    false
}

fn class_attr_contains(tag: &str, class_name: &str) -> bool {
    for quote in ['"', '\''] {
        let needle = if quote == '"' { "class=\"" } else { "class='" };
        if let Some(index) = tag.find(needle) {
            let after = &tag[index + needle.len()..];
            if let Some(end) = after.find(quote) {
                return after[..end].split_whitespace().any(|class| class == class_name);
            }
        }
    }
    false
}
