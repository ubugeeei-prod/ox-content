// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! [`BinaryWriter`] — the top-level entry point for emitting Binary AST.

use oxc_allocator::{Allocator, Vec as ArenaVec};
use oxc_span::Span;

use crate::format::diagnostics;
use crate::format::header::{
    COMPAT_MODE_BIT, DIAGNOSTICS_OFFSET_FIELD, EXTENDED_DATA_OFFSET_FIELD, FLAGS_OFFSET,
    HEADER_SIZE, Header, NODE_COUNT_FIELD, NODES_OFFSET_FIELD, ROOT_ARRAY_OFFSET_FIELD,
    ROOT_COUNT_FIELD, SOURCE_TEXT_LENGTH_FIELD, STRING_DATA_OFFSET_FIELD,
    STRING_OFFSETS_OFFSET_FIELD, SUPPORTED_VERSION_BYTE, VERSION_OFFSET,
};
use crate::format::kind::Kind;
use crate::format::node_record::{
    COMMON_DATA_MASK, NEXT_SIBLING_OFFSET, NODE_RECORD_SIZE, STRING_INLINE_LENGTH_MAX,
    STRING_INLINE_OFFSET_MAX, TypeTag, pack_node_data, pack_string_inline,
};
use crate::format::root_index::ROOT_INDEX_ENTRY_SIZE;
use crate::format::string_field::StringField;

use super::extended_data::{ExtOffset, ExtendedDataBuilder};
use super::nodes::NodeIndex;
use super::string_table::{
    LeafStringPayload, StringIndex, StringTableBuilder, common_string_field, lookup_common,
};

/// Tracks one in-progress NodeList — the head index and element count that
/// will be patched into the owning parent's Extended Data block when the
/// list closes.
///
/// Constructed by [`BinaryWriter::begin_node_list_at`], updated by
/// [`BinaryWriter::record_list_child`], and consumed by
/// [`BinaryWriter::finalize_node_list`]. The struct intentionally has no
/// `Default` impl: an unfinished list must be wired up to an Extended Data
/// slot or it has no place to land.
#[derive(Debug)]
pub struct ListInProgress {
    parent_ext: ExtOffset,
    slot_offset: usize,
    head_index: u32,
    count: u16,
}

/// Top-level writer that owns one buffer per Binary AST section.
///
/// Construction: [`BinaryWriter::new`] pre-writes the 24-byte sentinel
/// `node[0]` so that real nodes start at index 1.
///
/// Lifecycle: parser code drives [`BinaryWriter`] through the
/// `write_*` helpers (see the [`super::nodes`] module). When all roots and
/// diagnostics have been written, [`BinaryWriter::finish`] concatenates the
/// per-section buffers and patches the [`Header`] with the resolved offsets,
/// returning the final Binary AST byte stream.
///
/// All buffers are arena-allocated against the borrow-checker-tracked
/// `'arena` lifetime, so the resulting bytes can be shared zero-copy with
/// NAPI/WASM bindings as long as the arena outlives the consumer.
pub struct BinaryWriter<'arena> {
    /// In-memory header; field offsets are patched in at [`Self::finish`].
    pub(crate) header: Header,
    /// Root index array buffer (`12N` bytes, see `format::root_index`).
    pub(crate) root_index_buffer: ArenaVec<'arena, u8>,
    /// Diagnostics entries (`(root_index, message_index)`); sorted at
    /// [`Self::finish`] before being serialized.
    pub(crate) diagnostics: ArenaVec<'arena, (u32, u32)>,
    /// Nodes section buffer (`24P` bytes), starting with the sentinel.
    pub(crate) nodes_buffer: ArenaVec<'arena, u8>,
    /// String table builder (handles dedup + offsets/data buffers).
    pub(crate) strings: StringTableBuilder<'arena>,
    /// Extended Data builder (handles 8-byte alignment).
    pub(crate) extended: ExtendedDataBuilder<'arena>,
    /// Total length of source-text bytes appended via
    /// [`StringTableBuilder::append_source_text`]. Stored separately from
    /// `strings.data_buffer.len()` because that buffer also contains
    /// interned strings.
    pub(crate) source_text_length: u32,
    /// `data_buffer` byte offset where the **most recently appended**
    /// source text region starts. Used by [`Self::intern_source_slice`] to
    /// translate a source-relative byte range into the absolute offset
    /// pair that lands in the `String Offsets` section.
    ///
    /// Updated on every [`Self::append_source_text`] call. The contract
    /// for `intern_source_slice` callers is that they must intern source
    /// slices belonging to the **latest** appended source text — which is
    /// what every `emit_block` call does (parse-then-emit per item in
    /// `parse_batch_to_bytes`).
    pub(crate) current_source_data_offset: u32,
    /// Byte length of the most recently appended source text. Together
    /// with [`Self::current_source_data_offset`] this defines the valid
    /// byte range that [`Self::intern_source_slice`] callers must keep
    /// their span within.
    pub(crate) current_source_length: u32,
    /// Raw start pointer of the most recently appended source text, stored
    /// as `usize` to avoid lifetime tracking on the writer.
    ///
    /// Used by [`Self::intern_source_slice_or_string`] to detect whether a
    /// borrowed `&str` value is a sub-slice of the appended source via
    /// pointer arithmetic. The pointer is only ever compared (never
    /// dereferenced through this field), and its underlying allocation is
    /// guaranteed to outlive every emit call within the same
    /// `parse_*_to_bytes` iteration that registered the source via
    /// [`Self::append_source_text`].
    pub(crate) current_source_ptr: usize,
    /// Per-parent backpatch table: `next_sibling_patch[parent_index]`
    /// stores the byte offset of the most recent child of `parent_index`
    /// (so the next call to [`Self::emit_node_record`] can patch its
    /// `next_sibling` field). `0` means "no previous sibling".
    pub(crate) next_sibling_patch: ArenaVec<'arena, u32>,
    /// Per-writer opt-in for emitting the trailing `description_raw_span`
    /// slot on `JsdocBlock` / `JsdocTag` ED records. Set via
    /// [`Self::set_preserve_whitespace_span`]; consumed by the per-node
    /// emit phase in `parser/context.rs`. Orthogonal to compat mode.
    pub(crate) preserve_whitespace_span: bool,
    /// Reference to the underlying arena, used by the per-node helpers when
    /// they need to allocate scratch space.
    pub(crate) arena: &'arena Allocator,
}

