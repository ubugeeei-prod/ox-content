// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Lazy structs for the 45 TypeNode kinds (`0x80 - 0xAC`).
//!
//! Each struct mirrors the comment-AST style in
//! [`super::comment_ast`]: a `Copy` value type holding
//! `(source_file, node_index, root_index)` plus per-Kind getters.
//!
//! In addition to the per-Kind structs, this module exposes the
//! [`LazyTypeNode`] enum for callers (such as `JsdocTag.parsed_type`) that
//! receive a TypeNode of unknown variant.

use crate::format::kind::Kind;
use crate::format::string_field::STRING_FIELD_SIZE;
use crate::writer::nodes::type_node::TYPE_LIST_PARENT_SLOT;

use super::super::helpers::{
    child_at_visitor_index, children_bitmask_payload, ext_offset, ext_string_leaf, first_child,
    read_list_metadata, read_next_sibling, read_string_field, read_u16, string_payload,
};
use super::super::source_file::LazySourceFile;
use super::LazyNode;

/// Generate a lazy TypeNode struct + its `LazyNode` impl in one go.
macro_rules! define_lazy_type_node {
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

// ===========================================================================
// Pattern 1 — String only (5 kinds)
// ===========================================================================

define_lazy_type_node!(LazyTypeName, Kind::TypeName, "Lazy view of `TypeName` (Kind 0x80).");
impl<'a> LazyTypeName<'a> {
    /// Identifier name string.
    pub fn value(&self) -> &'a str {
        string_payload(self.source_file, self.node_index).unwrap_or("")
    }
}

define_lazy_type_node!(LazyTypeNumber, Kind::TypeNumber, "Lazy view of `TypeNumber` (Kind 0x81).");
impl<'a> LazyTypeNumber<'a> {
    /// Numeric literal as written in the source.
    pub fn value(&self) -> &'a str {
        string_payload(self.source_file, self.node_index).unwrap_or("")
    }
}

define_lazy_type_node!(
    LazyTypeStringValue,
    Kind::TypeStringValue,
    "Lazy view of `TypeStringValue` (Kind 0x82)."
);
impl<'a> LazyTypeStringValue<'a> {
    /// Quote style (None=0 / Single=1 / Double=2).
    #[inline]
    pub fn quote(&self) -> u8 {
        self.common_data() & 0b11
    }
    /// String literal value.
    pub fn value(&self) -> &'a str {
        string_payload(self.source_file, self.node_index).unwrap_or("")
    }
}

define_lazy_type_node!(
    LazyTypeProperty,
    Kind::TypeProperty,
    "Lazy view of `TypeProperty` (Kind 0xA3)."
);
impl<'a> LazyTypeProperty<'a> {
    /// Quote style (3-state).
    #[inline]
    pub fn quote(&self) -> u8 {
        self.common_data() & 0b11
    }
    /// Property name string.
    pub fn value(&self) -> &'a str {
        string_payload(self.source_file, self.node_index).unwrap_or("")
    }
}

define_lazy_type_node!(
    LazyTypeSpecialNamePath,
    Kind::TypeSpecialNamePath,
    "Lazy view of `TypeSpecialNamePath` (Kind 0x8F)."
);
impl<'a> LazyTypeSpecialNamePath<'a> {
    /// Special path category (3 variants).
    #[inline]
    pub fn special_type(&self) -> u8 {
        self.common_data() & 0b11
    }
    /// Quote style (3-state).
    #[inline]
    pub fn quote(&self) -> u8 {
        (self.common_data() >> 2) & 0b11
    }
    /// Path string.
    pub fn value(&self) -> &'a str {
        string_payload(self.source_file, self.node_index).unwrap_or("")
    }
}

// ===========================================================================
// Pattern 2 — Children only (29 kinds)
//
// Most of these wrap zero or more child TypeNodes accessed via
// `child_at_visitor_index`. Phase 1.1b ships the structs + the most
// useful getters (`element` / `elements` / `left` / `right` / etc.); the
// long tail of trivial accessors can be filled in during Phase 1.2a as
// the parser starts emitting them.
// ===========================================================================

