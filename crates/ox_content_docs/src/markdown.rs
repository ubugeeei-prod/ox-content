//! Markdown rendering for generated API reference documentation.

use rustc_hash::{FxHashMap, FxHashSet};
use std::borrow::Cow;
// BTreeMap keeps generated API section and tag output deterministic.
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::OnceLock;

use phf::{phf_map, phf_set};
use regex::Regex;

use crate::model::{ApiDocEntry, ApiDocModule, ApiDocTag, ApiThrowsDoc};
#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::{join2, join3, join5, StringBuilder};

mod group_order;
mod implementation;
mod markdown_html;
mod markdown_pure;
mod options;
mod paths;
mod sort;
mod stats;
mod summary;

pub use group_order::{order_by_group_title, ordered_entry_kinds};
use implementation::annotate_implementation_relationships;
pub use options::{
    MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkStyle, MarkdownPathStrategy,
    MarkdownRenderStyle, MarkdownSingleEntryRoot, DOC_KIND_ORDER,
};
use paths::{
    capitalize_ascii, doc_page_href, doc_page_href_from, entry_anchor, file_name, file_stem,
    generate_source_href, generate_source_link, member_anchor, module_display_name,
    module_file_name, module_route_name,
};
use sort::sort_extracted_docs;
#[allow(unused_imports)]
pub use sort::SortStrategy;
pub use sort::{compare_entries, kind_order_slice, parse_sort_strategies};
use stats::{
    doc_kind_plural, effective_index_format, effective_members_format, effective_parameters_format,
    member_table_includes_kind, push_generated_by, push_stats, summarize_docs, summarize_entries,
    summarize_module, EntryStats,
};
use summary::{
    clean_summary_text, collapse_inline_whitespace, collapse_type_annotation_whitespace,
    markdown_index_summary, typedoc_index_summary,
};

type RegexCache = OnceLock<Option<Regex>>;

fn cached_regex(cache: &'static RegexCache, pattern: &'static str) -> Option<&'static Regex> {
    // Regex construction is expensive and these helpers run throughout doc
    // generation. Cache both success and failure in `OnceLock<Option<_>>` so a
    // bad pattern degrades to the fallback path without recompiling on every
    // call.
    cache.get_or_init(|| Regex::new(pattern).ok()).as_ref()
}

#[derive(Debug, Clone)]
struct EntryBadge {
    label: String,
    tone: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct SymbolLocation {
    // `Rc<str>` because `build_symbol_map` shares one module name across every
    // entry in a module and one file name across an entry and all its members;
    // cloning the location into the map is then a refcount bump, not a heap copy.
    module_name: Rc<str>,
    file_name: Rc<str>,
    anchor: Option<String>,
}

#[derive(Debug, Clone)]
struct MarkdownLinkContext<'a> {
    options: &'a MarkdownDocsOptions,
    current_file_name: &'a str,
    current_module_name: &'a str,
    symbol_map: &'a FxHashMap<String, Vec<SymbolLocation>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsdocInlineLinkKind {
    Link,
    LinkCode,
    LinkPlain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct JsdocInlineLink<'a> {
    kind: JsdocInlineLinkKind,
    target: &'a str,
    label: Option<&'a str>,
}

/// Generates Markdown documentation pages from extracted API docs.
#[must_use]
pub fn generate_markdown(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
) -> BTreeMap<String, String> {
    profile_span!("docs::generate_markdown");
    let mut result = BTreeMap::new();
    let mut sorted_docs = sort_extracted_docs(docs, options);
    annotate_implementation_relationships(&mut sorted_docs);
    let symbol_map = build_symbol_map(&sorted_docs, options);

    if options.group_by == "file" {
        if options.path_strategy == MarkdownPathStrategy::TypeDoc {
            return generate_typedoc_markdown(&sorted_docs, options, &symbol_map);
        }

        let mut doc_to_file = FxHashMap::default();

        for doc in &sorted_docs {
            let file_name = module_file_name(&doc.file);
            doc_to_file.insert(doc.file.clone(), file_name.clone());

            let markdown = generate_file_markdown(doc, options, &file_name, &symbol_map);
            result.insert(join2(&file_name, ".md"), markdown);
        }

        result.insert(
            "index.md".to_string(),
            generate_index(&sorted_docs, options, Some(&doc_to_file), Some(&symbol_map)),
        );
    } else {
        let mut by_kind: BTreeMap<String, Vec<ApiDocEntry>> = BTreeMap::new();

        for doc in &sorted_docs {
            for entry in &doc.entries {
                by_kind.entry(entry.kind.clone()).or_default().push(entry.clone());
            }
        }

        let strategies = options.sort.as_deref().map(parse_sort_strategies);
        let kind_order = kind_order_slice(options.kind_sort_order.as_deref());
        for entries in by_kind.values_mut() {
            if let Some(strategies) = &strategies {
                entries.sort_by(|a, b| compare_entries(a, b, strategies, &kind_order));
            } else {
                // Case-insensitive sort with a case-sensitive tiebreak. Caching the
                // (lowercase, original) key computes each side's lowercase form once
                // per entry instead of on every comparison (O(n) vs O(n log n)
                // allocations); the tuple's lexicographic order reproduces the
                // previous "lowercase, then original" ordering exactly.
                entries.sort_by_cached_key(|entry| (entry.name.to_lowercase(), entry.name.clone()));
            }
        }

        for (kind, entries) in &by_kind {
            result.insert(
                join3(kind, "s", ".md"),
                generate_category_markdown(kind, entries, options, &symbol_map),
            );
        }

        result.insert(
            "index.md".to_string(),
            generate_category_index(&by_kind, options, &symbol_map),
        );
    }

    result
}

/// Directory segment for each documentation kind under the TypeDoc path strategy.
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

fn typedoc_kind_title(kind: &str) -> &'static str {
    TYPEDOC_KIND_TITLE.get(kind).copied().unwrap_or("Symbols")
}

fn typedoc_kind_singular(kind: &str) -> &'static str {
    TYPEDOC_KIND_SINGULAR.get(kind).copied().unwrap_or("Symbol")
}

