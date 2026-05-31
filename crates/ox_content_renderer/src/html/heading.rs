//! Heading text extraction and slug generation.
//!
//! Heading IDs and inline TOCs must agree on the same slug rules. This module owns the
//! shared text collector and slugifier so both code paths reuse the same Unicode-aware
//! normalization behavior.

use ox_content_ast::Node;

pub(super) fn collect_heading_text(nodes: &[Node<'_>]) -> String {
    let mut text = String::new();
    collect_heading_text_into(nodes, &mut text);
    text
}

pub(super) fn collect_heading_text_into(nodes: &[Node<'_>], text: &mut String) {
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

pub(super) fn slugify_heading(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    slugify_heading_into(text, &mut out);
    out
}

/// Slugify `text` into `out`. `out` is **not** cleared by this function —
/// callers should clear it themselves so they can reuse a long-lived
/// scratch buffer across many headings without giving up the allocation.
pub(super) fn slugify_heading_into(text: &str, out: &mut String) {
    // Single-pass slugify. Hot path is the all-ASCII byte loop (no
    // UTF-8 decode, no `char::to_lowercase` iterator allocation per
    // character); we fall back to the char iterator only when a
    // non-ASCII byte appears.
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