/// Iterator yielding [`LazyTypeNode`] values walked through `next_sibling`.
///
/// Separate from the generic [`NodeListIter`] because `LazyTypeNode` is a
/// sum type and can't implement [`LazyNode`] (each variant has its own
/// `Kind`). Carries an explicit `remaining` count read from the parent's
/// Extended Data list-metadata slot.
#[derive(Debug, Clone, Copy)]
pub struct LazyTypeNodeListIter<'a> {
    source_file: &'a LazySourceFile<'a>,
    current_index: u32,
    remaining: u32,
    root_index: u32,
}

impl<'a> LazyTypeNodeListIter<'a> {
    /// Create a fresh iterator over `count` elements starting at
    /// `head_index`. Either `head_index = 0` or `count = 0` produces an
    /// immediately-empty iterator.
    #[inline]
    #[must_use]
    pub const fn new(
        source_file: &'a LazySourceFile<'a>,
        head_index: u32,
        count: u32,
        root_index: u32,
    ) -> Self {
        let (current, remaining) =
            if head_index == 0 || count == 0 { (0, 0) } else { (head_index, count) };
        LazyTypeNodeListIter { source_file, current_index: current, remaining, root_index }
    }

    /// Whether the iterator has been fully consumed.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.remaining == 0
    }

    /// Number of elements still to yield.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.remaining as usize
    }
}

impl<'a> Iterator for LazyTypeNodeListIter<'a> {
    type Item = LazyTypeNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.remaining > 0 {
            let item =
                LazyTypeNode::from_index(self.source_file, self.current_index, self.root_index);
            self.current_index = read_next_sibling(self.source_file, self.current_index);
            self.remaining -= 1;
            if let Some(node) = item {
                return Some(node);
            }
            // Skip non-TypeNode siblings (shouldn't happen with valid buffers).
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.remaining as usize;
        (n, Some(n))
    }
}

/// Helper: build a [`LazyTypeNodeListIter`] over the elements registered in
/// the parent's Extended Data list-metadata slot at
/// [`TYPE_LIST_PARENT_SLOT`].
fn nodelist_at<'a>(
    sf: &'a LazySourceFile<'a>,
    parent_index: u32,
    root_index: u32,
) -> LazyTypeNodeListIter<'a> {
    let ext = ext_offset(sf, parent_index) as usize;
    let (head, count) = read_list_metadata(sf, ext, TYPE_LIST_PARENT_SLOT);
    LazyTypeNodeListIter::new(sf, head, count, root_index)
}

/// Helper: get the n-th direct child as a TypeNode.
fn child_type_node<'a>(
    sf: &'a LazySourceFile<'a>,
    parent_index: u32,
    visitor_index: u8,
    root_index: u32,
) -> Option<LazyTypeNode<'a>> {
    let bitmask = children_bitmask_payload(sf, parent_index) as u8;
    let idx = child_at_visitor_index(sf, parent_index, bitmask, visitor_index)?;
    LazyTypeNode::from_index(sf, idx, root_index)
}

define_lazy_type_node!(LazyTypeUnion, Kind::TypeUnion, "Lazy view of `TypeUnion` (Kind 0x87).");
impl<'a> LazyTypeUnion<'a> {
    /// Union elements.
    pub fn elements(&self) -> LazyTypeNodeListIter<'a> {
        nodelist_at(self.source_file, self.node_index, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeIntersection,
    Kind::TypeIntersection,
    "Lazy view of `TypeIntersection` (Kind 0x88)."
);
impl<'a> LazyTypeIntersection<'a> {
    /// Intersection elements.
    pub fn elements(&self) -> LazyTypeNodeListIter<'a> {
        nodelist_at(self.source_file, self.node_index, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeGeneric,
    Kind::TypeGeneric,
    "Lazy view of `TypeGeneric` (Kind 0x89)."
);
impl<'a> LazyTypeGeneric<'a> {
    /// Bracket style (Angle=0 / Square=1).
    #[inline]
    pub fn brackets(&self) -> u8 {
        self.common_data() & 1
    }
    /// Whether the generic was written with a leading `.` (Closure form).
    #[inline]
    pub fn dot(&self) -> bool {
        (self.common_data() & 0b10) != 0
    }
    /// Left-hand side type (the parent's first direct child).
    pub fn left(&self) -> Option<LazyTypeNode<'a>> {
        let idx = first_child(self.source_file, self.node_index)?;
        LazyTypeNode::from_index(self.source_file, idx, self.root_index)
    }
    /// Generic argument elements (registered in the parent's ED block).
    pub fn elements(&self) -> LazyTypeNodeListIter<'a> {
        nodelist_at(self.source_file, self.node_index, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeFunction,
    Kind::TypeFunction,
    "Lazy view of `TypeFunction` (Kind 0x8A)."
);
impl<'a> LazyTypeFunction<'a> {
    /// Constructor flag.
    #[inline]
    pub fn constructor(&self) -> bool {
        (self.common_data() & 0b001) != 0
    }
    /// Arrow-form flag.
    #[inline]
    pub fn arrow(&self) -> bool {
        (self.common_data() & 0b010) != 0
    }
    /// Whether parameters were enclosed in parentheses.
    #[inline]
    pub fn parenthesis(&self) -> bool {
        (self.common_data() & 0b100) != 0
    }
    /// Parameter list child.
    pub fn parameters(&self) -> Option<LazyTypeParameterList<'a>> {
        let bitmask = children_bitmask_payload(self.source_file, self.node_index) as u8;
        let idx = child_at_visitor_index(self.source_file, self.node_index, bitmask, 0)?;
        Some(LazyTypeParameterList::from_index(self.source_file, idx, self.root_index))
    }
    /// Return type child.
    pub fn return_type(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 1, self.root_index)
    }
    /// Type-parameter list child.
    pub fn type_parameters(&self) -> Option<LazyTypeParameterList<'a>> {
        let bitmask = children_bitmask_payload(self.source_file, self.node_index) as u8;
        let idx = child_at_visitor_index(self.source_file, self.node_index, bitmask, 2)?;
        Some(LazyTypeParameterList::from_index(self.source_file, idx, self.root_index))
    }
}