fn typedoc_entry_page_title_len(entry: &ApiDocEntry) -> usize {
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
fn push_typedoc_entry_page_title(out: &mut String, entry: &ApiDocEntry) {
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

fn typedoc_entry_file_name(module_name: &str, entry: &ApiDocEntry) -> String {
    let segment = sanitize_doc_path_segment(&entry.name);
    join5(module_name, "/", typedoc_kind_segment(&entry.kind), "/", &segment)
}

fn typedoc_module_index_file_name(module_name: &str) -> String {
    join3(module_name, "/", "index")
}

fn sanitize_doc_path_segment(value: &str) -> String {
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

fn format_symbol_href(context: &MarkdownLinkContext<'_>, location: &SymbolLocation) -> String {
    if location.file_name.as_ref() == context.current_file_name {
        if let Some(anchor) = location.anchor.as_deref().filter(|anchor| !anchor.is_empty()) {
            join2("#", anchor)
        } else {
            doc_page_href_from(
                context.options,
                context.current_file_name,
                &location.file_name,
                None,
            )
        }
    } else {
        doc_page_href_from(
            context.options,
            context.current_file_name,
            &location.file_name,
            location.anchor.as_deref(),
        )
    }
}

fn resolve_symbol_location<'a>(
    symbol_name: &str,
    context: &'a MarkdownLinkContext<'_>,
) -> Option<&'a SymbolLocation> {
    let locations = context.symbol_map.get(symbol_name)?;
    locations
        .iter()
        .find(|location| location.module_name.as_ref() == context.current_module_name)
        .or_else(|| {
            locations
                .iter()
                .find(|location| location.file_name.as_ref() == context.current_file_name)
        })
        .or_else(|| locations.first())
}

fn resolve_jsdoc_link_target(
    target: &str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> Option<String> {
    let target = target.trim();
    if target.starts_with("http://") || target.starts_with("https://") {
        return Some(target.to_string());
    }

    let context = context?;
    resolve_symbol_location(target, context).map(|location| format_symbol_href(context, location))
}

fn parse_jsdoc_inline_link_body(body: &str) -> Option<(&str, Option<&str>)> {
    let body = body.trim();
    if body.is_empty() {
        return None;
    }

    let (target, label) =
        body.split_once('|').map_or((body, None), |(target, label)| (target, Some(label)));
    let target = target.trim();
    if target.is_empty() {
        return None;
    }

    Some((target, label.map(str::trim).filter(|label| !label.is_empty())))
}

fn parse_jsdoc_inline_link_at(text: &str, start: usize) -> Option<(JsdocInlineLink<'_>, usize)> {
    let after_open = text.get(start + 2..)?;
    let (kind, tag_len) = if after_open.starts_with("linkcode") {
        (JsdocInlineLinkKind::LinkCode, "linkcode".len())
    } else if after_open.starts_with("linkplain") {
        (JsdocInlineLinkKind::LinkPlain, "linkplain".len())
    } else if after_open.starts_with("link") {
        (JsdocInlineLinkKind::Link, "link".len())
    } else {
        return None;
    };

    let body_start = start + 2 + tag_len;
    if !text
        .get(body_start..)
        .and_then(|value| value.chars().next())
        .is_some_and(|value| value.is_whitespace() || value == '}')
    {
        return None;
    }

    let body_end = body_start + text.get(body_start..)?.find('}')?;
    let body = text.get(body_start..body_end)?;
    let (target, label) = parse_jsdoc_inline_link_body(body)?;

    Some((JsdocInlineLink { kind, target, label }, body_end + 1))
}

fn render_jsdoc_inline_link(
    link: &JsdocInlineLink<'_>,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let label = link.label.unwrap_or(link.target).trim();
    let label = if label.is_empty() { link.target.trim() } else { label };
    let label = if link.kind == JsdocInlineLinkKind::LinkCode {
        join3("`", label.trim_matches('`'), "`")
    } else {
        label.to_string()
    };

    if let Some(href) = resolve_jsdoc_link_target(link.target, context) {
        join5("[", &label, "](", &href, ")")
    } else {
        label
    }
}

fn convert_jsdoc_inline_links<'a>(
    text: &'a str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> Cow<'a, str> {
    let mut result = String::new();
    let mut cursor = 0;

    while let Some(start_offset) = text[cursor..].find("{@") {
        let start = cursor + start_offset;
        let Some((link, end)) = parse_jsdoc_inline_link_at(text, start) else {
            result.push_str(&text[cursor..start + 2]);
            cursor = start + 2;
            continue;
        };

        result.push_str(&text[cursor..start]);
        result.push_str(&render_jsdoc_inline_link(&link, context));
        cursor = end;
    }

    if cursor == 0 {
        return Cow::Borrowed(text);
    }

    result.push_str(&text[cursor..]);
    Cow::Owned(result)
}

fn process_doc_text<'a>(text: &'a str, context: Option<&MarkdownLinkContext<'_>>) -> Cow<'a, str> {
    // Resolve `[Symbol]` references first, then `{@link}` inline tags. Both
    // passes borrow the input untouched when there is nothing to rewrite, so a
    // description with no links allocates nothing.
    match context {
        Some(context) => match convert_symbol_links(text, context) {
            Cow::Borrowed(borrowed) => convert_jsdoc_inline_links(borrowed, Some(context)),
            Cow::Owned(owned) => {
                Cow::Owned(convert_jsdoc_inline_links(&owned, Some(context)).into_owned())
            }
        },
        None => convert_jsdoc_inline_links(text, None),
    }
}

fn generate_file_markdown(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    current_file_name: &str,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
) -> String {
    profile_span!("docs::render_file");
    let display_name = file_name(&doc.file);
    let mut markdown = String::new();
    markdown.push_str("# ");
    markdown.push_str(&display_name);
    markdown.push_str("\n\n");

    if let Some(github_url) = &options.github_url {
        markdown.push_str(&generate_source_link(&doc.file, github_url, None, None));
        markdown.push_str("\n\n");
    }

    markdown.push_str("> ");
    let mut count = StringBuilder::new();
    count.push_usize(doc.entries.len());
    markdown.push_str(&count.into_string());
    markdown.push_str(" documented symbol");
    if doc.entries.len() != 1 {
        markdown.push('s');
    }
    markdown.push_str(". ");
    markdown.push_str(
        "Read the signatures first, then expand each item for parameters, return types, and examples.\n\n",
    );

    push_stats(&mut markdown, options, &summarize_entries(&doc.entries), None);

    markdown.push_str("## Reference\n\n");
    if options.render_style == MarkdownRenderStyle::Html && doc.entries.len() > 1 {
        markdown.push_str(&markdown_html::render_details_controls_html(".ox-api-entry"));
        markdown.push_str("\n\n");
    }

    for entry in &doc.entries {
        markdown.push_str(&generate_entry_markdown(
            entry,
            options,
            Some(current_file_name),
            Some(current_file_name),
            Some(symbol_map),
        ));
    }

    markdown
}

/// Resolves, for each distinct symbol, the single module that owns its canonical
/// TypeDoc per-symbol page. A symbol re-exported from several entry points
/// otherwise produces an identical page under each one; TypeDoc emits one page.
///
/// A symbol is keyed by `(name, defining_file)` so that two distinct symbols
/// sharing a name (different source files) keep separate pages. The owner is:
///
/// 1. the module whose own entry-point source (`source_path`) is the symbol's
///    defining file (i.e. the symbol is declared in that entry point), else
/// 2. the first module that exports it, in the same order pages are emitted.
pub struct CanonicalOwners {
    owners: FxHashMap<(String, String), String>,
}

impl CanonicalOwners {
    pub fn compute(docs: &[ApiDocModule]) -> Self {
        // Build the owner table in the same deterministic order that pages are
        // emitted (see `sort_extracted_docs`) so the fallback "first exporter"
        // rule agrees between the page generator and the nav generator,
        // regardless of the caller's input order.
        let mut order: Vec<&ApiDocModule> = docs.iter().collect();
        order.sort_by_cached_key(|module| {
            let name = file_name(&module.file);
            (name.to_lowercase(), name)
        });

        let mut owners: FxHashMap<(String, String), String> = FxHashMap::default();
        let mut fallback: FxHashMap<(String, String), String> = FxHashMap::default();
        for doc in order {
            let module_name = module_file_name(&doc.file);
            for entry in &doc.entries {
                let key = (entry.name.clone(), entry.file.clone());
                fallback.entry(key.clone()).or_insert_with(|| module_name.clone());
                // Rule 1: the defining module wins, if it is itself an entry point.
                if !entry.file.is_empty() && doc.source_path == entry.file {
                    owners.entry(key).or_insert_with(|| module_name.clone());
                }
            }
        }
        // Rule 2: symbols with no defining-module match fall back to the first
        // module that exported them.
        for (key, module_name) in fallback {
            owners.entry(key).or_insert(module_name);
        }

        Self { owners }
    }

    /// The module name owning `entry`'s canonical page, if known.
    fn canonical_module(&self, entry: &ApiDocEntry) -> Option<&str> {
        self.owners.get(&(entry.name.clone(), entry.file.clone())).map(String::as_str)
    }

    /// True when `entry` should render its full page under `doc` (rather than be
    /// a re-export reference to another module's canonical page).
    pub fn is_canonical(&self, doc: &ApiDocModule, entry: &ApiDocEntry) -> bool {
        self.canonical_module(entry) == Some(module_file_name(&doc.file).as_str())
    }
}

fn generate_typedoc_markdown(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
) -> BTreeMap<String, String> {
    profile_span!("docs::render_typedoc");
    let mut result = BTreeMap::new();
    let owners = CanonicalOwners::compute(docs);
    let flatten_single_entry =
        options.single_entry_root == MarkdownSingleEntryRoot::Flatten && docs.len() == 1;

    if flatten_single_entry {
        if let Some(doc) = docs.first() {
            let module_name = module_file_name(&doc.file);
            result.insert(
                "index.md".to_string(),
                generate_typedoc_module_index_for_file(
                    doc,
                    options,
                    &module_name,
                    TypedocModuleIndexPage {
                        current_file_name: "index",
                        title: "API Documentation",
                        include_generated_by: true,
                    },
                    symbol_map,
                    &owners,
                ),
            );
        }
    } else {
        result
            .insert("index.md".to_string(), generate_typedoc_root_index(docs, options, symbol_map));
    }

    for doc in docs {
        let module_name = module_file_name(&doc.file);
        if !flatten_single_entry {
            let module_index_file_name = typedoc_module_index_file_name(&module_name);
            result.insert(
                join2(&module_index_file_name, ".md"),
                generate_typedoc_module_index(doc, options, &module_name, symbol_map, &owners),
            );
        }

        // Both render styles group a symbol's overload signatures onto a single
        // page so every public call signature survives (TypeDoc parity); see
        // `generate_typedoc_entry_page_grouped`.
        for (entry_file_name, entries) in typedoc_canonical_groups(doc, &owners, &module_name) {
            result.insert(
                join2(&entry_file_name, ".md"),
                generate_typedoc_entry_page_grouped(
                    &entries,
                    options,
                    &module_name,
                    &entry_file_name,
                    symbol_map,
                ),
            );
        }
    }

    result
}

/// Groups a module's canonical entries by their TypeDoc page file name, preserving
/// first-seen (source) order. Entries that map to the same page — a symbol's
/// overload signatures plus its implementation — are collected together so the
/// renderer can emit one page per symbol instead of overwriting it per entry.
fn typedoc_canonical_groups<'a>(
    doc: &'a ApiDocModule,
    owners: &CanonicalOwners,
    module_name: &str,
) -> Vec<(String, Vec<&'a ApiDocEntry>)> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: FxHashMap<String, Vec<&ApiDocEntry>> = FxHashMap::default();
    for entry in &doc.entries {
        if !owners.is_canonical(doc, entry) {
            continue;
        }
        let file_name = typedoc_entry_file_name(module_name, entry);
        if !groups.contains_key(&file_name) {
            order.push(file_name.clone());
        }
        groups.entry(file_name).or_default().push(entry);
    }
    order
        .into_iter()
        .map(|file_name| {
            let entries = groups.remove(&file_name).unwrap_or_default();
            (file_name, entries)
        })
        .collect()
}

/// Renders one TypeDoc symbol page for a group of entries that share a page.
///
/// A single entry (the common case) renders exactly as before. When a symbol has
/// overloads, the implementation signature (the body-carrying entry) is hidden and
/// each public overload is rendered as a `## Call Signature` section; a lone public
/// overload collapses back to the normal single-signature page.
fn generate_typedoc_entry_page_grouped(
    entries: &[&ApiDocEntry],
    options: &MarkdownDocsOptions,
    module_name: &str,
    file_name: &str,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
) -> String {
    profile_span!("docs::render_entry_page");
    if entries.len() == 1 {
        return generate_typedoc_entry_page(
            entries[0],
            options,
            module_name,
            file_name,
            symbol_map,
        );
    }

    // Overload signatures have no body; the implementation does. Hide the
    // implementation and render the public signatures, matching TypeDoc.
    let implementation = entries.iter().copied().rfind(|entry| entry.has_body);
    let public: Vec<&ApiDocEntry> = {
        let signatures =
            entries.iter().copied().filter(|entry| !entry.has_body).collect::<Vec<_>>();
        if signatures.is_empty() {
            entries.to_vec()
        } else {
            signatures
        }
    };

    // A single public signature is just a normal symbol page (implementation
    // omitted) — no `## Call Signature` wrapper, like TypeDoc.
    if public.len() == 1 {
        return generate_typedoc_entry_page(public[0], options, module_name, file_name, symbol_map);
    }

    let link_context = MarkdownLinkContext {
        options,
        current_file_name: file_name,
        current_module_name: module_name,
        symbol_map,
    };
    // The H1 title is name + kind only (functions render `Function: name()` with no
    // generics), so any overload yields the same title. The body is rendered in the
    // configured style; both keep all public signatures and hide the implementation.
    let body = if options.render_style == MarkdownRenderStyle::Markdown {
        markdown_pure::render_overload_body_pure(
            &public,
            implementation,
            options,
            Some(&link_context),
            2,
        )
    } else {
        markdown_html::render_overload_body_html(
            &public,
            implementation,
            options,
            Some(&link_context),
        )
    };
    let mut markdown =
        String::with_capacity(typedoc_entry_page_title_len(public[0]) + 4 + body.len() + 1);
    markdown.push_str("# ");
    push_typedoc_entry_page_title(&mut markdown, public[0]);
    markdown.push_str("\n\n");
    if !body.is_empty() {
        markdown.push_str(&body);
        if options.render_style == MarkdownRenderStyle::Markdown {
            markdown.push('\n');
        }
    }
    markdown
}

fn generate_typedoc_root_index(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
) -> String {
    let link_context = MarkdownLinkContext {
        options,
        current_file_name: "index",
        current_module_name: "",
        symbol_map,
    };
    let mut markdown = "# API Documentation\n\n".to_string();
    push_generated_by(&mut markdown, options);
    push_stats(&mut markdown, options, &summarize_docs(docs), Some(docs.len()));
    markdown.push_str("## Modules\n\n");

    if effective_index_format(options) == MarkdownDisplayFormat::Table {
        markdown.push_str("| Module | Description |\n| ------ | ------ |\n");
        for doc in docs {
            let module_name = module_route_name(doc);
            let display_name = module_display_name(doc);
            let href = doc_page_href_from(
                options,
                "index",
                &typedoc_module_index_file_name(&module_name),
                None,
            );
            let summary = typedoc_index_summary(&doc.description, &link_context);
            markdown.push_str("| [");
            markdown.push_str(&display_name);
            markdown.push_str("](");
            markdown.push_str(&href);
            markdown.push_str(") | ");
            markdown.push_str(&summary);
            markdown.push_str(" |\n");
        }
        return markdown;
    }

    for doc in docs {
        let module_name = module_route_name(doc);
        let display_name = module_display_name(doc);
        let href = doc_page_href_from(
            options,
            "index",
            &typedoc_module_index_file_name(&module_name),
            None,
        );
        let summary =
            clean_summary_text(&process_doc_text(&doc.description, Some(&link_context)), 88);
        markdown.push_str("- [");
        markdown.push_str(&display_name);
        markdown.push_str("](");
        markdown.push_str(&href);
        if summary.is_empty() {
            markdown.push_str(")\n");
        } else {
            markdown.push_str(") - ");
            markdown.push_str(&summary);
            markdown.push('\n');
        }
    }

    markdown
}

fn generate_typedoc_module_index(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    module_name: &str,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
    owners: &CanonicalOwners,
) -> String {
    let current_file_name = typedoc_module_index_file_name(module_name);
    let display_name = module_display_name(doc);
    generate_typedoc_module_index_for_file(
        doc,
        options,
        module_name,
        TypedocModuleIndexPage {
            current_file_name: &current_file_name,
            title: &display_name,
            include_generated_by: false,
        },
        symbol_map,
        owners,
    )
}

struct TypedocModuleIndexPage<'a> {
    current_file_name: &'a str,
    title: &'a str,
    include_generated_by: bool,
}

