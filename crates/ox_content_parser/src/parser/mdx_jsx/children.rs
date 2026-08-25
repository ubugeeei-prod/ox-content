use memchr::memchr;
use ox_content_allocator::{Allocator, Vec};
use ox_content_ast::{Node, Span};

pub(super) struct JsxChildSource<'a> {
    pub source: &'a str,
    pub offsets: Option<Vec<'a, u32>>,
}

pub(super) fn normalize_indentation<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> JsxChildSource<'a> {
    let common_indent = common_line_indent(source);
    if common_indent == 0 {
        return JsxChildSource { source, offsets: None };
    }

    let bytes = source.as_bytes();
    let mut normalized =
        ox_content_allocator::String::with_capacity_in(source.len(), allocator.bump());
    let mut offsets: Vec<'a, u32> = allocator.new_vec();
    let mut line_start = 0usize;

    while line_start < bytes.len() {
        let (line_end, next_line) = line_bounds(bytes, line_start);
        let content_start = strip_indent_columns(bytes, line_start, line_end, common_indent);
        push_mapped_slice(source, content_start, next_line, &mut normalized, &mut offsets);
        line_start = next_line;
    }

    if offsets.is_empty() {
        offsets.push(0);
    }

    JsxChildSource { source: normalized.into_bump_str(), offsets: Some(offsets) }
}

pub(super) fn remap_node_spans<'a>(node: &mut Node<'a>, source_offset: u32, offsets: &[u32]) {
    match node {
        Node::Paragraph(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_node_spans(child, source_offset, offsets);
            }
        }
        Node::Heading(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_node_spans(child, source_offset, offsets);
            }
        }
        Node::ThematicBreak(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::BlockQuote(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_node_spans(child, source_offset, offsets);
            }
        }
        Node::List(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for item in &mut node.children {
                remap_list_item_spans(item, source_offset, offsets);
            }
        }
        Node::ListItem(node) => remap_list_item_spans(node, source_offset, offsets),
        Node::CodeBlock(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::Html(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::Table(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for row in &mut node.children {
                remap_span(&mut row.span, source_offset, offsets);
                for cell in &mut row.children {
                    remap_span(&mut cell.span, source_offset, offsets);
                    for child in &mut cell.children {
                        remap_node_spans(child, source_offset, offsets);
                    }
                }
            }
        }
        Node::Text(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::Emphasis(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_node_spans(child, source_offset, offsets);
            }
        }
        Node::Strong(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_node_spans(child, source_offset, offsets);
            }
        }
        Node::InlineCode(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::Break(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::Link(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_node_spans(child, source_offset, offsets);
            }
        }
        Node::Image(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::Delete(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_node_spans(child, source_offset, offsets);
            }
        }
        Node::FootnoteReference(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::Definition(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::FootnoteDefinition(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_node_spans(child, source_offset, offsets);
            }
        }
        Node::MdxJsxFlowElement(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for attribute in &mut node.attributes {
                remap_mdx_attribute_entry(attribute, source_offset, offsets);
            }
            for child in &mut node.children {
                remap_node_spans(child, source_offset, offsets);
            }
        }
        Node::MdxJsxTextElement(node) => {
            remap_span(&mut node.span, source_offset, offsets);
            for attribute in &mut node.attributes {
                remap_mdx_attribute_entry(attribute, source_offset, offsets);
            }
            for child in &mut node.children {
                remap_node_spans(child, source_offset, offsets);
            }
        }
        Node::MdxjsEsm(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::MdxFlowExpression(node) => remap_span(&mut node.span, source_offset, offsets),
        Node::MdxTextExpression(node) => remap_span(&mut node.span, source_offset, offsets),
    }
}

fn common_line_indent(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut line_start = 0usize;
    let mut common = None;

    while line_start < bytes.len() {
        let (line_end, next_line) = line_bounds(bytes, line_start);
        if let Some(first_non_ws) = first_non_whitespace(bytes, line_start, line_end) {
            let indent = indent_width(bytes, line_start, first_non_ws);
            common = Some(common.map_or(indent, |value: usize| value.min(indent)));
            if common == Some(0) {
                break;
            }
        }
        line_start = next_line;
    }

    common.unwrap_or(0)
}

fn line_bounds(bytes: &[u8], line_start: usize) -> (usize, usize) {
    let line_end =
        memchr(b'\n', &bytes[line_start..]).map_or(bytes.len(), |offset| line_start + offset);
    let next_line = if line_end < bytes.len() { line_end + 1 } else { line_end };
    (line_end, next_line)
}

fn first_non_whitespace(bytes: &[u8], line_start: usize, line_end: usize) -> Option<usize> {
    let mut cursor = line_start;
    while cursor < line_end && matches!(bytes[cursor], b' ' | b'\t') {
        cursor += 1;
    }
    (cursor < line_end).then_some(cursor)
}

fn indent_width(bytes: &[u8], line_start: usize, first_non_ws: usize) -> usize {
    let mut columns = 0usize;
    for &byte in &bytes[line_start..first_non_ws] {
        match byte {
            b'\t' => columns = (columns / 4 + 1) * 4,
            _ => columns += 1,
        }
    }
    columns
}

fn strip_indent_columns(bytes: &[u8], line_start: usize, line_end: usize, columns: usize) -> usize {
    let mut cursor = line_start;
    let mut column = 0usize;
    while cursor < line_end && column < columns {
        match bytes[cursor] {
            b' ' => {
                column += 1;
                cursor += 1;
            }
            b'\t' => {
                column = (column / 4 + 1) * 4;
                cursor += 1;
            }
            _ => break,
        }
    }
    cursor
}

fn push_mapped_slice(
    source: &str,
    start: usize,
    end: usize,
    normalized: &mut ox_content_allocator::String<'_>,
    offsets: &mut Vec<'_, u32>,
) {
    if start >= end {
        if offsets.is_empty() {
            offsets.push(start as u32);
        }
        return;
    }

    if offsets.is_empty() {
        offsets.push(start as u32);
    }
    normalized.push_str(&source[start..end]);

    let mut offset = start + 1;
    while offsets.len() < normalized.len() + 1 {
        offsets.push(offset.min(end) as u32);
        offset += 1;
    }
}

fn remap_span(span: &mut Span, source_offset: u32, offsets: &[u32]) {
    span.start = source_offset + boundary_offset(span.start as usize, offsets);
    span.end = source_offset + boundary_offset(span.end as usize, offsets);
}

fn boundary_offset(index: usize, offsets: &[u32]) -> u32 {
    offsets.get(index).copied().unwrap_or_else(|| offsets.last().copied().unwrap_or_default())
}

fn remap_list_item_spans<'a>(
    list_item: &mut ox_content_ast::ListItem<'a>,
    source_offset: u32,
    offsets: &[u32],
) {
    remap_span(&mut list_item.span, source_offset, offsets);
    for child in &mut list_item.children {
        remap_node_spans(child, source_offset, offsets);
    }
}

fn remap_mdx_attribute_entry(
    entry: &mut ox_content_ast::MdxJsxAttributeEntry<'_>,
    source_offset: u32,
    offsets: &[u32],
) {
    match entry {
        ox_content_ast::MdxJsxAttributeEntry::Attribute(attribute) => {
            remap_span(&mut attribute.span, source_offset, offsets);
            if let Some(ox_content_ast::MdxJsxAttributeValue::Expression(expr)) =
                &mut attribute.value
            {
                remap_span(&mut expr.span, source_offset, offsets);
            }
        }
        ox_content_ast::MdxJsxAttributeEntry::Expression(expr) => {
            remap_span(&mut expr.span, source_offset, offsets);
        }
    }
}
