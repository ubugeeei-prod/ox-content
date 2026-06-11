use phf::phf_set;
use rustc_hash::FxHashSet;

use super::links::{format_symbol_href, resolve_symbol_location, MarkdownLinkContext};

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

/// Tokenizes a TypeScript type annotation and resolves its identifiers against the
/// symbol map. Returns `None` when no identifier resolves to a link, so callers can
/// keep their existing single-code-span rendering (zero output churn for unlinkable
/// types). String and template literals are read as opaque text so literal types
/// like `"Command"` never produce false links.
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

            if !skip.contains(ident) && !TS_INTRINSIC_TYPES.contains(ident) {
                if let Some(location) = resolve_symbol_location(ident, context) {
                    fragments.push(TypeFragment::Link {
                        name: ident.to_string(),
                        href: format_symbol_href(context, location),
                    });
                    has_link = true;
                    continue;
                }
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