fn generate_typedoc_module_index_for_file(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    module_name: &str,
    page: TypedocModuleIndexPage<'_>,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
    owners: &CanonicalOwners,
) -> String {
    profile_span!("docs::render_module_index");
    let link_context = MarkdownLinkContext {
        options,
        current_file_name: page.current_file_name,
        current_module_name: module_name,
        symbol_map,
    };
    let mut builder = StringBuilder::with_capacity(page.title.len() + 4);
    builder.push_str("# ");
    builder.push_str(page.title);
    builder.push_str("\n\n");
    let mut markdown = builder.into_string();
    if page.include_generated_by {
        push_generated_by(&mut markdown, options);
    }

    // Module-level `@experimental` / `@deprecated`: GitHub alerts in markdown,
    // a badge row in HTML — so both styles surface module lifecycle state.
    if options.render_style == MarkdownRenderStyle::Markdown {
        markdown_pure::push_lifecycle_alerts(&mut markdown, &doc.tags, Some(&link_context));
    } else {
        markdown.push_str(&markdown_html::render_module_lifecycle_badges_html(&doc.tags));
    }

    let description = process_doc_text(&doc.description, Some(&link_context));
    let description = description.trim();
    if !description.is_empty() {
        markdown.push_str(description);
        markdown.push_str("\n\n");
    }

    if !doc.examples.is_empty() {
        if options.render_style == MarkdownRenderStyle::Markdown {
            markdown.push_str(&render_module_examples_markdown(&doc.examples));
        } else {
            markdown.push_str(&markdown_html::render_module_examples_html(&doc.examples));
        }
    }

    if let Some(github_url) = &options.github_url {
        // Link to the entry point's real source file. `doc.file` is the module
        // route name (e.g. `default`) under the TypeDoc strategy, which would
        // produce a dead link; `doc.source_path` holds the actual entry-point
        // path. Omit the link when no source path is available (matching the
        // per-symbol pages and TypeDoc, which has no module source line).
        if !doc.source_path.is_empty() {
            markdown.push_str(&generate_source_link(&doc.source_path, github_url, None, None));
            markdown.push_str("\n\n");
        }
    }

    push_stats(&mut markdown, options, &summarize_module(doc), None);

    let index_format = effective_index_format(options);

    // Collect the kind sections (in the historical order) plus the References
    // section, then order them by `group_order` before rendering. TypeDoc treats
    // References as just another group, so it participates in the ordering too.
    let mut sections: Vec<(String, IndexSection)> = Vec::new();
    let kind_order = kind_order_slice(options.kind_sort_order.as_deref());
    for kind in ordered_entry_kinds(&doc.entries, &kind_order) {
        // Only entries whose canonical page lives in this module are listed in
        // the kind sections; re-exports are collected into "References" below.
        let entries = doc
            .entries
            .iter()
            .filter(|entry| entry.kind == kind && owners.is_canonical(doc, entry))
            .collect::<Vec<_>>();
        if entries.is_empty() {
            continue;
        }
        let title = typedoc_kind_title(&kind).to_string();
        sections.push((title, IndexSection::Kind { kind, entries }));
    }

    // Symbols this module re-exports but does not own: link to the canonical page
    // instead of emitting a duplicate (matches TypeDoc's "References" section).
    // Overloads share a name, so collapse them to a single reference.
    let mut seen_references = rustc_hash::FxHashSet::default();
    let references = doc
        .entries
        .iter()
        .filter(|entry| !owners.is_canonical(doc, entry))
        .filter_map(|entry| owners.canonical_module(entry).map(|owner| (entry, owner.to_string())))
        .filter(|(entry, _)| seen_references.insert(entry.name.as_str()))
        .collect::<Vec<_>>();
    if !references.is_empty() {
        sections.push(("References".to_string(), IndexSection::References(references)));
    }

    for (_title, section) in order_by_group_title(sections, options.group_order.as_deref()) {
        match section {
            IndexSection::Kind { kind, entries } => render_typedoc_kind_section(
                &mut markdown,
                &kind,
                &entries,
                &link_context,
                index_format,
            ),
            IndexSection::References(references) => {
                render_typedoc_references_section(&mut markdown, &references, &link_context);
            }
        }
    }

    markdown
}

