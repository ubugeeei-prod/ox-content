use phf::phf_set;
use rustc_hash::FxHashSet;

use super::links::{MarkdownLinkContext, format_symbol_href, resolve_symbol_location};

/// A fragment of a tokenized TypeScript type annotation.
pub(super) enum TypeFragment {
    /// Punctuation / separators / whitespace between identifiers (raw, unescaped).
    Text(String),
    /// An identifier that did not resolve to a known symbol (render as code).
    Code(String),
    /// An identifier that resolved to a symbol page (render as a linked code span).
    Link { name: String, href: String },
}

/// TypeScript intrinsic / primitive type names. These are language built-ins, so
/// they are never linked inside a type annotation even when a same-named symbol
/// exists in the docs (e.g. a `string()` / `boolean()` combinator). This matches
/// TypeDoc, which renders intrinsic types as plain code. Applies to type
/// annotations only - JSDoc `{@link}` / `[Symbol]` references are unaffected.
static TS_INTRINSIC_TYPES: phf::Set<&'static str> = phf_set! {
    "any",
    "bigint",
    "boolean",
    "false",
    "never",
    "null",
    "number",
    "object",
    "string",
    "symbol",
    "this",
    "true",
    "undefined",
    "unknown",
    "void",
};

fn is_type_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$'
}

fn is_type_ident_part(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

/// Modifiers that may precede a member name in an object type (`readonly x: T`,
/// `get x(): T`). They are skipped when looking for what starts the member.
static MEMBER_MODIFIERS: phf::Set<&'static str> = phf_set! {
    "get",
    "readonly",
    "set",
};

/// What precedes a name, skipping spaces, member modifiers and a Flow variance
/// sigil (`+x: T`).
enum NameStart {
    /// The start of the annotation.
    Start,
    /// A rest parameter or tuple element (`...x: T`).
    Rest,
    /// The byte before the name: a delimiter, or the last byte of a preceding
    /// word. The Flow exact object opener `{|` reads as `{`.
    Byte(u8),
}

fn skip_spaces_forward(bytes: &[u8], mut index: usize) -> usize {
    while bytes.get(index) == Some(&b' ') {
        index += 1;
    }
    index
}

fn skip_spaces_backward(bytes: &[u8], mut index: usize) -> usize {
    while index > 0 && bytes[index - 1] == b' ' {
        index -= 1;
    }
    index
}

/// The word that ends at `end` (spaces skipped), with its start index.
fn word_before(value: &str, end: usize) -> Option<(usize, &str)> {
    let bytes = value.as_bytes();
    let end = skip_spaces_backward(bytes, end);
    let mut start = end;
    while start > 0 && is_type_ident_part(bytes[start - 1]) {
        start -= 1;
    }
    (start < end).then(|| (start, &value[start..end]))
}

/// The word that starts at `start` (spaces skipped).
fn word_after(value: &str, start: usize) -> Option<&str> {
    let bytes = value.as_bytes();
    let start = skip_spaces_forward(bytes, start);
    let mut end = start;
    while end < bytes.len() && is_type_ident_part(bytes[end]) {
        end += 1;
    }
    (start < end).then(|| &value[start..end])
}

fn name_start(value: &str, start: usize) -> NameStart {
    let bytes = value.as_bytes();
    let mut index = skip_spaces_backward(bytes, start);
    loop {
        if index == 0 {
            return NameStart::Start;
        }
        match bytes[index - 1] {
            b'+' | b'-' => index = skip_spaces_backward(bytes, index - 1),
            b'.' if index >= 3 && &bytes[index - 3..index] == b"..." => return NameStart::Rest,
            b'|' if index >= 2 && bytes[index - 2] == b'{' => return NameStart::Byte(b'{'),
            byte if is_type_ident_part(byte) => match word_before(value, index) {
                Some((word_start, word)) if MEMBER_MODIFIERS.contains(word) => {
                    index = skip_spaces_backward(bytes, word_start);
                }
                _ => return NameStart::Byte(byte),
            },
            byte => return NameStart::Byte(byte),
        }
    }
}

