use std::mem::size_of;

use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

use super::format::{
    FLAG_MDX_SELF_CLOSING, KIND_HEADING, KIND_MDX_ESM, KIND_MDX_FLOW_EXPRESSION, KIND_MDX_JSX_FLOW,
    KIND_MDX_JSX_TEXT, KIND_MDX_TEXT_EXPRESSION, KIND_PARAGRAPH, KIND_ROOT, KIND_TEXT,
    MDAST_PAYLOAD_VERSION, MDAST_SECTION_ALIGNS, MDAST_SECTION_CHILD_INDICES, MDAST_SECTION_NODES,
    MDAST_SECTION_STRINGS, NODE_RECORD_LEN, NONE_U32,
};
use super::to_mdast_raw;
use crate::transfer::{
    TRANSFER_HEADER_LEN, TRANSFER_MAGIC, TRANSFER_SECTION_RECORD_LEN, TRANSFER_VERSION,
    TransferPayloadKind,
};

fn parse_to_raw_bytes(source: &str) -> Vec<u8> {
    parse_to_raw_bytes_with_options(source, ParserOptions::gfm())
}

fn parse_to_raw_bytes_with_options(source: &str, options: ParserOptions) -> Vec<u8> {
    let allocator = Allocator::new();
    let parser = Parser::with_options(&allocator, source, options);
    let document = parser.parse().expect("markdown should parse");
    to_mdast_raw(&document).expect("raw mdast serialization should succeed")
}

fn read_u8(bytes: &[u8], offset: usize) -> u8 {
    bytes[offset]
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("u16 slice"))
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("u32 slice"))
}

fn find_section(bytes: &[u8], id: u32) -> (usize, usize) {
    let section_count = read_u32(bytes, 12) as usize;
    for index in 0..section_count {
        let base = TRANSFER_HEADER_LEN + index * TRANSFER_SECTION_RECORD_LEN;
        if read_u32(bytes, base) == id {
            return (read_u32(bytes, base + 4) as usize, read_u32(bytes, base + 8) as usize);
        }
    }

    panic!("missing section {id}");
}

fn node_base(nodes_offset: usize, index: usize) -> usize {
    nodes_offset + index * NODE_RECORD_LEN
}

fn find_node_base(bytes: &[u8], kind: u8) -> usize {
    let (nodes_offset, nodes_len) = find_section(bytes, MDAST_SECTION_NODES);
    (0..nodes_len / NODE_RECORD_LEN)
        .map(|index| node_base(nodes_offset, index))
        .find(|base| read_u8(bytes, *base) == kind)
        .unwrap_or_else(|| panic!("missing raw mdast node kind {kind}"))
}

fn read_node_string(bytes: &[u8], base: usize, slot: usize) -> Option<&str> {
    let (strings_offset, strings_len) = find_section(bytes, MDAST_SECTION_STRINGS);
    let slot_base = base + 28 + slot * 8;
    let offset = read_u32(bytes, slot_base);
    if offset == NONE_U32 {
        return None;
    }
    let len = read_u32(bytes, slot_base + 4) as usize;
    let start = strings_offset + offset as usize;
    assert!(start + len <= strings_offset + strings_len);
    Some(std::str::from_utf8(&bytes[start..start + len]).expect("raw mdast string is UTF-8"))
}