define_lazy_type_node!(LazyTypeObject, Kind::TypeObject, "Lazy view of `TypeObject` (Kind 0x8B).");
impl<'a> LazyTypeObject<'a> {
    /// Field separator style (`bits[0:2]`).
    #[inline]
    pub fn separator(&self) -> u8 {
        self.common_data() & 0b111
    }
    /// Field elements.
    pub fn elements(&self) -> LazyTypeNodeListIter<'a> {
        nodelist_at(self.source_file, self.node_index, self.root_index)
    }
}

define_lazy_type_node!(LazyTypeTuple, Kind::TypeTuple, "Lazy view of `TypeTuple` (Kind 0x8C).");
impl<'a> LazyTypeTuple<'a> {
    /// Tuple elements.
    pub fn elements(&self) -> LazyTypeNodeListIter<'a> {
        nodelist_at(self.source_file, self.node_index, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeParenthesis,
    Kind::TypeParenthesis,
    "Lazy view of `TypeParenthesis` (Kind 0x8D)."
);
impl<'a> LazyTypeParenthesis<'a> {
    /// Wrapped type.
    pub fn element(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 0, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeNamePath,
    Kind::TypeNamePath,
    "Lazy view of `TypeNamePath` (Kind 0x8E)."
);
impl<'a> LazyTypeNamePath<'a> {
    /// Path connector category (`bits[0:1]`).
    #[inline]
    pub fn path_type(&self) -> u8 {
        self.common_data() & 0b11
    }
    /// Left-hand side.
    pub fn left(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 0, self.root_index)
    }
    /// Right-hand side.
    pub fn right(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 1, self.root_index)
    }
}

/// Macro for the simple "1-child + position" modifier types
/// (`Nullable` / `NotNullable` / `Optional` / `Variadic`).
macro_rules! define_modifier_type {
    ($name:ident, $kind:expr, $doc:expr) => {
        define_lazy_type_node!($name, $kind, $doc);
        impl<'a> $name<'a> {
            /// Modifier position (Prefix=0 / Suffix=1).
            #[inline]
            pub fn position(&self) -> u8 {
                self.common_data() & 1
            }
            /// Wrapped type.
            pub fn element(&self) -> Option<LazyTypeNode<'a>> {
                child_type_node(self.source_file, self.node_index, 0, self.root_index)
            }
        }
    };
}
define_modifier_type!(
    LazyTypeNullable,
    Kind::TypeNullable,
    "Lazy view of `TypeNullable` (Kind 0x90)."
);
define_modifier_type!(
    LazyTypeNotNullable,
    Kind::TypeNotNullable,
    "Lazy view of `TypeNotNullable` (Kind 0x91)."
);
define_modifier_type!(
    LazyTypeOptional,
    Kind::TypeOptional,
    "Lazy view of `TypeOptional` (Kind 0x92)."
);

