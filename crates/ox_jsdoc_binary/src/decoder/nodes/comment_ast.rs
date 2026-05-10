// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Lazy structs for the 15 comment AST kinds (`0x01 - 0x0F`).
//!
//! Every struct is a `Copy` value type wrapping
//! `(source_file, node_index, root_index)` (16 bytes on 64-bit targets).
//! All string fields read inline [`crate::format::string_field::StringField`]
//! slots from Extended Data — there is no longer a String Offsets table to
//! consult.

use std::borrow::Cow;

use crate::format::kind::Kind;
use crate::format::node_record::{KIND_OFFSET, NODE_RECORD_SIZE};
use crate::format::string_field::STRING_FIELD_SIZE;
use crate::writer::nodes::comment_ast::{
    JSDOC_BLOCK_BASIC_SIZE, JSDOC_BLOCK_COMPAT_SIZE, JSDOC_BLOCK_DESC_LINES_SLOT,
    JSDOC_BLOCK_HAS_DESCRIPTION_RAW_SPAN_BIT, JSDOC_BLOCK_INLINE_TAGS_SLOT, JSDOC_BLOCK_TAGS_SLOT,
    JSDOC_TAG_BASIC_SIZE, JSDOC_TAG_COMPAT_SIZE, JSDOC_TAG_DESC_LINES_SLOT,
    JSDOC_TAG_HAS_DESCRIPTION_RAW_SPAN_BIT, JSDOC_TAG_INLINE_TAGS_SLOT, JSDOC_TAG_TYPE_LINES_SLOT,
};

use super::super::helpers::{
    child_at_visitor_index, ext_offset, read_list_metadata, read_string_field, string_payload,
};
use super::super::source_file::LazySourceFile;
use super::super::text::parsed_preserving_whitespace;
use super::type_node::LazyTypeNode;
use super::{LazyNode, NodeListIter};

/// Generate a lazy comment AST struct + its `LazyNode` impl in one go.
macro_rules! define_lazy_comment_node {
    ($name:ident, $kind:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy)]
        pub struct $name<'a> {
            source_file: &'a LazySourceFile<'a>,
            node_index: u32,
            root_index: u32,
        }

        impl<'a> LazyNode<'a> for $name<'a> {
            const KIND: Kind = $kind;

            #[inline]
            fn from_index(
                source_file: &'a LazySourceFile<'a>,
                node_index: u32,
                root_index: u32,
            ) -> Self {
                $name { source_file, node_index, root_index }
            }

            #[inline]
            fn source_file(&self) -> &'a LazySourceFile<'a> {
                self.source_file
            }

            #[inline]
            fn node_index(&self) -> u32 {
                self.node_index
            }

            #[inline]
            fn root_index(&self) -> u32 {
                self.root_index
            }
        }
    };
}

/// Read an Optional `StringField` slot at `field_offset` inside a node's
/// Extended Data record. Returns `None` when the slot equals
/// [`crate::format::string_field::StringField::NONE`].
#[inline]
fn resolve_string_field_slot<'a>(
    sf: &LazySourceFile<'a>,
    ext_byte: usize,
    field_offset: usize,
) -> Option<&'a str> {
    let field = read_string_field(sf.bytes(), ext_byte + field_offset);
    sf.get_string_by_field(field)
}

/// Read a Required `StringField` slot at `field_offset` inside a node's
/// Extended Data record. Returns `""` if the slot is mistakenly the NONE
/// sentinel (defensive — the writer never emits NONE for a Required slot).
#[inline]
fn ext_string<'a>(sf: &LazySourceFile<'a>, node_index: u32, field_offset: usize) -> &'a str {
    let ext = ext_offset(sf, node_index) as usize;
    let field = read_string_field(sf.bytes(), ext + field_offset);
    sf.get_string_by_field(field).unwrap_or("")
}

// ---------------------------------------------------------------------------
// 0x01 LazyJsdocBlock
// ---------------------------------------------------------------------------
define_lazy_comment_node!(
    LazyJsdocBlock,
    Kind::JsdocBlock,
    "Lazy view of a `JsdocBlock` (Kind 0x01, root node)."
);

