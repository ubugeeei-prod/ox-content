use rustc_hash::FxHashSet;

use ox_content_ast::{Document, Node};

pub fn collect_anchors(source: &str, document: &Document<'_>) -> FxHashSet<String> {
    let mut anchors = FxHashSet::default();
    collect_anchors_into(source, &document.children, &mut anchors);
    anchors
}

fn collect_anchors_into(source: &str, nodes: &[Node<'_>], out: &mut FxHashSet<String>) {
    for node in nodes {
        if let Node::Heading(heading) = node {
            let text = inline_text(source, &heading.children);
            out.insert(slugify(&text));
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

/// GitHub-style heading slug. Lowercase, strip everything that is not
/// `[a-z0-9 -]`, collapse spaces into `-`. Matches the slug rules
/// `ox_content_renderer` uses, so anchors emitted by the renderer for a
/// given heading round-trip through the checker.
fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.extend(ch.to_lowercase());
        } else if ch == ' ' || ch == '-' || ch == '_' {
            out.push('-');
        }
        // Drop everything else (punctuation, emoji, etc.).
    }
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    out.trim_matches('-').to_string()
}
