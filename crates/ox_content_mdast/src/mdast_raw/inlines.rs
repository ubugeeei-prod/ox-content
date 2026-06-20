use ox_content_ast::{
    Break, Definition, Delete, Emphasis, FootnoteDefinition, FootnoteReference, Image, InlineCode,
    Link, Strong, Text,
};

use super::format::{
    RawNodeRecord, KIND_BREAK, KIND_DEFINITION, KIND_DELETE, KIND_EMPHASIS,
    KIND_FOOTNOTE_DEFINITION, KIND_FOOTNOTE_REFERENCE, KIND_IMAGE, KIND_INLINE_CODE, KIND_LINK,
    KIND_STRONG, KIND_TEXT,
};
use super::serializer::MdastRawSerializer;

impl MdastRawSerializer {
    pub(super) fn write_text(&mut self, node: &Text<'_>) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_TEXT, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.value))?;
        self.push_record(record)
    }

    pub(super) fn write_emphasis(&mut self, node: &Emphasis<'_>) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_EMPHASIS, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        self.push_record(record)
    }

    pub(super) fn write_strong(&mut self, node: &Strong<'_>) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_STRONG, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        self.push_record(record)
    }

    pub(super) fn write_inline_code(
        &mut self,
        node: &InlineCode<'_>,
    ) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_INLINE_CODE, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.value))?;
        self.push_record(record)
    }

    pub(super) fn write_break(&mut self, node: &Break) -> crate::transfer::Result<u32> {
        self.push_record(RawNodeRecord::new(KIND_BREAK, node.span))
    }

    pub(super) fn write_link(&mut self, node: &Link<'_>) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_LINK, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        self.write_string_into_slot(&mut record, 0, Some(node.url))?;
        self.write_string_into_slot(&mut record, 1, node.title)?;
        self.push_record(record)
    }

    pub(super) fn write_image(&mut self, node: &Image<'_>) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_IMAGE, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.url))?;
        self.write_string_into_slot(&mut record, 1, Some(node.alt))?;
        self.write_string_into_slot(&mut record, 2, node.title)?;
        self.push_record(record)
    }

    pub(super) fn write_delete(&mut self, node: &Delete<'_>) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_DELETE, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        self.push_record(record)
    }

    pub(super) fn write_footnote_reference(
        &mut self,
        node: &FootnoteReference<'_>,
    ) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_FOOTNOTE_REFERENCE, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.identifier))?;
        self.write_string_into_slot(&mut record, 1, node.label)?;
        self.push_record(record)
    }

    pub(super) fn write_definition(
        &mut self,
        node: &Definition<'_>,
    ) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_DEFINITION, node.span);
        self.write_string_into_slot(&mut record, 0, Some(node.identifier))?;
        self.write_string_into_slot(&mut record, 1, node.label)?;
        self.write_string_into_slot(&mut record, 2, Some(node.url))?;
        self.write_string_into_slot(&mut record, 3, node.title)?;
        self.push_record(record)
    }

    pub(super) fn write_footnote_definition(
        &mut self,
        node: &FootnoteDefinition<'_>,
    ) -> crate::transfer::Result<u32> {
        let mut record = RawNodeRecord::new(KIND_FOOTNOTE_DEFINITION, node.span);
        self.write_child_nodes(&mut record, &node.children)?;
        self.write_string_into_slot(&mut record, 0, Some(node.identifier))?;
        self.write_string_into_slot(&mut record, 1, node.label)?;
        self.push_record(record)
    }
}
