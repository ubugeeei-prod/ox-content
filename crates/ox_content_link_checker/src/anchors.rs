use rustc_hash::{FxHashMap, FxHashSet};

use ox_content_ast::{Document, Node};

pub fn collect_anchors(source: &str, document: &Document<'_>) -> FxHashSet<String> {
    let mut anchors = FxHashSet::default();
    let mut counts = FxHashMap::default();
    collect_anchors_into(source, &document.children, &mut anchors, &mut counts);
    anchors
}

fn collect_anchors_into(
    source: &str,
    nodes: &[Node<'_>],
    out: &mut FxHashSet<String>,
    counts: &mut FxHashMap<String, u32>,
) {
    for node in nodes {
        match node {
            Node::Heading(heading) => {
                let text = inline_text(source, &heading.children);
                let slug = slugify(&text);
                let count = counts.entry(slug.clone()).or_insert(0);
                let unique = if *count == 0 { slug } else { format!("{slug}-{count}") };
                *count += 1;
                out.insert(unique);
            }
            Node::BlockQuote(block) => {
                collect_anchors_into(source, &block.children, out, counts);
            }
            Node::List(list) => {
                for item in &list.children {
                    collect_anchors_into(source, &item.children, out, counts);
                }
            }
            _ => {}
        }
    }
}

fn inline_text(source: &str, nodes: &[Node<'_>]) -> String {
    let mut buf = String::new();
    flatten(source, nodes, &mut buf);
    buf
}

fn flatten(source: &str, nodes: &[Node<'_>], buf: &mut String) {
    for node in nodes {
        match node {
            Node::Text(t) => buf.push_str(t.value),
            Node::InlineCode(c) => buf.push_str(c.value),
            Node::Emphasis(e) => flatten(source, &e.children, buf),
            Node::Strong(s) => flatten(source, &s.children, buf),
            Node::Delete(d) => flatten(source, &d.children, buf),
            Node::Link(l) => flatten(source, &l.children, buf),
            _ => {
                let span = node.span();
                let text = &source[span.start as usize..span.end as usize];
                buf.push_str(text);
            }
        }
    }
}

/// Keep this byte-for-byte compatible with the renderer's Unicode-aware
/// heading slugger. Punctuation collapses to one separator, non-Latin
/// alphanumerics are preserved, and an empty result becomes `section`.
fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_was_separator = true;
    for ch in input.chars() {
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
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("section");
    }
    out
}
