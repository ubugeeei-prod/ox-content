use phf::phf_map;

use super::{capitalize_ascii, entry_anchor};
use crate::model::ApiDocEntry;
use crate::string_builder::{join3, join5, StringBuilder};

static TYPEDOC_KIND_SEGMENT: phf::Map<&'static str, &'static str> = phf_map! {
    "function" => "functions",
    "class" => "classes",
    "interface" => "interfaces",
    "type" => "type-aliases",
    "enum" => "enumerations",
    "variable" => "variables",
    "const" => "variables",
    "module" => "modules",
};

/// Plural category heading for each documentation kind.
static TYPEDOC_KIND_TITLE: phf::Map<&'static str, &'static str> = phf_map! {
    "function" => "Functions",
    "class" => "Classes",
    "interface" => "Interfaces",
    "type" => "Type Aliases",
    "enum" => "Enumerations",
    "variable" => "Variables",
    "const" => "Variables",
    "module" => "Modules",
};

/// Singular category label used as the first column header of a module index
/// table (matches TypeDoc, e.g. `Function`, `Type Alias`).
static TYPEDOC_KIND_SINGULAR: phf::Map<&'static str, &'static str> = phf_map! {
    "function" => "Function",
    "class" => "Class",
    "interface" => "Interface",
    "type" => "Type Alias",
    "enum" => "Enumeration",
    "variable" => "Variable",
    "const" => "Variable",
    "module" => "Module",
};

fn typedoc_kind_segment(kind: &str) -> &'static str {
    TYPEDOC_KIND_SEGMENT.get(kind).copied().unwrap_or("symbols")
}

pub(super) fn typedoc_kind_title(kind: &str) -> &'static str {
    TYPEDOC_KIND_TITLE.get(kind).copied().unwrap_or("Symbols")
}

pub(super) fn typedoc_kind_singular(kind: &str) -> &'static str {
    TYPEDOC_KIND_SINGULAR.get(kind).copied().unwrap_or("Symbol")
}

pub(super) fn typedoc_entry_page_title_len(entry: &ApiDocEntry) -> usize {
    let mut len = typedoc_kind_singular(&entry.kind).len() + ": ".len() + entry.name.len();
    if entry.kind == "function" {
        len += "()".len();
    } else if !entry.type_parameters.is_empty() {
        len += "<>".len();
        len += entry.type_parameters.iter().map(|type_param| type_param.name.len()).sum::<usize>();
        len += ", ".len() * entry.type_parameters.len().saturating_sub(1);
    }
    len
}

/// Appends a TypeDoc-style H1 title for a per-symbol page, e.g.
/// `Function: args()`, `Interface: Command<G>`, `Variable: CLI_OPTIONS_DEFAULT`.
/// Functions append `()` (no type parameters); other kinds append `<...>` when
/// generic.
pub(super) fn push_typedoc_entry_page_title(out: &mut String, entry: &ApiDocEntry) {
    out.push_str(typedoc_kind_singular(&entry.kind));
    out.push_str(": ");
    out.push_str(&entry.name);
    if entry.kind == "function" {
        out.push_str("()");
    } else if !entry.type_parameters.is_empty() {
        out.push('<');
        for (index, type_param) in entry.type_parameters.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(&type_param.name);
        }
        out.push('>');
    }
}

pub(super) fn typedoc_entry_file_name(module_name: &str, entry: &ApiDocEntry) -> String {
    let segment = sanitize_doc_path_segment(&entry.name);
    join5(module_name, "/", typedoc_kind_segment(&entry.kind), "/", &segment)
}

pub(super) fn typedoc_module_index_file_name(module_name: &str) -> String {
    join3(module_name, "/", "index")
}

pub(super) fn sanitize_doc_path_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | '?' | '#' | '[' | ']' | '<' | '>' | ':' | '"' | '|' | '*' => '-',
            _ => ch,
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "symbol".to_string()
    } else {
        sanitized
    }
}

pub(super) fn plural_kind_file_name(kind: &str) -> String {
    let mut file_name = StringBuilder::with_capacity(kind.len() + 1);
    file_name.push_str(kind);
    file_name.push_char('s');
    file_name.into_string()
}

pub(super) fn anchor_href(name: &str) -> String {
    let anchor = entry_anchor(name);
    let mut href = StringBuilder::with_capacity(anchor.len() + 1);
    href.push_char('#');
    href.push_str(&anchor);
    href.into_string()
}

pub(super) fn plural_kind_title(kind: &str) -> String {
    let mut title = capitalize_ascii(kind);
    title.push('s');
    title
}
