// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Node kinds (62 in total) and category checks.
//!
//! See `design/007-binary-ast/ast-nodes.md#kind-number-space` for the full
//! number-space layout. Quick summary:
//!
//! ```text
//! 0x00         Sentinel               (1)
//! 0x01 - 0x0F  Comment AST            (15 used + reserved)
//! 0x10 - 0x3F  Comment AST headroom   (reserved)
//! 0x40 - 0x7E  Globally reserved      (reserved)
//! 0x7F         NodeList               (1)
//! 0x80 - 0xAC  TypeNode               (45 used)
//! 0xAD - 0xFF  TypeNode headroom      (reserved)
//! ```
//!
//! Categories are designed so that hot-path checks compile to a single
//! instruction:
//!
//! - [`is_type_node`]: `(kind & 0x80) != 0` (1 MSB test)
//! - [`is_node_list`]: `kind == 0x7F` (1 compare)
//! - [`is_sentinel`]:  `kind == 0x00` (1 compare)

use core::fmt;

/// All 62 node kinds defined by the Binary AST format.
///
/// Discriminants are stable across releases (per the protocol-compatibility
/// rules in `ast-nodes.md`); deleted variants are kept as comments rather
/// than reusing their slot.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    // -- Special ----------------------------------------------------------
    /// `node[0]` sentinel; never written for any real node.
    Sentinel = 0x00,

    // -- Comment AST (15 kinds, 0x01 - 0x0F) ------------------------------
    /// Root `JsdocBlock` for one `/** ... */` block.
    JsdocBlock = 0x01,
    /// `JsdocDescriptionLine`.
    JsdocDescriptionLine = 0x02,
    /// `JsdocTag` (block tag).
    JsdocTag = 0x03,
    /// `JsdocTagName` (the `@name` part of a tag).
    JsdocTagName = 0x04,
    /// `JsdocTagNameValue` (e.g. the `name` after the type in `@param`).
    JsdocTagNameValue = 0x05,
    /// `JsdocTypeSource` (raw `{...}` text inside a tag).
    JsdocTypeSource = 0x06,
    /// `JsdocTypeLine` (one source-preserving type-source line).
    JsdocTypeLine = 0x07,
    /// `JsdocInlineTag` (e.g. `{@link Foo}`).
    JsdocInlineTag = 0x08,
    /// `JsdocGenericTagBody` (`JsdocTagBody::Generic` variant).
    JsdocGenericTagBody = 0x09,
    /// `JsdocBorrowsTagBody` (`JsdocTagBody::Borrows` variant).
    JsdocBorrowsTagBody = 0x0A,
    /// `JsdocRawTagBody` (`JsdocTagBody::Raw` variant).
    JsdocRawTagBody = 0x0B,
    /// `JsdocParameterName` (`JsdocTagValue::Parameter` variant).
    JsdocParameterName = 0x0C,
    /// `JsdocNamepathSource` (`JsdocTagValue::Namepath` variant).
    JsdocNamepathSource = 0x0D,
    /// `JsdocIdentifier` (`JsdocTagValue::Identifier` variant).
    JsdocIdentifier = 0x0E,
    /// `JsdocText` (`JsdocTagValue::Raw` variant).
    JsdocText = 0x0F,

    // -- Special list wrapper ---------------------------------------------
    /// Special wrapper for variable-length child arrays.
    ///
    /// **Deprecated** as of the NodeList-elimination format change. Lists
    /// are now stored as direct children of the parent, with
    /// `(head_index: u32, count: u16)` metadata embedded inline in the
    /// parent's Extended Data block (see
    /// [`crate::format::extended_data::LIST_METADATA_SIZE`]). The discriminant
    /// `0x7F` remains reserved so legacy buffers still parse, but encoders
    /// no longer emit nodes of this Kind.
    NodeList = 0x7F,

    // -- TypeNode (45 kinds, 0x80 - 0xAC) ---------------------------------
    /// `TypeName` (basic identifier type).
    TypeName = 0x80,
    /// `TypeNumber` (numeric literal type).
    TypeNumber = 0x81,
    /// `TypeStringValue` (string literal type).
    TypeStringValue = 0x82,
    /// `TypeNull`.
    TypeNull = 0x83,
    /// `TypeUndefined`.
    TypeUndefined = 0x84,
    /// `TypeAny` (`*` or `any`).
    TypeAny = 0x85,
    /// `TypeUnknown`.
    TypeUnknown = 0x86,
    /// `TypeUnion`.
    TypeUnion = 0x87,
    /// `TypeIntersection`.
    TypeIntersection = 0x88,
    /// `TypeGeneric` (e.g. `Foo<T>` or `Foo[]`).
    TypeGeneric = 0x89,
    /// `TypeFunction`.
    TypeFunction = 0x8A,
    /// `TypeObject`.
    TypeObject = 0x8B,
    /// `TypeTuple`.
    TypeTuple = 0x8C,
    /// `TypeParenthesis`.
    TypeParenthesis = 0x8D,
    /// `TypeNamePath`.
    TypeNamePath = 0x8E,
    /// `TypeSpecialNamePath`.
    TypeSpecialNamePath = 0x8F,
    /// `TypeNullable` (`?T` / `T?`).
    TypeNullable = 0x90,
    /// `TypeNotNullable` (`!T` / `T!`).
    TypeNotNullable = 0x91,
    /// `TypeOptional` (`T=`).
    TypeOptional = 0x92,
    /// `TypeVariadic` (`...T`).
    TypeVariadic = 0x93,
    /// `TypeConditional`.
    TypeConditional = 0x94,
    /// `TypeInfer`.
    TypeInfer = 0x95,
    /// `TypeKeyOf`.
    TypeKeyOf = 0x96,
    /// `TypeTypeOf`.
    TypeTypeOf = 0x97,
    /// `TypeImport`.
    TypeImport = 0x98,
    /// `TypePredicate`.
    TypePredicate = 0x99,
    /// `TypeAsserts`.
    TypeAsserts = 0x9A,
    /// `TypeAssertsPlain`.
    TypeAssertsPlain = 0x9B,
    /// `TypeReadonlyArray`.
    TypeReadonlyArray = 0x9C,
    /// `TypeTemplateLiteral`.
    TypeTemplateLiteral = 0x9D,
    /// `TypeUniqueSymbol`.
    TypeUniqueSymbol = 0x9E,
    /// `TypeSymbol` (Closure-style `Symbol(...)` calls).
    TypeSymbol = 0x9F,
    /// `TypeObjectField`.
    TypeObjectField = 0xA0,
    /// `TypeJsdocObjectField`.
    TypeJsdocObjectField = 0xA1,
    /// `TypeKeyValue` (`a: T`).
    TypeKeyValue = 0xA2,
    /// `TypeProperty`.
    TypeProperty = 0xA3,
    /// `TypeIndexSignature` (`[K]: T`).
    TypeIndexSignature = 0xA4,
    /// `TypeMappedType` (`[K in U]: T`).
    TypeMappedType = 0xA5,
    /// `TypeTypeParameter`.
    TypeTypeParameter = 0xA6,
    /// `TypeCallSignature`.
    TypeCallSignature = 0xA7,
    /// `TypeConstructorSignature`.
    TypeConstructorSignature = 0xA8,
    /// `TypeMethodSignature`.
    TypeMethodSignature = 0xA9,
    /// `TypeIndexedAccessIndex`.
    TypeIndexedAccessIndex = 0xAA,
    /// `TypeParameterList`.
    TypeParameterList = 0xAB,
    /// `TypeReadonlyProperty`.
    TypeReadonlyProperty = 0xAC,
}

