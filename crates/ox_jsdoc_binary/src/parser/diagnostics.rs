// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Parser-level recovery and structural diagnostics.
//!
//! Verbatim port of `crates/ox_jsdoc/src/parser/diagnostics.rs`. Tag-policy
//! validation lives in `validator` (Phase 1.3+).
//!
//! In the typed-AST parser these diagnostics are wrapped in
//! `oxc_diagnostics::OxcDiagnostic`. The binary parser keeps a leaner
//! representation that points at an interned message string + the
//! offending span; consumers can convert into `OxcDiagnostic` at the API
//! boundary if richer formatting is needed.

/// Parser-level recovery and structural errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserDiagnosticKind {
    /// Input text does not start with `/**`.
    NotAJSDocBlock,
    /// Input text does not end with `*/`.
    UnclosedBlockComment,
    /// Source span exceeds the u32 byte offset range.
    SpanOverflow,
    /// `{@...}` inline tag is missing its closing `}`.
    UnclosedInlineTag,
    /// `{...}` type expression is missing its closing `}`.
    UnclosedTypeExpression,
    /// Triple-backtick fenced code block is missing its closing fence.
    UnclosedFence,
    /// Tag header could not be tokenised (e.g. `@` followed by non-identifier).
    InvalidTagStart,
    /// Inline tag body could not be tokenised.
    InvalidInlineTagStart,
}

/// Human-readable message text for a parser diagnostic.
#[must_use]
pub const fn parser_diagnostic_message(kind: ParserDiagnosticKind) -> &'static str {
    match kind {
        ParserDiagnosticKind::NotAJSDocBlock => "input is not a JSDoc block comment",
        ParserDiagnosticKind::UnclosedBlockComment => "JSDoc block comment is not closed",
        ParserDiagnosticKind::SpanOverflow => "JSDoc comment span exceeds u32 byte offset range",
        ParserDiagnosticKind::UnclosedInlineTag => "inline tag is not closed",
        ParserDiagnosticKind::UnclosedTypeExpression => "type expression is not closed",
        ParserDiagnosticKind::UnclosedFence => "fenced code block is not closed",
        ParserDiagnosticKind::InvalidTagStart => "invalid block tag start",
        ParserDiagnosticKind::InvalidInlineTagStart => "invalid inline tag start",
    }
}

/// Type parser diagnostics for malformed `{...}` expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDiagnosticKind {
    /// No prefix parslet matched the current token.
    NoParsletFound,
    /// Expected a specific token but found something else.
    ExpectedToken,
    /// Generic type parameter list `<...>` is not closed.
    UnclosedGeneric,
    /// Parenthesized type `(...)` is not closed.
    UnclosedParenthesis,
    /// Tuple type `[...]` is not closed.
    UnclosedTuple,
    /// Object type `{...}` is not closed.
    UnclosedObject,
    /// Template literal type is not closed.
    UnclosedTemplateLiteral,
    /// General invalid type expression.
    InvalidTypeExpression,
    /// Unexpected token after a complete type expression.
    EarlyEndOfParse,
}

/// Human-readable message text for a type-parser diagnostic.
#[must_use]
pub const fn type_diagnostic_message(kind: TypeDiagnosticKind) -> &'static str {
    match kind {
        TypeDiagnosticKind::NoParsletFound => "unexpected token in type expression",
        TypeDiagnosticKind::ExpectedToken => "expected token in type expression",
        TypeDiagnosticKind::UnclosedGeneric => "generic type parameter list is not closed",
        TypeDiagnosticKind::UnclosedParenthesis => "parenthesized type is not closed",
        TypeDiagnosticKind::UnclosedTuple => "tuple type is not closed",
        TypeDiagnosticKind::UnclosedObject => "object type is not closed",
        TypeDiagnosticKind::UnclosedTemplateLiteral => "template literal type is not closed",
        TypeDiagnosticKind::InvalidTypeExpression => "invalid type expression",
        TypeDiagnosticKind::EarlyEndOfParse => "unexpected token after type expression",
    }
}

/// Either a parser-level or a type-parser-level diagnostic kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// Structural parser diagnostic.
    Parser(ParserDiagnosticKind),
    /// Type expression parser diagnostic.
    Type(TypeDiagnosticKind),
}

impl DiagnosticKind {
    /// Resolve into a static message string.
    #[inline]
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Parser(k) => parser_diagnostic_message(k),
            Self::Type(k) => type_diagnostic_message(k),
        }
    }
}
