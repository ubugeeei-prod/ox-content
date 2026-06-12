use rustc_hash::FxHashMap;

use ox_content_ast::{Document, Heading, Node};

/// Table of contents entry.
#[derive(serde::Serialize)]
pub struct TocEntry {
    pub depth: u8,
    pub text: String,
    pub slug: String,
}

/// Extracts table of contents from document headings.
pub fn extract_toc(doc: &Document, max_depth: u8) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    let mut slug_counts = FxHashMap::default();

    for node in &doc.children {
        if let Node::Heading(heading) = node {
            if heading.depth <= max_depth {
                let text = extract_heading_text(heading);
                let slug = unique_slug(slugify(&text), &mut slug_counts);
                entries.push(TocEntry { depth: heading.depth, text, slug });
            }
        }
    }

    entries
}

/// Extracts plain text from a heading node.
fn extract_heading_text(heading: &Heading) -> String {
    let mut text = String::new();
    for child in &heading.children {
        collect_text(child, &mut text);
    }
    text
}

/// Recursively collects text from nodes.
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

/// Converts text to URL-friendly slug.
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn unique_slug(slug: String, counts: &mut FxHashMap<String, usize>) -> String {
    let slug = if slug.is_empty() { "section".to_string() } else { slug };
    let count = counts.entry(slug.clone()).or_insert(0);
    let unique = if *count == 0 { slug } else { format!("{slug}-{count}") };
    *count += 1;
    unique
}

#[cfg(test)]
mod tests {
    use ox_content_allocator::Allocator;
    use ox_content_parser::Parser;

    use super::extract_toc;

    #[test]
    fn toc_slugs_are_unique_and_match_heading_ids() {
        let allocator = Allocator::new();
        let doc = Parser::new(&allocator, "## Setup!\n## Setup?\n##").parse().unwrap();

        let toc = extract_toc(&doc, 3);

        assert_eq!(toc[0].slug, "setup");
        assert_eq!(toc[1].slug, "setup-1");
        assert_eq!(toc[2].slug, "section");
    }
}