/// A renderable section of a TypeDoc module index, kept title-tagged so
/// `group_order` can reorder kinds and References together.
enum IndexSection<'a> {
    Kind { kind: String, entries: Vec<&'a ApiDocEntry> },
    References(Vec<(&'a ApiDocEntry, String)>),
}

/// Renders one `## {kind title}` section (table or list) for a module index.
fn render_typedoc_kind_section(
    markdown: &mut String,
    kind: &str,
    entries: &[&ApiDocEntry],
    link_context: &MarkdownLinkContext<'_>,
    index_format: MarkdownDisplayFormat,
) {
    let options = link_context.options;
    let module_name = link_context.current_module_name;
    let current_file_name = link_context.current_file_name;
    markdown.push_str("## ");
    markdown.push_str(typedoc_kind_title(kind));
    markdown.push_str("\n\n");
    let mut seen = rustc_hash::FxHashSet::default();
    if index_format == MarkdownDisplayFormat::List {
        for entry in entries {
            // Overloads share a name (and page); collapse them to one row.
            if !seen.insert(entry.name.as_str()) {
                continue;
            }
            let href = doc_page_href_from(
                options,
                current_file_name,
                &typedoc_entry_file_name(module_name, entry),
                None,
            );
            let summary =
                clean_summary_text(&process_doc_text(&entry.description, Some(link_context)), 88);
            markdown.push_str("- [");
            markdown.push_str(&entry.name);
            markdown.push_str("](");
            markdown.push_str(&href);
            if summary.is_empty() {
                markdown.push_str(")\n");
            } else {
                markdown.push_str(") - ");
                markdown.push_str(&summary);
                markdown.push('\n');
            }
        }
        markdown.push('\n');
        return;
    }

    // Render a compact `Name | Description` table (matching TypeDoc) rather than a
    // bullet list with the full signature inlined; the signature stays on the
    // per-symbol page.
    markdown.push_str("| ");
    markdown.push_str(typedoc_kind_singular(kind));
    markdown.push_str(" | Description |\n| ------ | ------ |\n");
    for entry in entries {
        // Overloads share a name (and page); collapse them to one row.
        if !seen.insert(entry.name.as_str()) {
            continue;
        }
        let href = doc_page_href_from(
            options,
            current_file_name,
            &typedoc_entry_file_name(module_name, entry),
            None,
        );
        let summary = typedoc_index_summary(&entry.description, link_context);
        markdown.push_str("| [");
        markdown.push_str(&entry.name);
        markdown.push_str("](");
        markdown.push_str(&href);
        markdown.push_str(") | ");
        markdown.push_str(&summary);
        markdown.push_str(" |\n");
    }
    markdown.push('\n');
}

/// Renders the `## References` section for a module index.
fn render_typedoc_references_section(
    markdown: &mut String,
    references: &[(&ApiDocEntry, String)],
    link_context: &MarkdownLinkContext<'_>,
) {
    let options = link_context.options;
    let current_file_name = link_context.current_file_name;
    markdown.push_str("## References\n\n");
    for (index, (entry, owner)) in references.iter().enumerate() {
        // TypeDoc separates consecutive reference entries with a thematic break.
        if index > 0 {
            markdown.push_str("***\n\n");
        }
        let href = doc_page_href_from(
            options,
            current_file_name,
            &typedoc_entry_file_name(owner, entry),
            None,
        );
        markdown.push_str("### ");
        markdown.push_str(&entry.name);
        markdown.push_str("\n\nRe-exports [");
        markdown.push_str(&entry.name);
        markdown.push_str("](");
        markdown.push_str(&href);
        markdown.push_str(")\n\n");
    }
}

fn generate_typedoc_entry_page(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    module_name: &str,
    current_file_name: &str,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
) -> String {
    let link_context = MarkdownLinkContext {
        options,
        current_file_name,
        current_module_name: module_name,
        symbol_map,
    };
    // TypeDoc-style H1 includes the declaration kind (and generics / `()`),
    // e.g. `# Function: cli()`, `# Interface: Command<G>`.
    let title_len = typedoc_entry_page_title_len(entry);
    let body = if options.render_style == MarkdownRenderStyle::Markdown {
        // Per-symbol page: title is `# {title}` (H1), so sections render at H2.
        markdown_pure::render_entry_body_pure(entry, options, Some(&link_context), 2)
    } else {
        markdown_html::render_entry_page_html(entry, options, Some(&link_context))
    };
    let mut markdown = String::with_capacity(title_len + 4 + body.len() + 1);
    markdown.push_str("# ");
    push_typedoc_entry_page_title(&mut markdown, entry);
    markdown.push_str("\n\n");
    if !body.is_empty() {
        markdown.push_str(&body);
        if options.render_style == MarkdownRenderStyle::Markdown {
            markdown.push('\n');
        }
    }
    markdown
}

fn normalize_signature(signature: Option<&str>) -> Option<String> {
    let signature = signature?;
    let normalized = signature.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut value = normalized.as_str();

    for prefix in [
        "export ",
        "declare ",
        "abstract ",
        "async function ",
        "function ",
        "class ",
        "interface ",
        "type ",
    ] {
        if let Some(stripped) = value.strip_prefix(prefix) {
            value = stripped;
        }
    }

    Some(value.trim().to_string()).filter(|value| !value.is_empty())
}

fn format_kind_label(kind: &str) -> &str {
    match kind {
        "function" => "fn",
        "interface" => "interface",
        "class" => "class",
        "type" => "type",
        "const" => "const",
        _ => kind,
    }
}

fn format_count_label(count: usize, singular: &str, plural: Option<&str>) -> String {
    let label = if count == 1 { singular } else { plural.unwrap_or(singular) };
    let mut out = StringBuilder::new();
    out.push_usize(count);
    out.push_char(' ');
    out.push_str(label);
    out.into_string()
}

fn plural_kind_file_name(kind: &str) -> String {
    let mut file_name = StringBuilder::with_capacity(kind.len() + 1);
    file_name.push_str(kind);
    file_name.push_char('s');
    file_name.into_string()
}

fn anchor_href(name: &str) -> String {
    let anchor = entry_anchor(name);
    let mut href = StringBuilder::with_capacity(anchor.len() + 1);
    href.push_char('#');
    href.push_str(&anchor);
    href.into_string()
}

fn plural_kind_title(kind: &str) -> String {
    let mut title = capitalize_ascii(kind);
    title.push('s');
    title
}

fn member_symbol_name(entry_name: &str, member_name: &str) -> String {
    let mut symbol_name = StringBuilder::with_capacity(entry_name.len() + member_name.len() + 1);
    symbol_name.push_str(entry_name);
    symbol_name.push_char('.');
    symbol_name.push_str(member_name);
    symbol_name.into_string()
}

fn entry_tag_value<'a>(entry: &'a ApiDocEntry, tag_name: &str) -> Option<&'a str> {
    entry.tags.iter().find(|tag| tag.tag == tag_name).map(|tag| tag.value.as_str())
}

