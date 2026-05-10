// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Token definitions for the JSDoc type expression Pratt parser.
//!
//! Verbatim port of `crates/ox_jsdoc/src/type_parser/token.rs`.

/// Token kind for JSDoc type expressions.
///
/// `#[repr(u8)]` keeps `Token` at 12 bytes (Copy-friendly).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum TokenKind {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Pipe,
    Amp,
    Lt,
    Gt,
    Semicolon,
    Comma,
    Star,
    Question,
    Bang,
    Eq,
    Colon,
    Dot,
    At,
    Hash,
    Tilde,
    Slash,
    Arrow,
    Ellipsis,
    Null,
    Undefined,
    Function,
    This,
    New,
    Module,
    Event,
    Extends,
    External,
    Typeof,
    Keyof,
    Readonly,
    Import,
    Infer,
    Is,
    In,
    Asserts,
    Unique,
    Symbol,
    Identifier,
    StringValue,
    TemplateLiteral,
    Number,
    EOF,
}

impl TokenKind {
    /// Whether this token kind is a keyword that can also serve as an
    /// identifier in name contexts.
    #[inline]
    #[must_use]
    pub fn is_keyword(self) -> bool {
        matches!(
            self,
            Self::Null
                | Self::Undefined
                | Self::Function
                | Self::This
                | Self::New
                | Self::Module
                | Self::Event
                | Self::Extends
                | Self::External
                | Self::Typeof
                | Self::Keyof
                | Self::Readonly
                | Self::Import
                | Self::Infer
                | Self::Is
                | Self::In
                | Self::Asserts
                | Self::Unique
                | Self::Symbol
        )
    }

    /// Whether this token can appear as a type name in prefix position.
    #[inline]
    #[must_use]
    pub fn is_base_name_token(self) -> bool {
        matches!(
            self,
            Self::Module
                | Self::Keyof
                | Self::Event
                | Self::External
                | Self::Readonly
                | Self::Is
                | Self::Typeof
                | Self::In
                | Self::Null
                | Self::Undefined
                | Self::Function
                | Self::Asserts
                | Self::Infer
                | Self::Extends
                | Self::Import
                | Self::Unique
                | Self::Symbol
        )
    }
}

/// A single token produced by the lexer (12 bytes, `Copy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// Absolute start byte offset in the source.
    pub start: u32,
    /// Absolute end byte offset in the source.
    pub end: u32,
    /// The kind of this token.
    pub kind: TokenKind,
}

impl Token {
    /// Construct a new token.
    #[inline]
    #[must_use]
    pub fn new(kind: TokenKind, start: u32, end: u32) -> Self {
        Self { start, end, kind }
    }

    /// Construct an EOF token at the given offset.
    #[inline]
    #[must_use]
    pub fn eof(offset: u32) -> Self {
        Self { start: offset, end: offset, kind: TokenKind::EOF }
    }
}
