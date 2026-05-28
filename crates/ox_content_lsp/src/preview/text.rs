use ox_content_ast::{Heading, Node};

use crate::frontmatter::FrontmatterBlock;

pub fn preview_title(block: Option<&FrontmatterBlock>, nodes: &[Node<'_>]) -> Option<String> {
    if let Some(title) = block
        .and_then(|block| block.value.as_ref())
        .and_then(|value| value.as_object())
        .and_then(|value| value.get("title"))
        .and_then(serde_json::Value::as_str)
    {
        return Some(title.to_string());
    }

    nodes.iter().find_map(|node| match node {
        Node::Heading(heading) if heading.depth == 1 => Some(heading_text(heading)),
        _ => None,
    })
}

pub fn heading_text(heading: &Heading<'_>) -> String {
    let mut text = String::new();
    for child in &heading.children {
        collect_text(child, &mut text);
    }
    text
}

fn collect_text(node: &Node<'_>, text: &mut String) {
    match node {
        Node::Text(value) => text.push_str(value.value),
        Node::InlineCode(value) => text.push_str(value.value),
        Node::Emphasis(value) => {
            for child in &value.children {
                collect_text(child, text);
            }
        }
        Node::Strong(value) => {
            for child in &value.children {
                collect_text(child, text);
            }
        }
        Node::Delete(value) => {
            for child in &value.children {
                collect_text(child, text);
            }
        }
        Node::Link(value) => {
            for child in &value.children {
                collect_text(child, text);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use ox_content_allocator::Allocator;
    use ox_content_parser::{Parser, ParserOptions};

    use super::*;

    fn first_heading<'a>(allocator: &'a Allocator, source: &'a str) -> Heading<'a> {
        let parser = Parser::with_options(allocator, source, ParserOptions::default());
        let mut doc = parser.parse().unwrap();
        let node = doc.children.pop().expect("at least one node");
        match node {
            Node::Heading(heading) => heading,
            other => panic!("expected heading, got {other:?}"),
        }
    }

    #[test]
    fn heading_text_flattens_inline_styling() {
        let allocator = Allocator::new();
        let heading = first_heading(
            &allocator,
            "# Plain *italic* and **bold** and `code` and [link](https://x)\n",
        );
        assert_eq!(heading_text(&heading), "Plain italic and bold and code and link");
    }

    #[test]
    fn preview_title_prefers_frontmatter_over_heading() {
        // When frontmatter has `title`, it wins over the first H1. This is
        // the contract the VS Code preview panel relies on.
        let zero_range = tower_lsp::lsp_types::Range::default();
        let fm = crate::frontmatter::FrontmatterBlock {
            block_range: zero_range,
            content_range: zero_range,
            content_start_offset: 0,
            content_end_offset: 0,
            block_end_offset: 0,
            value: Some(serde_json::json!({ "title": "From FM" })),
            diagnostics: Vec::new(),
            top_level_keys: Vec::new(),
        };
        let allocator = Allocator::new();
        let heading_node = Node::Heading(first_heading(&allocator, "# Heading Wins When Empty\n"));
        let nodes = [heading_node];
        assert_eq!(preview_title(Some(&fm), &nodes).as_deref(), Some("From FM"));
    }

    #[test]
    fn preview_title_falls_back_to_first_h1() {
        let allocator = Allocator::new();
        let heading_node = Node::Heading(first_heading(&allocator, "# Heading Wins\n"));
        let nodes = [heading_node];
        assert_eq!(preview_title(None, &nodes).as_deref(), Some("Heading Wins"));
    }

    #[test]
    fn preview_title_returns_none_when_no_signal() {
        let nodes: [Node<'_>; 0] = [];
        assert_eq!(preview_title(None, &nodes), None);
    }
}