/// JSDoc tags folded into a dedicated `Since` element (TypeDoc parity) instead of
/// the generic tag list. `@version` is normalized alongside `@since`. Shared by
/// both renderers (`super::SINCE_TAGS`).
const SINCE_TAGS: [&str; 2] = ["since", "version"];

/// JSDoc lifecycle tags surfaced as structured callouts — GitHub alerts in the
/// markdown renderer, badges in the HTML renderer — rather than generic tags.
fn is_lifecycle_tag(tag: &str) -> bool {
    matches!(tag, "deprecated" | "experimental")
}

fn is_throws_tag(tag: &str) -> bool {
    matches!(tag, "throws" | "exception")
}

fn rendered_throws<'a>(throws: &'a [ApiThrowsDoc], tags: &[ApiDocTag]) -> Cow<'a, [ApiThrowsDoc]> {
    if !throws.is_empty() {
        return Cow::Borrowed(throws);
    }

    Cow::Owned(tags.iter().filter_map(api_throws_from_tag).collect())
}

fn api_throws_from_tag(tag: &ApiDocTag) -> Option<ApiThrowsDoc> {
    if !is_throws_tag(&tag.tag) {
        return None;
    }
    let value = tag.value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some((type_annotation, description)) = parse_throws_tag_value(value) {
        return Some(ApiThrowsDoc { type_annotation: Some(type_annotation), description });
    }
    Some(ApiThrowsDoc { type_annotation: None, description: value.to_string() })
}

