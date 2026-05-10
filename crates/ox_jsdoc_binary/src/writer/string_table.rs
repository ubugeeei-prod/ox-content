// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! String table builder used by [`super::BinaryWriter`].
//!
//! Owns the **String Offsets** + **String Data** buffers. Strings are
//! deduplicated by content via a HashMap so that repeated values
//! (`param`, `returns`, etc.) shared across a batch of comments are
//! interned only once.
//!
//! Two intern result types coexist:
//!
//! - [`StringIndex`] — index into the String Offsets table; used for
//!   string-leaf nodes (TypeTag::String) and the diagnostics section's
//!   `message_index`. Embedded as a 30-bit payload in Node Data, so each
//!   leaf node costs 0 extra bytes beyond its 24-byte record.
//! - [`crate::format::string_field::StringField`] — inline `(offset,
//!   length)` pair returned for Extended Data string slots; readers
//!   bypass the offsets table entirely.
//!
//! Both forms share the same `data_buffer`; dedup hits return whichever
//! representation the caller asked for, allocating the other lazily on
//! first use. See `design/007-binary-ast/format.md#string-table`.

use core::num::NonZeroU32;
use std::sync::OnceLock;

use oxc_allocator::{Allocator, Vec as ArenaVec};
use rustc_hash::FxHashMap;

use crate::format::string_field::StringField;

/// Index into the **String Offsets** table.
///
/// Newtype wrapper so a string index cannot accidentally be confused with a
/// node index, an extended-data offset, or a raw `u32`. String-type Node
/// Data leaves can reference up to 30 bits (the
/// [`crate::format::node_record::STRING_PAYLOAD_NONE_SENTINEL`] reserves
/// the last value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringIndex(NonZeroU32);

impl StringIndex {
    /// Construct a [`StringIndex`] from a raw `u32`.
    ///
    /// Returns `None` only when `value == u32::MAX` (so `value + 1`
    /// overflows the `NonZeroU32` storage). Index 0 itself is a valid
    /// string index, so the wrapper internally stores `index + 1`.
    #[inline]
    #[must_use]
    pub const fn from_u32(value: u32) -> Option<Self> {
        match NonZeroU32::new(value.wrapping_add(1)) {
            Some(nz) => Some(StringIndex(nz)),
            None => None,
        }
    }

    /// Get the raw `u32` index.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0.get() - 1
    }
}

/// String-leaf payload — selects between the inline `(offset, length)` packing
/// (fast path, `TypeTag::StringInline`) and the legacy `StringIndex` (fallback,
/// `TypeTag::String`). Used by `emit_string_node` to pick the right Node Data
/// tag at write time.
///
/// **Selection rule**: short strings (`length <= 255`) whose data-buffer
/// offset fits in 22 bits use [`Self::Inline`]; everything else falls back to
/// [`Self::Index`]. The decoder dispatches on the same boundary.
#[derive(Debug, Clone, Copy)]
pub enum LeafStringPayload {
    /// Short string packed directly into Node Data (`TypeTag::StringInline`).
    Inline {
        /// Byte offset within the String Data section. Must fit in 22 bits.
        offset: u32,
        /// UTF-8 byte length (0..=255).
        length: u8,
    },
    /// Long string or out-of-range offset; resolved through the String
    /// Offsets table (`TypeTag::String`).
    Index(StringIndex),
}

/// Number of common strings pre-interned by [`StringTableBuilder::new`].
/// Useful for tests that assert against the post-construction state.
pub const COMMON_STRING_COUNT: u32 = COMMON_STRINGS.len() as u32;

// -- Per-entry [`COMMON_STRINGS`] indices ------------------------------------
//
// Hot-path callers (e.g. `parser/context.rs::emit_block_inner`) look up the
// pre-computed StringField via [`common_string_field`]. Using the named
// constants instead of the generic [`lookup_common`] / [`StringTableBuilder::intern`]
// path skips the length-bucketed match + dedup probe entirely (~2% self time
// in the parse_batch_to_bytes profile).