define_lazy_type_node!(
    LazyTypeVariadic,
    Kind::TypeVariadic,
    "Lazy view of `TypeVariadic` (Kind 0x93)."
);
impl<'a> LazyTypeVariadic<'a> {
    /// Modifier position.
    #[inline]
    pub fn position(&self) -> u8 {
        self.common_data() & 1
    }
    /// Whether the variadic was written with `[]` brackets.
    #[inline]
    pub fn square_brackets(&self) -> bool {
        (self.common_data() & 0b10) != 0
    }
    /// Wrapped type.
    pub fn element(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 0, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeConditional,
    Kind::TypeConditional,
    "Lazy view of `TypeConditional` (Kind 0x94)."
);
impl<'a> LazyTypeConditional<'a> {
    /// `T` in `T extends U ? X : Y`.
    pub fn check_type(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 0, self.root_index)
    }
    /// `U`.
    pub fn extends_type(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 1, self.root_index)
    }
    /// `X` (true branch).
    pub fn true_type(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 2, self.root_index)
    }
    /// `Y` (false branch).
    pub fn false_type(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 3, self.root_index)
    }
}

/// Macro for "1 child = element" types.
macro_rules! define_single_child_type {
    ($name:ident, $kind:expr, $doc:expr) => {
        define_lazy_type_node!($name, $kind, $doc);
        impl<'a> $name<'a> {
            /// Wrapped child type.
            pub fn element(&self) -> Option<LazyTypeNode<'a>> {
                child_type_node(self.source_file, self.node_index, 0, self.root_index)
            }
        }
    };
}
define_single_child_type!(LazyTypeInfer, Kind::TypeInfer, "Lazy view of `TypeInfer` (Kind 0x95).");
define_single_child_type!(LazyTypeKeyOf, Kind::TypeKeyOf, "Lazy view of `TypeKeyOf` (Kind 0x96).");
define_single_child_type!(
    LazyTypeTypeOf,
    Kind::TypeTypeOf,
    "Lazy view of `TypeTypeOf` (Kind 0x97)."
);
define_single_child_type!(
    LazyTypeImport,
    Kind::TypeImport,
    "Lazy view of `TypeImport` (Kind 0x98)."
);
define_single_child_type!(
    LazyTypeAssertsPlain,
    Kind::TypeAssertsPlain,
    "Lazy view of `TypeAssertsPlain` (Kind 0x9B)."
);
define_single_child_type!(
    LazyTypeReadonlyArray,
    Kind::TypeReadonlyArray,
    "Lazy view of `TypeReadonlyArray` (Kind 0x9C)."
);
define_single_child_type!(
    LazyTypeIndexedAccessIndex,
    Kind::TypeIndexedAccessIndex,
    "Lazy view of `TypeIndexedAccessIndex` (Kind 0xAA)."
);
define_single_child_type!(
    LazyTypeReadonlyProperty,
    Kind::TypeReadonlyProperty,
    "Lazy view of `TypeReadonlyProperty` (Kind 0xAC)."
);

/// Macro for the "left + right" 2-child types (Predicate / Asserts).
macro_rules! define_left_right_type {
    ($name:ident, $kind:expr, $doc:expr) => {
        define_lazy_type_node!($name, $kind, $doc);
        impl<'a> $name<'a> {
            /// Left-hand operand.
            pub fn left(&self) -> Option<LazyTypeNode<'a>> {
                child_type_node(self.source_file, self.node_index, 0, self.root_index)
            }
            /// Right-hand operand.
            pub fn right(&self) -> Option<LazyTypeNode<'a>> {
                child_type_node(self.source_file, self.node_index, 1, self.root_index)
            }
        }
    };
}
define_left_right_type!(
    LazyTypePredicate,
    Kind::TypePredicate,
    "Lazy view of `TypePredicate` (Kind 0x99)."
);
define_left_right_type!(
    LazyTypeAsserts,
    Kind::TypeAsserts,
    "Lazy view of `TypeAsserts` (Kind 0x9A)."
);

