use rustc_hash::FxHashMap;

use ox_content_ast::{Document, Heading, Node};

use crate::TocEntry;

pub(super) fn extract_toc(doc: &Document, max_depth: u8) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    let mut slug_counts = FxHashMap::default();

    for node in &doc.children {
        if let Node::Heading(heading) = node {
            if heading.depth <= max_depth {
                let text = extract_heading_text(heading);
                let slug = unique_slug(slugify(&text), &mut slug_counts);
                push_nested_toc_entry(
                    &mut entries,
                    TocEntry { depth: heading.depth, text, slug, children: Vec::new() },
                );
            }
        }
    }

    entries
}

fn push_nested_toc_entry(entries: &mut Vec<TocEntry>, entry: TocEntry) {
    if let Some(last) = entries.last_mut() {
        if last.depth < entry.depth {
            push_nested_toc_entry(&mut last.children, entry);
            return;
        }
    }

    entries.push(entry);
}

fn extract_heading_text(heading: &Heading) -> String {
    let mut text = String::new();
    for child in &heading.children {
        collect_text(child, &mut text);
    }
    text
}

fn collect_text(node: &Node, text: &mut String) {
    match node {
        Node::Text(t) => text.push_str(t.value),
        Node::Emphasis(e) => {
            for child in &e.children {
                collect_text(child, text);
            }
        }
        Node::Strong(s) => {
            for child in &s.children {
                collect_text(child, text);
            }
        }
        Node::InlineCode(c) => text.push_str(c.value),
        Node::Delete(d) => {
            for child in &d.children {
                collect_text(child, text);
            }
        }
        Node::Link(l) => {
            for child in &l.children {
                collect_text(child, text);
            }
        }
        _ => {}
    }
}

fn slugify(text: &str) -> String {
    let mapped: String = text
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { ' ' })
        .collect();
    // Join the whitespace-split tokens with '-' directly, skipping the
    // intermediate `Vec<&str>` and the separate `join` allocation. This TOC
    // slugger runs for every heading in NAPI transforms; `mapped.len()` is a
    // safe upper bound for the slug because separators only shrink it.
    let mut slug = String::with_capacity(mapped.len());
    for token in mapped.split_whitespace() {
        if !slug.is_empty() {
            slug.push('-');
        }
        slug.push_str(token);
    }
    slug
}

fn unique_slug(slug: String, counts: &mut FxHashMap<String, usize>) -> String {
    let slug = if slug.is_empty() { "section".to_string() } else { slug };
    let count = counts.entry(slug.clone()).or_insert(0);
    let unique = if *count == 0 { slug } else { format!("{slug}-{count}") };
    *count += 1;
    unique
}