/// Index of `""` inside [`COMMON_STRINGS`].
pub const COMMON_EMPTY: u32 = 0;
/// Index of `" "` (single space) inside [`COMMON_STRINGS`].
pub const COMMON_SPACE: u32 = 1;
/// Index of `"*"` (margin delimiter) inside [`COMMON_STRINGS`].
pub const COMMON_STAR: u32 = 2;
/// Index of `"*/"` (block terminal) inside [`COMMON_STRINGS`].
pub const COMMON_SLASH_STAR: u32 = 3;
/// Index of `"\n"` inside [`COMMON_STRINGS`].
pub const COMMON_LF: u32 = 4;
/// Index of `"\t"` inside [`COMMON_STRINGS`].
pub const COMMON_TAB: u32 = 5;
/// Index of `"\r\n"` inside [`COMMON_STRINGS`].
pub const COMMON_CRLF: u32 = 6;
/// Index of `"/**"` inside [`COMMON_STRINGS`].
pub const COMMON_SLASH_STAR_STAR: u32 = 7;

/// Strings that the writer pre-interns at construction so the common case
/// (delimiters, whitespace, well-known tag names) skips the HashMap entirely.
///
/// Each entry's array index is its predetermined `StringIndex`. The
/// length-bucketed `lookup_common` helper below MUST stay in sync — adding
/// or reordering entries here without updating it will cause the fast path
/// to return wrong indices.
const COMMON_STRINGS: &[&str] = &[
    // 0..=4: source-preserving leaves
    "",
    " ",
    "*",
    "*/",
    "\n",
    // 5..=7: less common whitespace
    "\t",
    "\r\n",
    "/**",
    // 8..=27: most common JSDoc tag names (eslint-plugin-jsdoc usage data)
    "param",
    "returns",
    "return",
    "throws",
    "type",
    "see",
    "example",
    "deprecated",
    "since",
    "default",
    "author",
    "internal",
    "private",
    "public",
    "protected",
    "static",
    "this",
    "override",
    "readonly",
    "yields",
];

/// Length-bucketed perfect-hash style match for [`COMMON_STRINGS`]. Returns
/// the predetermined index when `value` is a common string, otherwise `None`.
///
/// The `match value.len()` arm lets the compiler skip whole comparison
/// chains for non-matching lengths in one branch, which is what makes this
/// path cheaper than the generic `HashMap::get`.
#[inline]
pub(crate) fn lookup_common(value: &str) -> Option<u32> {
    match value.len() {
        0 => Some(0), // ""
        1 => match value.as_bytes()[0] {
            b' ' => Some(1),
            b'*' => Some(2),
            b'\n' => Some(4),
            b'\t' => Some(5),
            _ => None,
        },
        2 => match value {
            "*/" => Some(3),
            "\r\n" => Some(6),
            _ => None,
        },
        3 => match value {
            "/**" => Some(7),
            "see" => Some(13),
            _ => None,
        },
        4 => match value {
            "type" => Some(12),
            "this" => Some(24),
            _ => None,
        },
        5 => match value {
            "param" => Some(8),
            "since" => Some(16),
            _ => None,
        },
        6 => match value {
            "return" => Some(10),
            "throws" => Some(11),
            "author" => Some(18),
            "public" => Some(21),
            "static" => Some(23),
            "yields" => Some(27),
            _ => None,
        },
        7 => match value {
            "returns" => Some(9),
            "example" => Some(14),
            "default" => Some(17),
            "private" => Some(20),
            _ => None,
        },
        8 => match value {
            "internal" => Some(19),
            "readonly" => Some(26),
            "override" => Some(25),
            _ => None,
        },
        9 => match value {
            "protected" => Some(22),
            _ => None,
        },
        10 => match value {
            "deprecated" => Some(15),
            _ => None,
        },
        _ => None,
    }
}

/// Pre-computed `(start, end)` u32-LE pairs for [`COMMON_STRINGS`], cached
/// on first use so every [`StringTableBuilder::new`] is just two memcpys
/// rather than 28 individual HashMap inserts and arena `alloc_str` calls.
fn prelude_offsets() -> &'static [u8] {
    static CACHE: OnceLock<Vec<u8>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut buf = Vec::with_capacity(COMMON_STRINGS.len() * 8);
        let mut pos = 0u32;
        for s in COMMON_STRINGS {
            let start = pos;
            pos += s.len() as u32;
            let end = pos;
            buf.extend_from_slice(&start.to_le_bytes());
            buf.extend_from_slice(&end.to_le_bytes());
        }
        buf
    })
}

/// Pre-computed concatenated bytes for [`COMMON_STRINGS`] — same caching
/// rationale as [`prelude_offsets`].
fn prelude_data() -> &'static [u8] {
    static CACHE: OnceLock<Vec<u8>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut buf = Vec::new();
        for s in COMMON_STRINGS {
            buf.extend_from_slice(s.as_bytes());
        }
        buf
    })
}