impl<'arena> BinaryWriter<'arena> {
    /// Create a fresh writer bound to the supplied arena.
    ///
    /// Pre-allocates the per-section buffers and writes the all-zero
    /// `node[0]` sentinel. After construction, calling [`Self::finish`]
    /// without writing any roots yields a valid empty Binary AST buffer.
    #[must_use]
    pub fn new(arena: &'arena Allocator) -> Self {
        let mut nodes_buffer = ArenaVec::new_in(arena);
        // Pre-write the all-zero sentinel `node[0]` so real nodes start at
        // index 1 and `parent_index = 0` / `next_sibling = 0` mean
        // "no link" without a special case.
        nodes_buffer.extend(core::iter::repeat_n(0u8, NODE_RECORD_SIZE));

        let mut header = Header::default();
        header.version = SUPPORTED_VERSION_BYTE;

        BinaryWriter {
            header,
            root_index_buffer: ArenaVec::new_in(arena),
            diagnostics: ArenaVec::new_in(arena),
            nodes_buffer,
            strings: StringTableBuilder::new(arena),
            extended: ExtendedDataBuilder::new(arena),
            source_text_length: 0,
            current_source_data_offset: 0,
            current_source_length: 0,
            current_source_ptr: 0,
            next_sibling_patch: ArenaVec::new_in(arena),
            preserve_whitespace_span: false,
            arena,
        }
    }

    /// Truncate the writer back to its post-[`Self::new`] state without
    /// freeing any arena memory. Per-section buffers retain their capacity
    /// so subsequent emit calls reuse the existing allocations instead of
    /// growing from zero on every per-comment writer construction.
    ///
    /// Pairs with [`crate::parser::parse_into`] /
    /// [`crate::parser::parse_batch_into`] for hot-loop callers (lint
    /// runners, doc generators) that want per-comment `parse()` semantics
    /// without paying the per-call writer construction cost.
    ///
    /// Restores after `reset()`:
    /// - the all-zero `node[0]` sentinel (preserved as the first 24 bytes
    ///   of `nodes_buffer`),
    /// - the [`super::string_table::COMMON_STRINGS`] prelude in the string
    ///   table,
    /// - `compat_mode` / `preserve_whitespace_span` cleared (callers must
    ///   re-enable them on the recycled writer when needed).
    pub fn reset(&mut self) {
        // Header: keep version, clear flags (compat_mode / preserve_ws span
        // bits are caller-controlled and must be re-set on the recycled writer).
        self.header.flags = 0;

        // Nodes buffer: keep the 24-byte sentinel, drop everything after it.
        // truncate retains capacity.
        self.nodes_buffer.truncate(NODE_RECORD_SIZE);

        self.root_index_buffer.truncate(0);
        self.diagnostics.truncate(0);
        self.next_sibling_patch.truncate(0);

        self.strings.reset();
        self.extended.reset();

        self.source_text_length = 0;
        self.current_source_data_offset = 0;
        self.current_source_length = 0;
        self.current_source_ptr = 0;

        self.preserve_whitespace_span = false;
    }

    /// Emit one 24-byte node record into the Nodes section and return its
    /// new [`NodeIndex`].
    ///
    /// Side effect: updates `next_sibling_patch` so the next sibling of
    /// `parent_index` will be backpatched to point at the freshly-emitted
    /// node. `parent_index = 0` means "child of the sentinel" (i.e. a
    /// root).
    ///
    /// `common_data` is masked to its lower 6 bits before being stored, so
    /// callers can pass the raw bit field without worrying about the
    /// reserved upper 2 bits.
    pub(crate) fn emit_node_record(
        &mut self,
        parent_index: u32,
        kind: Kind,
        common_data: u8,
        span: Span,
        node_data: u32,
    ) -> NodeIndex {
        let new_index = self.node_count();
        let new_byte_offset = self.nodes_buffer.len() as u32;

        // Build the node record directly into the buffer's spare capacity
        // via a single 24-byte aligned struct write — measurably faster than
        // the previous "stack record + extend_from_slice memcpy" path on the
        // typescript-checker.ts fixture (probe showed ~12% of parse_to_bytes
        // is spent in this single emit, with stack-build vs memcpy split
        // roughly even).
        //
        // SAFETY:
        // - `reserve(NODE_RECORD_SIZE)` guarantees the spare capacity has
        //   at least 24 bytes.
        // - `NodeRecord` is `#[repr(C)]` with the exact byte layout the
        //   format spec requires (verified by the assertion below).
        // - `write_unaligned` is correct because the buffer pointer's
        //   alignment is unknown; we don't rely on natural alignment.
        // - `set_len` is sound because we just initialised those bytes.
        #[repr(C)]
        struct NodeRecord {
            kind: u8,
            common_data: u8,
            padding: u16,
            span_start: u32,
            span_end: u32,
            node_data: u32,
            parent_index: u32,
            next_sibling: u32,
        }
        const _: () = assert!(core::mem::size_of::<NodeRecord>() == NODE_RECORD_SIZE);

        let cur_len = self.nodes_buffer.len();
        self.nodes_buffer.reserve(NODE_RECORD_SIZE);
        unsafe {
            let dst = self.nodes_buffer.as_mut_ptr().add(cur_len) as *mut NodeRecord;
            dst.write_unaligned(NodeRecord {
                kind: kind.as_u8(),
                common_data: common_data & COMMON_DATA_MASK,
                padding: 0,
                span_start: span.start.to_le(),
                span_end: span.end.to_le(),
                node_data: node_data.to_le(),
                parent_index: parent_index.to_le(),
                next_sibling: 0,
            });
            self.nodes_buffer.set_len(cur_len + NODE_RECORD_SIZE);
        }

        // Backpatch the previous sibling's `next_sibling` to this node, if
        // any.
        let parent_idx = parent_index as usize;
        if parent_idx >= self.next_sibling_patch.len() {
            self.next_sibling_patch.resize(parent_idx + 1, 0);
        }
        let prev_byte_offset = self.next_sibling_patch[parent_idx];
        if prev_byte_offset != 0 {
            let patch_at = prev_byte_offset as usize + NEXT_SIBLING_OFFSET;
            let bytes = new_index.to_le_bytes();
            self.nodes_buffer[patch_at..patch_at + 4].copy_from_slice(&bytes);
        }
        self.next_sibling_patch[parent_idx] = new_byte_offset;

        NodeIndex::new(new_index).expect("node_index 0 is reserved for the sentinel")
    }

