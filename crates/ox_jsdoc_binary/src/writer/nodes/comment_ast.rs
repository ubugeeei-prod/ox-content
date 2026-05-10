// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! `write_*` helpers for the 15 comment AST kinds (`0x01 - 0x0F`).
//!
//! Convention: every helper takes the writer first, then the node's
//! `Span`, then its `parent_index` (`0` = sentinel parent = root),
//! followed by the per-Kind payload parameters. Each helper returns the
//! [`NodeIndex`] of the freshly written node so that the parser can wire
//! it as a child of its parent (via backpatched `next_sibling`).
//!
//! String reference encoding:
//!
//! - **String-leaf nodes** (`JsdocTagName`, `JsdocText`, `JsdocTypeSource`,
//!   `JsdocRawTagBody`, `JsdocNamepathSource`, `JsdocIdentifier`,
//!   `JsdocTagNameValue`, plus the basic-mode forms of
//!   `JsdocDescriptionLine` and `JsdocTypeLine`) use `TypeTag::String`
//!   with a 30-bit `StringIndex` packed into Node Data — no Extended Data
//!   record allocated.
//! - **Extended Data string slots** (e.g. `JsdocBlock.delimiter`,
//!   `JsdocTag.description`, `JsdocInlineTag.text`) use inline 6-byte
//!   `StringField` slots, bypassing the offsets-table indirection.

use oxc_span::Span;

use crate::format::extended_data::LIST_METADATA_SIZE;
use crate::format::kind::Kind;
use crate::format::string_field::{STRING_FIELD_SIZE, StringField};

use super::super::{BinaryWriter, ExtOffset, LeafStringPayload};
use super::NodeIndex;

// ---------------------------------------------------------------------------
// 0x01 JsdocBlock (Extended, root)
// ---------------------------------------------------------------------------

/// Number of NodeList slots embedded in a `JsdocBlock` Extended Data block:
/// description_lines, tags, inline_tags.
const JSDOC_BLOCK_LIST_COUNT: usize = 3;
/// Byte offset where the per-list metadata region begins inside a
/// `JsdocBlock` Extended Data block (right after the 8 inline StringFields).
pub(crate) const JSDOC_BLOCK_LISTS_BASE: usize = 1 + 1 + 8 * STRING_FIELD_SIZE;
/// `description_lines` list metadata slot offset (basic-mode ED).
pub const JSDOC_BLOCK_DESC_LINES_SLOT: usize = JSDOC_BLOCK_LISTS_BASE;
/// `tags` list metadata slot offset (basic-mode ED).
pub const JSDOC_BLOCK_TAGS_SLOT: usize = JSDOC_BLOCK_LISTS_BASE + LIST_METADATA_SIZE;
/// `inline_tags` list metadata slot offset (basic-mode ED).
pub const JSDOC_BLOCK_INLINE_TAGS_SLOT: usize = JSDOC_BLOCK_LISTS_BASE + 2 * LIST_METADATA_SIZE;

/// Basic-mode Extended Data size for [`JsdocBlock`]:
/// `1 + 1 + 8 × StringField + 3 × ListMetadata = 68` bytes.
pub(crate) const JSDOC_BLOCK_BASIC_SIZE: usize =
    JSDOC_BLOCK_LISTS_BASE + JSDOC_BLOCK_LIST_COUNT * LIST_METADATA_SIZE;
/// Compat-mode Extended Data size for [`JsdocBlock`]: basic + `22` bytes
/// of compat tail (line indices + `has_preterminal_*_description` flags).
/// `description_raw_span` is **not** part of the compat tail in Phase 5 —
/// it is opt-in per the per-node Common Data bit (see [`Self::write_jsdoc_block`]).
///
/// Inter-record 8-byte alignment is handled by the writer
/// (`crate::writer::extended_data::ExtendedDataWriter::reserve_mut` pads
/// before each new record), so this constant does not include any extra
/// trailing padding.
pub(crate) const JSDOC_BLOCK_COMPAT_SIZE: usize = JSDOC_BLOCK_BASIC_SIZE + 22;
/// Compat tail base offset (start of the compat-only region).
const JSDOC_BLOCK_COMPAT_TAIL_BASE: usize = JSDOC_BLOCK_BASIC_SIZE;

