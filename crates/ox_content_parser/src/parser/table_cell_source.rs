use memchr::memchr;
use ox_content_allocator::{Allocator, Vec};
use ox_content_ast::{MdxJsxAttributeEntry, MdxJsxAttributeValue, Node, Span};

pub(super) struct TableCellContent<'a> {
    pub content: &'a str,
    pub source_offsets: Option<Vec<'a, u32>>,
}

/// Removes the table-level escape from pipes before inline parsing.
///
/// GFM treats `\|` as a literal pipe even inside code spans. Normal inline
/// parsing already handles the escape in prose, but code spans preserve
/// backslashes, so the table parser must consume it first.
pub(super) fn unescape_table_pipes<'a>(
    allocator: &'a Allocator,
    content: &'a str,
) -> TableCellContent<'a> {
    let bytes = content.as_bytes();
    // Most cells contain no pipe at all. Search only actual pipe positions
    // instead of examining every byte, then reuse the first escaped match
    // when constructing the remapped source below.
    let Some(first_pipe) = escaped_pipe_scan_start(bytes) else {
        return TableCellContent { content, source_offsets: None };
    };

    let mut unescaped =
        ox_content_allocator::String::with_capacity_in(content.len(), allocator.bump());
    let mut source_offsets: Vec<'a, u32> = allocator.new_vec();
    let mut copied_through = 0;
    let mut search_start = first_pipe;
    while let Some(relative) = memchr(b'|', &bytes[search_start..]) {
        let pipe = search_start + relative;
        if is_escaped_table_pipe(bytes, pipe) {
            push_mapped_table_cell_slice(
                content,
                copied_through,
                pipe - 1,
                &mut unescaped,
                &mut source_offsets,
            );
            unescaped.push('|');
            source_offsets.push((pipe + 1) as u32);
            copied_through = pipe + 1;
        }
        search_start = pipe + 1;
    }
    push_mapped_table_cell_slice(
        content,
        copied_through,
        content.len(),
        &mut unescaped,
        &mut source_offsets,
    );
    TableCellContent { content: unescaped.into_bump_str(), source_offsets: Some(source_offsets) }
}

#[inline]
fn escaped_pipe_scan_start(bytes: &[u8]) -> Option<usize> {
    // Vector dispatch costs more than the scan for short labels. Keep the
    // scalar path there; long prose cells benefit from skipping whole runs.
    if bytes.len() < 64 {
        return bytes
            .iter()
            .enumerate()
            .any(|(index, &byte)| byte == b'|' && is_escaped_table_pipe(bytes, index))
            .then_some(0);
    }
    memchr::memchr_iter(b'|', bytes).find(|&index| is_escaped_table_pipe(bytes, index))
}

fn push_mapped_table_cell_slice(
    source: &str,
    start: usize,
    end: usize,
    unescaped: &mut ox_content_allocator::String<'_>,
    source_offsets: &mut Vec<'_, u32>,
) {
    if start >= end {
        if source_offsets.is_empty() {
            source_offsets.push(start as u32);
        }
        return;
    }

    if source_offsets.is_empty() {
        source_offsets.push(start as u32);
    }
    unescaped.push_str(&source[start..end]);

    let mut source_offset = start + 1;
    while source_offsets.len() < unescaped.len() + 1 {
        source_offsets.push(source_offset.min(end) as u32);
        source_offset += 1;
    }
}

pub(super) fn remap_table_cell_inline_spans(
    node: &mut Node<'_>,
    source_offset: u32,
    offsets: &[u32],
) {
    match node {
        Node::Text(node) => remap_table_cell_span(&mut node.span, source_offset, offsets),
        Node::Emphasis(node) => {
            remap_table_cell_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_table_cell_inline_spans(child, source_offset, offsets);
            }
        }
        Node::Strong(node) => {
            remap_table_cell_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_table_cell_inline_spans(child, source_offset, offsets);
            }
        }
        Node::InlineCode(node) => remap_table_cell_span(&mut node.span, source_offset, offsets),
        Node::InlineMath(node) => remap_table_cell_span(&mut node.span, source_offset, offsets),
        Node::Break(node) => remap_table_cell_span(&mut node.span, source_offset, offsets),
        Node::Link(node) => {
            remap_table_cell_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_table_cell_inline_spans(child, source_offset, offsets);
            }
        }
        Node::Image(node) => remap_table_cell_span(&mut node.span, source_offset, offsets),
        Node::Delete(node) => {
            remap_table_cell_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_table_cell_inline_spans(child, source_offset, offsets);
            }
        }
        Node::Superscript(node) => {
            remap_table_cell_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_table_cell_inline_spans(child, source_offset, offsets);
            }
        }
        Node::Subscript(node) => {
            remap_table_cell_span(&mut node.span, source_offset, offsets);
            for child in &mut node.children {
                remap_table_cell_inline_spans(child, source_offset, offsets);
            }
        }
        Node::FootnoteReference(node) => {
            remap_table_cell_span(&mut node.span, source_offset, offsets);
        }
        Node::Html(node) => remap_table_cell_span(&mut node.span, source_offset, offsets),
        Node::MdxJsxTextElement(node) => {
            remap_table_cell_span(&mut node.span, source_offset, offsets);
            for attribute in &mut node.attributes {
                remap_table_cell_mdx_attribute_entry(attribute, source_offset, offsets);
            }
            for child in &mut node.children {
                remap_table_cell_inline_spans(child, source_offset, offsets);
            }
        }
        Node::MdxTextExpression(node) => {
            remap_table_cell_span(&mut node.span, source_offset, offsets);
        }
        Node::Paragraph(_)
        | Node::Heading(_)
        | Node::ThematicBreak(_)
        | Node::BlockQuote(_)
        | Node::List(_)
        | Node::ListItem(_)
        | Node::CodeBlock(_)
        | Node::MathBlock(_)
        | Node::Table(_)
        | Node::DefinitionList(_)
        | Node::DefinitionListTerm(_)
        | Node::DefinitionListDefinition(_)
        | Node::Definition(_)
        | Node::FootnoteDefinition(_)
        | Node::MdxJsxFlowElement(_)
        | Node::MdxjsEsm(_)
        | Node::MdxFlowExpression(_) => {}
    }
}

fn remap_table_cell_mdx_attribute_entry(
    entry: &mut MdxJsxAttributeEntry<'_>,
    source_offset: u32,
    offsets: &[u32],
) {
    match entry {
        MdxJsxAttributeEntry::Attribute(attribute) => {
            remap_table_cell_span(&mut attribute.span, source_offset, offsets);
            if let Some(MdxJsxAttributeValue::Expression(expr)) = &mut attribute.value {
                remap_table_cell_span(&mut expr.span, source_offset, offsets);
            }
        }
        MdxJsxAttributeEntry::Expression(expr) => {
            remap_table_cell_span(&mut expr.span, source_offset, offsets);
        }
    }
}

fn remap_table_cell_span(span: &mut Span, source_offset: u32, offsets: &[u32]) {
    span.start = source_offset + table_cell_boundary_offset(span.start as usize, offsets);
    span.end = source_offset + table_cell_boundary_offset(span.end as usize, offsets);
}

fn table_cell_boundary_offset(index: usize, offsets: &[u32]) -> u32 {
    offsets.get(index).copied().unwrap_or_else(|| offsets.last().copied().unwrap_or_default())
}

pub(super) fn is_escaped_table_pipe(bytes: &[u8], pipe: usize) -> bool {
    bytes[..pipe].iter().rev().take_while(|&&byte| byte == b'\\').count() % 2 == 1
}

#[cfg(test)]
mod tests;