    /// Convenience for **String-type** leaves: emit a node whose Node Data
    /// payload is a 30-bit String Offsets index. Used by string-leaf Kinds
    /// where embedding the index in Node Data is cheaper than allocating a
    /// 6-byte Extended Data record.
    ///
    /// Dispatches on the [`LeafStringPayload`] variant to pick the right
    /// `TypeTag`: `Inline` short strings pack `(offset, length)` directly into
    /// the 30-bit Node Data payload (`TypeTag::StringInline`), `Index`
    /// fallback uses the legacy String Offsets table indirection
    /// (`TypeTag::String`).
    ///
    /// `#[inline(always)]` because the only work this fn does on top of
    /// `emit_node_record` is one `pack_node_data` call; `#[inline]` alone
    /// is not enough to convince LLVM to inline through the per-Kind
    /// `write_jsdoc_*` helpers in `writer/nodes/`.
    #[inline(always)]
    pub(crate) fn emit_string_node(
        &mut self,
        parent_index: u32,
        kind: Kind,
        common_data: u8,
        span: Span,
        payload: LeafStringPayload,
    ) -> NodeIndex {
        let node_data = match payload {
            LeafStringPayload::Inline { offset, length } => {
                pack_node_data(TypeTag::StringInline, pack_string_inline(offset, length))
            }
            LeafStringPayload::Index(idx) => pack_node_data(TypeTag::String, idx.as_u32()),
        };
        self.emit_node_record(parent_index, kind, common_data, span, node_data)
    }

    /// Convenience for **Children-type** nodes: emit a node whose Node Data
    /// payload is a 30-bit visitor-order Children bitmask.
    #[inline(always)]
    pub(crate) fn emit_children_node(
        &mut self,
        parent_index: u32,
        kind: Kind,
        common_data: u8,
        span: Span,
        children_bitmask: u32,
    ) -> NodeIndex {
        let node_data = pack_node_data(TypeTag::Children, children_bitmask);
        self.emit_node_record(parent_index, kind, common_data, span, node_data)
    }

    /// Convenience for **Extended-type** nodes: emit a node whose Node Data
    /// payload is the supplied Extended Data byte offset.
    #[inline(always)]
    pub(crate) fn emit_extended_node(
        &mut self,
        parent_index: u32,
        kind: Kind,
        common_data: u8,
        span: Span,
        ext_offset: ExtOffset,
    ) -> NodeIndex {
        let node_data = pack_node_data(TypeTag::Extended, ext_offset.as_u32());
        self.emit_node_record(parent_index, kind, common_data, span, node_data)
    }

    /// Set the `compat_mode` flag bit on the header.
    ///
    /// Must be called before any node is written, since the bit affects the
    /// per-Kind Extended Data layouts emitted by `write_*` helpers.
    pub fn set_compat_mode(&mut self, enabled: bool) {
        if enabled {
            self.header.flags |= COMPAT_MODE_BIT;
        } else {
            self.header.flags &= !COMPAT_MODE_BIT;
        }
    }

    /// Whether `compat_mode` is currently enabled. `write_*` helpers consult
    /// this to decide whether to emit the compat extension region.
    #[inline]
    #[must_use]
    pub const fn compat_mode(&self) -> bool {
        self.header.compat_mode()
    }

    /// Enable / disable per-node emission of the `description_raw_span` slot
    /// on `JsdocBlock` / `JsdocTag`. When enabled, the parser-side emitter
    /// passes the span to the writer and the per-node `has_description_raw_span`
    /// Common Data bit is set. When disabled (default), the bit stays clear
    /// and the 8-byte slot is omitted entirely.
    ///
    /// Must be called before any node is written, since the flag affects the
    /// per-record ED size emitted by `write_jsdoc_block` / `write_jsdoc_tag`.
    /// Fully orthogonal to [`Self::set_compat_mode`].
    ///
    /// See `design/008-oxlint-oxfmt-support/README.md` §4.2.
    pub fn set_preserve_whitespace_span(&mut self, enabled: bool) {
        self.preserve_whitespace_span = enabled;
    }

    /// Whether the per-node `description_raw_span` opt-in is currently
    /// enabled. The parser's emit phase consults this to decide whether to
    /// pass the span through to the per-node `write_*` helper.
    #[inline]
    #[must_use]
    pub const fn preserve_whitespace_span(&self) -> bool {
        self.preserve_whitespace_span
    }

    /// Append one root entry to the Root Index Array.
    ///
    /// `node_index = 0` indicates parse failure (per
    /// [`crate::format::root_index::PARSE_FAILURE_SENTINEL`]); when used,
    /// at least one matching diagnostic must subsequently be emitted via
    /// [`Self::push_diagnostic`].
    pub fn push_root(&mut self, node_index: u32, source_offset_in_data: u32, base_offset: u32) {
        self.root_index_buffer.extend_from_slice(&node_index.to_le_bytes());
        self.root_index_buffer.extend_from_slice(&source_offset_in_data.to_le_bytes());
        self.root_index_buffer.extend_from_slice(&base_offset.to_le_bytes());
    }

    /// Append one diagnostic entry. The entries are sorted by `root_index`
    /// ascending at [`Self::finish`] (so callers may insert them in any
    /// order).
    pub fn push_diagnostic(&mut self, root_index: u32, message: &str) {
        let message_index = self.strings.intern_for_leaf(message);
        self.diagnostics.push((root_index, message_index.as_u32()));
    }