/// Size of the optional `description_raw_span` slot appended at the **end**
/// of every `JsdocBlock` / `JsdocTag` Extended Data record when the
/// `has_description_raw_span` Common Data bit is set. See
/// `design/008-oxlint-oxfmt-support/README.md` §4.2 for the wire layout
/// and §5.2 for the per-mode total ED sizes.
pub(crate) const DESCRIPTION_RAW_SPAN_SIZE: usize = 8;

/// Common Data bit signalling presence of `description_raw_span` on a
/// `JsdocBlock` Extended Data record. Bit 0 of the 6-bit Common Data
/// field. (bits 1–5 reserved.)
pub(crate) const JSDOC_BLOCK_HAS_DESCRIPTION_RAW_SPAN_BIT: u8 = 1 << 0;

/// Write a `JsdocBlock` (Kind `0x01`, Extended type).
///
/// Extended Data layout (basic 68 bytes; compat tail adds 22 bytes via
/// [`write_jsdoc_block_compat_tail`]; an optional 8-byte
/// `description_raw_span` is appended at the **very end** when
/// `description_raw_span` is `Some`):
///
/// ```text
/// byte 0      : Children bitmask (u8, retained for visitor framework)
/// byte 1      : padding (u8)
/// byte 2-7    : description     (StringField, NONE if absent)
/// byte 8-13   : delimiter
/// byte 14-19  : post_delimiter
/// byte 20-25  : terminal
/// byte 26-31  : line_end
/// byte 32-37  : initial
/// byte 38-43  : delimiter_line_break
/// byte 44-49  : preterminal_line_break
/// byte 50-55  : description_lines list metadata (head: u32, count: u16)
/// byte 56-61  : tags list metadata
/// byte 62-67  : inline_tags list metadata
/// [compat tail (bytes 68-89) — only present in compat mode]
/// [description_raw_span (last 8 bytes) — only when bit 0 of Common Data set]
/// ```
///
/// Per-mode total sizes (matrix from
/// `design/008-oxlint-oxfmt-support/README.md` §5.2):
///
/// | compat | preserve | total |
/// |--------|----------|-------|
/// | false  | false    | 68    |
/// | false  | true     | 76    |
/// | true   | false    | 90    |
/// | true   | true     | 98    |
///
/// When `description_raw_span` is `Some`, the [`JSDOC_BLOCK_HAS_DESCRIPTION_RAW_SPAN_BIT`]
/// is set in Common Data so the decoder knows to read the trailing 8 bytes.
///
/// Returns `(NodeIndex, ExtOffset)` so the caller can patch the per-list
/// metadata via [`BinaryWriter::begin_node_list_at`] /
/// [`BinaryWriter::finalize_node_list`] once all children of each list have
/// been emitted.
#[allow(clippy::too_many_arguments)]
pub fn write_jsdoc_block(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    description: Option<StringField>,
    delimiter: StringField,
    post_delimiter: StringField,
    terminal: StringField,
    line_end: StringField,
    initial: StringField,
    delimiter_line_break: StringField,
    preterminal_line_break: StringField,
    children_bitmask: u8,
    description_raw_span: Option<(u32, u32)>,
) -> (NodeIndex, ExtOffset) {
    let base_size =
        if writer.compat_mode() { JSDOC_BLOCK_COMPAT_SIZE } else { JSDOC_BLOCK_BASIC_SIZE };
    let total_size = if description_raw_span.is_some() {
        base_size + DESCRIPTION_RAW_SPAN_SIZE
    } else {
        base_size
    };
    let (off, dst) = writer.extended.reserve_mut(total_size);
    dst[0] = children_bitmask;
    dst[1] = 0;
    write_opt_string_field(dst, 2, description);
    write_string_field(dst, 8, delimiter);
    write_string_field(dst, 14, post_delimiter);
    write_string_field(dst, 20, terminal);
    write_string_field(dst, 26, line_end);
    write_string_field(dst, 32, initial);
    write_string_field(dst, 38, delimiter_line_break);
    write_string_field(dst, 44, preterminal_line_break);
    // List metadata bytes 50..68 are zeroed by reserve_mut and patched later
    // via begin_node_list_at / finalize_node_list.

    // Phase 5: optional description_raw_span at the very end of the ED.
    let common_data = if let Some((raw_start, raw_end)) = description_raw_span {
        let span_off = total_size - DESCRIPTION_RAW_SPAN_SIZE;
        dst[span_off..span_off + 4].copy_from_slice(&raw_start.to_le_bytes());
        dst[span_off + 4..span_off + 8].copy_from_slice(&raw_end.to_le_bytes());
        JSDOC_BLOCK_HAS_DESCRIPTION_RAW_SPAN_BIT
    } else {
        0
    };

    let idx = writer.emit_extended_node(parent_index, Kind::JsdocBlock, common_data, span, off);
    (idx, off)
}