fn parse_throws_tag_value(value: &str) -> Option<(String, String)> {
    let rest = value.strip_prefix('{')?;
    let end = rest.find('}')?;
    let type_annotation = rest[..end].trim();
    if type_annotation.is_empty() {
        return None;
    }
    let description = rest[end + 1..].trim().trim_start_matches('-').trim().to_string();
    Some((type_annotation.to_string(), description))
}

/// True when a tag is rendered as a structured element (lifecycle callout / Since
/// / Throws) and therefore must not also appear in the generic tag list. Shared
/// by both renderers so the generic-tag exclusion stays consistent.
fn is_structured_tag(name: &str) -> bool {
    is_lifecycle_tag(name) || SINCE_TAGS.contains(&name) || is_throws_tag(name)
}

fn get_entry_badges(entry: &ApiDocEntry) -> Vec<EntryBadge> {
    let mut badges = Vec::new();

    if entry_tag_value(entry, "deprecated").is_some() {
        badges.push(EntryBadge { label: "deprecated".to_string(), tone: Some("warning") });
    }
    if entry_tag_value(entry, "experimental").is_some() {
        badges.push(EntryBadge { label: "experimental".to_string(), tone: Some("warning") });
    }
    if !entry.params.is_empty() {
        badges.push(EntryBadge {
            label: format_count_label(entry.params.len(), "param", Some("params")),
            tone: None,
        });
    }
    if !entry.members.is_empty() {
        badges.push(EntryBadge {
            label: format_count_label(entry.members.len(), "member", Some("members")),
            tone: None,
        });
    }
    if let Some(returns) = &entry.returns {
        let mut label =
            StringBuilder::with_capacity("returns ".len() + returns.type_annotation.len());
        label.push_str("returns ");
        label.push_str(&returns.type_annotation);
        badges.push(EntryBadge { label: label.into_string(), tone: None });
    }
    if !entry.examples.is_empty() {
        badges.push(EntryBadge {
            label: format_count_label(entry.examples.len(), "example", Some("examples")),
            tone: None,
        });
    }
    if let Some(since) = entry_tag_value(entry, "since") {
        let mut label = StringBuilder::with_capacity("since ".len() + since.len());
        label.push_str("since ");
        label.push_str(since);
        badges.push(EntryBadge { label: label.into_string(), tone: None });
    }
    if let Some(version) = entry_tag_value(entry, "version") {
        let mut label = StringBuilder::with_capacity("version ".len() + version.len());
        label.push_str("version ");
        label.push_str(version);
        badges.push(EntryBadge { label: label.into_string(), tone: None });
    }
    if entry.private {
        badges.push(EntryBadge { label: "private".to_string(), tone: Some("warning") });
    }

    badges
}

/// A parsed `@example` body.
enum ExampleBlock<'a> {
    /// Pure code: a single fenced block (unwrapped, with its language) or a bare
    /// code body (defaulting to `ts`). Rendered inside a code fence / `<pre>`.
    Code { code: &'a str, language: &'a str },
    /// Mixed Markdown (prose and/or fenced code). Rendered as Markdown as-is so it
    /// is not wrapped in an extra code fence.
    Markdown(&'a str),
}

/// True when any line is a code-fence line (opens with ```` ``` ````). Counts fence
/// *lines* only, so a stray ```` ``` ```` inside a single-line string literal is
/// ignored.
fn example_has_fence_line(text: &str) -> bool {
    text.lines().any(|line| line.trim_start().starts_with("```"))
}

/// Classifies an `@example` body. A whole-body single fence is unwrapped to
/// [`ExampleBlock::Code`]; a body that still contains a fence line (prose + code,
/// multiple blocks, …) is kept verbatim as [`ExampleBlock::Markdown`] so it is not
/// double-wrapped; a fence-free body is treated as bare code (`ts`).
fn parse_example_block(example: &str) -> ExampleBlock<'_> {
    static FENCE_RE: RegexCache = OnceLock::new();

    let trimmed = example.trim();
    if let Some(fence_re) = cached_regex(&FENCE_RE, r"(?s)^```([\w-]+)?[^\n]*\n(.*?)\n?```$") {
        if let Some(captures) = fence_re.captures(trimmed) {
            let language = captures.get(1).map_or("ts", |value| value.as_str());
            let code = captures.get(2).map_or("", |value| value.as_str());
            // Only a single whole-body fence when the inner code has no further
            // fence line; otherwise the body is multiple blocks → Markdown.
            if !example_has_fence_line(code) {
                return ExampleBlock::Code { code, language };
            }
        }
    }

    if example_has_fence_line(trimmed) {
        ExampleBlock::Markdown(trimmed)
    } else {
        ExampleBlock::Code { code: trimmed, language: "ts" }
    }
}