impl<'a> LazyJsdocBlock<'a> {
    /// Children bitmask (`bit0=descLines`, `bit1=tags`, `bit2=inlineTags`).
    /// Retained in Extended Data byte 0 for the visitor framework even
    /// though the lazy decoder now reads list head/count from the per-list
    /// metadata slots (bytes 50-67).
    #[inline]
    #[allow(dead_code)]
    fn children_bitmask(&self) -> u8 {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        self.source_file.bytes()[ext]
    }

    /// Description string (None when absent).
    pub fn description(&self) -> Option<&'a str> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        resolve_string_field_slot(self.source_file, ext, 2)
    }

    /// Source-preserving delimiter strings stored in Extended Data.
    pub fn delimiter(&self) -> &'a str {
        ext_string(self.source_file, self.node_index, 8)
    }
    /// `post_delimiter`.
    pub fn post_delimiter(&self) -> &'a str {
        ext_string(self.source_file, self.node_index, 14)
    }
    /// `terminal` source string (`"*/"`).
    pub fn terminal(&self) -> &'a str {
        ext_string(self.source_file, self.node_index, 20)
    }
    /// `line_end` source string.
    pub fn line_end(&self) -> &'a str {
        ext_string(self.source_file, self.node_index, 26)
    }
    /// `initial` source string.
    pub fn initial(&self) -> &'a str {
        ext_string(self.source_file, self.node_index, 32)
    }
    /// `delimiter_line_break` source string.
    pub fn delimiter_line_break(&self) -> &'a str {
        ext_string(self.source_file, self.node_index, 38)
    }
    /// `preterminal_line_break` source string.
    pub fn preterminal_line_break(&self) -> &'a str {
        ext_string(self.source_file, self.node_index, 44)
    }

    /// Top-level description lines.
    pub fn description_lines(&self) -> NodeListIter<'a, LazyJsdocDescriptionLine<'a>> {
        self.list_at(JSDOC_BLOCK_DESC_LINES_SLOT)
    }
    /// Block tags.
    pub fn tags(&self) -> NodeListIter<'a, LazyJsdocTag<'a>> {
        self.list_at(JSDOC_BLOCK_TAGS_SLOT)
    }
    /// Inline tags found in the top-level description.
    pub fn inline_tags(&self) -> NodeListIter<'a, LazyJsdocInlineTag<'a>> {
        self.list_at(JSDOC_BLOCK_INLINE_TAGS_SLOT)
    }

    // -- compat-mode-only line metadata (Extended Data byte 70+) ------------

    /// 0-based line index of the closing `*/` line. Returns `None` when the
    /// buffer was not written in compat mode.
    pub fn end_line(&self) -> Option<u32> {
        if !self.source_file.compat_mode {
            return None;
        }
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        // Compat tail starts at byte 68 (basic ED size). end_line is the
        // first u32 after a 2-byte alignment padding → byte 70.
        Some(super::super::helpers::read_u32(self.source_file.bytes(), ext + 70))
    }

    /// Raw description slice (with `*` prefix and blank lines intact).
    /// Returns `None` when the buffer was not parsed with
    /// `preserve_whitespace = true` (the per-node
    /// `has_description_raw_span` Common Data bit is clear), or when the
    /// block has no description.
    ///
    /// Phase 5 layout: the span sits at the **last 8 bytes** of the ED
    /// record (offset = compat ? 90 : 68 = the basic / compat ED size).
    /// See `design/008-oxlint-oxfmt-support/README.md` §4.2 / §4.3.
    pub fn description_raw(&self) -> Option<&'a str> {
        if (self.common_data() & JSDOC_BLOCK_HAS_DESCRIPTION_RAW_SPAN_BIT) == 0 {
            return None;
        }
        let span_off = if self.source_file.compat_mode {
            JSDOC_BLOCK_COMPAT_SIZE
        } else {
            JSDOC_BLOCK_BASIC_SIZE
        };
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        let bytes = self.source_file.bytes();
        let start = super::super::helpers::read_u32(bytes, ext + span_off);
        let end = super::super::helpers::read_u32(bytes, ext + span_off + 4);
        self.source_file.slice_source_text(self.root_index, start, end)
    }

    /// Description text. When `preserve_whitespace` is `true`, blank lines
    /// and indentation past the `* ` prefix are preserved (algorithm in
    /// [`super::super::text::parsed_preserving_whitespace`]). When `false`,
    /// returns the compact view ([`Self::description`]). Returns `None`
    /// when no description is present, or when `preserve_whitespace = true`
    /// is requested on a buffer that wasn't parsed with the matching
    /// `preserve_whitespace = true` parse option.
    pub fn description_text(&self, preserve_whitespace: bool) -> Option<Cow<'a, str>> {
        if preserve_whitespace {
            self.description_raw().map(|raw| Cow::Owned(parsed_preserving_whitespace(raw)))
        } else {
            self.description().map(Cow::Borrowed)
        }
    }

    /// Helper: read the per-list `(head, count)` metadata at `slot_offset`
    /// inside this block's Extended Data record and produce an iterator.
    #[inline]
    fn list_at<T: LazyNode<'a>>(&self, slot_offset: usize) -> NodeListIter<'a, T> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        let (head, count) = read_list_metadata(self.source_file, ext, slot_offset);
        NodeListIter::new(self.source_file, head, count, self.root_index)
    }
}

