use std::mem::size_of;

use napi::bindgen_prelude::Uint8Array;

use ox_content_allocator::Vec as ArenaVec;
use ox_content_ast::{
    AlignKind, BlockQuote, Break, CodeBlock, Definition, Delete, Document, Emphasis,
    FootnoteDefinition, FootnoteReference, Heading, Html, Image, InlineCode, Link, List, ListItem,
    Node, Paragraph, Span, Strong, Table, TableCell, TableRow, Text, ThematicBreak,
};

use crate::transfer::{as_u32, TransferBufferBuilder, TransferPayloadKind};

const MDAST_PAYLOAD_VERSION: u32 = 1;
const MDAST_SECTION_NODES: u32 = 1;
const MDAST_SECTION_CHILD_INDICES: u32 = 2;
const MDAST_SECTION_ALIGNS: u32 = 3;
const MDAST_SECTION_STRINGS: u32 = 4;
pub const MDAST_SECTION_CONTENT: u32 = 5;
pub const MDAST_SECTION_FRONTMATTER: u32 = 6;
pub const MDAST_SECTION_SOURCE_ORIGIN: u32 = 7;

const NODE_RECORD_LEN: usize = 60;
const NONE_U32: u32 = u32::MAX;

const FLAG_ORDERED: u8 = 1 << 0;
const FLAG_SPREAD: u8 = 1 << 1;
const FLAG_CHECKED_PRESENT: u8 = 1 << 2;
const FLAG_CHECKED_VALUE: u8 = 1 << 3;

const ALIGN_NONE: u8 = 0;
const ALIGN_LEFT: u8 = 1;
const ALIGN_CENTER: u8 = 2;
const ALIGN_RIGHT: u8 = 3;

const KIND_ROOT: u8 = 0;
const KIND_PARAGRAPH: u8 = 1;
const KIND_HEADING: u8 = 2;
const KIND_THEMATIC_BREAK: u8 = 3;
const KIND_BLOCKQUOTE: u8 = 4;
const KIND_LIST: u8 = 5;
const KIND_LIST_ITEM: u8 = 6;
const KIND_CODE: u8 = 7;
const KIND_HTML: u8 = 8;
const KIND_TABLE: u8 = 9;
const KIND_TABLE_ROW: u8 = 10;
const KIND_TABLE_CELL: u8 = 11;
const KIND_TEXT: u8 = 12;
const KIND_EMPHASIS: u8 = 13;
const KIND_STRONG: u8 = 14;
const KIND_INLINE_CODE: u8 = 15;
const KIND_BREAK: u8 = 16;
const KIND_LINK: u8 = 17;
const KIND_IMAGE: u8 = 18;
const KIND_DELETE: u8 = 19;
const KIND_FOOTNOTE_REFERENCE: u8 = 20;
const KIND_DEFINITION: u8 = 21;
const KIND_FOOTNOTE_DEFINITION: u8 = 22;

#[derive(Clone, Copy)]
struct RawNodeRecord {
    kind: u8,
    flags: u8,
    reserved: u16,
    span_start: u32,
    span_end: u32,
    child_start: u32,
    child_len: u32,
    num0: u32,
    num1: u32,
    str0_offset: u32,
    str0_len: u32,
    str1_offset: u32,
    str1_len: u32,
    str2_offset: u32,
    str2_len: u32,
    str3_offset: u32,
    str3_len: u32,
}

impl RawNodeRecord {
    fn new(kind: u8, span: Span) -> Self {
        Self {
            kind,
            flags: 0,
            reserved: 0,
            span_start: span.start,
            span_end: span.end,
            child_start: 0,
            child_len: 0,
            num0: 0,
            num1: 0,
            str0_offset: NONE_U32,
            str0_len: 0,
            str1_offset: NONE_U32,
            str1_len: 0,
            str2_offset: NONE_U32,
            str2_len: 0,
            str3_offset: NONE_U32,
            str3_len: 0,
        }
    }
}

pub fn to_mdast_raw(document: &Document<'_>) -> napi::Result<Uint8Array> {
    to_mdast_raw_with_sections(document, Vec::new())
}

pub fn to_mdast_raw_with_sections(
    document: &Document<'_>,
    extra_sections: Vec<(u32, Vec<u8>)>,
) -> napi::Result<Uint8Array> {
    let mut serializer = MdastRawSerializer::default();
    let root_index = serializer.write_document(document);
    serializer.finish(root_index, extra_sections)
}