fn render_module_examples_markdown(examples: &[String]) -> String {
    let mut out = String::new();
    out.push_str("## ");
    out.push_str(if examples.len() == 1 { "Example" } else { "Examples" });
    out.push_str("\n\n");
    for example in examples {
        match parse_example_block(example) {
            ExampleBlock::Code { code, language } => {
                out.push_str("```");
                out.push_str(language);
                out.push('\n');
                out.push_str(code);
                out.push_str("\n```\n\n");
            }
            ExampleBlock::Markdown(markdown) => {
                out.push_str(markdown);
                out.push_str("\n\n");
            }
        }
    }
    out
}

fn render_overview_line(
    entry: &ApiDocEntry,
    href: &str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let signature = normalize_signature(entry.signature.as_deref());
    let summary = clean_summary_text(&process_doc_text(&entry.description, context), 88);
    let mut line = StringBuilder::new();
    line.push_str("- [`");
    line.push_str(&entry.name);
    line.push_str("`](");
    line.push_str(href);
    line.push_str(") `");
    line.push_str(&entry.kind);
    line.push_char('`');

    if let Some(signature) = signature {
        line.push_str(" `");
        line.push_str(&signature);
        line.push_char('`');
    }

    if !summary.is_empty() {
        line.push_str(" - ");
        line.push_str(&summary);
    }

    line.push_char('\n');
    line.into_string()
}

fn render_overview_table_row(
    entry: &ApiDocEntry,
    href: &str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let summary = markdown_index_summary(&entry.description, context);
    let mut row = StringBuilder::new();
    row.push_str("| [`");
    row.push_str(&entry.name);
    row.push_str("`](");
    row.push_str(href);
    row.push_str(") | `");
    row.push_str(&entry.kind);
    row.push_str("` | ");
    row.push_str(&summary);
    row.push_str(" |\n");
    row.into_string()
}

fn generate_entry_markdown(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    current_file_name: Option<&str>,
    current_module_name: Option<&str>,
    symbol_map: Option<&FxHashMap<String, Vec<SymbolLocation>>>,
) -> String {
    let link_context = current_file_name.zip(current_module_name).zip(symbol_map).map(
        |((current_file_name, current_module_name), symbol_map)| MarkdownLinkContext {
            options,
            current_file_name,
            current_module_name,
            symbol_map,
        },
    );
    let link_context = link_context.as_ref();

    if options.render_style == MarkdownRenderStyle::Markdown {
        // Flat entry heading is `### {name}` (H3), so sections render at H4.
        let body = markdown_pure::render_entry_body_pure(entry, options, link_context, 4);
        let mut builder = StringBuilder::with_capacity(entry.name.len() + 6);
        builder.push_str("### ");
        builder.push_str(&entry.name);
        builder.push_str("\n\n");
        let mut markdown = builder.into_string();
        if !body.is_empty() {
            markdown.push_str(&body);
            markdown.push_str("\n\n");
        }
        return markdown;
    }

    markdown_html::render_entry_html(entry, options, link_context)
}

fn generate_index(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
    doc_to_file: Option<&FxHashMap<String, String>>,
    symbol_map: Option<&FxHashMap<String, Vec<SymbolLocation>>>,
) -> String {
    let link_context = symbol_map.map(|symbol_map| MarkdownLinkContext {
        options,
        current_file_name: "index",
        current_module_name: "",
        symbol_map,
    });
    let mut markdown = "# API Documentation\n\n".to_string();
    push_generated_by(&mut markdown, options);
    markdown.push_str(
        "> Use search scopes like `@api transform` to limit results to the generated API reference.\n\n",
    );
    push_stats(
        &mut markdown,
        options,
        &summarize_entries(docs.iter().flat_map(|doc| doc.entries.iter())),
        Some(docs.len()),
    );

    markdown.push_str("## Modules\n\n");
    let index_format = effective_index_format(options);
    if options.render_style == MarkdownRenderStyle::Html
        && matches!(index_format, MarkdownDisplayFormat::List | MarkdownDisplayFormat::Table)
    {
        markdown.push_str(&markdown_html::render_module_index_html(
            docs,
            options,
            doc_to_file,
            index_format,
            link_context.as_ref(),
        ));
        return markdown;
    }

    if options.render_style == MarkdownRenderStyle::Html && docs.len() > 1 {
        markdown.push_str(&markdown_html::render_details_controls_html(".ox-api-module"));
        markdown.push_str("\n\n");
    }

    for doc in docs {
        let display_name = file_stem(&doc.file);
        let mut file_name = display_name.clone();

        if let Some(doc_to_file) = doc_to_file {
            if let Some(mapped) = doc_to_file.get(&doc.file) {
                file_name.clone_from(mapped);
            }
        } else if file_name == "index" {
            file_name = "index-module".to_string();
        }

        let count_label = format_count_label(doc.entries.len(), "symbol", Some("symbols"));

        if options.render_style == MarkdownRenderStyle::Markdown {
            markdown.push_str("### [");
            markdown.push_str(&display_name);
            markdown.push_str("](");
            markdown.push_str(&doc_page_href(options, &file_name, None));
            markdown.push_str(") — ");
            markdown.push_str(&count_label);
            markdown.push_str("\n\n");
            if effective_index_format(options) == MarkdownDisplayFormat::Table {
                markdown.push_str("| Name | Kind | Description |\n| --- | --- | --- |\n");
                for entry in &doc.entries {
                    let href = doc_page_href(options, &file_name, Some(&entry_anchor(&entry.name)));
                    markdown.push_str(&render_overview_table_row(
                        entry,
                        &href,
                        link_context.as_ref(),
                    ));
                }
            } else {
                for entry in &doc.entries {
                    let href = doc_page_href(options, &file_name, Some(&entry_anchor(&entry.name)));
                    markdown.push_str(&render_overview_line(entry, &href, link_context.as_ref()));
                }
            }
            markdown.push('\n');
            continue;
        }

        markdown.push_str(&markdown_html::render_module_section_html(
            doc,
            options,
            &file_name,
            &display_name,
            &count_label,
            link_context.as_ref(),
        ));
    }

    markdown
}

