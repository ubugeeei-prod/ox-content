use ox_content_ast::{ListItem, Node, Span};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn offset_span(span: &mut Span, offset: u32) {
        span.start += offset;
        span.end += offset;
    }

    pub(super) fn offset_node_spans(node: &mut Node<'a>, offset: u32) {
        match node {
            Node::Paragraph(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::Heading(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::ThematicBreak(node) => Self::offset_span(&mut node.span, offset),
            Node::BlockQuote(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::List(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_list_item_spans(child, offset);
                }
            }
            Node::ListItem(node) => Self::offset_list_item_spans(node, offset),
            Node::CodeBlock(node) => Self::offset_span(&mut node.span, offset),
            Node::Html(node) => Self::offset_span(&mut node.span, offset),
            Node::Table(node) => {
                Self::offset_span(&mut node.span, offset);
                for row in &mut node.children {
                    Self::offset_span(&mut row.span, offset);
                    for cell in &mut row.children {
                        Self::offset_span(&mut cell.span, offset);
                        for child in &mut cell.children {
                            Self::offset_node_spans(child, offset);
                        }
                    }
                }
            }
            Node::Text(node) => Self::offset_span(&mut node.span, offset),
            Node::Emphasis(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::Strong(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::InlineCode(node) => Self::offset_span(&mut node.span, offset),
            Node::Break(node) => Self::offset_span(&mut node.span, offset),
            Node::Link(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::Image(node) => Self::offset_span(&mut node.span, offset),
            Node::Delete(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
            Node::FootnoteReference(node) => Self::offset_span(&mut node.span, offset),
            Node::Definition(node) => Self::offset_span(&mut node.span, offset),
            Node::FootnoteDefinition(node) => {
                Self::offset_span(&mut node.span, offset);
                for child in &mut node.children {
                    Self::offset_node_spans(child, offset);
                }
            }
        }
    }

    pub(super) fn offset_list_item_spans(list_item: &mut ListItem<'a>, offset: u32) {
        Self::offset_span(&mut list_item.span, offset);
        for child in &mut list_item.children {
            Self::offset_node_spans(child, offset);
        }
    }
}