#[derive(Default)]
struct MdastRawSerializer {
    nodes: Vec<RawNodeRecord>,
    child_indices: Vec<u32>,
    aligns: Vec<u8>,
    strings: Vec<u8>,
}

impl MdastRawSerializer {
    fn finish(
        self,
        root_index: u32,
        extra_sections: Vec<(u32, Vec<u8>)>,
    ) -> napi::Result<Uint8Array> {
        let nodes_len = self.nodes.len() * NODE_RECORD_LEN;
        let child_indices_len = self.child_indices.len() * size_of::<u32>();
        let mut nodes_buffer = Vec::with_capacity(nodes_len);
        let mut child_indices_buffer = Vec::with_capacity(child_indices_len);

        for node in &self.nodes {
            write_node_record(&mut nodes_buffer, node);
        }

        for child_index in &self.child_indices {
            push_u32(&mut child_indices_buffer, *child_index);
        }

        let mut builder = TransferBufferBuilder::new(
            TransferPayloadKind::Mdast,
            MDAST_PAYLOAD_VERSION,
            root_index,
        );
        builder.push_section(MDAST_SECTION_NODES, nodes_buffer);
        builder.push_section(MDAST_SECTION_CHILD_INDICES, child_indices_buffer);
        builder.push_section(MDAST_SECTION_ALIGNS, self.aligns);
        builder.push_section(MDAST_SECTION_STRINGS, self.strings);
        for (id, bytes) in extra_sections {
            builder.push_section(id, bytes);
        }
        builder.finish()
    }

