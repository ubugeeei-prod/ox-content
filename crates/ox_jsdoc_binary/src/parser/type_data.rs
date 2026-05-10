// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Intermediate type-expression AST used by the binary parser.
//!
//! Mirrors `crates/ox_jsdoc/src/type_parser/ast.rs` (`TypeNode` and its 45
//! variant structs) but uses the std heap rather than an arena so the
//! binary parser does not need to thread an [`oxc_allocator::Allocator`]
//! through its Pratt parser. The lifetime `'a` is the source-text borrow.
//!
//! After parsing, the [`crate::parser::type_emit`] module walks these
//! values in DFS pre-order and writes them to a [`BinaryWriter`].
//!
//! [`BinaryWriter`]: crate::writer::BinaryWriter

use oxc_span::Span;

/// Position of a prefix/suffix modifier (`?T` vs `T?`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierPosition {
    /// Modifier appears before the operand (`?T`).
    Prefix,
    /// Modifier appears after the operand (`T?`).
    Suffix,
}

/// Brackets used for generic types (`Array<T>` vs `T[]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericBrackets {
    /// `<T>` angle brackets.
    Angle,
    /// `T[]` square brackets.
    Square,
}

/// Quote style for string literals and property keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteStyle {
    /// `'...'`
    Single,
    /// `"..."`
    Double,
}

/// Separator used in object types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectSeparator {
    /// `,`
    Comma,
    /// `;`
    Semicolon,
    /// `\n`
    Linebreak,
    /// `,\n`
    CommaAndLinebreak,
    /// `;\n`
    SemicolonAndLinebreak,
}

/// Name path type for `A.B`, `A#B`, `A~B`, `A["key"]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamePathType {
    /// `.` property access.
    Property,
    /// `#` instance member.
    Instance,
    /// `~` inner member.
    Inner,
    /// `[...]` bracket access.
    PropertyBrackets,
}

/// Special name path prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialPathType {
    /// `module:x`
    Module,
    /// `event:x`
    Event,
    /// `external:x`
    External,
}

/// Variadic position (`...T` vs `T...` vs bare `...`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariadicPosition {
    /// `...T`
    Prefix,
    /// `T...`
    Suffix,
}

/// Parse mode for the type parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    /// JSDoc mode: supports JSDoc-specific syntax with loose lexer.
    Jsdoc,
    /// Closure mode: supports Closure Compiler syntax with loose lexer.
    Closure,
    /// TypeScript mode: strict lexer.
    Typescript,
}

impl ParseMode {
    /// Whether the lexer should use loose rules (NaN, Infinity, hyphens).
    #[inline]
    #[must_use]
    pub const fn is_loose(self) -> bool {
        matches!(self, Self::Jsdoc | Self::Closure)
    }

    /// Whether this is jsdoc mode.
    #[inline]
    #[must_use]
    pub const fn is_jsdoc(self) -> bool {
        matches!(self, Self::Jsdoc)
    }

    /// Whether this is closure mode.
    #[inline]
    #[must_use]
    pub const fn is_closure(self) -> bool {
        matches!(self, Self::Closure)
    }

    /// Whether this is typescript mode.
    #[inline]
    #[must_use]
    pub const fn is_typescript(self) -> bool {
        matches!(self, Self::Typescript)
    }
}

// ============================================================================
// TypeNodeData enum + 45 variant structs (mirrors `TypeNode`).
// ============================================================================