/// Pre-computed [`StringField`] for each entry in [`COMMON_STRINGS`].
///
/// Built once at first use by walking `COMMON_STRINGS` and tracking the
/// running data-buffer offset. Returned to callers by [`lookup_common`] hits
/// without touching `data_buffer` or the dedup map.
fn common_string_fields() -> &'static [StringField] {
    static CACHE: OnceLock<Vec<StringField>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut fields = Vec::with_capacity(COMMON_STRINGS.len());
        let mut pos = 0u32;
        for s in COMMON_STRINGS {
            let length = u16::try_from(s.len()).expect("common string longer than u16");
            fields.push(StringField { offset: pos, length });
            pos += s.len() as u32;
        }
        fields
    })
}

/// Resolve the pre-computed [`StringField`] for the [`lookup_common`]
/// fast-path index `idx`. Used by writer combined-intern helpers to
/// short-circuit the COMMON_STRINGS dedup without copying bytes.
#[inline]
pub fn common_string_field(idx: u32) -> StringField {
    common_string_fields()[idx as usize]
}

/// One entry in the dedup table — tracks both representations a string
/// might be requested as. The `index` is `None` until a string-leaf caller
/// first asks for the [`StringIndex`] form (lazy offsets-table allocation).
#[derive(Debug, Clone, Copy)]
struct DedupEntry {
    field: StringField,
    index: Option<StringIndex>,
}

/// Builds the String Offsets + String Data buffers incrementally.
///
/// Internally:
/// - `data_buffer` accumulates UTF-8 bytes (zero-copy slice from source
///   text whenever possible),
/// - `offsets_buffer` accumulates `(start, end)` u32 pairs (8 bytes each)
///   only for strings actually requested as [`StringIndex`],
/// - a hash map from `&'arena str` to a [`DedupEntry`] enables both forms
///   of dedup without redundant byte writes.
pub struct StringTableBuilder<'arena> {
    /// `8K` bytes of `(start, end)` u32 pairs, one per StringIndex requested.
    pub(crate) offsets_buffer: ArenaVec<'arena, u8>,
    /// Raw concatenated UTF-8 bytes for every interned string and every
    /// appended sourceText.
    pub(crate) data_buffer: ArenaVec<'arena, u8>,
    /// Number of [`StringIndex`] entries allocated so far. Equal to
    /// `offsets_buffer.len() / 8`.
    pub(crate) count: u32,
    /// Reference to the underlying arena, used to allocate dedup keys with
    /// `'arena` lifetime.
    arena: &'arena Allocator,
    /// Dedup map: arena-allocated string → its [`DedupEntry`]. Stored on
    /// the `std` heap (not the arena) because `HashMap` cannot live inside
    /// `oxc_allocator` without a custom allocator binding.
    ///
    /// Uses `FxHashMap` (rustc-hash) instead of the default `SipHash` for
    /// ~2x faster hashing on the unique-string slow path.
    dedup: FxHashMap<&'arena str, DedupEntry>,
    /// 1-slot recent-call cache. Stores the most recently interned
    /// `&str`'s pointer + length and the resulting `(StringField, Option<StringIndex>)`.
    /// Hot when callers re-intern the same literal back-to-back
    /// (e.g. `intern("")` for empty source slots, or `intern("param")` /
    /// `intern("returns")` repeatedly across a batch).
    ///
    /// The pointer is only ever compared (never dereferenced) and its
    /// underlying allocation outlives the builder by virtue of the arena.
    last_key_ptr: *const u8,
    last_key_len: usize,
    last_entry: DedupEntry,
}

impl<'arena> StringTableBuilder<'arena> {
    /// Create a builder seeded with [`COMMON_STRINGS`] at fixed indices.
    ///
    /// The seeding is two cheap memcpys (offsets + data); the dedup
    /// HashMap stays empty so common strings never enter it — the fast
    /// path in [`Self::intern`] returns early via [`lookup_common`]
    /// before consulting the map.
    #[must_use]
    pub fn new(arena: &'arena Allocator) -> Self {
        let mut offsets_buffer = ArenaVec::new_in(arena);
        offsets_buffer.extend_from_slice(prelude_offsets());

        let mut data_buffer = ArenaVec::new_in(arena);
        data_buffer.extend_from_slice(prelude_data());

        StringTableBuilder {
            offsets_buffer,
            data_buffer,
            count: COMMON_STRING_COUNT,
            arena,
            dedup: FxHashMap::default(),
            // Sentinel pointer never matches a real `&str` (length-0 reads
            // are guarded by the `len` check too).
            last_key_ptr: core::ptr::null(),
            last_key_len: 0,
            last_entry: DedupEntry { field: StringField::NONE, index: None },
        }
    }