/// Patch the compat-mode tail on a previously written `JsdocBlock` Extended
/// Data record. Only call this when [`BinaryWriter::compat_mode`] is `true`
/// (the basic write helper already reserved the extra bytes).
///
/// Tail layout (offsets relative to the parent's Extended Data start; the
/// region begins at byte 68 = `JSDOC_BLOCK_COMPAT_TAIL_BASE`):
/// ```text
/// byte 68-69 : padding (u16, zero-fill)
/// byte 70-73 : end_line (u32)
/// byte 74-77 : description_start_line (u32, 0xFFFF_FFFF = None)
/// byte 78-81 : description_end_line   (u32, 0xFFFF_FFFF = None)
/// byte 82-85 : last_description_line  (u32, 0xFFFF_FFFF = None)
/// byte 86    : has_preterminal_description (u8)
/// byte 87    : has_preterminal_tag_description (u8, 0xFF = None)
/// byte 88-89 : padding (u16, zero-fill)
/// ```
///
/// `description_raw_span` is **not** part of the compat tail in Phase 5 —
/// it is written by [`write_jsdoc_block`] at the very end of the ED record
/// when opted in (see `design/008-oxlint-oxfmt-support/README.md` §4.2).
#[allow(clippy::too_many_arguments)]
pub fn write_jsdoc_block_compat_tail(
    writer: &mut BinaryWriter<'_>,
    ext_offset: ExtOffset,
    end_line: u32,
    description_start_line: Option<u32>,
    description_end_line: Option<u32>,
    last_description_line: Option<u32>,
    has_preterminal_description: u8,
    has_preterminal_tag_description: Option<u8>,
) {
    debug_assert!(
        writer.compat_mode(),
        "write_jsdoc_block_compat_tail called but compat_mode is off"
    );
    let dst = writer.extended.slice_mut(ext_offset, JSDOC_BLOCK_COMPAT_SIZE);
    let base = JSDOC_BLOCK_COMPAT_TAIL_BASE;
    // bytes [base..base+2] are u32 alignment padding (already zero)
    dst[base + 2..base + 6].copy_from_slice(&end_line.to_le_bytes());
    dst[base + 6..base + 10]
        .copy_from_slice(&opt_u32_sentinel(description_start_line).to_le_bytes());
    dst[base + 10..base + 14]
        .copy_from_slice(&opt_u32_sentinel(description_end_line).to_le_bytes());
    dst[base + 14..base + 18]
        .copy_from_slice(&opt_u32_sentinel(last_description_line).to_le_bytes());
    dst[base + 18] = has_preterminal_description;
    dst[base + 19] = has_preterminal_tag_description.unwrap_or(0xFF);
    // bytes [base+20..base+22] trailing alignment padding (already zero)
}