/// Parsed type expression. Tree of variants matching the binary AST's 45
/// TypeNode kinds.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum TypeNodeData<'a> {
    Name(TypeName<'a>),
    Number(TypeNumber<'a>),
    StringValue(TypeStringValue<'a>),
    Null(TypeNull),
    Undefined(TypeUndefined),
    Any(TypeAny),
    Unknown(TypeUnknown),
    Union(TypeUnion<'a>),
    Intersection(TypeIntersection<'a>),
    Generic(TypeGeneric<'a>),
    Function(TypeFunction<'a>),
    Object(TypeObject<'a>),
    Tuple(TypeTuple<'a>),
    Parenthesis(TypeParenthesis<'a>),
    NamePath(TypeNamePath<'a>),
    SpecialNamePath(TypeSpecialNamePath<'a>),
    Nullable(TypeNullable<'a>),
    NotNullable(TypeNotNullable<'a>),
    Optional(TypeOptional<'a>),
    Variadic(TypeVariadic<'a>),
    Conditional(TypeConditional<'a>),
    Infer(TypeInfer<'a>),
    KeyOf(TypeKeyOf<'a>),
    TypeOf(TypeTypeOf<'a>),
    Import(TypeImport<'a>),
    Predicate(TypePredicate<'a>),
    Asserts(TypeAsserts<'a>),
    AssertsPlain(TypeAssertsPlain<'a>),
    ReadonlyArray(TypeReadonlyArray<'a>),
    TemplateLiteral(TypeTemplateLiteral<'a>),
    UniqueSymbol(TypeUniqueSymbol),
    Symbol(TypeSymbol<'a>),
    ObjectField(TypeObjectField<'a>),
    JsdocObjectField(TypeJsdocObjectField<'a>),
    KeyValue(TypeKeyValue<'a>),
    Property(TypeProperty<'a>),
    IndexSignature(TypeIndexSignature<'a>),
    MappedType(TypeMappedType<'a>),
    TypeParameter(TypeTypeParameter<'a>),
    CallSignature(TypeCallSignature<'a>),
    ConstructorSignature(TypeConstructorSignature<'a>),
    MethodSignature(TypeMethodSignature<'a>),
    IndexedAccessIndex(TypeIndexedAccessIndex<'a>),
    ParameterList(TypeParameterList<'a>),
    ReadonlyProperty(TypeReadonlyProperty<'a>),
}

impl<'a> TypeNodeData<'a> {
    /// Source span covered by this type expression.
    #[inline]
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Name(n) => n.span,
            Self::Number(n) => n.span,
            Self::StringValue(n) => n.span,
            Self::Null(n) => n.span,
            Self::Undefined(n) => n.span,
            Self::Any(n) => n.span,
            Self::Unknown(n) => n.span,
            Self::Union(n) => n.span,
            Self::Intersection(n) => n.span,
            Self::Generic(n) => n.span,
            Self::Function(n) => n.span,
            Self::Object(n) => n.span,
            Self::Tuple(n) => n.span,
            Self::Parenthesis(n) => n.span,
            Self::NamePath(n) => n.span,
            Self::SpecialNamePath(n) => n.span,
            Self::Nullable(n) => n.span,
            Self::NotNullable(n) => n.span,
            Self::Optional(n) => n.span,
            Self::Variadic(n) => n.span,
            Self::Conditional(n) => n.span,
            Self::Infer(n) => n.span,
            Self::KeyOf(n) => n.span,
            Self::TypeOf(n) => n.span,
            Self::Import(n) => n.span,
            Self::Predicate(n) => n.span,
            Self::Asserts(n) => n.span,
            Self::AssertsPlain(n) => n.span,
            Self::ReadonlyArray(n) => n.span,
            Self::TemplateLiteral(n) => n.span,
            Self::UniqueSymbol(n) => n.span,
            Self::Symbol(n) => n.span,
            Self::ObjectField(n) => n.span,
            Self::JsdocObjectField(n) => n.span,
            Self::KeyValue(n) => n.span,
            Self::Property(n) => n.span,
            Self::IndexSignature(n) => n.span,
            Self::MappedType(n) => n.span,
            Self::TypeParameter(n) => n.span,
            Self::CallSignature(n) => n.span,
            Self::ConstructorSignature(n) => n.span,
            Self::MethodSignature(n) => n.span,
            Self::IndexedAccessIndex(n) => n.span,
            Self::ParameterList(n) => n.span,
            Self::ReadonlyProperty(n) => n.span,
        }
    }
}

#[allow(missing_docs)]
mod structs {
    use super::*;

    #[derive(Debug)]
    pub struct TypeName<'a> {
        pub span: Span,
        pub value: &'a str,
    }
    #[derive(Debug)]
    pub struct TypeNumber<'a> {
        pub span: Span,
        pub value: &'a str,
    }
    #[derive(Debug)]
    pub struct TypeStringValue<'a> {
        pub span: Span,
        pub value: &'a str,
        pub quote: QuoteStyle,
    }
    #[derive(Debug)]
    pub struct TypeNull {
        pub span: Span,
    }
    #[derive(Debug)]
    pub struct TypeUndefined {
        pub span: Span,
    }
    #[derive(Debug)]
    pub struct TypeAny {
        pub span: Span,
    }
    #[derive(Debug)]
    pub struct TypeUnknown {
        pub span: Span,
    }
    #[derive(Debug)]
    pub struct TypeUnion<'a> {
        pub span: Span,
        pub elements: Vec<Box<TypeNodeData<'a>>>,
    }
    #[derive(Debug)]
    pub struct TypeIntersection<'a> {
        pub span: Span,
        pub elements: Vec<Box<TypeNodeData<'a>>>,
    }
    #[derive(Debug)]
    pub struct TypeGeneric<'a> {
        pub span: Span,
        pub left: Box<TypeNodeData<'a>>,
        pub elements: Vec<Box<TypeNodeData<'a>>>,
        pub brackets: GenericBrackets,
        pub dot: bool,
    }
    #[derive(Debug)]
    pub struct TypeFunction<'a> {
        pub span: Span,
        pub parameters: Vec<Box<TypeNodeData<'a>>>,
        pub return_type: Option<Box<TypeNodeData<'a>>>,
        pub type_parameters: Vec<Box<TypeNodeData<'a>>>,
        pub constructor: bool,
        pub arrow: bool,
        pub parenthesis: bool,
    }
    #[derive(Debug)]
    pub struct TypeObject<'a> {
        pub span: Span,
        pub elements: Vec<Box<TypeNodeData<'a>>>,
        pub separator: Option<ObjectSeparator>,
    }
    #[derive(Debug)]
    pub struct TypeTuple<'a> {
        pub span: Span,
        pub elements: Vec<Box<TypeNodeData<'a>>>,
    }
    #[derive(Debug)]
    pub struct TypeParenthesis<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeNamePath<'a> {
        pub span: Span,
        pub left: Box<TypeNodeData<'a>>,
        pub right: Box<TypeNodeData<'a>>,
        pub path_type: NamePathType,
    }
    #[derive(Debug)]
    pub struct TypeSpecialNamePath<'a> {
        pub span: Span,
        pub value: &'a str,
        pub special_type: SpecialPathType,
        pub quote: Option<QuoteStyle>,
    }
    #[derive(Debug)]
    pub struct TypeNullable<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
        pub position: ModifierPosition,
    }
    #[derive(Debug)]
    pub struct TypeNotNullable<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
        pub position: ModifierPosition,
    }
    #[derive(Debug)]
    pub struct TypeOptional<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
        pub position: ModifierPosition,
    }
    #[derive(Debug)]
    pub struct TypeVariadic<'a> {
        pub span: Span,
        pub element: Option<Box<TypeNodeData<'a>>>,
        pub position: Option<VariadicPosition>,
        pub square_brackets: bool,
    }
    #[derive(Debug)]
    pub struct TypeConditional<'a> {
        pub span: Span,
        pub checks_type: Box<TypeNodeData<'a>>,
        pub extends_type: Box<TypeNodeData<'a>>,
        pub true_type: Box<TypeNodeData<'a>>,
        pub false_type: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeInfer<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeKeyOf<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeTypeOf<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeImport<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypePredicate<'a> {
        pub span: Span,
        pub left: Box<TypeNodeData<'a>>,
        pub right: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeAsserts<'a> {
        pub span: Span,
        pub left: Box<TypeNodeData<'a>>,
        pub right: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeAssertsPlain<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeReadonlyArray<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeTemplateLiteral<'a> {
        pub span: Span,
        pub literals: Vec<&'a str>,
        pub interpolations: Vec<Box<TypeNodeData<'a>>>,
    }
    #[derive(Debug)]
    pub struct TypeUniqueSymbol {
        pub span: Span,
    }
    #[derive(Debug)]
    pub struct TypeSymbol<'a> {
        pub span: Span,
        pub value: &'a str,
        pub element: Option<Box<TypeNodeData<'a>>>,
    }
    #[derive(Debug)]
    pub struct TypeObjectField<'a> {
        pub span: Span,
        pub key: Box<TypeNodeData<'a>>,
        pub right: Option<Box<TypeNodeData<'a>>>,
        pub optional: bool,
        pub readonly: bool,
        pub quote: Option<QuoteStyle>,
    }
    #[derive(Debug)]
    pub struct TypeJsdocObjectField<'a> {
        pub span: Span,
        pub left: Box<TypeNodeData<'a>>,
        pub right: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeKeyValue<'a> {
        pub span: Span,
        pub key: &'a str,
        pub right: Option<Box<TypeNodeData<'a>>>,
        pub optional: bool,
        pub variadic: bool,
    }
    #[derive(Debug)]
    pub struct TypeProperty<'a> {
        pub span: Span,
        pub value: &'a str,
        pub quote: Option<QuoteStyle>,
    }
    #[derive(Debug)]
    pub struct TypeIndexSignature<'a> {
        pub span: Span,
        pub key: &'a str,
        pub right: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeMappedType<'a> {
        pub span: Span,
        pub key: &'a str,
        pub right: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeTypeParameter<'a> {
        pub span: Span,
        pub name: Box<TypeNodeData<'a>>,
        pub constraint: Option<Box<TypeNodeData<'a>>>,
        pub default_value: Option<Box<TypeNodeData<'a>>>,
    }
    #[derive(Debug)]
    pub struct TypeCallSignature<'a> {
        pub span: Span,
        pub parameters: Vec<Box<TypeNodeData<'a>>>,
        pub return_type: Box<TypeNodeData<'a>>,
        pub type_parameters: Vec<Box<TypeNodeData<'a>>>,
    }
    #[derive(Debug)]
    pub struct TypeConstructorSignature<'a> {
        pub span: Span,
        pub parameters: Vec<Box<TypeNodeData<'a>>>,
        pub return_type: Box<TypeNodeData<'a>>,
        pub type_parameters: Vec<Box<TypeNodeData<'a>>>,
    }
    #[derive(Debug)]
    pub struct TypeMethodSignature<'a> {
        pub span: Span,
        pub name: &'a str,
        pub parameters: Vec<Box<TypeNodeData<'a>>>,
        pub return_type: Box<TypeNodeData<'a>>,
        pub type_parameters: Vec<Box<TypeNodeData<'a>>>,
        pub quote: Option<QuoteStyle>,
    }
    #[derive(Debug)]
    pub struct TypeIndexedAccessIndex<'a> {
        pub span: Span,
        pub right: Box<TypeNodeData<'a>>,
    }
    #[derive(Debug)]
    pub struct TypeParameterList<'a> {
        pub span: Span,
        pub elements: Vec<Box<TypeNodeData<'a>>>,
    }
    #[derive(Debug)]
    pub struct TypeReadonlyProperty<'a> {
        pub span: Span,
        pub element: Box<TypeNodeData<'a>>,
    }
}

pub use structs::*;