/// First reserved Kind discriminant in the global-reserved range
/// (`0x40 - 0x7E`); see `ast-nodes.md`.
pub const FIRST_RESERVED_KIND: u8 = 0x40;
/// Last reserved Kind discriminant in the global-reserved range.
pub const LAST_RESERVED_KIND: u8 = 0x7E;
/// First TypeNode Kind discriminant (`0x80`); the upper-bit cluster.
pub const FIRST_TYPE_NODE_KIND: u8 = 0x80;
/// Highest TypeNode Kind discriminant currently in use (`0xAC`).
pub const LAST_TYPE_NODE_KIND_IN_USE: u8 = 0xAC;
/// First Comment AST Kind discriminant (`0x01`).
pub const FIRST_COMMENT_AST_KIND: u8 = 0x01;
/// Last Comment AST Kind discriminant currently in use (`0x0F`).
pub const LAST_COMMENT_AST_KIND_IN_USE: u8 = 0x0F;
/// Sentinel discriminant (`0x00`).
pub const SENTINEL_KIND: u8 = 0x00;
/// `NodeList` discriminant (`0x7F`).
pub const NODE_LIST_KIND: u8 = 0x7F;

/// Error returned by [`Kind::from_u8`] for unknown discriminants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownKind(pub u8);

impl fmt::Display for UnknownKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown Kind discriminant 0x{:02X}", self.0)
    }
}