    /// Intern `value` as a [`StringField`] (Extended Data slot path).
    ///
    /// Fast path: [`lookup_common`] returns the pre-computed StringField
    /// for the well-known strings without HashMap lookup.
    ///
    /// Slow path: regular HashMap dedup; on miss, append to `data_buffer`
    /// only (no `offsets_buffer` write) and cache the resulting
    /// StringField for future hits.
    pub fn intern(&mut self, value: &str) -> StringField {
        // 1-slot recent-call cache: skip lookup_common + HashMap probe
        // when the caller passes the same `&str` (same backing allocation
        // and length) twice in a row.
        if value.as_ptr() == self.last_key_ptr && value.len() == self.last_key_len {
            return self.last_entry.field;
        }
        if let Some(idx) = lookup_common(value) {
            let field = common_string_field(idx);
            self.update_recent_cache(value, DedupEntry { field, index: None });
            return field;
        }
        if let Some(entry) = self.dedup.get(value) {
            let snapshot = *entry;
            self.update_recent_cache(value, snapshot);
            return snapshot.field;
        }
        let field = self.append_no_dedup(value);
        let key: &'arena str = self.arena.alloc_str(value);
        let entry = DedupEntry { field, index: None };
        self.dedup.insert(key, entry);
        self.update_recent_cache(value, entry);
        field
    }

    /// Intern `value` as a [`StringIndex`] — used by string-leaf nodes
    /// (TypeTag::String) and the diagnostics section's `message_index`.
    ///
    /// Fast path: [`lookup_common`] returns the pre-seeded index.
    ///
    /// Slow path: HashMap dedup. On hit, allocates a StringIndex lazily
    /// (so ED-only strings never pay the `offsets_buffer` write).
    pub fn intern_for_leaf(&mut self, value: &str) -> StringIndex {
        // 1-slot recent-call cache hit: only valid if the cached entry
        // already has an `index` allocated (recent path may have stored
        // a StringField-only entry).
        if value.as_ptr() == self.last_key_ptr && value.len() == self.last_key_len {
            if let Some(idx) = self.last_entry.index {
                return idx;
            }
        }
        if let Some(idx) = lookup_common(value) {
            // SAFETY: idx < COMMON_STRINGS.len() < u32::MAX.
            return StringIndex::from_u32(idx).expect("common index in range");
        }
        if let Some(entry) = self.dedup.get(value) {
            if let Some(idx) = entry.index {
                let snapshot = *entry;
                self.update_recent_cache(value, snapshot);
                return idx;
            }
            // Same string was previously interned as StringField only —
            // promote it now by allocating an offsets-table entry.
            let field = entry.field;
            let idx = self.assign_index(field);
            let entry = DedupEntry { field, index: Some(idx) };
            // Re-fetch & write back. (`get_mut` was unavailable while we
            // held the borrow on `assign_index` to mutate offsets_buffer.)
            if let Some(entry_mut) = self.dedup.get_mut(value) {
                *entry_mut = entry;
            }
            self.update_recent_cache(value, entry);
            return idx;
        }
        // First-touch: append data_buffer + offsets_buffer together.
        let field = self.append_no_dedup(value);
        let idx = self.assign_index(field);
        let key: &'arena str = self.arena.alloc_str(value);
        let entry = DedupEntry { field, index: Some(idx) };
        self.dedup.insert(key, entry);
        self.update_recent_cache(value, entry);
        idx
    }

    /// Append `value` as a fresh [`StringField`] without [`lookup_common`]
    /// or dedup. Use this for strings dominated by per-call unique values
    /// (description-line text, raw `{type}` source, etc.) where paying the
    /// FxHash + lookup work for a key that will never be revisited is
    /// pure overhead.
    pub fn intern_unique(&mut self, value: &str) -> StringField {
        self.append_no_dedup(value)
    }