// ---------------------------------------------------------------------------
// 0x02 LazyJsdocDescriptionLine
// ---------------------------------------------------------------------------
define_lazy_comment_node!(
    LazyJsdocDescriptionLine,
    Kind::JsdocDescriptionLine,
    "Lazy view of a `JsdocDescriptionLine` (Kind 0x02)."
);

impl<'a> LazyJsdocDescriptionLine<'a> {
    /// Description content. Basic mode reads it from the String-payload
    /// (Node Data); compat mode reads from byte 0-5 of the Extended Data
    /// record.
    pub fn description(&self) -> &'a str {
        if self.source_file.compat_mode {
            ext_string(self.source_file, self.node_index, 0)
        } else {
            string_payload(self.source_file, self.node_index).unwrap_or("")
        }
    }
}

// ---------------------------------------------------------------------------
// 0x03 LazyJsdocTag
// ---------------------------------------------------------------------------
define_lazy_comment_node!(LazyJsdocTag, Kind::JsdocTag, "Lazy view of a `JsdocTag` (Kind 0x03).");

impl<'a> LazyJsdocTag<'a> {
    /// Whether the tag was written with bracket syntax such as `[id]`.
    #[inline]
    pub fn optional(&self) -> bool {
        (self.common_data() & 0b0000_0001) != 0
    }