define_lazy_type_node!(
    LazyTypeObjectField,
    Kind::TypeObjectField,
    "Lazy view of `TypeObjectField` (Kind 0xA0)."
);
impl<'a> LazyTypeObjectField<'a> {
    /// `?` modifier flag.
    #[inline]
    pub fn optional(&self) -> bool {
        (self.common_data() & 0b0001) != 0
    }
    /// `readonly` modifier flag.
    #[inline]
    pub fn readonly(&self) -> bool {
        (self.common_data() & 0b0010) != 0
    }
    /// Quote style for the field key (3-state).
    #[inline]
    pub fn quote(&self) -> u8 {
        (self.common_data() >> 2) & 0b11
    }
    /// Field key.
    pub fn key(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 0, self.root_index)
    }
    /// Field value type.
    pub fn right(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 1, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeJsdocObjectField,
    Kind::TypeJsdocObjectField,
    "Lazy view of `TypeJsdocObjectField` (Kind 0xA1)."
);
impl<'a> LazyTypeJsdocObjectField<'a> {
    /// Field key.
    pub fn key(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 0, self.root_index)
    }
    /// Field value type.
    pub fn right(&self) -> Option<LazyTypeNode<'a>> {
        child_type_node(self.source_file, self.node_index, 1, self.root_index)
    }
}

/// Macro for the call-style signature types (CallSignature / ConstructorSignature).
macro_rules! define_signature_type {
    ($name:ident, $kind:expr, $doc:expr) => {
        define_lazy_type_node!($name, $kind, $doc);
        impl<'a> $name<'a> {
            /// Parameter list.
            pub fn parameters(&self) -> Option<LazyTypeParameterList<'a>> {
                let bitmask = children_bitmask_payload(self.source_file, self.node_index) as u8;
                let idx = child_at_visitor_index(self.source_file, self.node_index, bitmask, 0)?;
                Some(LazyTypeParameterList::from_index(self.source_file, idx, self.root_index))
            }
            /// Return type.
            pub fn return_type(&self) -> Option<LazyTypeNode<'a>> {
                child_type_node(self.source_file, self.node_index, 1, self.root_index)
            }
            /// Type-parameter list.
            pub fn type_parameters(&self) -> Option<LazyTypeParameterList<'a>> {
                let bitmask = children_bitmask_payload(self.source_file, self.node_index) as u8;
                let idx = child_at_visitor_index(self.source_file, self.node_index, bitmask, 2)?;
                Some(LazyTypeParameterList::from_index(self.source_file, idx, self.root_index))
            }
        }
    };
}
define_signature_type!(
    LazyTypeCallSignature,
    Kind::TypeCallSignature,
    "Lazy view of `TypeCallSignature` (Kind 0xA7)."
);
define_signature_type!(
    LazyTypeConstructorSignature,
    Kind::TypeConstructorSignature,
    "Lazy view of `TypeConstructorSignature` (Kind 0xA8)."
);

define_lazy_type_node!(
    LazyTypeTypeParameter,
    Kind::TypeTypeParameter,
    "Lazy view of `TypeTypeParameter` (Kind 0xA6)."
);
impl<'a> LazyTypeTypeParameter<'a> {
    /// Type-parameter children.
    pub fn elements(&self) -> LazyTypeNodeListIter<'a> {
        nodelist_at(self.source_file, self.node_index, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeParameterList,
    Kind::TypeParameterList,
    "Lazy view of `TypeParameterList` (Kind 0xAB)."
);
impl<'a> LazyTypeParameterList<'a> {
    /// Parameter elements.
    pub fn elements(&self) -> LazyTypeNodeListIter<'a> {
        nodelist_at(self.source_file, self.node_index, self.root_index)
    }
}

// ===========================================================================
// Pattern 3 — Mixed string + children (6 kinds)
// ===========================================================================