// ---------------------------------------------------------------------------
// 0x02 JsdocDescriptionLine (String basic / Extended compat)
// ---------------------------------------------------------------------------

/// Compat-mode Extended Data size for [`JsdocDescriptionLine`]: 4 ×
/// StringField (description + 3 optional delimiter strings).
const JSDOC_DESCRIPTION_LINE_COMPAT_SIZE: usize = 4 * STRING_FIELD_SIZE;

/// Write a `JsdocDescriptionLine` (Kind `0x02`, basic mode).
///
/// Node Data: `TypeTag::String` carrying the description's StringIndex.
/// No Extended Data record is allocated.
pub fn write_jsdoc_description_line(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    description: LeafStringPayload,
) -> NodeIndex {
    writer.emit_string_node(parent_index, Kind::JsdocDescriptionLine, 0, span, description)
}

/// Write a `JsdocDescriptionLine` (Kind `0x02`, compat mode).
///
/// Extended Data layout (24 bytes):
/// - byte 0-5   : description    (StringField, required)
/// - byte 6-11  : delimiter      (StringField, NONE if absent)
/// - byte 12-17 : post_delimiter (StringField, NONE if absent)
/// - byte 18-23 : initial        (StringField, NONE if absent)
pub fn write_jsdoc_description_line_compat(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    description: StringField,
    delimiter: Option<StringField>,
    post_delimiter: Option<StringField>,
    initial: Option<StringField>,
) -> NodeIndex {
    debug_assert!(writer.compat_mode());
    let (off, dst) = writer.extended.reserve_mut(JSDOC_DESCRIPTION_LINE_COMPAT_SIZE);
    write_string_field(dst, 0, description);
    write_opt_string_field(dst, 6, delimiter);
    write_opt_string_field(dst, 12, post_delimiter);
    write_opt_string_field(dst, 18, initial);
    writer.emit_extended_node(parent_index, Kind::JsdocDescriptionLine, 0, span, off)
}

// ---------------------------------------------------------------------------
// 0x03 JsdocTag (Extended)
// ---------------------------------------------------------------------------

/// Number of NodeList slots embedded in a `JsdocTag` Extended Data block:
/// type_lines, description_lines, inline_tags.
const JSDOC_TAG_LIST_COUNT: usize = 3;
/// Byte offset where the per-list metadata region begins inside a
/// `JsdocTag` Extended Data block (right after the 3 inline StringFields).
pub(crate) const JSDOC_TAG_LISTS_BASE: usize = 1 + 1 + 3 * STRING_FIELD_SIZE;
/// `type_lines` list metadata slot offset.
pub const JSDOC_TAG_TYPE_LINES_SLOT: usize = JSDOC_TAG_LISTS_BASE;
/// `description_lines` list metadata slot offset.
pub const JSDOC_TAG_DESC_LINES_SLOT: usize = JSDOC_TAG_LISTS_BASE + LIST_METADATA_SIZE;
/// `inline_tags` list metadata slot offset.
pub const JSDOC_TAG_INLINE_TAGS_SLOT: usize = JSDOC_TAG_LISTS_BASE + 2 * LIST_METADATA_SIZE;

/// Basic-mode Extended Data size for [`JsdocTag`]:
/// `1 + 1 + 3 × StringField + 3 × ListMetadata = 38` bytes.
pub(crate) const JSDOC_TAG_BASIC_SIZE: usize =
    JSDOC_TAG_LISTS_BASE + JSDOC_TAG_LIST_COUNT * LIST_METADATA_SIZE;