    /// Borrow the underlying string table builder. Used by per-Kind
    /// `write_*` helpers to intern delimiter / description strings.
    pub fn strings(&mut self) -> &mut StringTableBuilder<'arena> {
        &mut self.strings
    }

    /// Borrow the underlying Extended Data builder.
    pub fn extended(&mut self) -> &mut ExtendedDataBuilder<'arena> {
        &mut self.extended
    }

    /// Convenience: intern a string into the table. Returns the
    /// [`StringField`] that can be embedded in an Extended Data record.
    pub fn intern_string(&mut self, value: &str) -> StringField {
        self.strings.intern(value)
    }

    /// Intern a string and return its [`StringIndex`] (String Offsets table
    /// path). Used by string-leaf Kinds (`emit_string_node`) where the
    /// 30-bit index packed into Node Data is cheaper than a 6-byte ED
    /// record.
    pub fn intern_string_index(&mut self, value: &str) -> StringIndex {
        self.strings.intern_for_leaf(value)
    }

    /// Intern a string and return a [`LeafStringPayload`] suitable for
    /// [`Self::emit_string_node`]. Picks the inline path when both
    /// `value.len() <= 255` and the resulting String Data offset fits in
    /// 22 bits; otherwise falls back to the legacy [`StringIndex`] path.
    ///
    /// This is the Path B-leaf entry point: it elides the 8-byte append to
    /// `offsets_buffer` for short strings, replacing it with a pure
    /// `(offset, length)` pack into Node Data. See
    /// `tasks/benchmark/results/2026-04-23-…` for the design rationale.
    pub fn intern_string_payload(&mut self, value: &str) -> LeafStringPayload {
        let field = self.strings.intern(value);
        if (value.len() as u32) <= STRING_INLINE_LENGTH_MAX
            && field.offset <= STRING_INLINE_OFFSET_MAX
        {
            LeafStringPayload::Inline { offset: field.offset, length: value.len() as u8 }
        } else {
            LeafStringPayload::Index(self.strings.intern_for_leaf(value))
        }
    }

    /// Skip-dedup variant of [`Self::intern_string`] for callers who know
    /// their string is dominated by per-call unique content (description
    /// text, raw type source). Trades a small amount of binary growth
    /// (duplicate content stored twice) for the FxHash + lookup work the
    /// dedup map would otherwise perform on every call.
    pub fn intern_string_unique(&mut self, value: &str) -> StringField {
        self.strings.intern_unique(value)
    }

    /// Convenience: append a sourceText prefix and remember its byte length
    /// so [`Header.source_text_length`] is set correctly at [`Self::finish`].
    ///
    /// Also caches the appended region's data-buffer offset / length so
    /// the next [`Self::emit_block`] cycle can use [`Self::intern_source_slice`]
    /// to zero-copy intern source-derived strings (description text,
    /// tag names, raw type sources) without re-copying their bytes into
    /// the data buffer.
    pub fn append_source_text(&mut self, value: &str) -> u32 {
        let offset = self.strings.append_source_text(value);
        self.source_text_length = self.source_text_length.saturating_add(value.len() as u32);
        self.current_source_data_offset = offset;
        self.current_source_length = value.len() as u32;
        // Stash the source's start address as a usize so subsequent
        // `intern_source_slice_or_string` calls can detect borrowed sub-slices
        // via pointer arithmetic (no dereference). The pointer is valid for
        // the remainder of the current `parse_*_to_bytes` iteration; we never
        // store one across iterations.
        self.current_source_ptr = value.as_ptr() as usize;
        offset
    }

    /// Zero-copy intern: register a String Offsets entry that points into
    /// the **most recently appended** source text (see
    /// [`Self::append_source_text`]) without writing the bytes a second
    /// time into the String Data section.
    ///
    /// `source_byte_start` and `source_byte_end` are byte offsets relative
    /// to the start of the latest source text (i.e. matching the spans
    /// produced by `parse_block_into_data` when called with
    /// `base_offset = 0`). The caller must ensure the range falls within
    /// `[0, current_source_length]`.
    ///
    /// This is the Path-A optimization: it eliminates the per-call
    /// `data_buffer.extend_from_slice(value.as_bytes())` that
    /// [`Self::intern_string_unique`] does for every source-slice field
    /// (description, tag name, raw type, parameter name, …), trading the
    /// duplicated bytes for an offsets-only registration. See
    /// `.notes/binary-ast-emit-phase-format-analysis.md` for context.
    #[inline]
    pub fn intern_source_slice(
        &mut self,
        source_byte_start: u32,
        source_byte_end: u32,
    ) -> StringField {
        debug_assert!(
            source_byte_end <= self.current_source_length,
            "intern_source_slice end {source_byte_end} > current source length {}",
            self.current_source_length
        );
        let absolute_start = self.current_source_data_offset.saturating_add(source_byte_start);
        let absolute_end = self.current_source_data_offset.saturating_add(source_byte_end);
        self.strings.intern_at_offset(absolute_start, absolute_end)
    }

    /// String-leaf-targeted variant of [`Self::intern_source_slice`].
    ///
    /// Returns a [`StringIndex`] (allocates a String Offsets entry) so the
    /// caller can pass it to [`Self::emit_string_node`]. Used by
    /// description-line / type-line basic-mode emission where the source
    /// slice becomes the String-payload of a leaf node.
    ///
    /// The bytes themselves are **not** copied; only the (start, end) pair
    /// is appended to the offsets table.
    #[inline]
    pub fn intern_source_slice_for_leaf(
        &mut self,
        source_byte_start: u32,
        source_byte_end: u32,
    ) -> StringIndex {
        debug_assert!(
            source_byte_end <= self.current_source_length,
            "intern_source_slice_for_leaf end {source_byte_end} > current source length {}",
            self.current_source_length
        );
        let absolute_start = self.current_source_data_offset.saturating_add(source_byte_start);
        let absolute_end = self.current_source_data_offset.saturating_add(source_byte_end);
        self.strings.intern_at_offset_for_leaf(absolute_start, absolute_end)
    }

    /// Path B-leaf variant of [`Self::intern_source_slice_for_leaf`]: when
    /// the source slice is short enough (length ≤ 255 and offset ≤ 4 MB),
    /// returns a [`LeafStringPayload::Inline`] that packs `(offset, length)`
    /// directly into Node Data — skipping the 8-byte append to
    /// `offsets_buffer` and the [`StringIndex`] payload allocation entirely.
    /// Long slices fall back to the legacy `intern_at_offset_for_leaf` path.
    #[inline]
    pub fn intern_source_slice_for_leaf_payload(
        &mut self,
        source_byte_start: u32,
        source_byte_end: u32,
    ) -> LeafStringPayload {
        debug_assert!(
            source_byte_end <= self.current_source_length,
            "intern_source_slice_for_leaf_payload end {source_byte_end} > current source length {}",
            self.current_source_length
        );
        let absolute_start = self.current_source_data_offset.saturating_add(source_byte_start);
        let absolute_end = self.current_source_data_offset.saturating_add(source_byte_end);
        let length = absolute_end - absolute_start;
        if length <= STRING_INLINE_LENGTH_MAX && absolute_start <= STRING_INLINE_OFFSET_MAX {
            LeafStringPayload::Inline { offset: absolute_start, length: length as u8 }
        } else {
            LeafStringPayload::Index(
                self.strings.intern_at_offset_for_leaf(absolute_start, absolute_end),
            )
        }
    }

    /// Intern `value` choosing the cheapest of three paths automatically,
    /// using the supplied `span` as a hint for the zero-copy case.
    ///
    /// 1. **Common-string fast path** — when `value` is a pre-seeded
    ///    common string (`*`, `*/`, `param`, …), return its predetermined
    ///    index with no state mutation. Same cost as
    ///    [`Self::intern_string`] on a hit.
    /// 2. **Zero-copy source slice** — when `value.len()` equals the span
    ///    length and the span fits inside the most recently appended source
    ///    text, register an offsets-only entry pointing at those source
    ///    bytes (no `data_buffer` copy, no HashMap probe). Equivalent to
    ///    [`Self::intern_source_slice`] but autoguarded.
    /// 3. **Unique fresh entry** — otherwise (synthesized strings,
    ///    quote-stripped variants whose lengths differ from the span,
    ///    parent-spanning aggregates whose span covers more than just
    ///    `value`), append `value` bytes via [`Self::intern_string_unique`].
    ///
    /// The length-equality check is what makes path 2 safe even when the
    /// caller doesn't know up-front whether `value` is a true sub-slice of
    /// the source (e.g. `TypeProperty.value` may have its outer quotes
    /// stripped while its span still includes them). See
    /// `.notes/binary-ast-emit-intern-audit.md` for the per-caller analysis.
    ///
    /// `#[inline]` because the per-Kind `parse_*` callers in
    /// `parser/context.rs` invoke this once per emitted string field; cross-
    /// crate inlining is not implicit for `pub fn` even at `-O3`.
    #[inline]
    pub fn intern_source_or_string(&mut self, value: &str, span: Span) -> StringField {
        if let Some(idx) = lookup_common(value) {
            return common_string_field(idx);
        }
        let span_len = span.end.saturating_sub(span.start);
        if span_len as usize == value.len() && span.end <= self.current_source_length {
            return self.intern_source_slice(span.start, span.end);
        }
        self.strings.intern_unique(value)
    }

    /// String-leaf-targeted variant of [`Self::intern_source_or_string`]
    /// — returns a [`StringIndex`] suitable for [`Self::emit_string_node`].
    ///
    /// Mirrors the same three-path decision tree (common-string fast path,
    /// zero-copy source slice, dedup'd unique entry) but allocates a
    /// String Offsets index for the result.
    #[inline]
    pub fn intern_source_or_string_for_leaf(&mut self, value: &str, span: Span) -> StringIndex {
        if let Some(idx) = lookup_common(value) {
            return StringIndex::from_u32(idx).expect("common index in range");
        }
        let span_len = span.end.saturating_sub(span.start);
        if span_len as usize == value.len() && span.end <= self.current_source_length {
            return self.intern_source_slice_for_leaf(span.start, span.end);
        }
        self.strings.intern_for_leaf(value)
    }

    /// Path B-leaf variant of [`Self::intern_source_or_string_for_leaf`].
    ///
    /// Returns a [`LeafStringPayload`] that picks the inline path
    /// (`TypeTag::StringInline`) when the resulting string fits the
    /// `(offset ≤ 4 MB, length ≤ 255)` constraints, falling back to the
    /// legacy `StringIndex` path (`TypeTag::String`) otherwise.
    ///
    /// Implements the same three-path decision tree as the non-payload
    /// sibling: common-string fast path, zero-copy source slice, dedup'd
    /// unique entry. Each path is short-circuited to inline when the slot
    /// fits the inline encoding.
    #[inline]
    pub fn intern_source_or_string_for_leaf_payload(
        &mut self,
        value: &str,
        span: Span,
    ) -> LeafStringPayload {
        if let Some(idx) = lookup_common(value) {
            // COMMON_STRINGS live at the start of the data buffer (well
            // within 4 MB) and are all <= 10 bytes, so they always inline.
            let field = common_string_field(idx);
            return LeafStringPayload::Inline { offset: field.offset, length: field.length as u8 };
        }
        let span_len = span.end.saturating_sub(span.start);
        if span_len as usize == value.len() && span.end <= self.current_source_length {
            return self.intern_source_slice_for_leaf_payload(span.start, span.end);
        }
        // Synthesized / quote-stripped value → dedup via HashMap. Inline
        // when the dedup'd field fits the (offset ≤ 4 MB, length ≤ 255)
        // window; otherwise fall back to the legacy leaf path.
        let field = self.strings.intern(value);
        if (value.len() as u32) <= STRING_INLINE_LENGTH_MAX
            && field.offset <= STRING_INLINE_OFFSET_MAX
        {
            LeafStringPayload::Inline { offset: field.offset, length: value.len() as u8 }
        } else {
            LeafStringPayload::Index(self.strings.intern_for_leaf(value))
        }
    }

    /// Span-less sibling of [`Self::intern_source_or_string`] for callers that
    /// hold an `&str` without an explicit byte range (e.g. fields surfaced
    /// by `Option<&str>` getters where the parser merged or normalized the
    /// underlying source bytes).
    ///
    /// Detection is via pointer arithmetic against the most recently
    /// appended source text — when `value` lies inside that buffer's
    /// allocation, register an offsets-only entry pointing at it; otherwise
    /// fall through to the common-string fast path or a unique fresh entry.
    /// The pointer comparison never dereferences either pointer, so the
    /// check is safe even when the source allocation has since been
    /// mutated (it cannot have moved while we still hold the borrow).
    ///
    /// Pointer-arithmetic identification handles the synthesized vs
    /// source-slice ambiguity in `normalize_lines` results: single-line
    /// descriptions remain a sub-slice of the source and take path 2;
    /// multi-line joins live in the parser's scratch String (separate
    /// allocation) and fall through to path 3.
    ///
    /// `#[inline]` because this is the single hottest writer entry point
    /// (per `examples/profile_parse_batch.rs` samply runs ≈ 14.8% self
    /// time): cross-crate inlining with the per-Kind `parse_*` callers
    /// in `parser/context.rs` lets LLVM fold the lookup_common hit and
    /// the pointer-comparison branches into the surrounding emission.
    #[inline]
    pub fn intern_source_slice_or_string(&mut self, value: &str) -> StringField {
        if let Some(idx) = lookup_common(value) {
            return common_string_field(idx);
        }
        let value_ptr = value.as_ptr() as usize;
        let source_start = self.current_source_ptr;
        if source_start != 0 {
            let source_end = source_start.saturating_add(self.current_source_length as usize);
            let value_end = value_ptr.saturating_add(value.len());
            if value_ptr >= source_start && value_end <= source_end {
                let offset = (value_ptr - source_start) as u32;
                return self.intern_source_slice(offset, offset + value.len() as u32);
            }
        }
        self.strings.intern_unique(value)
    }

    /// String-leaf-targeted variant of [`Self::intern_source_slice_or_string`]
    /// — returns a [`StringIndex`] suitable for [`Self::emit_string_node`].
    #[inline]
    pub fn intern_source_slice_or_string_for_leaf(&mut self, value: &str) -> StringIndex {
        if let Some(idx) = lookup_common(value) {
            return StringIndex::from_u32(idx).expect("common index in range");
        }
        let value_ptr = value.as_ptr() as usize;
        let source_start = self.current_source_ptr;
        if source_start != 0 {
            let source_end = source_start.saturating_add(self.current_source_length as usize);
            let value_end = value_ptr.saturating_add(value.len());
            if value_ptr >= source_start && value_end <= source_end {
                let offset = (value_ptr - source_start) as u32;
                return self.intern_source_slice_for_leaf(offset, offset + value.len() as u32);
            }
        }
        self.strings.intern_for_leaf(value)
    }

    /// Number of node records currently in the Nodes section (including the
    /// `node[0]` sentinel).
    #[inline]
    #[must_use]
    pub fn node_count(&self) -> u32 {
        (self.nodes_buffer.len() / NODE_RECORD_SIZE) as u32
    }

    /// Open a NodeList cursor at `(parent_ext + slot_offset)`.
    ///
    /// Pattern (per parent that owns one or more lists):
    /// 1. Emit the parent via `write_*` helper. The helper now returns
    ///    `(NodeIndex, ExtOffset)` so the caller can address the ED block.
    /// 2. `begin_node_list_at(ext, slot_offset)` opens a cursor for one list.
    /// 3. After each child emit, [`Self::record_list_child`] updates the
    ///    cursor's head/count.
    /// 4. [`Self::finalize_node_list`] patches `(head: u32, count: u16)` into
    ///    the parent's ED block at the recorded slot.
    #[inline]
    #[must_use]
    pub fn begin_node_list_at(&self, parent_ext: ExtOffset, slot_offset: usize) -> ListInProgress {
        ListInProgress { parent_ext, slot_offset, head_index: 0, count: 0 }
    }

    /// Record one child added to the in-progress list. `child_index` is the
    /// `u32` returned by the per-child `write_*` helper (or `emit_type_node`).
    #[inline]
    pub fn record_list_child(&mut self, list: &mut ListInProgress, child_index: u32) {
        if list.count == 0 {
            list.head_index = child_index;
        }
        list.count = list.count.checked_add(1).expect("NodeList exceeds u16::MAX elements");
    }

    /// Patch `(head_index: u32, count: u16)` into the parent's Extended Data
    /// block at `(parent_ext + slot_offset)`. Must be called exactly once per
    /// list opened via [`Self::begin_node_list_at`].
    pub fn finalize_node_list(&mut self, list: ListInProgress) {
        let base = list.parent_ext.as_u32() as usize + list.slot_offset;
        let buf = &mut self.extended.buffer[base..base + 6];
        buf[0..4].copy_from_slice(&list.head_index.to_le_bytes());
        buf[4..6].copy_from_slice(&list.count.to_le_bytes());
    }

    /// Number of roots currently in the Root Index Array.
    #[inline]
    #[must_use]
    pub fn root_count(&self) -> u32 {
        (self.root_index_buffer.len() / ROOT_INDEX_ENTRY_SIZE) as u32
    }

    /// Reference to the arena passed at [`Self::new`]. Useful when
    /// per-Kind helpers need scratch allocations.
    #[inline]
    #[must_use]
    pub fn arena(&self) -> &'arena Allocator {
        self.arena
    }

    /// Finish writing and produce the concatenated Binary AST byte stream.
    ///
    /// At this point the writer:
    /// - sorts the diagnostic entries by `root_index` ascending,
    /// - resolves each section's start offset and patches the [`Header`],
    /// - writes Header (40 bytes) + all section buffers in canonical order
    ///   (Root index array → String Offsets → String Data → Extended Data
    ///   → Diagnostics → Nodes).
    ///
    /// The returned `Vec<u8>` is owned (not arena-backed) so it can be sent
    /// across NAPI/WASM boundaries without lifetime concerns.
    #[must_use]
    pub fn finish(mut self) -> Vec<u8> {
        let layout = self.prepare_finish_layout();
        let mut out: Vec<u8> = vec![0u8; layout.total_size];
        write_finish_layout(&self, &layout, &mut out);
        out
    }

    /// Finish writing into the writer's owning arena and return the bytes
    /// as `&'arena [u8]`. Avoids the heap [`Vec<u8>`] allocation +
    /// [`Allocator::alloc_slice_copy`] step that callers like [`crate::parser::parse`]
    /// would otherwise pay to materialize a `&'arena [u8]` view of
    /// [`Self::finish`]'s output.
    ///
    /// Output bytes are byte-for-byte identical to [`Self::finish`] for the
    /// same writer state.
    #[must_use]
    pub fn finish_into_arena(self) -> &'arena [u8] {
        // Delegate to the borrow-based variant. `self` is dropped at the
        // end of this scope; its arena-backed buffers are kept alive by
        // the arena, and the returned slice is valid for `'arena`.
        let mut writer = self;
        writer.finish_into_arena_reusing()
    }

    /// Like [`Self::finish_into_arena`] but takes `&mut self` instead of
    /// consuming, so the writer can subsequently be [`Self::reset`]ed and
    /// reused for the next per-comment parse.
    ///
    /// Used by [`crate::parser::parse_into`] /
    /// [`crate::parser::parse_batch_into`] to amortize writer construction
    /// across many parse calls.
    ///
    /// The returned slice borrows from the writer's arena (lifetime
    /// `'arena`), independent of the writer itself, so [`Self::reset`] can
    /// be called immediately afterwards without invalidating it.
    ///
    /// Output bytes are byte-for-byte identical to [`Self::finish`] for the
    /// same writer state.
    #[must_use]
    pub fn finish_into_arena_reusing(&mut self) -> &'arena [u8] {
        let layout = self.prepare_finish_layout();
        let arena = self.arena;
        let mut out: ArenaVec<'arena, u8> = ArenaVec::with_capacity_in(layout.total_size, arena);
        out.resize(layout.total_size, 0);
        write_finish_layout(self, &layout, &mut out);
        out.into_bump_slice()
    }

    /// Sort diagnostics + compute layout. Mutating helper shared by
    /// [`Self::finish`] and [`Self::finish_into_arena`] — the only
    /// pre-write side effect either path needs.
    fn prepare_finish_layout(&mut self) -> FinishLayout {
        // -- 1. sort diagnostics by root_index ascending --------------------
        // Empty skip: most batches finish without diagnostics (parse success
        // path), so avoid the function-call overhead entirely. Unstable
        // sort is sufficient because each push is unique enough; the order
        // among entries with the same root_index is not load-bearing.
        if !self.diagnostics.is_empty() {
            self.diagnostics.sort_unstable_by_key(|(root_index, _)| *root_index);
        }

        // -- 2. compute counts and section sizes ----------------------------
        let root_array_size = self.root_index_buffer.len();
        let string_offsets_size = self.strings.offsets_buffer.len();
        let string_data_size = self.strings.data_buffer.len();
        let extended_data_size = self.extended.buffer.len();
        let diagnostics_size = diagnostics::section_size(self.diagnostics.len());
        let nodes_size = self.nodes_buffer.len();

        // -- 3. compute absolute section offsets ----------------------------
        //
        // Padding requirement: every section that contains u32 reads must
        // start at a 4-byte aligned offset so the JS-side decoder can use
        // `Uint32Array[idx]` (5-10× faster than `DataView.getUint32`)
        // instead of going through DataView. The `string_data` section
        // contains arbitrary UTF-8 byte lengths, so the boundary right
        // after it (Extended Data start) and every later boundary need
        // to round up.
        let root_array_offset = HEADER_SIZE as u32;
        let string_offsets_offset = root_array_offset + root_array_size as u32;
        let string_data_offset = string_offsets_offset + string_offsets_size as u32;
        // Pad after String Data so Extended Data starts 4-byte aligned.
        let extended_data_offset = align_up_4(string_data_offset + string_data_size as u32);
        // Pad after Extended Data so Diagnostics starts 4-byte aligned.
        let diagnostics_offset = align_up_4(extended_data_offset + extended_data_size as u32);
        // diagnostics_size is `4 + 8M` (always 4-aligned), so the next
        // boundary is automatically 4-aligned. Compute defensively anyway.
        let nodes_offset = align_up_4(diagnostics_offset + diagnostics_size as u32);

        let total_size = nodes_offset as usize + nodes_size;

        FinishLayout {
            root_array_offset,
            string_offsets_offset,
            string_data_offset,
            extended_data_offset,
            diagnostics_offset,
            nodes_offset,
            root_array_size,
            string_offsets_size,
            string_data_size,
            extended_data_size,
            nodes_size,
            total_size,
        }
    }
}