    /// `default_value` from `[id=foo]` syntax.
    pub fn default_value(&self) -> Option<&'a str> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        resolve_string_field_slot(self.source_file, ext, 2)
    }
    /// Joined description text.
    pub fn description(&self) -> Option<&'a str> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        resolve_string_field_slot(self.source_file, ext, 8)
    }
    /// Raw description slice (with `*` prefix and blank lines intact).
    /// Returns `None` when the buffer was not parsed with
    /// `preserve_whitespace = true` (the per-node
    /// `has_description_raw_span` Common Data bit is clear), or when the
    /// tag has no description.
    ///
    /// Phase 5 layout: the span sits at the **last 8 bytes** of the ED
    /// record (offset = compat ? 80 : 38 = the basic / compat ED size).
    /// See `design/008-oxlint-oxfmt-support/README.md` §4.2 / §4.3.
    pub fn description_raw(&self) -> Option<&'a str> {
        if (self.common_data() & JSDOC_TAG_HAS_DESCRIPTION_RAW_SPAN_BIT) == 0 {
            return None;
        }
        let span_off =
            if self.source_file.compat_mode { JSDOC_TAG_COMPAT_SIZE } else { JSDOC_TAG_BASIC_SIZE };
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        let bytes = self.source_file.bytes();
        let start = super::super::helpers::read_u32(bytes, ext + span_off);
        let end = super::super::helpers::read_u32(bytes, ext + span_off + 4);
        self.source_file.slice_source_text(self.root_index, start, end)
    }
    /// Description text. Identical contract to
    /// [`LazyJsdocBlock::description_text`].
    pub fn description_text(&self, preserve_whitespace: bool) -> Option<Cow<'a, str>> {
        if preserve_whitespace {
            self.description_raw().map(|raw| Cow::Owned(parsed_preserving_whitespace(raw)))
        } else {
            self.description().map(Cow::Borrowed)
        }
    }
    /// Raw body when the tag uses the `Raw` body variant.
    pub fn raw_body(&self) -> Option<&'a str> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        resolve_string_field_slot(self.source_file, ext, 14)
    }

    #[inline]
    fn children_bitmask(&self) -> u8 {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        self.source_file.bytes()[ext]
    }

    /// Tag name child (`@name`). Always present.
    pub fn tag(&self) -> LazyJsdocTagName<'a> {
        let bitmask = self.children_bitmask();
        let idx = child_at_visitor_index(self.source_file, self.node_index, bitmask, 0)
            .expect("JsdocTag.tag is required");
        LazyJsdocTagName::from_index(self.source_file, idx, self.root_index)
    }
    /// Raw `{...}` type source.
    pub fn raw_type(&self) -> Option<LazyJsdocTypeSource<'a>> {
        let bitmask = self.children_bitmask();
        child_at_visitor_index(self.source_file, self.node_index, bitmask, 1)
            .map(|idx| LazyJsdocTypeSource::from_index(self.source_file, idx, self.root_index))
    }
    /// Tag-name value (e.g. `id` in `@param id`).
    pub fn name(&self) -> Option<LazyJsdocTagNameValue<'a>> {
        let bitmask = self.children_bitmask();
        child_at_visitor_index(self.source_file, self.node_index, bitmask, 2)
            .map(|idx| LazyJsdocTagNameValue::from_index(self.source_file, idx, self.root_index))
    }
    /// `parsedType` child (any TypeNode variant).
    pub fn parsed_type(&self) -> Option<LazyTypeNode<'a>> {
        let bitmask = self.children_bitmask();
        let idx = child_at_visitor_index(self.source_file, self.node_index, bitmask, 3)?;
        LazyTypeNode::from_index(self.source_file, idx, self.root_index)
    }
    /// `body` child (one of `JsdocGenericTagBody` / `JsdocBorrowsTagBody` /
    /// `JsdocRawTagBody`). The variant is determined from the child's Kind
    /// byte; returns `None` when bit4 of the Children bitmask is unset or
    /// the child is not one of the three body kinds.
    pub fn body(&self) -> Option<LazyJsdocTagBody<'a>> {
        let bitmask = self.children_bitmask();
        let idx = child_at_visitor_index(self.source_file, self.node_index, bitmask, 4)?;
        let kind_byte = self.source_file.bytes()[self.source_file.nodes_offset as usize
            + idx as usize * NODE_RECORD_SIZE
            + KIND_OFFSET];
        let kind = Kind::from_u8(kind_byte).ok()?;
        match kind {
            Kind::JsdocGenericTagBody => Some(LazyJsdocTagBody::Generic(
                LazyJsdocGenericTagBody::from_index(self.source_file, idx, self.root_index),
            )),
            Kind::JsdocBorrowsTagBody => Some(LazyJsdocTagBody::Borrows(
                LazyJsdocBorrowsTagBody::from_index(self.source_file, idx, self.root_index),
            )),
            Kind::JsdocRawTagBody => Some(LazyJsdocTagBody::Raw(LazyJsdocRawTagBody::from_index(
                self.source_file,
                idx,
                self.root_index,
            ))),
            _ => None,
        }
    }

    /// Source-preserving description lines.
    pub fn description_lines(&self) -> NodeListIter<'a, LazyJsdocDescriptionLine<'a>> {
        self.list_at(JSDOC_TAG_DESC_LINES_SLOT)
    }
    /// Source-preserving type lines.
    pub fn type_lines(&self) -> NodeListIter<'a, LazyJsdocTypeLine<'a>> {
        self.list_at(JSDOC_TAG_TYPE_LINES_SLOT)
    }
    /// Inline tags found in this tag's description.
    pub fn inline_tags(&self) -> NodeListIter<'a, LazyJsdocInlineTag<'a>> {
        self.list_at(JSDOC_TAG_INLINE_TAGS_SLOT)
    }

    #[inline]
    fn list_at<T: LazyNode<'a>>(&self, slot_offset: usize) -> NodeListIter<'a, T> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        let (head, count) = read_list_metadata(self.source_file, ext, slot_offset);
        NodeListIter::new(self.source_file, head, count, self.root_index)
    }
}