define_lazy_type_node!(
    LazyTypeKeyValue,
    Kind::TypeKeyValue,
    "Lazy view of `TypeKeyValue` (Kind 0xA2)."
);
impl<'a> LazyTypeKeyValue<'a> {
    /// `?` modifier flag.
    #[inline]
    pub fn optional(&self) -> bool {
        (self.common_data() & 0b01) != 0
    }
    /// `...` variadic flag.
    #[inline]
    pub fn variadic(&self) -> bool {
        (self.common_data() & 0b10) != 0
    }
    /// Key string from Extended Data byte 0-5.
    pub fn key(&self) -> &'a str {
        ext_string_leaf(self.source_file, self.node_index)
    }
    /// Value type (first child if present).
    pub fn right(&self) -> Option<LazyTypeNode<'a>> {
        // Pattern 3 children are NOT gated by a Children bitmask in
        // Extended Data; they sit directly under the parent index.
        let child = first_child(self.source_file, self.node_index)?;
        LazyTypeNode::from_index(self.source_file, child, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeIndexSignature,
    Kind::TypeIndexSignature,
    "Lazy view of `TypeIndexSignature` (Kind 0xA4)."
);
impl<'a> LazyTypeIndexSignature<'a> {
    /// Key string.
    pub fn key(&self) -> &'a str {
        ext_string_leaf(self.source_file, self.node_index)
    }
    /// Value type (`right`).
    pub fn right(&self) -> Option<LazyTypeNode<'a>> {
        let child = first_child(self.source_file, self.node_index)?;
        LazyTypeNode::from_index(self.source_file, child, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeMappedType,
    Kind::TypeMappedType,
    "Lazy view of `TypeMappedType` (Kind 0xA5)."
);
impl<'a> LazyTypeMappedType<'a> {
    /// Key string.
    pub fn key(&self) -> &'a str {
        ext_string_leaf(self.source_file, self.node_index)
    }
    /// Value type (`right`).
    pub fn right(&self) -> Option<LazyTypeNode<'a>> {
        let child = first_child(self.source_file, self.node_index)?;
        LazyTypeNode::from_index(self.source_file, child, self.root_index)
    }
}

define_lazy_type_node!(
    LazyTypeMethodSignature,
    Kind::TypeMethodSignature,
    "Lazy view of `TypeMethodSignature` (Kind 0xA9)."
);
impl<'a> LazyTypeMethodSignature<'a> {
    /// Quote style for the method name (3-state).
    #[inline]
    pub fn quote(&self) -> u8 {
        self.common_data() & 0b11
    }
    /// `true` when the parameter NodeList was emitted.
    #[inline]
    pub fn has_parameters(&self) -> bool {
        (self.common_data() & 0b0100) != 0
    }
    /// `true` when the type-parameter NodeList was emitted.
    #[inline]
    pub fn has_type_parameters(&self) -> bool {
        (self.common_data() & 0b1000) != 0
    }
    /// Method name string.
    pub fn name(&self) -> &'a str {
        ext_string_leaf(self.source_file, self.node_index)
    }
}

define_lazy_type_node!(
    LazyTypeTemplateLiteral,
    Kind::TypeTemplateLiteral,
    "Lazy view of `TypeTemplateLiteral` (Kind 0x9D)."
);
impl<'a> LazyTypeTemplateLiteral<'a> {
    /// Number of literal segments.
    pub fn literal_count(&self) -> u16 {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        read_u16(self.source_file.bytes(), ext)
    }
    /// Get the n-th literal segment (panics when `index >= literal_count`).
    pub fn literal(&self, index: u16) -> &'a str {
        let ext = ext_offset(self.source_file, self.node_index) as usize;
        let field = read_string_field(
            self.source_file.bytes(),
            ext + 2 + index as usize * STRING_FIELD_SIZE,
        );
        self.source_file.get_string_by_field(field).unwrap_or("")
    }
}

define_lazy_type_node!(LazyTypeSymbol, Kind::TypeSymbol, "Lazy view of `TypeSymbol` (Kind 0x9F).");
impl<'a> LazyTypeSymbol<'a> {
    /// Whether the call carries an element argument.
    #[inline]
    pub fn has_element(&self) -> bool {
        (self.common_data() & 1) != 0
    }
    /// `Symbol(...)` callee text.
    pub fn value(&self) -> &'a str {
        ext_string_leaf(self.source_file, self.node_index)
    }
    /// Element argument when `has_element` is `true`.
    pub fn element(&self) -> Option<LazyTypeNode<'a>> {
        if !self.has_element() {
            return None;
        }
        let child = first_child(self.source_file, self.node_index)?;
        LazyTypeNode::from_index(self.source_file, child, self.root_index)
    }
}

// ===========================================================================
// Others — pure leaves (no payload)
// ===========================================================================
define_lazy_type_node!(LazyTypeNull, Kind::TypeNull, "Lazy view of `TypeNull` leaf (Kind 0x83).");
define_lazy_type_node!(
    LazyTypeUndefined,
    Kind::TypeUndefined,
    "Lazy view of `TypeUndefined` leaf (Kind 0x84)."
);
define_lazy_type_node!(LazyTypeAny, Kind::TypeAny, "Lazy view of `TypeAny` leaf (Kind 0x85).");
define_lazy_type_node!(
    LazyTypeUnknown,
    Kind::TypeUnknown,
    "Lazy view of `TypeUnknown` leaf (Kind 0x86)."
);
define_lazy_type_node!(
    LazyTypeUniqueSymbol,
    Kind::TypeUniqueSymbol,
    "Lazy view of `TypeUniqueSymbol` leaf (Kind 0x9E)."
);