    fn write_document(&mut self, document: &Document<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_ROOT, document.span);
        self.write_child_nodes(&mut record, &document.children);
        self.push_record(record)
    }

    fn write_node(&mut self, node: &Node<'_>) -> u32 {
        match node {
            Node::Paragraph(node) => self.write_paragraph(node),
            Node::Heading(node) => self.write_heading(node),
            Node::ThematicBreak(node) => self.write_thematic_break(node),
            Node::BlockQuote(node) => self.write_block_quote(node),
            Node::List(node) => self.write_list(node),
            Node::ListItem(node) => self.write_list_item(node),
            Node::CodeBlock(node) => self.write_code_block(node),
            Node::Html(node) => self.write_html(node),
            Node::Table(node) => self.write_table(node),
            Node::Text(node) => self.write_text(node),
            Node::Emphasis(node) => self.write_emphasis(node),
            Node::Strong(node) => self.write_strong(node),
            Node::InlineCode(node) => self.write_inline_code(node),
            Node::Break(node) => self.write_break(node),
            Node::Link(node) => self.write_link(node),
            Node::Image(node) => self.write_image(node),
            Node::Delete(node) => self.write_delete(node),
            Node::FootnoteReference(node) => self.write_footnote_reference(node),
            Node::Definition(node) => self.write_definition(node),
            Node::FootnoteDefinition(node) => self.write_footnote_definition(node),
        }
    }

    fn write_paragraph(&mut self, node: &Paragraph<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_PARAGRAPH, node.span);
        self.write_child_nodes(&mut record, &node.children);
        self.push_record(record)
    }

    fn write_heading(&mut self, node: &Heading<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_HEADING, node.span);
        self.write_child_nodes(&mut record, &node.children);
        record.num0 = u32::from(node.depth);
        self.push_record(record)
    }

    fn write_thematic_break(&mut self, node: &ThematicBreak) -> u32 {
        self.push_record(RawNodeRecord::new(KIND_THEMATIC_BREAK, node.span))
    }

    fn write_block_quote(&mut self, node: &BlockQuote<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_BLOCKQUOTE, node.span);
        self.write_child_nodes(&mut record, &node.children);
        self.push_record(record)
    }

    fn write_list(&mut self, node: &List<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_LIST, node.span);
        self.write_child_list_items(&mut record, &node.children);
        if node.ordered {
            record.flags |= FLAG_ORDERED;
        }
        if node.spread {
            record.flags |= FLAG_SPREAD;
        }
        record.num0 = node.start.unwrap_or(NONE_U32);
        self.push_record(record)
    }

    fn write_list_item(&mut self, node: &ListItem<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_LIST_ITEM, node.span);
        self.write_child_nodes(&mut record, &node.children);
        if node.spread {
            record.flags |= FLAG_SPREAD;
        }
        if let Some(checked) = node.checked {
            record.flags |= FLAG_CHECKED_PRESENT;
            if checked {
                record.flags |= FLAG_CHECKED_VALUE;
            }
        }
        self.push_record(record)
    }

    fn write_code_block(&mut self, node: &CodeBlock<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_CODE, node.span);
        self.write_string_into_slot(&mut record, 0, node.lang);
        self.write_string_into_slot(&mut record, 1, node.meta);
        self.write_string_into_slot(&mut record, 2, Some(node.value));
        self.push_record(record)
    }

    fn write_html(&mut self, node: &Html<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_HTML, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.value));
        self.push_record(record)
    }

    fn write_table(&mut self, node: &Table<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_TABLE, node.span);
        self.write_child_table_rows(&mut record, &node.children);
        let align_start = self.aligns.len();
        for align in &node.align {
            self.aligns.push(match align {
                AlignKind::None => ALIGN_NONE,
                AlignKind::Left => ALIGN_LEFT,
                AlignKind::Center => ALIGN_CENTER,
                AlignKind::Right => ALIGN_RIGHT,
            });
        }
        record.num0 = as_u32(align_start).expect("align overflow");
        record.num1 = as_u32(node.align.len()).expect("align overflow");
        self.push_record(record)
    }

    fn write_table_row(&mut self, node: &TableRow<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_TABLE_ROW, node.span);
        self.write_child_table_cells(&mut record, &node.children);
        self.push_record(record)
    }

    fn write_table_cell(&mut self, node: &TableCell<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_TABLE_CELL, node.span);
        self.write_child_nodes(&mut record, &node.children);
        self.push_record(record)
    }

    fn write_text(&mut self, node: &Text<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_TEXT, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.value));
        self.push_record(record)
    }

    fn write_emphasis(&mut self, node: &Emphasis<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_EMPHASIS, node.span);
        self.write_child_nodes(&mut record, &node.children);
        self.push_record(record)
    }

    fn write_strong(&mut self, node: &Strong<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_STRONG, node.span);
        self.write_child_nodes(&mut record, &node.children);
        self.push_record(record)
    }

    fn write_inline_code(&mut self, node: &InlineCode<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_INLINE_CODE, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.value));
        self.push_record(record)
    }

    fn write_break(&mut self, node: &Break) -> u32 {
        self.push_record(RawNodeRecord::new(KIND_BREAK, node.span))
    }

    fn write_link(&mut self, node: &Link<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_LINK, node.span);
        self.write_child_nodes(&mut record, &node.children);
        self.write_string_into_slot(&mut record, 0, Some(node.url));
        self.write_string_into_slot(&mut record, 1, node.title);
        self.push_record(record)
    }

    fn write_image(&mut self, node: &Image<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_IMAGE, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.url));
        self.write_string_into_slot(&mut record, 1, Some(node.alt));
        self.write_string_into_slot(&mut record, 2, node.title);
        self.push_record(record)
    }

    fn write_delete(&mut self, node: &Delete<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_DELETE, node.span);
        self.write_child_nodes(&mut record, &node.children);
        self.push_record(record)
    }

    fn write_footnote_reference(&mut self, node: &FootnoteReference<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_FOOTNOTE_REFERENCE, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.identifier));
        self.write_string_into_slot(&mut record, 1, node.label);
        self.push_record(record)
    }

    fn write_definition(&mut self, node: &Definition<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_DEFINITION, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.identifier));
        self.write_string_into_slot(&mut record, 1, node.label);
        self.write_string_into_slot(&mut record, 2, Some(node.url));
        self.write_string_into_slot(&mut record, 3, node.title);
        self.push_record(record)
    }

    fn write_footnote_definition(&mut self, node: &FootnoteDefinition<'_>) -> u32 {
        let mut record = RawNodeRecord::new(KIND_FOOTNOTE_DEFINITION, node.span);
        self.write_child_nodes(&mut record, &node.children);
        self.write_string_into_slot(&mut record, 0, Some(node.identifier));
        self.write_string_into_slot(&mut record, 1, node.label);
        self.push_record(record)
    }

    fn write_child_nodes(&mut self, record: &mut RawNodeRecord, children: &ArenaVec<'_, Node<'_>>) {
        let mut direct_children = Vec::with_capacity(children.len());
        for child in children {
            let index = self.write_node(child);
            direct_children.push(index);
        }
        let child_start = self.child_indices.len();
        self.child_indices.extend(direct_children);
        record.child_start = as_u32(child_start).expect("child overflow");
        record.child_len = as_u32(self.child_indices.len() - child_start).expect("child overflow");
    }

    fn write_child_list_items(
        &mut self,
        record: &mut RawNodeRecord,
        children: &ArenaVec<'_, ListItem<'_>>,
    ) {
        let mut direct_children = Vec::with_capacity(children.len());
        for child in children {
            let index = self.write_list_item(child);
            direct_children.push(index);
        }
        let child_start = self.child_indices.len();
        self.child_indices.extend(direct_children);
        record.child_start = as_u32(child_start).expect("child overflow");
        record.child_len = as_u32(self.child_indices.len() - child_start).expect("child overflow");
    }

    fn write_child_table_rows(
        &mut self,
        record: &mut RawNodeRecord,
        children: &ArenaVec<'_, TableRow<'_>>,
    ) {
        let mut direct_children = Vec::with_capacity(children.len());
        for child in children {
            let index = self.write_table_row(child);
            direct_children.push(index);
        }
        let child_start = self.child_indices.len();
        self.child_indices.extend(direct_children);
        record.child_start = as_u32(child_start).expect("child overflow");
        record.child_len = as_u32(self.child_indices.len() - child_start).expect("child overflow");
    }

    fn write_child_table_cells(
        &mut self,
        record: &mut RawNodeRecord,
        children: &ArenaVec<'_, TableCell<'_>>,
    ) {
        let mut direct_children = Vec::with_capacity(children.len());
        for child in children {
            let index = self.write_table_cell(child);
            direct_children.push(index);
        }
        let child_start = self.child_indices.len();
        self.child_indices.extend(direct_children);
        record.child_start = as_u32(child_start).expect("child overflow");
        record.child_len = as_u32(self.child_indices.len() - child_start).expect("child overflow");
    }

    fn write_string_into_slot(
        &mut self,
        record: &mut RawNodeRecord,
        slot: usize,
        value: Option<&str>,
    ) {
        let Some(value) = value else {
            return;
        };

        let offset = self.strings.len();
        self.strings.extend_from_slice(value.as_bytes());
        let offset = as_u32(offset).expect("string overflow");
        let len = as_u32(value.len()).expect("string overflow");

        match slot {
            0 => {
                record.str0_offset = offset;
                record.str0_len = len;
            }
            1 => {
                record.str1_offset = offset;
                record.str1_len = len;
            }
            2 => {
                record.str2_offset = offset;
                record.str2_len = len;
            }
            3 => {
                record.str3_offset = offset;
                record.str3_len = len;
            }
            _ => unreachable!(),
        }
    }

    fn push_record(&mut self, record: RawNodeRecord) -> u32 {
        let index = self.nodes.len();
        self.nodes.push(record);
        as_u32(index).expect("node overflow")
    }
}