/// Whether the identifier at `start..end` is a name the type declares rather than
/// a reference: a property or method name, a parameter name, an index signature
/// key, a tuple member label, or the parameter of a type predicate. TypeDoc links
/// references only, so these stay plain code even when an exported symbol has the
/// same name (e.g. `required` in `{ required: R }` next to a `required()` function).
fn is_declared_name(value: &str, start: usize, end: usize) -> bool {
    let bytes = value.as_bytes();
    let next = skip_spaces_forward(bytes, end);
    // An optional marker sits between a name and its annotation (`x?: T`, `x?(): T`).
    // A conditional type's `?` is never followed by `:`, and the name before it ends
    // the extends type, so it never follows the `{` or `;` that a method name follows.
    let after =
        if bytes.get(next) == Some(&b'?') { skip_spaces_forward(bytes, next + 1) } else { next };
    match bytes.get(after) {
        // `x: T`: a property, parameter, index signature key or tuple member. A
        // conditional type's branch is also followed by `:`, but it follows `?`,
        // `:` or an operator, never a delimiter that starts a member or parameter.
        Some(b':') => matches!(
            name_start(value, start),
            NameStart::Start | NameStart::Rest | NameStart::Byte(b'{' | b';' | b',' | b'(' | b'[')
        ),
        // `x(): T` / `x<T>(): T`: a method signature at the start of an object type
        // member. Elsewhere `x(` is a call (a mixin in a heritage clause) and `x<` a
        // generic reference, so both stay linkable.
        Some(b'(' | b'<') => matches!(name_start(value, start), NameStart::Byte(b'{' | b';')),
        // `x is T` / `asserts x`: the parameter of a type predicate.
        _ => {
            word_after(value, next) == Some("is")
                || matches!(word_before(value, start), Some((_, "asserts")))
        }
    }
}

/// Tokenizes a TypeScript type annotation and resolves its identifiers against the
/// symbol map. Returns `None` when no identifier resolves to a link, so callers can
/// keep their existing single-code-span rendering (zero output churn for unlinkable
/// types). String and template literals are read as opaque text so literal types
/// like `"Command"` never produce false links, and names the type declares
/// (properties, parameters, …) are never linked (see `is_declared_name`).
pub(super) fn resolve_type_fragments(
    value: &str,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &FxHashSet<&str>,
) -> Option<Vec<TypeFragment>> {
    let context = context?;
    let bytes = value.as_bytes();
    let mut fragments = Vec::new();
    let mut text_start = 0;
    let mut index = 0;
    let mut has_link = false;

    while index < bytes.len() {
        let byte = bytes[index];

        // String / template literals stay opaque text (no identifier linking inside).
        if byte == b'\'' || byte == b'"' || byte == b'`' {
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index += 2;
                    continue;
                }
                let closing = bytes[index] == byte;
                index += 1;
                if closing {
                    break;
                }
            }
            continue;
        }

        if is_type_ident_start(byte) {
            let start = index;
            index += 1;
            while index < bytes.len() && is_type_ident_part(bytes[index]) {
                index += 1;
            }
            let ident = &value[start..index];

            if text_start < start {
                fragments.push(TypeFragment::Text(value[text_start..start].to_string()));
            }
            text_start = index;

            if !skip.contains(ident)
                && !TS_INTRINSIC_TYPES.contains(ident)
                && let Some(location) = resolve_symbol_location(ident, context)
                && !is_declared_name(value, start, index)
            {
                fragments.push(TypeFragment::Link {
                    name: ident.to_string(),
                    href: format_symbol_href(context, location),
                });
                has_link = true;
                continue;
            }
            fragments.push(TypeFragment::Code(ident.to_string()));
            continue;
        }

        index += 1;
    }

    if text_start < value.len() {
        fragments.push(TypeFragment::Text(value[text_start..].to_string()));
    }

    has_link.then_some(fragments)
}
