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