fn write_node_record(buffer: &mut Vec<u8>, node: &RawNodeRecord) {
    buffer.push(node.kind);
    buffer.push(node.flags);
    buffer.extend_from_slice(&node.reserved.to_le_bytes());
    push_u32(buffer, node.span_start);
    push_u32(buffer, node.span_end);
    push_u32(buffer, node.child_start);
    push_u32(buffer, node.child_len);
    push_u32(buffer, node.num0);
    push_u32(buffer, node.num1);
    push_u32(buffer, node.str0_offset);
    push_u32(buffer, node.str0_len);
    push_u32(buffer, node.str1_offset);
    push_u32(buffer, node.str1_len);
    push_u32(buffer, node.str2_offset);
    push_u32(buffer, node.str2_len);
    push_u32(buffer, node.str3_offset);
    push_u32(buffer, node.str3_len);
}

fn push_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::transfer::{
        TRANSFER_HEADER_LEN, TRANSFER_MAGIC, TRANSFER_SECTION_RECORD_LEN, TRANSFER_VERSION,
    };
    use ox_content_allocator::Allocator;
    use ox_content_parser::{Parser, ParserOptions};

    fn parse_to_raw_bytes(source: &str) -> Vec<u8> {
        let allocator = Allocator::new();
        let parser = Parser::with_options(&allocator, source, ParserOptions::gfm());
        let document = parser.parse().expect("markdown should parse");
        let raw = to_mdast_raw(&document).expect("raw mdast serialization should succeed");
        raw.as_ref().to_vec()
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
        let (child_indices_offset, child_len_bytes) =
            find_section(&bytes, MDAST_SECTION_CHILD_INDICES);
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
}