/// Pre-computed section offsets + sizes for [`BinaryWriter::finish`] /
/// [`BinaryWriter::finish_into_arena`]. Computed once from
/// [`BinaryWriter::prepare_finish_layout`] and consumed by
/// [`write_finish_layout`].
struct FinishLayout {
    root_array_offset: u32,
    string_offsets_offset: u32,
    string_data_offset: u32,
    extended_data_offset: u32,
    diagnostics_offset: u32,
    nodes_offset: u32,
    root_array_size: usize,
    string_offsets_size: usize,
    string_data_size: usize,
    extended_data_size: usize,
    nodes_size: usize,
    total_size: usize,
}

/// Write the header + every section into a pre-allocated, zero-initialized
/// buffer of exactly [`FinishLayout::total_size`] bytes. Padding regions are
/// covered by the caller's zero-init.
fn write_finish_layout(writer: &BinaryWriter<'_>, layout: &FinishLayout, out: &mut [u8]) {
    debug_assert_eq!(out.len(), layout.total_size);

    let node_count = writer.node_count();
    let root_count = writer.root_count();
    let diagnostic_count = writer.diagnostics.len() as u32;

    // -- Header ---------------------------------------------------------
    out[VERSION_OFFSET] = writer.header.version;
    out[FLAGS_OFFSET] = writer.header.flags;
    // bytes 2-3 already zero (reserved)
    write_u32(out, ROOT_ARRAY_OFFSET_FIELD, layout.root_array_offset);
    write_u32(out, STRING_OFFSETS_OFFSET_FIELD, layout.string_offsets_offset);
    write_u32(out, STRING_DATA_OFFSET_FIELD, layout.string_data_offset);
    write_u32(out, EXTENDED_DATA_OFFSET_FIELD, layout.extended_data_offset);
    write_u32(out, DIAGNOSTICS_OFFSET_FIELD, layout.diagnostics_offset);
    write_u32(out, NODES_OFFSET_FIELD, layout.nodes_offset);
    write_u32(out, NODE_COUNT_FIELD, node_count);
    write_u32(out, SOURCE_TEXT_LENGTH_FIELD, writer.source_text_length);
    write_u32(out, ROOT_COUNT_FIELD, root_count);

    // -- Root index array ----------------------------------------------
    let root_start = layout.root_array_offset as usize;
    out[root_start..root_start + layout.root_array_size].copy_from_slice(&writer.root_index_buffer);

    // -- String Offsets / Data -----------------------------------------
    let so_start = layout.string_offsets_offset as usize;
    out[so_start..so_start + layout.string_offsets_size]
        .copy_from_slice(&writer.strings.offsets_buffer);

    let sd_start = layout.string_data_offset as usize;
    out[sd_start..sd_start + layout.string_data_size].copy_from_slice(&writer.strings.data_buffer);
    // String Data → Extended Data padding: already zero (out was zero-init).

    // -- Extended Data --------------------------------------------------
    let ed_start = layout.extended_data_offset as usize;
    out[ed_start..ed_start + layout.extended_data_size].copy_from_slice(&writer.extended.buffer);
    // Extended Data → Diagnostics padding: already zero.

    // -- Diagnostics: count header + 8-byte entries --------------------
    let diag_start = layout.diagnostics_offset as usize;
    out[diag_start..diag_start + 4].copy_from_slice(&diagnostic_count.to_le_bytes());
    let mut cursor = diag_start + 4;
    for (root_index, message_index) in &writer.diagnostics {
        out[cursor..cursor + 4].copy_from_slice(&root_index.to_le_bytes());
        out[cursor + 4..cursor + 8].copy_from_slice(&message_index.to_le_bytes());
        cursor += 8;
    }
    // Diagnostics → Nodes padding: already zero.

    // -- Nodes ----------------------------------------------------------
    let nodes_start = layout.nodes_offset as usize;
    out[nodes_start..nodes_start + layout.nodes_size].copy_from_slice(&writer.nodes_buffer);

    debug_assert_eq!(layout.extended_data_offset & 3, 0, "Extended Data must be 4-aligned");
    debug_assert_eq!(layout.diagnostics_offset & 3, 0, "Diagnostics must be 4-aligned");
    debug_assert_eq!(layout.nodes_offset & 3, 0, "Nodes must be 4-aligned");
}

