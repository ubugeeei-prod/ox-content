use ox_content_ast::{
    AlignKind, BlockQuote, CodeBlock, Heading, Html, List, ListItem, Paragraph, Table, TableCell,
    TableRow, ThematicBreak,
};

use crate::transfer::as_u32;

use super::format::{
    RawNodeRecord, ALIGN_CENTER, ALIGN_LEFT, ALIGN_NONE, ALIGN_RIGHT, FLAG_CHECKED_PRESENT,
    FLAG_CHECKED_VALUE, FLAG_ORDERED, FLAG_SPREAD, KIND_BLOCKQUOTE, KIND_CODE, KIND_HEADING,
    KIND_HTML, KIND_LIST, KIND_LIST_ITEM, KIND_PARAGRAPH, KIND_TABLE, KIND_TABLE_CELL,
    KIND_TABLE_ROW, KIND_THEMATIC_BREAK, NONE_U32,
};
use super::serializer::MdastRawSerializer;

impl MdastRawSerializer {
    pub(super) fn write_paragraph(&mut self, node: &Paragraph<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_PARAGRAPH, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        self.push_record(record)
    }

    pub(super) fn write_heading(&mut self, node: &Heading<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_HEADING, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        record.num0 = u32::from(node.depth);
        self.push_record(record)
    }

    pub(super) fn write_thematic_break(&mut self, node: &ThematicBreak) -> napi::Result<u32> {
        self.push_record(RawNodeRecord::new(KIND_THEMATIC_BREAK, node.span))
    }

    pub(super) fn write_block_quote(&mut self, node: &BlockQuote<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_BLOCKQUOTE, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        self.push_record(record)
    }

    pub(super) fn write_list(&mut self, node: &List<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_LIST, node.span);
        self.write_child_list_items(&mut record, &node.children)?;
        if node.ordered {
            record.flags |= FLAG_ORDERED;
        }
        if node.spread {
            record.flags |= FLAG_SPREAD;
        }
        record.num0 = node.start.unwrap_or(NONE_U32);
        self.push_record(record)
    }

    pub(super) fn write_list_item(&mut self, node: &ListItem<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_LIST_ITEM, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
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

    pub(super) fn write_code_block(&mut self, node: &CodeBlock<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_CODE, node.span);
        self.write_string_into_slot(&mut record, 0, node.lang)?;
        self.write_string_into_slot(&mut record, 1, node.meta)?;
        self.write_string_into_slot(&mut record, 2, Some(node.value))?;
        self.push_record(record)
    }

    pub(super) fn write_html(&mut self, node: &Html<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_HTML, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.value))?;
        self.push_record(record)
    }

    pub(super) fn write_table(&mut self, node: &Table<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_TABLE, node.span);
        self.write_child_table_rows(&mut record, &node.children)?;
        let align_start = self.aligns.len();
        for align in &node.align {
            self.aligns.push(match align {
                AlignKind::None => ALIGN_NONE,
                AlignKind::Left => ALIGN_LEFT,
                AlignKind::Center => ALIGN_CENTER,
                AlignKind::Right => ALIGN_RIGHT,
            });
        }
        record.num0 = as_u32(align_start)?;
        record.num1 = as_u32(node.align.len())?;
        self.push_record(record)
    }

    pub(super) fn write_table_row(&mut self, node: &TableRow<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_TABLE_ROW, node.span);
        self.write_child_table_cells(&mut record, &node.children)?;
        self.push_record(record)
    }

    pub(super) fn write_table_cell(&mut self, node: &TableCell<'_>) -> napi::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_TABLE_CELL, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        self.push_record(record)
    }
}