impl Kind {
    /// Convert a raw byte to a [`Kind`]. Returns [`UnknownKind`] when the
    /// byte falls in a reserved or undefined slot.
    ///
    /// This is the safe entry point used by decoders. Hot dispatch in the
    /// lazy decoder may use unchecked transmutes after [`is_known`] returns
    /// `true`.
    #[inline]
    pub const fn from_u8(value: u8) -> Result<Self, UnknownKind> {
        match value {
            0x00 => Ok(Kind::Sentinel),
            0x01 => Ok(Kind::JsdocBlock),
            0x02 => Ok(Kind::JsdocDescriptionLine),
            0x03 => Ok(Kind::JsdocTag),
            0x04 => Ok(Kind::JsdocTagName),
            0x05 => Ok(Kind::JsdocTagNameValue),
            0x06 => Ok(Kind::JsdocTypeSource),
            0x07 => Ok(Kind::JsdocTypeLine),
            0x08 => Ok(Kind::JsdocInlineTag),
            0x09 => Ok(Kind::JsdocGenericTagBody),
            0x0A => Ok(Kind::JsdocBorrowsTagBody),
            0x0B => Ok(Kind::JsdocRawTagBody),
            0x0C => Ok(Kind::JsdocParameterName),
            0x0D => Ok(Kind::JsdocNamepathSource),
            0x0E => Ok(Kind::JsdocIdentifier),
            0x0F => Ok(Kind::JsdocText),
            0x7F => Ok(Kind::NodeList),
            0x80 => Ok(Kind::TypeName),
            0x81 => Ok(Kind::TypeNumber),
            0x82 => Ok(Kind::TypeStringValue),
            0x83 => Ok(Kind::TypeNull),
            0x84 => Ok(Kind::TypeUndefined),
            0x85 => Ok(Kind::TypeAny),
            0x86 => Ok(Kind::TypeUnknown),
            0x87 => Ok(Kind::TypeUnion),
            0x88 => Ok(Kind::TypeIntersection),
            0x89 => Ok(Kind::TypeGeneric),
            0x8A => Ok(Kind::TypeFunction),
            0x8B => Ok(Kind::TypeObject),
            0x8C => Ok(Kind::TypeTuple),
            0x8D => Ok(Kind::TypeParenthesis),
            0x8E => Ok(Kind::TypeNamePath),
            0x8F => Ok(Kind::TypeSpecialNamePath),
            0x90 => Ok(Kind::TypeNullable),
            0x91 => Ok(Kind::TypeNotNullable),
            0x92 => Ok(Kind::TypeOptional),
            0x93 => Ok(Kind::TypeVariadic),
            0x94 => Ok(Kind::TypeConditional),
            0x95 => Ok(Kind::TypeInfer),
            0x96 => Ok(Kind::TypeKeyOf),
            0x97 => Ok(Kind::TypeTypeOf),
            0x98 => Ok(Kind::TypeImport),
            0x99 => Ok(Kind::TypePredicate),
            0x9A => Ok(Kind::TypeAsserts),
            0x9B => Ok(Kind::TypeAssertsPlain),
            0x9C => Ok(Kind::TypeReadonlyArray),
            0x9D => Ok(Kind::TypeTemplateLiteral),
            0x9E => Ok(Kind::TypeUniqueSymbol),
            0x9F => Ok(Kind::TypeSymbol),
            0xA0 => Ok(Kind::TypeObjectField),
            0xA1 => Ok(Kind::TypeJsdocObjectField),
            0xA2 => Ok(Kind::TypeKeyValue),
            0xA3 => Ok(Kind::TypeProperty),
            0xA4 => Ok(Kind::TypeIndexSignature),
            0xA5 => Ok(Kind::TypeMappedType),
            0xA6 => Ok(Kind::TypeTypeParameter),
            0xA7 => Ok(Kind::TypeCallSignature),
            0xA8 => Ok(Kind::TypeConstructorSignature),
            0xA9 => Ok(Kind::TypeMethodSignature),
            0xAA => Ok(Kind::TypeIndexedAccessIndex),
            0xAB => Ok(Kind::TypeParameterList),
            0xAC => Ok(Kind::TypeReadonlyProperty),
            other => Err(UnknownKind(other)),
        }
    }