/// Compat-mode Extended Data size for [`JsdocTag`]: basic + 42 bytes of
/// compat tail (7 × `StringField` for the delimiter group).
/// `description_raw_span` is **not** part of the compat tail in Phase 5 —
/// it is opt-in per the per-node Common Data bit (see [`Self::write_jsdoc_tag`]).
pub(crate) const JSDOC_TAG_COMPAT_SIZE: usize = JSDOC_TAG_BASIC_SIZE + 7 * STRING_FIELD_SIZE;
/// Compat tail base offset (start of the compat-only region).
const JSDOC_TAG_COMPAT_TAIL_BASE: usize = JSDOC_TAG_BASIC_SIZE;

/// Common Data bit signalling presence of `description_raw_span` on a
/// `JsdocTag` Extended Data record. Bit 1 of the 6-bit Common Data
/// field (bit 0 is `optional`; bits 2–5 reserved).
pub(crate) const JSDOC_TAG_HAS_DESCRIPTION_RAW_SPAN_BIT: u8 = 1 << 1;

/// Write a `JsdocTag` (Kind `0x03`, Extended type).
///
/// Common Data layout: bit 0 = `optional`, bit 1 =
/// `has_description_raw_span` (set when `description_raw_span` is `Some`).
/// bits 2–5 reserved.
///
/// Extended Data layout (basic 38 bytes; compat tail adds 42 bytes via
/// [`write_jsdoc_tag_compat_tail`]; an optional 8-byte
/// `description_raw_span` is appended at the **very end** when
/// `description_raw_span` is `Some`):
///
/// ```text
/// byte 0      : Children bitmask (u8, retained for visitor framework)
/// byte 1      : padding (u8)
/// byte 2-7    : default_value (StringField, NONE if absent)
/// byte 8-13   : description    (StringField, NONE if absent)
/// byte 14-19  : raw_body       (StringField, NONE if absent)
/// byte 20-25  : type_lines       list metadata (head: u32, count: u16)
/// byte 26-31  : description_lines list metadata
/// byte 32-37  : inline_tags      list metadata
/// [compat tail: 7 × StringField at bytes 38..=79 — only in compat mode]
/// [description_raw_span (last 8 bytes) — only when bit 1 of Common Data set]
/// ```
///
/// Per-mode total sizes (matrix from
/// `design/008-oxlint-oxfmt-support/README.md` §5.2):
///
/// | compat | preserve | total |
/// |--------|----------|-------|
/// | false  | false    | 38    |
/// | false  | true     | 46    |
/// | true   | false    | 80    |
/// | true   | true     | 88    |
///
/// Returns `(NodeIndex, ExtOffset)` so the caller can patch the per-list
/// metadata via [`BinaryWriter::begin_node_list_at`] /
/// [`BinaryWriter::finalize_node_list`].
#[allow(clippy::too_many_arguments)]
pub fn write_jsdoc_tag(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    optional: bool,
    default_value: Option<StringField>,
    description: Option<StringField>,
    raw_body: Option<StringField>,
    children_bitmask: u8,
    description_raw_span: Option<(u32, u32)>,
) -> (NodeIndex, ExtOffset) {
    let base_size = if writer.compat_mode() { JSDOC_TAG_COMPAT_SIZE } else { JSDOC_TAG_BASIC_SIZE };
    let total_size = if description_raw_span.is_some() {
        base_size + DESCRIPTION_RAW_SPAN_SIZE
    } else {
        base_size
    };
    let (off, dst) = writer.extended.reserve_mut(total_size);
    dst[0] = children_bitmask;
    dst[1] = 0;
    write_opt_string_field(dst, 2, default_value);
    write_opt_string_field(dst, 8, description);
    write_opt_string_field(dst, 14, raw_body);
    // List metadata bytes 20..38 are zeroed by reserve_mut and patched
    // later via begin_node_list_at / finalize_node_list.

    // Phase 5: optional description_raw_span at the very end of the ED.
    let common_data = {
        let mut cd = optional as u8;
        if let Some((raw_start, raw_end)) = description_raw_span {
            let span_off = total_size - DESCRIPTION_RAW_SPAN_SIZE;
            dst[span_off..span_off + 4].copy_from_slice(&raw_start.to_le_bytes());
            dst[span_off + 4..span_off + 8].copy_from_slice(&raw_end.to_le_bytes());
            cd |= JSDOC_TAG_HAS_DESCRIPTION_RAW_SPAN_BIT;
        }
        cd
    };

    let idx = writer.emit_extended_node(parent_index, Kind::JsdocTag, common_data, span, off);
    (idx, off)
}

