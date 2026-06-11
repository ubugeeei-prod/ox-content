use rustc_hash::FxHashSet;

use super::super::{
    collapse_inline_whitespace, collapse_type_annotation_whitespace, process_doc_text,
    resolve_type_fragments, MarkdownLinkContext, TypeFragment,
};
use crate::model::ApiTypeParamDoc;
use crate::string_builder::StringBuilder;

/// Inline Markdown for a doc-text fragment (resolves `{@link}`), single-line.
pub(super) fn inline(text: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    collapse_inline_whitespace(&process_doc_text(text, context)).into_owned()
}

/// Escapes a value for use inside a Markdown table cell.
fn table_cell(text: &str) -> String {
    if text.contains('|') {
        text.replace('|', "\\|")
    } else {
        text.to_string()
    }
}

/// Append a table-cell value directly to `out` (pipes escaped), avoiding the
/// intermediate `String` that [`table_cell`] allocates for every cell.
pub(super) fn push_table_cell(out: &mut String, text: &str) {
    if text.contains('|') {
        let mut rest = text;
        while let Some(index) = rest.find('|') {
            out.push_str(&rest[..index]);
            out.push_str("\\|");
            rest = &rest[index + 1..];
        }
        out.push_str(rest);
    } else {
        out.push_str(text);
    }
}

/// Inline code for normal Markdown text; empty string if blank.
pub(super) fn code_span(value: &str) -> String {
    let value = collapse_inline_whitespace(value);
    if value.is_empty() {
        String::new()
    } else {
        let mut code = StringBuilder::with_capacity(value.len() + 2);
        code.push_char('`');
        code.push_str(&value);
        code.push_char('`');
        code.into_string()
    }
}

/// Inline code for a Markdown table cell (`|` escaped); empty string if blank.
pub(super) fn code_cell(value: &str) -> String {
    let value = collapse_inline_whitespace(value);
    if value.is_empty() {
        String::new()
    } else {
        let cell = table_cell(&value);
        let mut code = StringBuilder::with_capacity(cell.len() + 2);
        code.push_char('`');
        code.push_str(&cell);
        code.push_char('`');
        code.into_string()
    }
}

/// Escapes Markdown-significant characters in a type annotation's non-identifier
/// text (generics, unions, arrays, …) so they render literally. Pipes are only
/// escaped inside table cells.
fn escape_type_text(text: &str, in_cell: bool) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '`' => out.push_str("\\`"),
            '<' => out.push_str("\\<"),
            '>' => out.push_str("\\>"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            '|' if in_cell => out.push_str("\\|"),
            _ => out.push(ch),
        }
    }
    out
}

/// Renders a TypeScript type annotation, linking known symbols. When no identifier
/// resolves to a symbol page the type is returned unchanged as a single inline-code
/// span (`code(value)`); otherwise it is fragmented TypeDoc-style: each identifier
/// is its own inline-code span (linked when resolvable) and punctuation is escaped.
fn linked_type(
    value: &str,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &FxHashSet<&str>,
    code: fn(&str) -> String,
    in_cell: bool,
) -> String {
    let value = collapse_type_annotation_whitespace(value);
    match resolve_type_fragments(&value, context, skip) {
        None => code(&value),
        Some(fragments) => {
            let mut out = String::new();
            for fragment in fragments {
                match fragment {
                    TypeFragment::Text(text) => out.push_str(&escape_type_text(&text, in_cell)),
                    TypeFragment::Code(text) => out.push_str(&code(&text)),
                    TypeFragment::Link { name, href } => {
                        out.push('[');
                        out.push_str(&code(&name));
                        out.push_str("](");
                        out.push_str(&href);
                        out.push(')');
                    }
                }
            }
            out
        }
    }
}

pub(super) fn linked_type_cell(value: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    linked_type(value, context, &FxHashSet::default(), code_cell, true)
}

pub(super) fn linked_type_span(value: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    linked_type(value, context, &FxHashSet::default(), code_span, false)
}

/// Builds the Name cell for a type parameter: `` `T` `` plus optional `*extends*`
/// constraint and `=` default. The constraint/default link known symbols; the
/// parameter's own name and its siblings (`skip`) are never linked.
pub(super) fn type_param_name_cell(
    type_param: &ApiTypeParamDoc,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &FxHashSet<&str>,
) -> String {
    type_param_name(type_param, context, skip, code_cell, true)
}

pub(super) fn type_param_name_span(
    type_param: &ApiTypeParamDoc,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &FxHashSet<&str>,
) -> String {
    type_param_name(type_param, context, skip, code_span, false)
}

fn type_param_name(
    type_param: &ApiTypeParamDoc,
    context: Option<&MarkdownLinkContext<'_>>,
    skip: &FxHashSet<&str>,
    code: fn(&str) -> String,
    in_cell: bool,
) -> String {
    let mut cell = code(&type_param.name);
    if let Some(constraint) = &type_param.constraint {
        cell.push_str(" *extends* ");
        cell.push_str(&linked_type(constraint, context, skip, code, in_cell));
    }
    if let Some(default) = &type_param.default {
        cell.push_str(" = ");
        cell.push_str(&linked_type(default, context, skip, code, in_cell));
    }
    cell
}