// ===========================================================================
// Sum type for any TypeNode variant
// ===========================================================================

/// Wrapper enum produced when the parent node only knows it has *some*
/// TypeNode child.
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub enum LazyTypeNode<'a> {
    Name(LazyTypeName<'a>),
    Number(LazyTypeNumber<'a>),
    StringValue(LazyTypeStringValue<'a>),
    Property(LazyTypeProperty<'a>),
    SpecialNamePath(LazyTypeSpecialNamePath<'a>),
    Union(LazyTypeUnion<'a>),
    Intersection(LazyTypeIntersection<'a>),
    Generic(LazyTypeGeneric<'a>),
    Function(LazyTypeFunction<'a>),
    Object(LazyTypeObject<'a>),
    Tuple(LazyTypeTuple<'a>),
    Parenthesis(LazyTypeParenthesis<'a>),
    NamePath(LazyTypeNamePath<'a>),
    Nullable(LazyTypeNullable<'a>),
    NotNullable(LazyTypeNotNullable<'a>),
    Optional(LazyTypeOptional<'a>),
    Variadic(LazyTypeVariadic<'a>),
    Conditional(LazyTypeConditional<'a>),
    Infer(LazyTypeInfer<'a>),
    KeyOf(LazyTypeKeyOf<'a>),
    TypeOf(LazyTypeTypeOf<'a>),
    Import(LazyTypeImport<'a>),
    Predicate(LazyTypePredicate<'a>),
    Asserts(LazyTypeAsserts<'a>),
    AssertsPlain(LazyTypeAssertsPlain<'a>),
    ReadonlyArray(LazyTypeReadonlyArray<'a>),
    ObjectField(LazyTypeObjectField<'a>),
    JsdocObjectField(LazyTypeJsdocObjectField<'a>),
    IndexedAccessIndex(LazyTypeIndexedAccessIndex<'a>),
    CallSignature(LazyTypeCallSignature<'a>),
    ConstructorSignature(LazyTypeConstructorSignature<'a>),
    TypeParameter(LazyTypeTypeParameter<'a>),
    ParameterList(LazyTypeParameterList<'a>),
    ReadonlyProperty(LazyTypeReadonlyProperty<'a>),
    KeyValue(LazyTypeKeyValue<'a>),
    IndexSignature(LazyTypeIndexSignature<'a>),
    MappedType(LazyTypeMappedType<'a>),
    MethodSignature(LazyTypeMethodSignature<'a>),
    TemplateLiteral(LazyTypeTemplateLiteral<'a>),
    Symbol(LazyTypeSymbol<'a>),
    Null(LazyTypeNull<'a>),
    Undefined(LazyTypeUndefined<'a>),
    Any(LazyTypeAny<'a>),
    Unknown(LazyTypeUnknown<'a>),
    UniqueSymbol(LazyTypeUniqueSymbol<'a>),
}

impl<'a> LazyTypeNode<'a> {
    /// Construct the appropriate variant by reading the node's Kind byte.
    pub fn from_index(
        source_file: &'a LazySourceFile<'a>,
        node_index: u32,
        root_index: u32,
    ) -> Option<Self> {
        let kind_byte = source_file.bytes()[source_file.nodes_offset as usize
            + node_index as usize * crate::format::node_record::NODE_RECORD_SIZE];
        let kind = Kind::from_u8(kind_byte).ok()?;
        if !crate::format::kind::is_type_node(kind_byte) {
            return None;
        }
        // Macro to map each TypeNode Kind to its variant constructor.
        macro_rules! dispatch {
            ($($kind:ident => $variant:ident($struct:ty)),* $(,)?) => {
                match kind {
                    $(
                        Kind::$kind => Some(LazyTypeNode::$variant(
                            <$struct>::from_index(source_file, node_index, root_index),
                        )),
                    )*
                    _ => None,
                }
            };
        }
        dispatch! {
            TypeName => Name(LazyTypeName<'a>),
            TypeNumber => Number(LazyTypeNumber<'a>),
            TypeStringValue => StringValue(LazyTypeStringValue<'a>),
            TypeProperty => Property(LazyTypeProperty<'a>),
            TypeSpecialNamePath => SpecialNamePath(LazyTypeSpecialNamePath<'a>),
            TypeUnion => Union(LazyTypeUnion<'a>),
            TypeIntersection => Intersection(LazyTypeIntersection<'a>),
            TypeGeneric => Generic(LazyTypeGeneric<'a>),
            TypeFunction => Function(LazyTypeFunction<'a>),
            TypeObject => Object(LazyTypeObject<'a>),
            TypeTuple => Tuple(LazyTypeTuple<'a>),
            TypeParenthesis => Parenthesis(LazyTypeParenthesis<'a>),
            TypeNamePath => NamePath(LazyTypeNamePath<'a>),
            TypeNullable => Nullable(LazyTypeNullable<'a>),
            TypeNotNullable => NotNullable(LazyTypeNotNullable<'a>),
            TypeOptional => Optional(LazyTypeOptional<'a>),
            TypeVariadic => Variadic(LazyTypeVariadic<'a>),
            TypeConditional => Conditional(LazyTypeConditional<'a>),
            TypeInfer => Infer(LazyTypeInfer<'a>),
            TypeKeyOf => KeyOf(LazyTypeKeyOf<'a>),
            TypeTypeOf => TypeOf(LazyTypeTypeOf<'a>),
            TypeImport => Import(LazyTypeImport<'a>),
            TypePredicate => Predicate(LazyTypePredicate<'a>),
            TypeAsserts => Asserts(LazyTypeAsserts<'a>),
            TypeAssertsPlain => AssertsPlain(LazyTypeAssertsPlain<'a>),
            TypeReadonlyArray => ReadonlyArray(LazyTypeReadonlyArray<'a>),
            TypeObjectField => ObjectField(LazyTypeObjectField<'a>),
            TypeJsdocObjectField => JsdocObjectField(LazyTypeJsdocObjectField<'a>),
            TypeIndexedAccessIndex => IndexedAccessIndex(LazyTypeIndexedAccessIndex<'a>),
            TypeCallSignature => CallSignature(LazyTypeCallSignature<'a>),
            TypeConstructorSignature => ConstructorSignature(LazyTypeConstructorSignature<'a>),
            TypeTypeParameter => TypeParameter(LazyTypeTypeParameter<'a>),
            TypeParameterList => ParameterList(LazyTypeParameterList<'a>),
            TypeReadonlyProperty => ReadonlyProperty(LazyTypeReadonlyProperty<'a>),
            TypeKeyValue => KeyValue(LazyTypeKeyValue<'a>),
            TypeIndexSignature => IndexSignature(LazyTypeIndexSignature<'a>),
            TypeMappedType => MappedType(LazyTypeMappedType<'a>),
            TypeMethodSignature => MethodSignature(LazyTypeMethodSignature<'a>),
            TypeTemplateLiteral => TemplateLiteral(LazyTypeTemplateLiteral<'a>),
            TypeSymbol => Symbol(LazyTypeSymbol<'a>),
            TypeNull => Null(LazyTypeNull<'a>),
            TypeUndefined => Undefined(LazyTypeUndefined<'a>),
            TypeAny => Any(LazyTypeAny<'a>),
            TypeUnknown => Unknown(LazyTypeUnknown<'a>),
            TypeUniqueSymbol => UniqueSymbol(LazyTypeUniqueSymbol<'a>),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn type_node_lazy_structs_fit_in_16_bytes() {
        macro_rules! assert_size {
            ($t:ty) => {
                assert!(size_of::<$t>() <= 16, concat!(stringify!($t), " > 16 bytes"));
            };
        }
        assert_size!(LazyTypeName<'static>);
        assert_size!(LazyTypeFunction<'static>);
        assert_size!(LazyTypeKeyValue<'static>);
        assert_size!(LazyTypeNull<'static>);
        assert_size!(LazyTypeMethodSignature<'static>);
        assert_size!(LazyTypeTemplateLiteral<'static>);
    }

    #[test]
    fn lazy_type_node_sum_fits_in_24_bytes() {
        assert!(size_of::<LazyTypeNode<'static>>() <= 24);
    }
}
