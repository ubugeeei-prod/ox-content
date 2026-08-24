//! Strict construction, span, and visitor tests for MDX AST nodes.

#![allow(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]

use ox_content_allocator::Allocator;

use crate::visit::Visit;
use crate::{
    Document, MdxFlowExpression, MdxJsxAttribute, MdxJsxAttributeEntry, MdxJsxAttributeValue,
    MdxJsxAttributeValueExpression, MdxJsxExpressionAttribute, MdxJsxFlowElement,
    MdxJsxTextElement, MdxTextExpression, MdxjsEsm, Node, Span, Text,
};

fn text<'a>(value: &'a str, span: Span) -> Node<'a> {
    Node::Text(Text { value, span })
}

#[derive(Default)]
struct KindVisitor {
    kinds: Vec<&'static str>,
    texts: Vec<String>,
}

impl<'a> Visit<'a> for KindVisitor {
    fn visit_text(&mut self, node: &Text<'a>) {
        self.kinds.push("text");
        self.texts.push(node.value.to_string());
    }

    fn visit_mdx_jsx_flow_element(&mut self, node: &MdxJsxFlowElement<'a>) {
        self.kinds.push("mdxJsxFlowElement");
        crate::visit::walk_mdx_jsx_flow_element(self, node);
    }

    fn visit_mdx_jsx_text_element(&mut self, node: &MdxJsxTextElement<'a>) {
        self.kinds.push("mdxJsxTextElement");
        crate::visit::walk_mdx_jsx_text_element(self, node);
    }

    fn visit_mdxjs_esm(&mut self, _node: &MdxjsEsm<'a>) {
        self.kinds.push("mdxjsEsm");
    }

    fn visit_mdx_flow_expression(&mut self, _node: &MdxFlowExpression<'a>) {
        self.kinds.push("mdxFlowExpression");
    }

    fn visit_mdx_text_expression(&mut self, _node: &MdxTextExpression<'a>) {
        self.kinds.push("mdxTextExpression");
    }
}

fn visit_kinds(node: &Node<'_>) -> (Vec<&'static str>, Vec<String>) {
    let mut visitor = KindVisitor::default();
    visitor.visit_node(node);
    (visitor.kinds, visitor.texts)
}

#[test]
fn flow_element_exposes_mdast_fields() {
    let allocator = Allocator::new();
    let mut attributes = allocator.new_vec();
    attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
        name: "title",
        value: Some(MdxJsxAttributeValue::Literal("hi")),
        span: Span::new(7, 17),
    }));
    let node = Node::MdxJsxFlowElement(allocator.boxed(MdxJsxFlowElement {
        name: Some("Alert"),
        attributes,
        children: allocator.new_vec(),
        self_closing: true,
        span: Span::new(0, 20),
    }));
    let Node::MdxJsxFlowElement(element) = &node else {
        panic!("expected flow element, got {node:?}");
    };
    assert_eq!(element.name, Some("Alert"));
    assert!(element.self_closing);
    assert_eq!(element.attributes.len(), 1);
    assert!(element.children.is_empty());
    assert_eq!(node.span(), Span::new(0, 20));
}

#[test]
fn text_element_exposes_mdast_fields() {
    let allocator = Allocator::new();
    let mut children = allocator.new_vec();
    children.push(text("ok", Span::new(8, 10)));
    let node = Node::MdxJsxTextElement(allocator.boxed(MdxJsxTextElement {
        name: Some("Badge"),
        attributes: allocator.new_vec(),
        children,
        self_closing: false,
        span: Span::new(0, 18),
    }));
    let Node::MdxJsxTextElement(element) = &node else {
        panic!("expected text element, got {node:?}");
    };
    assert_eq!(element.name, Some("Badge"));
    assert!(!element.self_closing);
    assert_eq!(element.children.len(), 1);
    assert_eq!(node.span(), Span::new(0, 18));
}

#[test]
fn fragment_name_is_none_for_flow_and_text() {
    let allocator = Allocator::new();
    let flow = MdxJsxFlowElement {
        name: None,
        attributes: allocator.new_vec(),
        children: allocator.new_vec(),
        self_closing: false,
        span: Span::new(0, 5),
    };
    let text_el = MdxJsxTextElement {
        name: None,
        attributes: allocator.new_vec(),
        children: allocator.new_vec(),
        self_closing: false,
        span: Span::new(0, 5),
    };
    assert_eq!(flow.name, None);
    assert_eq!(text_el.name, None);
}

