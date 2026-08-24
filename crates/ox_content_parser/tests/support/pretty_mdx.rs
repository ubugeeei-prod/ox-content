use ox_content_ast::{
    MdxFlowExpression, MdxJsxAttributeEntry, MdxJsxAttributeValue, MdxJsxFlowElement,
    MdxJsxTextElement, MdxTextExpression, MdxjsEsm, Node,
};

use super::{format_node, line, span};

pub(super) fn format_jsx_flow_element(
    node: &MdxJsxFlowElement<'_>,
    source: &str,
    depth: usize,
    out: &mut String,
) {
    format_mdx_jsx_element(
        format_args!(
            "MdxJsxFlowElement name={:?} self_closing={} {}",
            node.name,
            node.self_closing,
            span(node.span, source)
        ),
        &node.attributes,
        &node.children,
        source,
        depth,
        out,
    );
}

pub(super) fn format_jsx_text_element(
    node: &MdxJsxTextElement<'_>,
    source: &str,
    depth: usize,
    out: &mut String,
) {
    format_mdx_jsx_element(
        format_args!(
            "MdxJsxTextElement name={:?} self_closing={} {}",
            node.name,
            node.self_closing,
            span(node.span, source)
        ),
        &node.attributes,
        &node.children,
        source,
        depth,
        out,
    );
}

pub(super) fn format_esm(node: &MdxjsEsm<'_>, source: &str, depth: usize, out: &mut String) {
    line(out, depth, format_args!("MdxjsEsm value={:?} {}", node.value, span(node.span, source)));
}

pub(super) fn format_flow_expression(
    node: &MdxFlowExpression<'_>,
    source: &str,
    depth: usize,
    out: &mut String,
) {
    line(
        out,
        depth,
        format_args!("MdxFlowExpression value={:?} {}", node.value, span(node.span, source)),
    );
}

pub(super) fn format_text_expression(
    node: &MdxTextExpression<'_>,
    source: &str,
    depth: usize,
    out: &mut String,
) {
    line(
        out,
        depth,
        format_args!("MdxTextExpression value={:?} {}", node.value, span(node.span, source)),
    );
}

fn format_mdx_jsx_element(
    header: std::fmt::Arguments<'_>,
    attributes: &[MdxJsxAttributeEntry<'_>],
    children: &[Node<'_>],
    source: &str,
    depth: usize,
    out: &mut String,
) {
    line(out, depth, header);
    for attribute in attributes {
        format_mdx_attribute(attribute, source, depth + 1, out);
    }
    for child in children {
        format_node(child, source, depth + 1, out);
    }
}

fn format_mdx_attribute(
    entry: &MdxJsxAttributeEntry<'_>,
    source: &str,
    depth: usize,
    out: &mut String,
) {
    match entry {
        MdxJsxAttributeEntry::Attribute(attribute) => {
            let value = match &attribute.value {
                None => "boolean".to_string(),
                Some(MdxJsxAttributeValue::Literal(value)) => {
                    format!("literal({value:?})")
                }
                Some(MdxJsxAttributeValue::Expression(expr)) => {
                    format!("expression({:?})", expr.value)
                }
            };
            line(
                out,
                depth,
                format_args!(
                    "Attr name={:?} value={value} {}",
                    attribute.name,
                    span(attribute.span, source)
                ),
            );
        }
        MdxJsxAttributeEntry::Expression(expr) => {
            line(
                out,
                depth,
                format_args!("AttrExpr value={:?} {}", expr.value, span(expr.span, source)),
            );
        }
    }
}