    /// Materialize a [`StringField`] pointing at an existing range of
    /// `data_buffer` — typically the source text region appended via
    /// [`Self::append_source_text`] — **without copying any bytes**.
    #[inline]
    pub fn intern_at_offset(&mut self, start: u32, end: u32) -> StringField {
        debug_assert!(
            (end as usize) <= self.data_buffer.len(),
            "intern_at_offset range [{start}, {end}) extends past data_buffer length {}",
            self.data_buffer.len()
        );
        debug_assert!(start <= end, "intern_at_offset start > end");
        debug_assert!(
            (end - start) <= u16::MAX as u32,
            "source slice length {} exceeds u16 (StringField max)",
            end - start
        );
        StringField { offset: start, length: (end - start) as u16 }
    }

    /// Allocate a [`StringIndex`] pointing at an existing range of
    /// `data_buffer` (typically a slice of the just-appended source
    /// text). Writes a `(start, end)` pair into `offsets_buffer` so the
    /// reader can resolve the index without scanning.
    ///
    /// Used by string-leaf emitters (description-line / type-line basic)
    /// where the source slice becomes the String payload of a leaf node.
    pub fn intern_at_offset_for_leaf(&mut self, start: u32, end: u32) -> StringIndex {
        debug_assert!(
            (end as usize) <= self.data_buffer.len(),
            "intern_at_offset_for_leaf range [{start}, {end}) extends past data_buffer length {}",
            self.data_buffer.len()
        );
        debug_assert!(start <= end, "intern_at_offset_for_leaf start > end");
        push_offset_pair(&mut self.offsets_buffer, start, end);
        let idx = StringIndex::from_u32(self.count).expect("string index overflow");
        self.count = self.count.checked_add(1).expect("string table overflow");
        idx
    }

    /// Append a sourceText prefix to `data_buffer` (no offsets-table entry).
    pub fn append_source_text(&mut self, value: &str) -> u32 {
        let offset = self.data_buffer.len() as u32;
        self.data_buffer.extend_from_slice(value.as_bytes());
        offset
    }

    /// Truncate the builder back to its post-[`Self::new`] state without
    /// freeing arena memory. The arena-backed `offsets_buffer` /
    /// `data_buffer` keep their capacity, so subsequent
    /// [`Self::intern_for_leaf`] / [`Self::append_source_text`] calls reuse
    /// the existing allocations instead of growing from zero on every
    /// per-comment writer construction.
    ///
    /// Used by [`crate::writer::BinaryWriter::reset`] to recycle a single
    /// writer across many parse calls (see
    /// [`crate::parser::parse_into`] / [`crate::parser::parse_batch_into`]).
    pub(crate) fn reset(&mut self) {
        // Restore the prelude bytes. Truncate first so capacity is retained,
        // then `extend_from_slice` reuses that capacity for the prelude.
        let prelude_offsets_bytes = prelude_offsets();
        let prelude_data_bytes = prelude_data();
        self.offsets_buffer.truncate(0);
        self.offsets_buffer.extend_from_slice(prelude_offsets_bytes);
        self.data_buffer.truncate(0);
        self.data_buffer.extend_from_slice(prelude_data_bytes);
        self.count = COMMON_STRING_COUNT;
        // Wipe dedup but keep the HashMap's bucket allocation (FxHashMap's
        // `clear` retains capacity).
        self.dedup.clear();
        self.last_key_ptr = core::ptr::null();
        self.last_key_len = 0;
        self.last_entry = DedupEntry { field: StringField::NONE, index: None };
    }

    /// Number of strings registered in the offsets table so far.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.count
    }

    /// Whether nothing has been registered in the offsets table yet.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Internal: refresh the 1-slot recent-call cache so the next intern
    /// of the same `&str` slice can short-circuit before HashMap probe.
    #[inline]
    fn update_recent_cache(&mut self, value: &str, entry: DedupEntry) {
        self.last_key_ptr = value.as_ptr();
        self.last_key_len = value.len();
        self.last_entry = entry;
    }

    /// Internal: append `value` to `data_buffer` and return the resulting
    /// [`StringField`]. Shared between [`Self::intern`] (after dedup miss)
    /// and [`Self::intern_unique`].
    ///
    /// `value.len()` is cast directly to `u16`; the encoder's contract is
    /// that no individual JSDoc field exceeds 65 KiB (description text is
    /// the worst case in practice). A debug-assert catches the overflow in
    /// development builds without paying the panic check on the release
    /// hot path.
    #[inline]
    fn append_no_dedup(&mut self, value: &str) -> StringField {
        debug_assert!(
            value.len() <= u16::MAX as usize,
            "string length {} exceeds u16 (StringField max)",
            value.len()
        );
        let offset = self.data_buffer.len() as u32;
        self.data_buffer.extend_from_slice(value.as_bytes());
        let length = value.len() as u16;
        StringField { offset, length }
    }

    /// Internal: allocate a fresh [`StringIndex`] for an existing
    /// [`StringField`] by writing a (start, end) pair into `offsets_buffer`.
    fn assign_index(&mut self, field: StringField) -> StringIndex {
        let idx = StringIndex::from_u32(self.count).expect("string index overflow");
        self.count = self.count.checked_add(1).expect("string table overflow");
        let end = field.offset + field.length as u32;
        push_offset_pair(&mut self.offsets_buffer, field.offset, end);
        idx
    }
}

