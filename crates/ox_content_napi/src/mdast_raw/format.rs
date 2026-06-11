use ox_content_ast::Span;

pub(super) const MDAST_PAYLOAD_VERSION: u32 = 1;
pub(super) const MDAST_SECTION_NODES: u32 = 1;
pub(super) const MDAST_SECTION_CHILD_INDICES: u32 = 2;
pub(super) const MDAST_SECTION_ALIGNS: u32 = 3;
pub(super) const MDAST_SECTION_STRINGS: u32 = 4;
pub const MDAST_SECTION_CONTENT: u32 = 5;
pub const MDAST_SECTION_FRONTMATTER: u32 = 6;
pub const MDAST_SECTION_SOURCE_ORIGIN: u32 = 7;

pub(super) const NODE_RECORD_LEN: usize = 60;
pub(super) const NONE_U32: u32 = u32::MAX;

pub(super) const FLAG_ORDERED: u8 = 1 << 0;
pub(super) const FLAG_SPREAD: u8 = 1 << 1;
pub(super) const FLAG_CHECKED_PRESENT: u8 = 1 << 2;
pub(super) const FLAG_CHECKED_VALUE: u8 = 1 << 3;

pub(super) const ALIGN_NONE: u8 = 0;
pub(super) const ALIGN_LEFT: u8 = 1;
pub(super) const ALIGN_CENTER: u8 = 2;
pub(super) const ALIGN_RIGHT: u8 = 3;

pub(super) const KIND_ROOT: u8 = 0;
pub(super) const KIND_PARAGRAPH: u8 = 1;
pub(super) const KIND_HEADING: u8 = 2;
pub(super) const KIND_THEMATIC_BREAK: u8 = 3;
pub(super) const KIND_BLOCKQUOTE: u8 = 4;
pub(super) const KIND_LIST: u8 = 5;
pub(super) const KIND_LIST_ITEM: u8 = 6;
pub(super) const KIND_CODE: u8 = 7;
pub(super) const KIND_HTML: u8 = 8;
pub(super) const KIND_TABLE: u8 = 9;
pub(super) const KIND_TABLE_ROW: u8 = 10;
pub(super) const KIND_TABLE_CELL: u8 = 11;
pub(super) const KIND_TEXT: u8 = 12;
pub(super) const KIND_EMPHASIS: u8 = 13;
pub(super) const KIND_STRONG: u8 = 14;
pub(super) const KIND_INLINE_CODE: u8 = 15;
pub(super) const KIND_BREAK: u8 = 16;
pub(super) const KIND_LINK: u8 = 17;
pub(super) const KIND_IMAGE: u8 = 18;
pub(super) const KIND_DELETE: u8 = 19;
pub(super) const KIND_FOOTNOTE_REFERENCE: u8 = 20;
pub(super) const KIND_DEFINITION: u8 = 21;
pub(super) const KIND_FOOTNOTE_DEFINITION: u8 = 22;

#[derive(Clone, Copy)]
pub(super) struct RawNodeRecord {
    pub(super) kind: u8,
    pub(super) flags: u8,
    reserved: u16,
    pub(super) span_start: u32,
    pub(super) span_end: u32,
    pub(super) child_start: u32,
    pub(super) child_len: u32,
    pub(super) num0: u32,
    pub(super) num1: u32,
    pub(super) str0_offset: u32,
    pub(super) str0_len: u32,
    pub(super) str1_offset: u32,
    pub(super) str1_len: u32,
    pub(super) str2_offset: u32,
    pub(super) str2_len: u32,
    pub(super) str3_offset: u32,
    pub(super) str3_len: u32,
}

impl RawNodeRecord {
    pub(super) fn new(kind: u8, span: Span) -> Self {
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

pub(super) fn write_node_record(buffer: &mut Vec<u8>, node: &RawNodeRecord) {
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

pub(super) fn push_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}