/// Write the 7 delimiter [`StringField`]s (42 bytes total) at the compat
/// tail of a previously written `JsdocTag` Extended Data record.
///
/// `description_raw_span` is **not** part of the compat tail in Phase 5 —
/// it is written by [`write_jsdoc_tag`] at the very end of the ED record
/// when opted in (see `design/008-oxlint-oxfmt-support/README.md` §4.2).
#[allow(clippy::too_many_arguments)]
pub fn write_jsdoc_tag_compat_tail(
    writer: &mut BinaryWriter<'_>,
    ext_offset: ExtOffset,
    delimiter: StringField,
    post_delimiter: StringField,
    post_tag: StringField,
    post_type: StringField,
    post_name: StringField,
    initial: StringField,
    line_end: StringField,
) {
    debug_assert!(writer.compat_mode());
    let dst = writer.extended.slice_mut(ext_offset, JSDOC_TAG_COMPAT_SIZE);
    let base = JSDOC_TAG_COMPAT_TAIL_BASE;
    write_string_field(dst, base, delimiter);
    write_string_field(dst, base + STRING_FIELD_SIZE, post_delimiter);
    write_string_field(dst, base + 2 * STRING_FIELD_SIZE, post_tag);
    write_string_field(dst, base + 3 * STRING_FIELD_SIZE, post_type);
    write_string_field(dst, base + 4 * STRING_FIELD_SIZE, post_name);
    write_string_field(dst, base + 5 * STRING_FIELD_SIZE, initial);
    write_string_field(dst, base + 6 * STRING_FIELD_SIZE, line_end);
}

// ---------------------------------------------------------------------------
// 0x04 JsdocTagName (String leaf)
// ---------------------------------------------------------------------------

/// Write a `JsdocTagName` leaf (Kind `0x04`, String type).
pub fn write_jsdoc_tag_name(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    value: LeafStringPayload,
) -> NodeIndex {
    writer.emit_string_node(parent_index, Kind::JsdocTagName, 0, span, value)
}

// ---------------------------------------------------------------------------
// 0x05 JsdocTagNameValue (String leaf)
// ---------------------------------------------------------------------------

/// Write a `JsdocTagNameValue` leaf (Kind `0x05`, String type).
pub fn write_jsdoc_tag_name_value(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    raw: LeafStringPayload,
) -> NodeIndex {
    writer.emit_string_node(parent_index, Kind::JsdocTagNameValue, 0, span, raw)
}

// ---------------------------------------------------------------------------
// 0x06 JsdocTypeSource (String leaf)
// ---------------------------------------------------------------------------

/// Write a `JsdocTypeSource` leaf (Kind `0x06`, String type).
pub fn write_jsdoc_type_source(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    raw: LeafStringPayload,
) -> NodeIndex {
    writer.emit_string_node(parent_index, Kind::JsdocTypeSource, 0, span, raw)
}

// ---------------------------------------------------------------------------
// 0x07 JsdocTypeLine (String basic / Extended compat)
// ---------------------------------------------------------------------------

/// Compat-mode Extended Data size for [`JsdocTypeLine`]: 4 × StringField
/// (raw_type + 3 optional delimiter strings).
const JSDOC_TYPE_LINE_COMPAT_SIZE: usize = 4 * STRING_FIELD_SIZE;

/// Write a `JsdocTypeLine` (Kind `0x07`, basic mode).
///
/// Node Data: `TypeTag::String` carrying the raw_type's StringIndex.
pub fn write_jsdoc_type_line(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    raw_type: LeafStringPayload,
) -> NodeIndex {
    writer.emit_string_node(parent_index, Kind::JsdocTypeLine, 0, span, raw_type)
}