#[test]
fn boolean_attribute_has_no_value() {
    let attr = MdxJsxAttribute { name: "disabled", value: None, span: Span::new(7, 15) };
    assert_eq!(attr.name, "disabled");
    assert!(attr.value.is_none());
    assert_eq!(attr.span, Span::new(7, 15));
}

#[test]
fn literal_attribute_keeps_quoted_value() {
    let attr = MdxJsxAttribute {
        name: "className",
        value: Some(MdxJsxAttributeValue::Literal("note")),
        span: Span::new(7, 24),
    };
    match attr.value {
        Some(MdxJsxAttributeValue::Literal(value)) => assert_eq!(value, "note"),
        other => panic!("expected literal, got {other:?}"),
    }
}

#[test]
fn expression_attribute_value_keeps_source_without_braces() {
    let attr = MdxJsxAttribute {
        name: "title",
        value: Some(MdxJsxAttributeValue::Expression(MdxJsxAttributeValueExpression {
            value: "props.title",
            span: Span::new(13, 26),
        })),
        span: Span::new(7, 26),
    };
    match attr.value {
        Some(MdxJsxAttributeValue::Expression(expr)) => {
            assert_eq!(expr.value, "props.title");
            assert!(!expr.value.contains('{'));
            assert_eq!(expr.span, Span::new(13, 26));
        }
        other => panic!("expected expression value, got {other:?}"),
    }
}

#[test]
fn spread_attribute_is_expression_entry() {
    let entry = MdxJsxAttributeEntry::Expression(MdxJsxExpressionAttribute {
        value: "...props",
        span: Span::new(7, 17),
    });
    match entry {
        MdxJsxAttributeEntry::Expression(expr) => {
            assert_eq!(expr.value, "...props");
            assert_eq!(expr.span, Span::new(7, 17));
        }
        MdxJsxAttributeEntry::Attribute(_) => panic!("expected spread/expression attribute"),
    }
}

#[test]
fn esm_node_keeps_source_and_span() {
    let node = Node::MdxjsEsm(MdxjsEsm { value: "import X from './x'", span: Span::new(0, 19) });
    assert_eq!(node.span(), Span::new(0, 19));
    match node {
        Node::MdxjsEsm(esm) => {
            assert!(esm.value.starts_with("import "));
            assert_eq!(esm.value, "import X from './x'");
        }
        other => panic!("expected esm, got {other:?}"),
    }
}

#[test]
fn flow_and_text_expressions_keep_source_without_braces() {
    let flow = Node::MdxFlowExpression(MdxFlowExpression { value: "1 + 1", span: Span::new(0, 7) });
    let text_expr =
        Node::MdxTextExpression(MdxTextExpression { value: "name", span: Span::new(6, 12) });
    assert_eq!(flow.span(), Span::new(0, 7));
    assert_eq!(text_expr.span(), Span::new(6, 12));
    match flow {
        Node::MdxFlowExpression(node) => {
            assert_eq!(node.value, "1 + 1");
            assert!(!node.value.contains('{'));
        }
        other => panic!("expected flow expression, got {other:?}"),
    }
    match text_expr {
        Node::MdxTextExpression(node) => assert_eq!(node.value, "name"),
        other => panic!("expected text expression, got {other:?}"),
    }
}

#[test]
fn visitor_walks_flow_element_children_not_attributes() {
    let allocator = Allocator::new();
    let mut attributes = allocator.new_vec();
    attributes.push(MdxJsxAttributeEntry::Attribute(MdxJsxAttribute {
        name: "title",
        value: Some(MdxJsxAttributeValue::Literal("ignored")),
        span: Span::new(7, 22),
    }));
    let mut children = allocator.new_vec();
    children.push(text("child", Span::new(24, 29)));
    let node = Node::MdxJsxFlowElement(allocator.boxed(MdxJsxFlowElement {
        name: Some("Alert"),
        attributes,
        children,
        self_closing: false,
        span: Span::new(0, 38),
    }));
    let (kinds, texts) = visit_kinds(&node);
    assert_eq!(kinds, ["mdxJsxFlowElement", "text"]);
    assert_eq!(texts, ["child"]);
}