// ---------------------------------------------------------------------------
// 0x04-0x06, 0x0B, 0x0D-0x0F: Extended-type string-leaf nodes (6-byte ED)
// ---------------------------------------------------------------------------

macro_rules! define_string_leaf {
    ($name:ident, $kind:expr, $accessor:ident, $doc:expr) => {
        define_lazy_comment_node!($name, $kind, $doc);
        impl<'a> $name<'a> {
            #[doc = concat!("Resolve the underlying string value of this `", stringify!($name), "`.")]
            pub fn $accessor(&self) -> &'a str {
                string_payload(self.source_file, self.node_index).unwrap_or("")
            }
        }
    };
}

define_string_leaf!(
    LazyJsdocTagName,
    Kind::JsdocTagName,
    value,
    "Lazy view of a `JsdocTagName` leaf (Kind 0x04)."
);
define_string_leaf!(
    LazyJsdocTagNameValue,
    Kind::JsdocTagNameValue,
    raw,
    "Lazy view of a `JsdocTagNameValue` leaf (Kind 0x05)."
);
define_string_leaf!(
    LazyJsdocTypeSource,
    Kind::JsdocTypeSource,
    raw,
    "Lazy view of a `JsdocTypeSource` leaf (Kind 0x06)."
);
define_string_leaf!(
    LazyJsdocRawTagBody,
    Kind::JsdocRawTagBody,
    raw,
    "Lazy view of a `JsdocRawTagBody` leaf (Kind 0x0B)."
);
define_string_leaf!(
    LazyJsdocNamepathSource,
    Kind::JsdocNamepathSource,
    raw,
    "Lazy view of a `JsdocNamepathSource` leaf (Kind 0x0D)."
);
define_string_leaf!(
    LazyJsdocIdentifier,
    Kind::JsdocIdentifier,
    name,
    "Lazy view of a `JsdocIdentifier` leaf (Kind 0x0E)."
);
define_string_leaf!(
    LazyJsdocText,
    Kind::JsdocText,
    value,
    "Lazy view of a `JsdocText` leaf (Kind 0x0F)."
);

// ---------------------------------------------------------------------------
// 0x07 LazyJsdocTypeLine (Extended)
// ---------------------------------------------------------------------------
define_lazy_comment_node!(
    LazyJsdocTypeLine,
    Kind::JsdocTypeLine,
    "Lazy view of a `JsdocTypeLine` (Kind 0x07)."
);

impl<'a> LazyJsdocTypeLine<'a> {
    /// Raw `{...}` line content. Basic mode reads it from the
    /// String-payload (Node Data); compat mode reads from byte 0-5 of the
    /// Extended Data record.
    pub fn raw_type(&self) -> &'a str {
        if self.source_file.compat_mode {
            ext_string(self.source_file, self.node_index, 0)
        } else {
            string_payload(self.source_file, self.node_index).unwrap_or("")
        }
    }
}

// ---------------------------------------------------------------------------
// 0x08 LazyJsdocInlineTag
// ---------------------------------------------------------------------------
define_lazy_comment_node!(
    LazyJsdocInlineTag,
    Kind::JsdocInlineTag,
    "Lazy view of a `JsdocInlineTag` (Kind 0x08)."
);

impl<'a> LazyJsdocInlineTag<'a> {
    /// Inline tag format (`bits[0:2]` of Common Data).
    #[inline]
    pub fn format(&self) -> u8 {
        self.common_data() & 0b0000_0111
    }
    /// Optional name path or URL portion.
    pub fn namepath_or_url(&self) -> Option<&'a str> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        resolve_string_field_slot(self.source_file, ext, 0)
    }
    /// Optional display text portion.
    pub fn text(&self) -> Option<&'a str> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        resolve_string_field_slot(self.source_file, ext, STRING_FIELD_SIZE)
    }
    /// Raw body text fallback.
    pub fn raw_body(&self) -> Option<&'a str> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        resolve_string_field_slot(self.source_file, ext, 2 * STRING_FIELD_SIZE)
    }
}