/// Append an `(start, end)` u32-LE pair to the offsets buffer as a single
/// 8-byte write. Combines the two `extend_from_slice(&[u8; 4])` calls into
/// one realloc-check + one 8-byte memcpy — cuts `intern_at_offset_for_leaf`
/// self time roughly in half on `parse_batch_to_bytes`.
#[inline]
fn push_offset_pair(buf: &mut ArenaVec<'_, u8>, start: u32, end: u32) {
    let mut bytes = [0u8; 8];
    bytes[..4].copy_from_slice(&start.to_le_bytes());
    bytes[4..].copy_from_slice(&end.to_le_bytes());
    buf.extend_from_slice(&bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_index_round_trips() {
        for raw in [0u32, 1, 100, 0xFFFE, 0x3FFF_FFFE] {
            let idx = StringIndex::from_u32(raw).unwrap();
            assert_eq!(idx.as_u32(), raw);
        }
    }

    #[test]
    fn string_index_rejects_overflow() {
        assert!(StringIndex::from_u32(u32::MAX).is_none());
    }

    #[test]
    fn intern_dedups_repeated_values_as_string_field() {
        let arena = Allocator::default();
        let mut builder = StringTableBuilder::new(&arena);
        let a = builder.intern("custom_alpha");
        let b = builder.intern("custom_beta");
        let a_again = builder.intern("custom_alpha");
        assert_eq!(a, a_again);
        assert_ne!(a, b);
    }

    #[test]
    fn intern_for_leaf_is_lazy_about_offsets_entry() {
        let arena = Allocator::default();
        let mut builder = StringTableBuilder::new(&arena);
        let prelude_count = COMMON_STRING_COUNT;
        // Intern as field only — must not allocate an offsets entry.
        let _f = builder.intern("custom_xyz");
        assert_eq!(builder.len(), prelude_count, "no offsets entry for ED-only intern");
        // Now ask for the leaf form — that allocates one entry.
        let _idx = builder.intern_for_leaf("custom_xyz");
        assert_eq!(builder.len(), prelude_count + 1, "leaf intern allocates offsets entry");
        // Calling intern_for_leaf again must dedup, no extra allocation.
        let _idx2 = builder.intern_for_leaf("custom_xyz");
        assert_eq!(builder.len(), prelude_count + 1, "dedup on second leaf intern");
    }

    #[test]
    fn append_source_text_does_not_register_an_index() {
        let arena = Allocator::default();
        let mut builder = StringTableBuilder::new(&arena);
        let common_data_bytes = builder.data_buffer.len() as u32;
        let off = builder.append_source_text("/** @x */");
        assert_eq!(
            off, common_data_bytes,
            "source text starts immediately after the common-string prelude"
        );
        assert_eq!(
            builder.len(),
            COMMON_STRING_COUNT,
            "source text does not count as an offsets entry"
        );
    }

    #[test]
    fn intern_common_string_returns_predetermined_field() {
        let arena = Allocator::default();
        let mut builder = StringTableBuilder::new(&arena);
        let field = builder.intern("param");
        let expected = common_string_fields()[8];
        assert_eq!(field, expected);
    }

    #[test]
    fn intern_for_leaf_common_string_returns_predetermined_index() {
        let arena = Allocator::default();
        let mut builder = StringTableBuilder::new(&arena);
        // "param" is COMMON_STRINGS[8].
        assert_eq!(builder.intern_for_leaf("param").as_u32(), 8);
        let pre_count = builder.len();
        assert_eq!(builder.intern_for_leaf("param").as_u32(), 8);
        assert_eq!(builder.len(), pre_count, "fast path must not append");
    }
}