    /// Returns the raw u8 discriminant.
    #[inline]
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

// ---------------------------------------------------------------------------
// Category checks (single-instruction hot paths).
// ---------------------------------------------------------------------------

/// Is `kind` in the TypeNode range (`0x80 - 0xFF`, MSB set)?
///
/// Compiles to a single MSB test such as `TEST AL, 0x80` (x86) or
/// `TST W0, #0x80` (ARM).
#[inline]
#[must_use]
pub const fn is_type_node(kind: u8) -> bool {
    (kind & 0x80) != 0
}

/// Is `kind` exactly the `NodeList` discriminant (`0x7F`)?
#[inline]
#[must_use]
pub const fn is_node_list(kind: u8) -> bool {
    kind == NODE_LIST_KIND
}

/// Is `kind` the sentinel discriminant (`0x00`)?
#[inline]
#[must_use]
pub const fn is_sentinel(kind: u8) -> bool {
    kind == SENTINEL_KIND
}

/// Is `kind` in the comment-AST range (`0x01 - 0x3F`)?
///
/// Implementation: `(kind & 0xC0) == 0x00 && kind != 0x00`.
#[inline]
#[must_use]
pub const fn is_comment_ast(kind: u8) -> bool {
    (kind & 0xC0) == 0x00 && kind != SENTINEL_KIND
}

/// Is `kind` in the globally reserved range (`0x40 - 0x7E`, excluding NodeList)?
///
/// Implementation: `(kind & 0xC0) == 0x40 && kind != NodeList`.
#[inline]
#[must_use]
pub const fn is_reserved(kind: u8) -> bool {
    (kind & 0xC0) == 0x40 && kind != NODE_LIST_KIND
}

/// Is `kind` a defined node type (i.e. [`Kind::from_u8`] would return `Ok`)?
#[inline]
#[must_use]
pub const fn is_known(kind: u8) -> bool {
    Kind::from_u8(kind).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Total number of defined Kind values: 1 sentinel + 15 comment AST +
    /// 1 NodeList + 45 TypeNode = 62.
    const EXPECTED_KNOWN_KINDS: usize = 62;

    #[test]
    fn from_u8_recognizes_all_62_known_discriminants() {
        let mut count = 0usize;
        for byte in 0u8..=u8::MAX {
            if Kind::from_u8(byte).is_ok() {
                count += 1;
            }
        }
        assert_eq!(count, EXPECTED_KNOWN_KINDS);
    }

    #[test]
    fn from_u8_round_trips_for_all_known_kinds() {
        for byte in 0u8..=u8::MAX {
            if let Ok(kind) = Kind::from_u8(byte) {
                assert_eq!(kind.as_u8(), byte);
            }
        }
    }

    #[test]
    fn category_partition_covers_every_byte() {
        // Every byte must fall into exactly one of: sentinel, comment AST,
        // reserved, NodeList, TypeNode.
        for byte in 0u8..=u8::MAX {
            let mut hits = 0;
            if is_sentinel(byte) {
                hits += 1;
            }
            if is_comment_ast(byte) {
                hits += 1;
            }
            if is_reserved(byte) {
                hits += 1;
            }
            if is_node_list(byte) {
                hits += 1;
            }
            if is_type_node(byte) {
                hits += 1;
            }
            assert_eq!(hits, 1, "byte 0x{byte:02X} matched {hits} categories");
        }
    }

    #[test]
    fn type_node_range_count_matches_msb_bytes() {
        let count = (0u8..=u8::MAX).filter(|&b| is_type_node(b)).count();
        assert_eq!(count, 128);
    }

    #[test]
    fn comment_ast_range_count_is_63() {
        // 0x01 - 0x3F = 63 slots (15 currently used + 48 reserved for growth).
        let count = (0u8..=u8::MAX).filter(|&b| is_comment_ast(b)).count();
        assert_eq!(count, 63);
    }

    #[test]
    fn reserved_range_count_is_63() {
        // 0x40 - 0x7E = 63 slots (NodeList = 0x7F is excluded).
        let count = (0u8..=u8::MAX).filter(|&b| is_reserved(b)).count();
        assert_eq!(count, 63);
    }

    #[test]
    fn known_kinds_are_a_subset_of_categorized_bytes() {
        // Every defined Kind must be either Sentinel, NodeList,
        // a comment-AST byte, or a TypeNode byte (never reserved).
        for byte in 0u8..=u8::MAX {
            if Kind::from_u8(byte).is_ok() {
                assert!(
                    is_sentinel(byte)
                        || is_node_list(byte)
                        || is_comment_ast(byte)
                        || is_type_node(byte),
                    "known Kind 0x{byte:02X} fell into reserved range"
                );
                assert!(!is_reserved(byte));
            }
        }
    }

    #[test]
    fn unknown_kind_error_carries_the_byte() {
        let err = Kind::from_u8(0x40).unwrap_err();
        assert_eq!(err.0, 0x40);
    }

    #[test]
    fn first_and_last_constants_match_enum() {
        assert_eq!(Kind::Sentinel.as_u8(), SENTINEL_KIND);
        assert_eq!(Kind::JsdocBlock.as_u8(), FIRST_COMMENT_AST_KIND);
        assert_eq!(Kind::JsdocText.as_u8(), LAST_COMMENT_AST_KIND_IN_USE);
        assert_eq!(Kind::NodeList.as_u8(), NODE_LIST_KIND);
        assert_eq!(Kind::TypeName.as_u8(), FIRST_TYPE_NODE_KIND);
        assert_eq!(Kind::TypeReadonlyProperty.as_u8(), LAST_TYPE_NODE_KIND_IN_USE);
    }
}