/// Write a `JsdocTypeLine` (Kind `0x07`, compat mode).
///
/// Extended Data layout (24 bytes):
/// - byte 0-5   : raw_type       (StringField, required)
/// - byte 6-11  : delimiter      (StringField, NONE if absent)
/// - byte 12-17 : post_delimiter (StringField, NONE if absent)
/// - byte 18-23 : initial        (StringField, NONE if absent)
pub fn write_jsdoc_type_line_compat(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    raw_type: StringField,
    delimiter: Option<StringField>,
    post_delimiter: Option<StringField>,
    initial: Option<StringField>,
) -> NodeIndex {
    debug_assert!(writer.compat_mode());
    let (off, dst) = writer.extended.reserve_mut(JSDOC_TYPE_LINE_COMPAT_SIZE);
    write_string_field(dst, 0, raw_type);
    write_opt_string_field(dst, 6, delimiter);
    write_opt_string_field(dst, 12, post_delimiter);
    write_opt_string_field(dst, 18, initial);
    writer.emit_extended_node(parent_index, Kind::JsdocTypeLine, 0, span, off)
}

// ---------------------------------------------------------------------------
// 0x08 JsdocInlineTag (Extended)
// ---------------------------------------------------------------------------

/// Extended Data size for [`JsdocInlineTag`]: 3 × StringField = 18 bytes.
const JSDOC_INLINE_TAG_SIZE: usize = 3 * STRING_FIELD_SIZE;

/// Write a `JsdocInlineTag` (Kind `0x08`, Extended type).
///
/// Common Data: `bits[0:2] = format`. Extended Data: 18 bytes
/// (3 × StringField for `namepath_or_url`, `text`, `raw_body`).
pub fn write_jsdoc_inline_tag(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    format: u8,
    namepath_or_url: Option<StringField>,
    text: Option<StringField>,
    raw_body: Option<StringField>,
) -> NodeIndex {
    let (off, dst) = writer.extended.reserve_mut(JSDOC_INLINE_TAG_SIZE);
    write_opt_string_field(dst, 0, namepath_or_url);
    write_opt_string_field(dst, 6, text);
    write_opt_string_field(dst, 12, raw_body);
    writer.emit_extended_node(parent_index, Kind::JsdocInlineTag, format & 0b111, span, off)
}

// ---------------------------------------------------------------------------
// 0x09 JsdocGenericTagBody (Extended)
// ---------------------------------------------------------------------------

/// Extended Data size for [`JsdocGenericTagBody`]: bitmask + padding +
/// StringField = 8 bytes.
const JSDOC_GENERIC_TAG_BODY_SIZE: usize = 1 + 1 + STRING_FIELD_SIZE;

/// Write a `JsdocGenericTagBody` (Kind `0x09`, Extended type).
///
/// Common Data: `bit0 = has_dash_separator`. Extended Data: 8 bytes
/// (Children bitmask u8 + padding u8 + description StringField).
pub fn write_jsdoc_generic_tag_body(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    has_dash_separator: bool,
    description: Option<StringField>,
    children_bitmask: u8,
) -> NodeIndex {
    let (off, dst) = writer.extended.reserve_mut(JSDOC_GENERIC_TAG_BODY_SIZE);
    dst[0] = children_bitmask;
    dst[1] = 0;
    write_opt_string_field(dst, 2, description);
    writer.emit_extended_node(
        parent_index,
        Kind::JsdocGenericTagBody,
        has_dash_separator as u8,
        span,
        off,
    )
}

// ---------------------------------------------------------------------------
// 0x0A JsdocBorrowsTagBody (Children)
// ---------------------------------------------------------------------------

/// Write a `JsdocBorrowsTagBody` (Kind `0x0A`, Children type; 2 children:
/// `source` + `target`).
pub fn write_jsdoc_borrows_tag_body(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    children_bitmask: u32,
) -> NodeIndex {
    writer.emit_children_node(parent_index, Kind::JsdocBorrowsTagBody, 0, span, children_bitmask)
}