#[test]
fn visitor_walks_nested_jsx_and_text_expressions() {
    let allocator = Allocator::new();
    let mut inner_children = allocator.new_vec();
    inner_children.push(Node::MdxTextExpression(MdxTextExpression {
        value: "label",
        span: Span::new(14, 21),
    }));
    let mut inner_attrs = allocator.new_vec();
    inner_attrs.push(MdxJsxAttributeEntry::Expression(MdxJsxExpressionAttribute {
        value: "...rest",
        span: Span::new(8, 17),
    }));
    let mut outer_children = allocator.new_vec();
    outer_children.push(Node::MdxJsxTextElement(allocator.boxed(MdxJsxTextElement {
        name: Some("Badge"),
        attributes: inner_attrs,
        children: inner_children,
        self_closing: false,
        span: Span::new(7, 30),
    })));
    let node = Node::MdxJsxFlowElement(allocator.boxed(MdxJsxFlowElement {
        name: Some("Alert"),
        attributes: allocator.new_vec(),
        children: outer_children,
        self_closing: false,
        span: Span::new(0, 39),
    }));
    let (kinds, texts) = visit_kinds(&node);
    assert_eq!(kinds, ["mdxJsxFlowElement", "mdxJsxTextElement", "mdxTextExpression"]);
    assert!(texts.is_empty(), "expression nodes must not be flattened to text: {texts:?}");
}

#[test]
fn visitor_does_not_invent_children_for_esm_or_expressions() {
    let esm = Node::MdxjsEsm(MdxjsEsm { value: "export const n = 1", span: Span::new(0, 18) });
    let flow = Node::MdxFlowExpression(MdxFlowExpression { value: "n", span: Span::new(0, 3) });
    assert_eq!(visit_kinds(&esm).0, ["mdxjsEsm"]);
    assert_eq!(visit_kinds(&flow).0, ["mdxFlowExpression"]);
}

#[test]
fn document_visitor_reaches_mdx_children() {
    let allocator = Allocator::new();
    let mut children = allocator.new_vec();
    children
        .push(Node::MdxjsEsm(MdxjsEsm { value: "import X from './x'", span: Span::new(0, 19) }));
    children.push(Node::MdxJsxFlowElement(allocator.boxed(MdxJsxFlowElement {
        name: Some("X"),
        attributes: allocator.new_vec(),
        children: allocator.new_vec(),
        self_closing: true,
        span: Span::new(20, 25),
    })));
    let document = Document { children, span: Span::new(0, 25) };
    let mut visitor = KindVisitor::default();
    visitor.visit_document(&document);
    assert_eq!(visitor.kinds, ["mdxjsEsm", "mdxJsxFlowElement"]);
}

#[test]
fn every_mdx_node_debug_contains_its_mdast_type_name() {
    let allocator = Allocator::new();
    let cases = [
        (
            Node::MdxJsxFlowElement(allocator.boxed(MdxJsxFlowElement {
                name: Some("A"),
                attributes: allocator.new_vec(),
                children: allocator.new_vec(),
                self_closing: true,
                span: Span::new(0, 4),
            })),
            "MdxJsxFlowElement",
        ),
        (
            Node::MdxJsxTextElement(allocator.boxed(MdxJsxTextElement {
                name: Some("A"),
                attributes: allocator.new_vec(),
                children: allocator.new_vec(),
                self_closing: true,
                span: Span::new(0, 4),
            })),
            "MdxJsxTextElement",
        ),
        (Node::MdxjsEsm(MdxjsEsm { value: "export {}", span: Span::new(0, 9) }), "MdxjsEsm"),
        (
            Node::MdxFlowExpression(MdxFlowExpression { value: "1", span: Span::new(0, 3) }),
            "MdxFlowExpression",
        ),
        (
            Node::MdxTextExpression(MdxTextExpression { value: "1", span: Span::new(0, 3) }),
            "MdxTextExpression",
        ),
    ];
    for (node, expected) in cases {
        let debug = format!("{node:?}");
        assert!(debug.contains(expected), "Debug for {expected} was {debug}");
    }
}
