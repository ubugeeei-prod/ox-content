use ox_content_allocator::Allocator;
use ox_content_ast::Node;
use ox_content_parser::{Parser, ParserOptions};

#[path = "edge_cases/blocks.rs"]
mod blocks;
#[path = "edge_cases/html.rs"]
mod html;
#[path = "edge_cases/inline.rs"]
mod inline;
#[path = "edge_cases/lists.rs"]
mod lists;
#[path = "edge_cases/prepass.rs"]
mod prepass;
#[path = "edge_cases/tables.rs"]
mod tables;

fn parse_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
) -> ox_content_ast::Document<'a> {
    Parser::with_options(allocator, source, options).parse().unwrap()
}

fn first_text<'a>(node: &'a Node<'a>) -> Option<&'a str> {
    match node {
        Node::Text(text) => Some(text.value),
        Node::Paragraph(paragraph) => paragraph.children.iter().find_map(first_text),
        Node::Heading(heading) => heading.children.iter().find_map(first_text),
        Node::Emphasis(emphasis) => emphasis.children.iter().find_map(first_text),
        Node::Strong(strong) => strong.children.iter().find_map(first_text),
        Node::Delete(delete) => delete.children.iter().find_map(first_text),
        Node::Link(link) => link.children.iter().find_map(first_text),
        Node::List(list) => {
            list.children.iter().flat_map(|item| item.children.iter()).find_map(first_text)
        }
        Node::ListItem(item) => item.children.iter().find_map(first_text),
        _ => None,
    }
}

fn first_text_in_nodes<'a>(nodes: impl IntoIterator<Item = &'a Node<'a>>) -> Option<&'a str> {
    nodes.into_iter().find_map(first_text)
}

fn flatten_text(node: &Node<'_>) -> String {
    match node {
        Node::Text(text) => text.value.to_string(),
        Node::Paragraph(paragraph) => paragraph.children.iter().map(flatten_text).collect(),
        Node::Heading(heading) => heading.children.iter().map(flatten_text).collect(),
        Node::Emphasis(emphasis) => emphasis.children.iter().map(flatten_text).collect(),
        Node::Strong(strong) => strong.children.iter().map(flatten_text).collect(),
        Node::Delete(delete) => delete.children.iter().map(flatten_text).collect(),
        Node::Link(link) => link.children.iter().map(flatten_text).collect(),
        Node::List(list) => {
            list.children.iter().flat_map(|item| item.children.iter()).map(flatten_text).collect()
        }
        _ => String::new(),
    }
}