// ---------------------------------------------------------------------------
// 0x0B JsdocRawTagBody (String leaf)
// ---------------------------------------------------------------------------

/// Write a `JsdocRawTagBody` leaf (Kind `0x0B`, String type).
pub fn write_jsdoc_raw_tag_body(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    raw: LeafStringPayload,
) -> NodeIndex {
    writer.emit_string_node(parent_index, Kind::JsdocRawTagBody, 0, span, raw)
}

// ---------------------------------------------------------------------------
// 0x0C JsdocParameterName (Extended)
// ---------------------------------------------------------------------------

/// Extended Data size for [`JsdocParameterName`]: 2 × StringField = 12 bytes.
const JSDOC_PARAMETER_NAME_SIZE: usize = 2 * STRING_FIELD_SIZE;

/// Write a `JsdocParameterName` (Kind `0x0C`, Extended type).
///
/// Common Data: `bit0 = optional`. Extended Data: 12 bytes (`path`
/// StringField (required) + `default_value` StringField (Optional)).
pub fn write_jsdoc_parameter_name(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    optional: bool,
    path: StringField,
    default_value: Option<StringField>,
) -> NodeIndex {
    let (off, dst) = writer.extended.reserve_mut(JSDOC_PARAMETER_NAME_SIZE);
    write_string_field(dst, 0, path);
    write_opt_string_field(dst, 6, default_value);
    writer.emit_extended_node(parent_index, Kind::JsdocParameterName, optional as u8, span, off)
}

// ---------------------------------------------------------------------------
// 0x0D JsdocNamepathSource (String leaf)
// ---------------------------------------------------------------------------

/// Write a `JsdocNamepathSource` leaf (Kind `0x0D`, String type).
pub fn write_jsdoc_namepath_source(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    raw: LeafStringPayload,
) -> NodeIndex {
    writer.emit_string_node(parent_index, Kind::JsdocNamepathSource, 0, span, raw)
}

// ---------------------------------------------------------------------------
// 0x0E JsdocIdentifier (String leaf)
// ---------------------------------------------------------------------------

/// Write a `JsdocIdentifier` leaf (Kind `0x0E`, String type).
pub fn write_jsdoc_identifier(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    name: LeafStringPayload,
) -> NodeIndex {
    writer.emit_string_node(parent_index, Kind::JsdocIdentifier, 0, span, name)
}

// ---------------------------------------------------------------------------
// 0x0F JsdocText (String leaf)
// ---------------------------------------------------------------------------

/// Write a `JsdocText` leaf (Kind `0x0F`, String type).
pub fn write_jsdoc_text(
    writer: &mut BinaryWriter<'_>,
    span: Span,
    parent_index: u32,
    value: LeafStringPayload,
) -> NodeIndex {
    writer.emit_string_node(parent_index, Kind::JsdocText, 0, span, value)
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Write a [`StringField`] at `offset` inside `dst`. Caller guarantees that
/// `dst[offset..offset+6]` is in range.
#[inline]
fn write_string_field(dst: &mut [u8], offset: usize, field: StringField) {
    field.write_le(&mut dst[offset..offset + STRING_FIELD_SIZE]);
}

/// Write `opt.unwrap_or(StringField::NONE)` at `offset` inside `dst`.
#[inline]
fn write_opt_string_field(dst: &mut [u8], offset: usize, opt: Option<StringField>) {
    let field = opt.unwrap_or(StringField::NONE);
    field.write_le(&mut dst[offset..offset + STRING_FIELD_SIZE]);
}

/// Sentinel-conversion helper for compat-mode `Option<u32>` line indices.
#[inline]
fn opt_u32_sentinel(opt: Option<u32>) -> u32 {
    opt.unwrap_or(0xFFFF_FFFF)
}