fn generate_category_markdown(
    kind: &str,
    entries: &[ApiDocEntry],
    options: &MarkdownDocsOptions,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
) -> String {
    let category_file_name = plural_kind_file_name(kind);
    let link_context = MarkdownLinkContext {
        options,
        current_file_name: &category_file_name,
        current_module_name: "",
        symbol_map,
    };
    let kind_title = plural_kind_title(kind);
    let mut builder = StringBuilder::with_capacity(kind_title.len() + 4);
    builder.push_str("# ");
    builder.push_str(&kind_title);
    builder.push_str("\n\n");
    let mut markdown = builder.into_string();
    markdown.push_str("> ");
    let mut count = StringBuilder::new();
    count.push_usize(entries.len());
    markdown.push_str(&count.into_string());
    markdown.push_str(" documented ");
    markdown.push_str(kind);
    if entries.len() != 1 {
        markdown.push('s');
    }
    markdown.push_str(" collected across modules.\n\n");
    push_stats(&mut markdown, options, &summarize_entries(entries), None);

    markdown.push_str("## Overview\n\n");
    if effective_index_format(options) == MarkdownDisplayFormat::Table {
        markdown.push_str("| Name | Kind | Description |\n| --- | --- | --- |\n");
        for entry in entries {
            let href = anchor_href(&entry.name);
            markdown.push_str(&render_overview_table_row(entry, &href, Some(&link_context)));
        }
    } else {
        for entry in entries {
            let href = anchor_href(&entry.name);
            markdown.push_str(&render_overview_line(entry, &href, Some(&link_context)));
        }
    }
    markdown.push_str("\n## Reference\n\n");
    if options.render_style == MarkdownRenderStyle::Html && entries.len() > 1 {
        markdown.push_str(&markdown_html::render_details_controls_html(".ox-api-entry"));
        markdown.push_str("\n\n");
    }

    for entry in entries {
        markdown.push_str(&generate_entry_markdown(
            entry,
            options,
            Some(&category_file_name),
            Some(""),
            Some(symbol_map),
        ));
    }

    markdown
}

fn generate_category_index(
    by_kind: &BTreeMap<String, Vec<ApiDocEntry>>,
    options: &MarkdownDocsOptions,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
) -> String {
    let link_context = MarkdownLinkContext {
        options,
        current_file_name: "index",
        current_module_name: "",
        symbol_map,
    };
    let mut markdown = "# API Documentation\n\n".to_string();
    push_generated_by(&mut markdown, options);
    push_stats(
        &mut markdown,
        options,
        &summarize_entries(by_kind.values().flat_map(|entries| entries.iter())),
        None,
    );

    for (kind, entries) in by_kind {
        let kind_title = plural_kind_title(kind);
        let category_file_name = plural_kind_file_name(kind);
        markdown.push_str("## [");
        markdown.push_str(&kind_title);
        markdown.push_str("](");
        markdown.push_str(&doc_page_href(options, &category_file_name, None));
        markdown.push_str(")\n\n> ");
        let mut count = StringBuilder::new();
        count.push_usize(entries.len());
        markdown.push_str(&count.into_string());
        markdown.push_str(" item");
        if entries.len() != 1 {
            markdown.push('s');
        }
        markdown.push_str(".\n\n");

        if effective_index_format(options) == MarkdownDisplayFormat::Table {
            markdown.push_str("| Name | Kind | Description |\n| --- | --- | --- |\n");
            for entry in entries {
                let href =
                    doc_page_href(options, &category_file_name, Some(&entry_anchor(&entry.name)));
                markdown.push_str(&render_overview_table_row(entry, &href, Some(&link_context)));
            }
        } else {
            for entry in entries {
                let href =
                    doc_page_href(options, &category_file_name, Some(&entry_anchor(&entry.name)));
                markdown.push_str(&render_overview_line(entry, &href, Some(&link_context)));
            }
        }
        markdown.push('\n');
    }

    markdown
}

fn convert_symbol_links<'a>(text: &'a str, context: &MarkdownLinkContext<'_>) -> Cow<'a, str> {
    static SYMBOL_RE: RegexCache = OnceLock::new();

    let Some(symbol_re) = cached_regex(&SYMBOL_RE, r"\[([A-Z_]\w*)\]") else {
        return Cow::Borrowed(text);
    };
    let mut result = String::new();
    let mut last_index = 0;

    for captures in symbol_re.captures_iter(text) {
        let Some(mat) = captures.get(0) else {
            continue;
        };

        if text[mat.end()..].starts_with('(') {
            continue;
        }

        let symbol_name = captures.get(1).map_or("", |value| value.as_str());
        let Some(location) = resolve_symbol_location(symbol_name, context) else {
            continue;
        };

        result.push_str(&text[last_index..mat.start()]);
        result.push('[');
        result.push_str(symbol_name);
        result.push_str("](");
        result.push_str(&format_symbol_href(context, location));
        result.push(')');
        last_index = mat.end();
    }

    if last_index == 0 {
        return Cow::Borrowed(text);
    }

    result.push_str(&text[last_index..]);
    Cow::Owned(result)
}

/// A fragment of a tokenized TypeScript type annotation.
enum TypeFragment {
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
/// annotations only — JSDoc `{@link}` / `[Symbol]` references are unaffected.
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
fn resolve_type_fragments(
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

fn build_symbol_map(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
) -> FxHashMap<String, Vec<SymbolLocation>> {
    profile_span!("docs::build_symbol_map");
    let mut map = FxHashMap::default();
    // In the TypeDoc strategy a re-exported symbol has a single canonical page;
    // resolve every reference to that owner module so cross-links never point at
    // a duplicate page that is no longer emitted.
    let canonical = (options.group_by == "file"
        && options.path_strategy == MarkdownPathStrategy::TypeDoc)
        .then(|| CanonicalOwners::compute(docs));

    for doc in docs {
        // Interned once per module and shared by every entry + member below.
        let module_name: Rc<str> = Rc::from(module_file_name(&doc.file));
        for entry in &doc.entries {
            let (file_name, anchor): (Rc<str>, Option<String>) =
                match (options.group_by.as_str(), options.path_strategy) {
                    ("file", MarkdownPathStrategy::TypeDoc) => {
                        let owner_module = canonical
                            .as_ref()
                            .and_then(|owners| owners.canonical_module(entry))
                            .unwrap_or(&module_name);
                        (Rc::from(typedoc_entry_file_name(owner_module, entry)), None)
                    }
                    ("category", _) => (
                        Rc::from(plural_kind_file_name(&entry.kind)),
                        Some(entry_anchor(&entry.name)),
                    ),
                    _ => (Rc::clone(&module_name), Some(entry_anchor(&entry.name))),
                };
            insert_symbol_location(
                &mut map,
                entry.name.clone(),
                SymbolLocation {
                    module_name: Rc::clone(&module_name),
                    file_name: Rc::clone(&file_name),
                    anchor,
                },
            );
            for member in &entry.members {
                insert_symbol_location(
                    &mut map,
                    member_symbol_name(&entry.name, &member.name),
                    SymbolLocation {
                        module_name: Rc::clone(&module_name),
                        file_name: Rc::clone(&file_name),
                        anchor: Some(member_anchor(&entry.name, member, options.path_strategy)),
                    },
                );
            }
        }
    }

    map
}

fn insert_symbol_location(
    map: &mut FxHashMap<String, Vec<SymbolLocation>>,
    symbol_name: String,
    location: SymbolLocation,
) {
    map.entry(symbol_name).or_default().push(location);
}

#[cfg(test)]
mod tests;
