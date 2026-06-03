//! Error types for the parser.

use ox_content_ast::Span;
use thiserror::Error;

/// Result type for parsing operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// Parse error.
#[derive(Debug, Error)]
#[allow(clippy::disallowed_types)]
pub enum ParseError {
    /// Unexpected token encountered.
    #[error("unexpected token at {span:?}: expected {expected}, found {found}")]
    UnexpectedToken {
        /// The span where the error occurred.
        span: Span,
        /// Expected token description.
        expected: String,
        /// Found token description.
        found: String,
    },

    /// Unexpected end of input.
    #[error("unexpected end of input at {span:?}")]
    UnexpectedEof {
        /// The span where the error occurred.
        span: Span,
    },

    /// Invalid syntax.
    #[error("invalid syntax at {span:?}: {message}")]
    InvalidSyntax {
        /// The span where the error occurred.
        span: Span,
        /// Error message.
        message: String,
    },

    /// Nesting too deep.
    #[error("nesting too deep at {span:?}: maximum depth is {max_depth}")]
    NestingTooDeep {
        /// The span where the error occurred.
        span: Span,
        /// Maximum allowed depth.
        max_depth: usize,
    },
}

impl ParseError {
    /// Returns the span where the error occurred.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::UnexpectedToken { span, .. }
            | Self::UnexpectedEof { span }
            | Self::InvalidSyntax { span, .. }
            | Self::NestingTooDeep { span, .. } => *span,
        }
    }
}
