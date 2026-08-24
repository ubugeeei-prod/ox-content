use rustc_hash::FxHashMap;

use ox_content_ast::{Document, Heading, Node};
use ox_content_renderer::slugify_heading;

use crate::TocEntry;

pub(super) fn extract_toc(doc: &Document, max_depth: u8) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    let mut slug_counts = FxHashMap::default();

    for node in &doc.children {
        if let Node::Heading(heading) = node
            && heading.depth <= max_depth
        {
            let text = extract_heading_text(heading);
            let slug = unique_slug(slugify_heading(&text), &mut slug_counts);
            push_nested_toc_entry(
                &mut entries,
                TocEntry { depth: heading.depth, text, slug, children: Vec::new() },
            );
        }
    }

    entries
}

fn push_nested_toc_entry(entries: &mut Vec<TocEntry>, entry: TocEntry) {
    if let Some(last) = entries.last_mut()
        && last.depth < entry.depth
    {
        push_nested_toc_entry(&mut last.children, entry);
        return;
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

fn unique_slug(slug: String, counts: &mut FxHashMap<String, usize>) -> String {
    let slug = if slug.is_empty() { "section".to_string() } else { slug };
    let count = counts.entry(slug.clone()).or_insert(0);
    let unique = if *count == 0 { slug } else { format!("{slug}-{count}") };
    *count += 1;
    unique
}