/// Round `value` up to the next multiple of 4.
#[inline]
fn align_up_4(value: u32) -> u32 {
    (value + 3) & !3
}

/// Write a little-endian u32 at the given byte offset.
#[inline]
fn write_u32(buf: &mut [u8], offset: usize, value: u32) {
    buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::source_file::LazySourceFile;

    #[test]
    fn empty_buffer_roundtrips_through_lazy_source_file() {
        use crate::writer::string_table::COMMON_STRING_COUNT;

        let arena = Allocator::default();
        let writer = BinaryWriter::new(&arena);
        assert_eq!(writer.node_count(), 1, "sentinel node[0] is pre-written");
        assert_eq!(writer.root_count(), 0);
        assert!(!writer.compat_mode());
        // The string table is seeded with `COMMON_STRING_COUNT` entries
        // (delimiters, whitespace, common tag names) so per-call
        // `intern_for_leaf` can skip the HashMap.
        assert_eq!(writer.strings.len(), COMMON_STRING_COUNT);

        let bytes = writer.finish();

        let sf = LazySourceFile::new(&bytes).expect("empty buffer must parse");
        assert_eq!(sf.node_count, 1);
        assert_eq!(sf.root_count, 0);
        assert!(!sf.compat_mode);
        // Sections sit in canonical order; offsets shift by the size of
        // the common-string prelude.
        assert_eq!(sf.root_array_offset, 40);
        assert_eq!(sf.string_offsets_offset, 40);
        // Each interned entry occupies 8 bytes in the offsets table.
        let prelude_offsets_bytes = COMMON_STRING_COUNT * 8;
        assert_eq!(sf.string_data_offset, 40 + prelude_offsets_bytes);
    }

    #[test]
    fn set_compat_mode_round_trips() {
        let arena = Allocator::default();
        let mut writer = BinaryWriter::new(&arena);
        writer.set_compat_mode(true);
        let bytes = writer.finish();
        assert_eq!(bytes[FLAGS_OFFSET] & COMPAT_MODE_BIT, COMPAT_MODE_BIT);

        let sf = LazySourceFile::new(&bytes).unwrap();
        assert!(sf.compat_mode);
    }

    #[test]
    fn push_root_writes_12_byte_entry_in_canonical_order() {
        let arena = Allocator::default();
        let mut writer = BinaryWriter::new(&arena);
        writer.push_root(1, 0, 100);
        writer.push_root(0, 7, 200); // parse failure sentinel
        assert_eq!(writer.root_count(), 2);

        let bytes = writer.finish();
        let sf = LazySourceFile::new(&bytes).unwrap();
        assert_eq!(sf.root_count, 2);
        // Each entry is 12 bytes; the first one starts at root_array_offset.
        let root0 = sf.root_array_offset as usize;
        assert_eq!(read_u32_at(&bytes, root0), 1, "node_index of root 0");
        assert_eq!(read_u32_at(&bytes, root0 + 4), 0, "source_offset_in_data");
        assert_eq!(read_u32_at(&bytes, root0 + 8), 100, "base_offset");
        assert_eq!(read_u32_at(&bytes, root0 + 12), 0, "node_index of root 1 (failure)");
        assert_eq!(read_u32_at(&bytes, root0 + 20), 200);
    }

    #[test]
    fn push_diagnostic_sorts_by_root_index() {
        let arena = Allocator::default();
        let mut writer = BinaryWriter::new(&arena);
        // Insert out of order; finish() must sort ascending by root_index.
        writer.push_diagnostic(2, "second");
        writer.push_diagnostic(0, "zero");
        writer.push_diagnostic(1, "one");

        let bytes = writer.finish();
        let sf = LazySourceFile::new(&bytes).unwrap();
        let diag_offset = sf.diagnostics_offset as usize;
        assert_eq!(read_u32_at(&bytes, diag_offset), 3, "diagnostic count");

        // First entry: root_index = 0
        assert_eq!(read_u32_at(&bytes, diag_offset + 4), 0);
        // Second entry: root_index = 1
        assert_eq!(read_u32_at(&bytes, diag_offset + 4 + 8), 1);
        // Third entry: root_index = 2
        assert_eq!(read_u32_at(&bytes, diag_offset + 4 + 16), 2);
    }

    #[test]
    fn finish_records_source_text_length() {
        let arena = Allocator::default();
        let mut writer = BinaryWriter::new(&arena);
        let _ = writer.append_source_text("/** @param x */");
        let bytes = writer.finish();
        let sf = LazySourceFile::new(&bytes).unwrap();
        // sourceText length is in bytes, not chars; ASCII-only here so they match.
        let expected = "/** @param x */".len() as u32;
        assert_eq!(read_u32_at(&bytes, SOURCE_TEXT_LENGTH_FIELD), expected);
        // Spot-check the LazySourceFile path doesn't panic on the same buffer.
        assert_eq!(sf.node_count, 1);
    }

    fn read_u32_at(buf: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap())
    }
}