#[test]
fn serializes_heading_and_paragraph_tree() {
    let bytes = parse_to_raw_bytes("# Hello\n\nWorld");

    assert_eq!(read_u32(&bytes, 0), TRANSFER_MAGIC);
    assert_eq!(read_u16(&bytes, 4), TRANSFER_VERSION);
    assert_eq!(read_u16(&bytes, 6), TransferPayloadKind::Mdast.as_u16());
    assert_eq!(read_u32(&bytes, 8), MDAST_PAYLOAD_VERSION);
    assert_eq!(read_u32(&bytes, 12), 4);
    assert_eq!(read_u32(&bytes, 16), 4);

    let (nodes_offset, nodes_len) = find_section(&bytes, MDAST_SECTION_NODES);
    let (child_indices_offset, child_len_bytes) = find_section(&bytes, MDAST_SECTION_CHILD_INDICES);
    let (_, aligns_len) = find_section(&bytes, MDAST_SECTION_ALIGNS);
    let (_, strings_len) = find_section(&bytes, MDAST_SECTION_STRINGS);

    assert_eq!(nodes_len, 5 * NODE_RECORD_LEN);
    assert_eq!(child_len_bytes, 4 * size_of::<u32>());
    assert_eq!(aligns_len, 0);
    assert_eq!(strings_len, "HelloWorld".len());

    assert_eq!(read_u8(&bytes, node_base(nodes_offset, 0)), KIND_TEXT);
    assert_eq!(read_u8(&bytes, node_base(nodes_offset, 1)), KIND_HEADING);
    assert_eq!(read_u8(&bytes, node_base(nodes_offset, 2)), KIND_TEXT);
    assert_eq!(read_u8(&bytes, node_base(nodes_offset, 3)), KIND_PARAGRAPH);
    assert_eq!(read_u8(&bytes, node_base(nodes_offset, 4)), KIND_ROOT);

    assert_eq!(read_u32(&bytes, node_base(nodes_offset, 1) + 12), 0);
    assert_eq!(read_u32(&bytes, node_base(nodes_offset, 1) + 16), 1);
    assert_eq!(read_u32(&bytes, node_base(nodes_offset, 3) + 12), 1);
    assert_eq!(read_u32(&bytes, node_base(nodes_offset, 3) + 16), 1);
    assert_eq!(read_u32(&bytes, node_base(nodes_offset, 4) + 12), 2);
    assert_eq!(read_u32(&bytes, node_base(nodes_offset, 4) + 16), 2);

    assert_eq!(read_u32(&bytes, child_indices_offset), 0);
    assert_eq!(read_u32(&bytes, child_indices_offset + 4), 2);
    assert_eq!(read_u32(&bytes, child_indices_offset + 8), 1);
    assert_eq!(read_u32(&bytes, child_indices_offset + 12), 3);
}

#[test]
fn preserves_utf8_spans_as_byte_offsets() {
    let bytes = parse_to_raw_bytes("# あ");

    assert_eq!(read_u32(&bytes, 16), 2);

    let (nodes_offset, nodes_len) = find_section(&bytes, MDAST_SECTION_NODES);
    assert_eq!(nodes_len, 3 * NODE_RECORD_LEN);

    let text_base = node_base(nodes_offset, 0);
    let heading_base = node_base(nodes_offset, 1);
    let root_base = node_base(nodes_offset, 2);

    assert_eq!(read_u8(&bytes, text_base), KIND_TEXT);
    assert_eq!(read_u32(&bytes, text_base + 4), 2);
    assert_eq!(read_u32(&bytes, text_base + 8), 5);

    assert_eq!(read_u8(&bytes, heading_base), KIND_HEADING);
    assert_eq!(read_u32(&bytes, heading_base + 4), 0);
    assert_eq!(read_u32(&bytes, heading_base + 8), 5);

    assert_eq!(read_u8(&bytes, root_base), KIND_ROOT);
    assert_eq!(read_u32(&bytes, root_base + 4), 0);
    assert_eq!(read_u32(&bytes, root_base + 8), 5);
}

#[test]
fn serializes_every_mdx_kind_and_preserves_jsx_attributes() {
    let source = "import Alert from './Alert'\n\n{count}\n\n<Alert title=\"Hi\" count={count} {...props} />\n\nHello <Badge /> {name}\n";
    let bytes = parse_to_raw_bytes_with_options(source, ParserOptions::mdx());

    for kind in [
        KIND_MDX_ESM,
        KIND_MDX_FLOW_EXPRESSION,
        KIND_MDX_JSX_FLOW,
        KIND_MDX_JSX_TEXT,
        KIND_MDX_TEXT_EXPRESSION,
    ] {
        find_node_base(&bytes, kind);
    }

    let flow = find_node_base(&bytes, KIND_MDX_JSX_FLOW);
    assert_ne!(read_u8(&bytes, flow + 1) & FLAG_MDX_SELF_CLOSING, 0);
    assert_eq!(read_node_string(&bytes, flow, 0), Some("Alert"));

    let attributes: serde_json::Value = serde_json::from_str(
        read_node_string(&bytes, flow, 1).expect("MDX attributes must be transferred"),
    )
    .expect("MDX attributes must be valid JSON");
    assert_eq!(attributes[0]["name"], "title");
    assert_eq!(attributes[0]["value"], "Hi");
    assert_eq!(attributes[1]["value"]["type"], "mdxJsxAttributeValueExpression");
    assert_eq!(attributes[1]["value"]["value"], "count");
    assert_eq!(attributes[2]["type"], "mdxJsxExpressionAttribute");
    assert_eq!(attributes[2]["value"], "...props");
}
