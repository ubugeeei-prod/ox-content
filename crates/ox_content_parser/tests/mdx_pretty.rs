//! Pretty-printer contracts for MDX nodes.
//!
//! These tests construct AST nodes directly so the printer stays pinned
//! before the parser emits MDX syntax.

use ox_content_allocator::Allocator;
use ox_content_ast::{
    Document, MdxFlowExpression, MdxJsxAttribute, MdxJsxAttributeEntry, MdxJsxAttributeValue,
    MdxJsxAttributeValueExpression, MdxJsxExpressionAttribute, MdxJsxFlowElement,
    MdxJsxTextElement, MdxTextExpression, MdxjsEsm, Node, Span, Text,
};

#[path = "support/pretty.rs"]
mod pretty;

fn format_nodes<'a>(source: &str, nodes: ox_content_allocator::Vec<'a, Node<'a>>) -> String {
    let span = Span::new(0, source.len() as u32);
    let doc = Document { children: nodes, span };
    let mut out = String::new();
    pretty::format_document(&doc, source, &mut out);
    out
}

#[test]
fn pretty_prints_self_closing_flow_element_with_literal_attr() {
    let source = "<Alert title=\"hi\" />";
    let allocator = Allocator::new();
    let mut attributes = allocator.new_vec();
    attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
        name: "title",
        value: Some(MdxJsxAttributeValue::Literal("hi")),
        span: Span::new(7, 17),
    }));
    let mut children = allocator.new_vec();
    children.push(Node::MdxJsxFlowElement(allocator.boxed(MdxJsxFlowElement {
        name: Some("Alert"),
        attributes,
        children: allocator.new_vec(),
        self_closing: true,
        span: Span::new(0, 20),
    })));
    assert_eq!(
        format_nodes(source, children),
        "Document [0..20]\n  MdxJsxFlowElement name=Some(\"Alert\") self_closing=true [0..20]\n    Attr name=\"title\" value=literal(\"hi\") [7..17]\n"
    );
}

#[test]
fn pretty_prints_boolean_and_expression_and_spread_attrs() {
    let source = "<Btn disabled title={t} {...rest} />";
    let allocator = Allocator::new();
    let mut attributes = allocator.new_vec();
    attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
        name: "disabled",
        value: None,
        span: Span::new(5, 13),
    }));
    attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
        name: "title",
        value: Some(MdxJsxAttributeValue::Expression(MdxJsxAttributeValueExpression {
            value: "t",
            span: Span::new(20, 23),
        })),
        span: Span::new(14, 23),
    }));
    attributes.push(MdxJsxAttributeEntry::Expression(MdxJsxExpressionAttribute {
        value: "...rest",
        span: Span::new(24, 33),
    }));
    let mut children = allocator.new_vec();
    children.push(Node::MdxJsxFlowElement(allocator.boxed(MdxJsxFlowElement {
        name: Some("Btn"),
        attributes,
        children: allocator.new_vec(),
        self_closing: true,
        span: Span::new(0, 36),
    })));
    assert_eq!(
        format_nodes(source, children),
        "Document [0..36]\n  MdxJsxFlowElement name=Some(\"Btn\") self_closing=true [0..36]\n    Attr name=\"disabled\" value=boolean [5..13]\n    Attr name=\"title\" value=expression(\"t\") [14..23]\n    AttrExpr value=\"...rest\" [24..33]\n"
    );
}

#[test]
fn pretty_prints_fragment_and_text_element_children() {
    let source = "<>A<Badge>x</Badge></>";
    let allocator = Allocator::new();
    let mut badge_children = allocator.new_vec();
    badge_children.push(Node::Text(Text { value: "x", span: Span::new(10, 11) }));
    let mut fragment_children = allocator.new_vec();
    fragment_children.push(Node::Text(Text { value: "A", span: Span::new(2, 3) }));
    fragment_children.push(Node::MdxJsxTextElement(allocator.boxed(MdxJsxTextElement {
        name: Some("Badge"),
        attributes: allocator.new_vec(),
        children: badge_children,
        self_closing: false,
        span: Span::new(3, 19),
    })));
    let mut children = allocator.new_vec();
    children.push(Node::MdxJsxFlowElement(allocator.boxed(MdxJsxFlowElement {
        name: None,
        attributes: allocator.new_vec(),
        children: fragment_children,
        self_closing: false,
        span: Span::new(0, 22),
    })));
    assert_eq!(
        format_nodes(source, children),
        "Document [0..22]\n  MdxJsxFlowElement name=None self_closing=false [0..22]\n    Text \"A\" [2..3]\n    MdxJsxTextElement name=Some(\"Badge\") self_closing=false [3..19]\n      Text \"x\" [10..11]\n"
    );
}

#[test]
fn pretty_prints_esm_and_expressions() {
    let source = "import X from './x'\n{1 + 1}\nHello {name}";
    let allocator = Allocator::new();
    let mut children = allocator.new_vec();
    children
        .push(Node::MdxjsEsm(MdxjsEsm { value: "import X from './x'", span: Span::new(0, 19) }));
    children.push(Node::MdxFlowExpression(MdxFlowExpression {
        value: "1 + 1",
        span: Span::new(20, 27),
    }));
    children.push(Node::MdxTextExpression(MdxTextExpression {
        value: "name",
        span: Span::new(34, 40),
    }));
    assert_eq!(
        format_nodes(source, children),
        "Document [0..40]\n  MdxjsEsm value=\"import X from './x'\" [0..19]\n  MdxFlowExpression value=\"1 + 1\" [20..27]\n  MdxTextExpression value=\"name\" [34..40]\n"
    );
}
