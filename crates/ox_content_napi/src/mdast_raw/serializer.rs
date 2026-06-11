use std::mem::size_of;

use napi::bindgen_prelude::Uint8Array;

use ox_content_allocator::Vec as ArenaVec;
use ox_content_ast::{Document, ListItem, Node, TableCell, TableRow};

use crate::transfer::{as_u32, TransferBufferBuilder, TransferPayloadKind};

use super::format::{
    push_u32, write_node_record, RawNodeRecord, KIND_ROOT, MDAST_PAYLOAD_VERSION,
    MDAST_SECTION_ALIGNS, MDAST_SECTION_CHILD_INDICES, MDAST_SECTION_NODES, MDAST_SECTION_STRINGS,
    NODE_RECORD_LEN,
};

#[derive(Default)]
pub(super) struct MdastRawSerializer {
    pub(super) nodes: Vec<RawNodeRecord>,
    pub(super) child_indices: Vec<u32>,
    pub(super) aligns: Vec<u8>,
    pub(super) strings: Vec<u8>,
}

impl MdastRawSerializer {
    pub(super) fn finish(
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

    pub(super) fn write_document(&mut self, document: &Document<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_ROOT, document.span);
        self.write_child_nodes(&mut record, &document.children)?;
        self.push_record(record)
    }

    fn write_node(&mut self, node: &Node<'_>) -> napi::Result<u32> {
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

    pub(super) fn write_child_nodes(
        &mut self,
        record: &mut RawNodeRecord,
        children: &ArenaVec<'_, Node<'_>>,
    ) -> napi::Result<()> {
        let mut direct_children = Vec::with_capacity(children.len());
        for child in children {
            let index = self.write_node(child)?;
            direct_children.push(index);
        }
        self.write_child_index_range(record, direct_children)
    }

    pub(super) fn write_child_list_items(
        &mut self,
        record: &mut RawNodeRecord,
        children: &ArenaVec<'_, ListItem<'_>>,
    ) -> napi::Result<()> {
        let mut direct_children = Vec::with_capacity(children.len());
        for child in children {
            let index = self.write_list_item(child)?;
            direct_children.push(index);
        }
        self.write_child_index_range(record, direct_children)
    }

    pub(super) fn write_child_table_rows(
        &mut self,
        record: &mut RawNodeRecord,
        children: &ArenaVec<'_, TableRow<'_>>,
    ) -> napi::Result<()> {
        let mut direct_children = Vec::with_capacity(children.len());
        for child in children {
            let index = self.write_table_row(child)?;
            direct_children.push(index);
        }
        self.write_child_index_range(record, direct_children)
    }

    pub(super) fn write_child_table_cells(
        &mut self,
        record: &mut RawNodeRecord,
        children: &ArenaVec<'_, TableCell<'_>>,
    ) -> napi::Result<()> {
        let mut direct_children = Vec::with_capacity(children.len());
        for child in children {
            let index = self.write_table_cell(child)?;
            direct_children.push(index);
        }
        self.write_child_index_range(record, direct_children)
    }

    fn write_child_index_range(
        &mut self,
        record: &mut RawNodeRecord,
        direct_children: Vec<u32>,
    ) -> napi::Result<()> {
        let child_start = self.child_indices.len();
        self.child_indices.extend(direct_children);
        record.child_start = as_u32(child_start)?;
        record.child_len = as_u32(self.child_indices.len() - child_start)?;
        Ok(())
    }

    pub(super) fn write_string_into_slot(
        &mut self,
        record: &mut RawNodeRecord,
        slot: usize,
        value: Option<&str>,
    ) -> napi::Result<()> {
        let Some(value) = value else {
            return Ok(());
        };

        let offset = self.strings.len();
        self.strings.extend_from_slice(value.as_bytes());
        let offset = as_u32(offset)?;
        let len = as_u32(value.len())?;

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
            _ => {
                let mut message = String::from("invalid raw mdast string slot: ");
                message.push_str(&slot.to_string());
                return Err(napi::Error::from_reason(message));
            }
        }
        Ok(())
    }

    pub(super) fn push_record(&mut self, record: RawNodeRecord) -> napi::Result<u32> {
        let index = self.nodes.len();
        let index = as_u32(index)?;
        self.nodes.push(record);
        Ok(index)
    }
}
