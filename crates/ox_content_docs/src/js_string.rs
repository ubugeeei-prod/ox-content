use std::borrow::Cow;

use oxc_ast::ast::StringLiteral;

/// Returns the value of a string literal as text.
///
/// JavaScript strings can hold lone surrogates, which Rust strings cannot, so
/// those are written as `\uXXXX` escapes. Other values are borrowed as is.
pub fn string_literal_text<'a>(literal: &StringLiteral<'a>) -> Cow<'a, str> {
    let value = literal.value;
    value.as_str().map_or_else(|| Cow::Owned(value.display().to_string()), Cow::Borrowed)
}