// ---------------------------------------------------------------------------
// 0x09 LazyJsdocGenericTagBody
// ---------------------------------------------------------------------------
define_lazy_comment_node!(
    LazyJsdocGenericTagBody,
    Kind::JsdocGenericTagBody,
    "Lazy view of a `JsdocGenericTagBody` (Kind 0x09)."
);

impl<'a> LazyJsdocGenericTagBody<'a> {
    /// `true` when the tag separator was `-`.
    #[inline]
    pub fn has_dash_separator(&self) -> bool {
        (self.common_data() & 0b0000_0001) != 0
    }
    /// `description` string after the dash separator.
    pub fn description(&self) -> Option<&'a str> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        resolve_string_field_slot(self.source_file, ext, 2)
    }
}

// ---------------------------------------------------------------------------
// 0x0A LazyJsdocBorrowsTagBody
// ---------------------------------------------------------------------------
define_lazy_comment_node!(
    LazyJsdocBorrowsTagBody,
    Kind::JsdocBorrowsTagBody,
    "Lazy view of a `JsdocBorrowsTagBody` (Kind 0x0A)."
);

// ---------------------------------------------------------------------------
// 0x0C LazyJsdocParameterName
// ---------------------------------------------------------------------------
define_lazy_comment_node!(
    LazyJsdocParameterName,
    Kind::JsdocParameterName,
    "Lazy view of a `JsdocParameterName` (Kind 0x0C)."
);

impl<'a> LazyJsdocParameterName<'a> {
    /// `true` when the parameter was wrapped in brackets (`[id]`).
    #[inline]
    pub fn optional(&self) -> bool {
        (self.common_data() & 0b0000_0001) != 0
    }
    /// The path text (required, byte 0-5).
    pub fn path(&self) -> &'a str {
        ext_string(self.source_file, self.node_index, 0)
    }
    /// Default value from `[id=foo]` syntax (Optional, byte 6-11).
    pub fn default_value(&self) -> Option<&'a str> {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        resolve_string_field_slot(self.source_file, ext, STRING_FIELD_SIZE)
    }
}

// ---------------------------------------------------------------------------
// Variant wrappers
// ---------------------------------------------------------------------------

/// Body of a `JsdocTag` — one of three variants distinguished by the child Kind.
#[derive(Debug, Clone, Copy)]
pub enum LazyJsdocTagBody<'a> {
    /// Generic body (`@param {T} name - desc`).
    Generic(LazyJsdocGenericTagBody<'a>),
    /// Borrows body (`@borrows from as to`).
    Borrows(LazyJsdocBorrowsTagBody<'a>),
    /// Raw text body.
    Raw(LazyJsdocRawTagBody<'a>),
}

/// Tag value — first non-type token after the tag name.
#[derive(Debug, Clone, Copy)]
pub enum LazyJsdocTagValue<'a> {
    /// Parameter-style name (e.g. `[id=foo]`).
    Parameter(LazyJsdocParameterName<'a>),
    /// Namepath token.
    Namepath(LazyJsdocNamepathSource<'a>),
    /// Bare identifier.
    Identifier(LazyJsdocIdentifier<'a>),
    /// Raw text fallback.
    Raw(LazyJsdocText<'a>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn lazy_comment_structs_are_compact() {
        macro_rules! assert_size {
            ($t:ty) => {
                assert!(size_of::<$t>() <= 16, concat!(stringify!($t), " exceeds 16 bytes"));
            };
        }
        assert_size!(LazyJsdocBlock<'static>);
        assert_size!(LazyJsdocDescriptionLine<'static>);
        assert_size!(LazyJsdocTag<'static>);
        assert_size!(LazyJsdocTagName<'static>);
        assert_size!(LazyJsdocText<'static>);
        assert_size!(LazyJsdocInlineTag<'static>);
        assert_size!(LazyJsdocParameterName<'static>);
    }

    #[test]
    fn variant_wrappers_fit_in_24_bytes() {
        assert!(size_of::<LazyJsdocTagBody<'static>>() <= 24);
        assert!(size_of::<LazyJsdocTagValue<'static>>() <= 24);
    }
}
