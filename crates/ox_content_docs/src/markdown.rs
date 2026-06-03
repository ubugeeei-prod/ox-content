//! Markdown rendering for generated API reference documentation.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::OnceLock;

use phf::phf_map;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::model::{ApiDocEntry, ApiDocMember, ApiDocModule};
use crate::string_builder::{join2, join3, join4, join5, StringBuilder};

mod markdown_html;
mod markdown_pure;

const DOC_KIND_ORDER: [&str; 7] =
    ["function", "class", "interface", "type", "enum", "variable", "module"];

type RegexCache = OnceLock<Option<Regex>>;

fn cached_regex(cache: &'static RegexCache, pattern: &'static str) -> Option<&'static Regex> {
    // Regex construction is expensive and these helpers run throughout doc
    // generation. Cache both success and failure in `OnceLock<Option<_>>` so a
    // bad pattern degrades to the fallback path without recompiling on every
    // call.
    cache.get_or_init(|| Regex::new(pattern).ok()).as_ref()
}

/// Options for generated API Markdown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkdownDocsOptions {
    /// Grouping mode: `file` or `category`.
    #[serde(default = "default_group_by")]
    pub group_by: String,
    /// GitHub repository URL for source links.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_url: Option<String>,
    /// Internal documentation link style.
    #[serde(default)]
    pub link_style: MarkdownLinkStyle,
    /// Optional absolute route prefix for generated documentation links.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_path: Option<String>,
    /// Output path strategy.
    ///
    /// Only applies when `group_by` is `"file"`. Category grouping always emits
    /// flat `{kind}s.md` pages regardless of this setting.
    #[serde(default)]
    pub path_strategy: MarkdownPathStrategy,
    /// Rendering style: HTML-laced Markdown (default) or pure Markdown.
    #[serde(default)]
    pub render_style: MarkdownRenderStyle,
    /// Display format for index items.
    #[serde(default)]
    pub index_format: MarkdownDisplayFormat,
    /// Display format for value and type parameters.
    #[serde(default)]
    pub parameters_format: MarkdownDisplayFormat,
    /// Display format for interface property groups.
    #[serde(default)]
    pub interface_properties_format: MarkdownDisplayFormat,
    /// Display format for class property groups.
    #[serde(default)]
    pub class_properties_format: MarkdownDisplayFormat,
    /// Display format for type alias property groups.
    #[serde(default)]
    pub type_alias_properties_format: MarkdownDisplayFormat,
    /// Display format for enum member groups.
    #[serde(default)]
    pub enum_members_format: MarkdownDisplayFormat,
    /// Display format for property-owned object literal members.
    #[serde(default)]
    pub property_members_format: MarkdownDisplayFormat,
    /// Display format for type declaration members.
    #[serde(default)]
    pub type_declaration_format: MarkdownDisplayFormat,
    /// Whether to emit the stats summary line on index/overview pages. Defaults
    /// to `true` (historical behavior); set to `false` for TypeDoc-like output
    /// without stats.
    #[serde(default = "default_render_stats")]
    pub render_stats: bool,
    /// TypeDoc-style group order for module index sections and nav groups. `None`
    /// keeps the historical fixed order; `Some` reorders groups by title, placing
    /// unlisted groups alphabetically at `*` (or at the end when `*` is absent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_order: Option<Vec<String>>,
}

/// Internal documentation link style.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarkdownLinkStyle {
    /// Link to emitted Markdown files, such as `./context.md#symbol`.
    #[default]
    Markdown,
    /// Link to clean routes, such as `./context#symbol`.
    Clean,
}

/// Generated Markdown output path strategy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MarkdownPathStrategy {
    /// Keep the historical flat module/category files with entry anchors.
    #[default]
    Flat,
    /// Emit TypeDoc-style module/kind/symbol pages.
    TypeDoc,
}

/// API documentation rendering style.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarkdownRenderStyle {
    /// Emit HTML-laced Markdown: collapsible `<details>` entries, stat blocks,
    /// member tables and other ox-content theme scaffolding. This is the default
    /// and preserves the historical output.
    #[default]
    Html,
    /// Emit pure Markdown: headings, tables and fenced code with no raw HTML
    /// scaffolding. Suitable for plain Markdown hosts such as VitePress.
    Markdown,
}

/// TypeDoc-compatible Markdown display format.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MarkdownDisplayFormat {
    /// Use the renderer's default behavior.
    #[default]
    None,
    /// Render supported sections as Markdown lists.
    List,
    /// Render supported sections as Markdown tables.
    Table,
}

impl Default for MarkdownDocsOptions {
    fn default() -> Self {
        Self {
            group_by: default_group_by(),
            github_url: None,
            link_style: MarkdownLinkStyle::Markdown,
            base_path: None,
            path_strategy: MarkdownPathStrategy::Flat,
            render_style: MarkdownRenderStyle::Html,
            index_format: MarkdownDisplayFormat::None,
            parameters_format: MarkdownDisplayFormat::None,
            interface_properties_format: MarkdownDisplayFormat::None,
            class_properties_format: MarkdownDisplayFormat::None,
            type_alias_properties_format: MarkdownDisplayFormat::None,
            enum_members_format: MarkdownDisplayFormat::None,
            property_members_format: MarkdownDisplayFormat::None,
            type_declaration_format: MarkdownDisplayFormat::None,
            render_stats: true,
            group_order: None,
        }
    }
}

fn default_group_by() -> String {
    "file".to_string()
}

fn default_render_stats() -> bool {
    true
}

#[derive(Debug, Clone, Default)]
struct EntryStats {
    entries: usize,
    by_kind: [usize; DOC_KIND_ORDER.len()],
    members: usize,
    params: usize,
    returns: usize,
    examples: usize,
    deprecated: usize,
}

#[derive(Debug, Clone)]
struct EntryBadge {
    label: String,
    tone: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct SymbolLocation {
    module_name: String,
    file_name: String,
    anchor: Option<String>,
}

#[derive(Debug, Clone)]
struct MarkdownLinkContext<'a> {
    options: &'a MarkdownDocsOptions,
    current_file_name: &'a str,
    current_module_name: &'a str,
    symbol_map: &'a HashMap<String, Vec<SymbolLocation>>,
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
    let mut result = BTreeMap::new();
    let sorted_docs = sort_extracted_docs(docs);
    let symbol_map = build_symbol_map(&sorted_docs, options);

    if options.group_by == "file" {
        if options.path_strategy == MarkdownPathStrategy::TypeDoc {
            return generate_typedoc_markdown(&sorted_docs, options, &symbol_map);
        }

        let mut doc_to_file = HashMap::new();

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

        for entries in by_kind.values_mut() {
            // Case-insensitive sort with a case-sensitive tiebreak. Caching the
            // (lowercase, original) key computes each side's lowercase form once
            // per entry instead of on every comparison (O(n) vs O(n log n)
            // allocations); the tuple's lexicographic order reproduces the
            // previous "lowercase, then original" ordering exactly.
            entries.sort_by_cached_key(|entry| (entry.name.to_lowercase(), entry.name.clone()));
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

fn doc_page_href(options: &MarkdownDocsOptions, file_name: &str, anchor: Option<&str>) -> String {
    doc_page_href_from(options, "", file_name, anchor)
}

fn doc_page_href_from(
    options: &MarkdownDocsOptions,
    current_file_name: &str,
    target_file_name: &str,
    anchor: Option<&str>,
) -> String {
    if target_file_name == current_file_name {
        if let Some(anchor) = anchor.filter(|anchor| !anchor.is_empty()) {
            return join2("#", anchor);
        }
    }

    let mut href = String::new();
    let target_file_name = route_file_name(options, target_file_name);

    if let Some(base_path) =
        options.base_path.as_deref().map(str::trim).filter(|base| !base.is_empty())
    {
        let base_path = normalize_base_path(base_path);
        if !base_path.is_empty() {
            href.push_str(&base_path);
        }
        href.push('/');
        href.push_str(&target_file_name);
    } else {
        href.push_str(&relative_doc_href_path(current_file_name, &target_file_name));
    }

    if options.link_style == MarkdownLinkStyle::Markdown {
        href.push_str(".md");
    }
    if let Some(anchor) = anchor.filter(|anchor| !anchor.is_empty()) {
        href.push('#');
        href.push_str(anchor);
    }

    href
}

fn route_file_name(options: &MarkdownDocsOptions, file_name: &str) -> String {
    if options.link_style == MarkdownLinkStyle::Clean {
        file_name.strip_suffix("/index").unwrap_or(file_name).to_string()
    } else {
        file_name.to_string()
    }
}

fn relative_doc_href_path(current_file_name: &str, target_file_name: &str) -> String {
    let current_dir =
        current_file_name.rsplit_once('/').map_or("", |(directory, _)| directory).trim_matches('/');
    let current_parts = current_dir.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();
    let target_parts =
        target_file_name.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();

    let mut common = 0;
    while current_parts.get(common) == target_parts.get(common) {
        if current_parts.get(common).is_none() {
            break;
        }
        common += 1;
    }

    let mut parts = Vec::new();
    parts.extend(std::iter::repeat_n("..", current_parts.len().saturating_sub(common)));
    parts.extend(target_parts.iter().skip(common).copied());

    let path = if parts.is_empty() { target_file_name.to_string() } else { parts.join("/") };
    if path.starts_with("../") {
        path
    } else {
        join2("./", &path)
    }
}

fn normalize_base_path(base_path: &str) -> String {
    let base_path = base_path.trim().trim_end_matches('/');

    if base_path.is_empty() || base_path == "/" {
        return String::new();
    }

    if base_path.starts_with('/') {
        base_path.to_string()
    } else {
        join2("/", base_path)
    }
}

fn entry_anchor(name: &str) -> String {
    name.to_lowercase()
}

fn member_anchor(
    entry_name: &str,
    member: &ApiDocMember,
    path_strategy: MarkdownPathStrategy,
) -> String {
    match path_strategy {
        MarkdownPathStrategy::Flat => {
            join3(&entry_anchor(entry_name), "-", &entry_anchor(&member.name))
        }
        MarkdownPathStrategy::TypeDoc => {
            let prefix = match member.kind.as_str() {
                "constructor" => return "constructor".to_string(),
                "method" => "method",
                "getter" | "setter" => "accessor",
                "enumMember" => "enumeration-member",
                _ => "property",
            };
            join3(prefix, "-", &entry_anchor(&member.name))
        }
    }
}

fn module_file_name(file_path: &str) -> String {
    let mut file_name = file_stem(file_path);
    if file_name == "index" {
        file_name = "index-module".to_string();
    }
    sanitize_doc_path_segment(&file_name)
}

fn module_route_name(doc: &ApiDocModule) -> String {
    module_file_name(&doc.file)
}

fn module_display_name(doc: &ApiDocModule) -> String {
    if !doc.source_path.is_empty() {
        return doc.file.clone();
    }

    let display_name = file_stem(&doc.file);
    if display_name.is_empty() {
        doc.file.clone()
    } else {
        display_name
    }
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
    if location.file_name == context.current_file_name {
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
        .find(|location| location.module_name == context.current_module_name)
        .or_else(|| {
            locations.iter().find(|location| location.file_name == context.current_file_name)
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

fn convert_jsdoc_inline_links(text: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
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
        return text.to_string();
    }

    result.push_str(&text[cursor..]);
    result
}

fn process_doc_text(text: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    let text =
        context.map_or_else(|| text.to_string(), |context| convert_symbol_links(text, context));
    convert_jsdoc_inline_links(&text, context)
}

/// One-line summary for a module index table cell.
///
/// Resolves `{@link}`/`{@linkcode}` exactly like the per-symbol pages (keeping
/// the produced Markdown links and inline code), takes the first paragraph,
/// collapses it to a single line, and escapes table-cell pipes. Unlike
/// [`clean_summary_text`] it does not strip links/code, so the index matches
/// TypeDoc (e.g. `An object that contains [argument schema](…).`).
fn typedoc_index_summary(description: &str, context: &MarkdownLinkContext<'_>) -> String {
    markdown_index_summary(description, Some(context))
}

fn markdown_index_summary(description: &str, context: Option<&MarkdownLinkContext<'_>>) -> String {
    let resolved = process_doc_text(description, context);
    let first_paragraph = resolved.split("\n\n").next().unwrap_or_default();
    let one_line = first_paragraph.split_whitespace().collect::<Vec<_>>().join(" ");
    one_line.replace('|', "\\|")
}

fn clean_summary_text(text: &str, max_length: usize) -> String {
    static MARKDOWN_LINK_RE: RegexCache = OnceLock::new();
    static BRACKET_LINK_RE: RegexCache = OnceLock::new();
    static INLINE_CODE_RE: RegexCache = OnceLock::new();
    static WHITESPACE_RE: RegexCache = OnceLock::new();

    if text.is_empty() {
        return String::new();
    }

    let fallback = || text.split_whitespace().collect::<Vec<_>>().join(" ");
    let Some(markdown_link_re) = cached_regex(&MARKDOWN_LINK_RE, r"\[([^\]]+)\]\([^)]+\)") else {
        return truncate_summary_text(&fallback(), max_length);
    };
    let Some(bracket_link_re) = cached_regex(&BRACKET_LINK_RE, r"\[([^\]]+)\]") else {
        return truncate_summary_text(&fallback(), max_length);
    };
    let Some(inline_code_re) = cached_regex(&INLINE_CODE_RE, r"`([^`]+)`") else {
        return truncate_summary_text(&fallback(), max_length);
    };
    let Some(whitespace_re) = cached_regex(&WHITESPACE_RE, r"\s+") else {
        return truncate_summary_text(&fallback(), max_length);
    };

    // Summary cleanup is called for every entry in index views. `replace_all`
    // returns `Cow::Borrowed` when a pattern does not match, which is common
    // for short summaries, so thread the borrowed/owned value through each
    // regex stage and materialize only in `truncate_summary_text`.
    let s1 = markdown_link_re.replace_all(text, "$1");
    let s2 = bracket_link_re.replace_all(&s1, "$1");
    let s3 = inline_code_re.replace_all(&s2, "$1");
    let s4 = whitespace_re.replace_all(&s3, " ");

    truncate_summary_text(s4.trim(), max_length)
}

fn truncate_summary_text(text: &str, max_length: usize) -> String {
    if text.chars().count() <= max_length {
        return text.to_string();
    }

    let truncated: String = text.chars().take(max_length.saturating_sub(1)).collect();
    let trimmed = truncated.trim_end();
    let mut value = String::with_capacity(trimmed.len() + "…".len());
    value.push_str(trimmed);
    value.push('…');
    value
}

fn summarize_entries<'a>(entries: impl IntoIterator<Item = &'a ApiDocEntry>) -> EntryStats {
    let mut stats = EntryStats::default();

    for entry in entries {
        stats.entries += 1;
        if let Some(index) = doc_kind_index(&entry.kind) {
            stats.by_kind[index] += 1;
        }
        stats.members += entry.members.len();
        stats.params += entry.params.len();
        stats.returns += usize::from(entry.returns.is_some());
        stats.examples += entry.examples.len();
        stats.deprecated += usize::from(entry.tags.iter().any(|tag| tag.tag == "deprecated"));
    }

    stats
}

fn summarize_module(module: &ApiDocModule) -> EntryStats {
    let mut stats = summarize_entries(&module.entries);
    stats.examples += module.examples.len();
    stats
}

fn summarize_docs(docs: &[ApiDocModule]) -> EntryStats {
    let mut stats = summarize_entries(docs.iter().flat_map(|doc| doc.entries.iter()));
    stats.examples += docs.iter().map(|doc| doc.examples.len()).sum::<usize>();
    stats
}

fn doc_kind_index(kind: &str) -> Option<usize> {
    match kind {
        "function" => Some(0),
        "class" => Some(1),
        "interface" => Some(2),
        "type" => Some(3),
        "enum" => Some(4),
        "variable" => Some(5),
        "module" => Some(6),
        _ => None,
    }
}

fn doc_kind_plural(kind: &str) -> &'static str {
    match kind {
        "function" => "functions",
        "class" => "classes",
        "interface" => "interfaces",
        "type" => "types",
        "enum" => "enumerations",
        "variable" => "variables",
        "module" => "modules",
        _ => "symbols",
    }
}

fn normalize_doc_file_path(file_path: &str) -> String {
    let normalized = file_path.replace('\\', "/");

    for marker in ["npm/", "packages/", "crates/", "src/"] {
        if let Some(index) = normalized.find(marker) {
            if index == 0 || normalized.as_bytes().get(index - 1) == Some(&b'/') {
                return normalized[index..].to_string();
            }
        }
    }

    normalized.trim_start_matches('/').to_string()
}

fn sort_extracted_docs(docs: &[ApiDocModule]) -> Vec<ApiDocModule> {
    let mut sorted = docs.to_vec();

    for doc in &mut sorted {
        doc.entries.sort_by_cached_key(|entry| (entry.name.to_lowercase(), entry.name.clone()));
    }

    sorted.sort_by_cached_key(|module| {
        let name = file_name(&module.file);
        (name.to_lowercase(), name)
    });
    sorted
}

/// Renders the per-page stats summary in the configured render style.
fn render_stats_summary(
    options: &MarkdownDocsOptions,
    stats: &EntryStats,
    module_count: Option<usize>,
) -> String {
    match options.render_style {
        MarkdownRenderStyle::Html => markdown_html::render_stats_html(stats, module_count),
        MarkdownRenderStyle::Markdown => markdown_pure::render_stats_markdown(stats, module_count),
    }
}

/// Appends the stats summary plus its trailing blank line, unless stats are
/// disabled via `options.render_stats`. Centralizing the gate keeps every
/// index/overview generator from leaving a stray blank line when stats are
/// omitted.
fn push_stats(
    markdown: &mut String,
    options: &MarkdownDocsOptions,
    stats: &EntryStats,
    module_count: Option<usize>,
) {
    if !options.render_stats {
        return;
    }
    markdown.push_str(&render_stats_summary(options, stats, module_count));
    markdown.push_str("\n\n");
}

fn effective_display_format(
    options: &MarkdownDocsOptions,
    format: MarkdownDisplayFormat,
) -> MarkdownDisplayFormat {
    match (options.render_style, format) {
        (MarkdownRenderStyle::Markdown, MarkdownDisplayFormat::None) => MarkdownDisplayFormat::List,
        (_, format) => format,
    }
}

fn effective_index_format(options: &MarkdownDocsOptions) -> MarkdownDisplayFormat {
    effective_display_format(options, options.index_format)
}

fn effective_parameters_format(options: &MarkdownDocsOptions) -> MarkdownDisplayFormat {
    effective_display_format(options, options.parameters_format)
}

fn effective_members_format(
    options: &MarkdownDocsOptions,
    entry_kind: &str,
    group_title: &str,
) -> MarkdownDisplayFormat {
    let format = match (entry_kind, group_title) {
        ("class", "Properties" | "Static Properties") => options.class_properties_format,
        ("interface", "Properties") => options.interface_properties_format,
        ("type", "Properties") => options.type_alias_properties_format,
        ("enum", "Enum Members" | "Members") | ("type", "Enum Members") => {
            options.enum_members_format
        }
        _ => return MarkdownDisplayFormat::None,
    };
    effective_display_format(options, format)
}

fn generate_file_markdown(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    current_file_name: &str,
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
) -> String {
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
    owners: HashMap<(String, String), String>,
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

        let mut owners: HashMap<(String, String), String> = HashMap::new();
        let mut fallback: HashMap<(String, String), String> = HashMap::new();
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
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    let owners = CanonicalOwners::compute(docs);

    result.insert("index.md".to_string(), generate_typedoc_root_index(docs, options, symbol_map));

    for doc in docs {
        let module_name = module_file_name(&doc.file);
        let module_index_file_name = typedoc_module_index_file_name(&module_name);
        result.insert(
            join2(&module_index_file_name, ".md"),
            generate_typedoc_module_index(doc, options, &module_name, symbol_map, &owners),
        );

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
    let mut groups: HashMap<String, Vec<&ApiDocEntry>> = HashMap::new();
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
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
) -> String {
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
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
) -> String {
    let link_context = MarkdownLinkContext {
        options,
        current_file_name: "index",
        current_module_name: "",
        symbol_map,
    };
    let mut markdown = "# API Documentation\n\n".to_string();
    markdown.push_str("Generated by [Ox Content](https://github.com/ubugeeei-prod/ox-content)\n\n");
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
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
    owners: &CanonicalOwners,
) -> String {
    let current_file_name = typedoc_module_index_file_name(module_name);
    let link_context = MarkdownLinkContext {
        options,
        current_file_name: &current_file_name,
        current_module_name: module_name,
        symbol_map,
    };
    let display_name = module_display_name(doc);
    let mut builder = StringBuilder::with_capacity(display_name.len() + 4);
    builder.push_str("# ");
    builder.push_str(&display_name);
    builder.push_str("\n\n");
    let mut markdown = builder.into_string();

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
        markdown.push_str(&generate_source_link(&doc.file, github_url, None, None));
        markdown.push_str("\n\n");
    }

    push_stats(&mut markdown, options, &summarize_module(doc), None);

    let index_format = effective_index_format(options);

    // Collect the kind sections (in the historical order) plus the References
    // section, then order them by `group_order` before rendering. TypeDoc treats
    // References as just another group, so it participates in the ordering too.
    let mut sections: Vec<(String, IndexSection)> = Vec::new();
    for kind in ordered_entry_kinds(&doc.entries) {
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
    let mut seen_references = std::collections::HashSet::new();
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
    let mut seen = std::collections::HashSet::new();
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
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
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

fn ordered_entry_kinds(entries: &[ApiDocEntry]) -> Vec<String> {
    let mut kinds = Vec::new();
    for kind in DOC_KIND_ORDER {
        if entries.iter().any(|entry| entry.kind == kind) {
            kinds.push(kind.to_string());
        }
    }
    let mut extra = entries
        .iter()
        .map(|entry| entry.kind.clone())
        .filter(|kind| !DOC_KIND_ORDER.contains(&kind.as_str()))
        .collect::<Vec<_>>();
    extra.sort();
    extra.dedup();
    kinds.extend(extra);
    kinds
}

/// Reorders `(group_title, payload)` sections by a TypeDoc-style `group_order`.
///
/// `None` returns the input unchanged (preserving the caller's default order).
/// Otherwise titles listed before `*` lead in the given order, titles after `*`
/// trail in the given order, and titles not listed are placed at the `*` position
/// (or the end when there is no `*`) sorted alphabetically. Listed titles that are
/// not present are ignored.
pub fn order_by_group_title<T>(
    sections: Vec<(String, T)>,
    group_order: Option<&[String]>,
) -> Vec<(String, T)> {
    let Some(group_order) = group_order else {
        return sections;
    };
    let star = group_order.iter().position(|group| group == "*");
    let (head, tail): (&[String], &[String]) = match star {
        Some(index) => (&group_order[..index], &group_order[index + 1..]),
        None => (group_order, &group_order[group_order.len()..]),
    };

    let mut remaining: Vec<Option<(String, T)>> = sections.into_iter().map(Some).collect();
    let mut result = Vec::with_capacity(remaining.len());

    for title in head {
        if let Some(section) = take_section(&mut remaining, title) {
            result.push(section);
        }
    }

    let mut unspecified = Vec::new();
    for slot in &mut remaining {
        let is_tail = slot.as_ref().is_some_and(|(title, _)| tail.iter().any(|t| t == title));
        if !is_tail {
            if let Some(section) = slot.take() {
                unspecified.push(section);
            }
        }
    }
    unspecified.sort_by(|a, b| a.0.cmp(&b.0));
    result.extend(unspecified);

    for title in tail {
        if let Some(section) = take_section(&mut remaining, title) {
            result.push(section);
        }
    }
    result
}

fn take_section<T>(remaining: &mut [Option<(String, T)>], title: &str) -> Option<(String, T)> {
    remaining
        .iter_mut()
        .find(|slot| slot.as_ref().is_some_and(|(t, _)| t == title))
        .and_then(Option::take)
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

/// True when a tag is rendered as a structured element (lifecycle callout / Since)
/// and therefore must not also appear in the generic tag list. Shared by both
/// renderers so the generic-tag exclusion stays consistent.
fn is_structured_tag(name: &str) -> bool {
    is_lifecycle_tag(name) || SINCE_TAGS.contains(&name)
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

fn parse_example_block(example: &str) -> (&str, &str) {
    static FENCE_RE: RegexCache = OnceLock::new();

    let trimmed = example.trim();
    let Some(fence_re) = cached_regex(&FENCE_RE, r"(?s)^```([\w-]+)?[^\n]*\n(.*?)\n?```$") else {
        return (trimmed, "ts");
    };

    if let Some(captures) = fence_re.captures(trimmed) {
        let language = captures.get(1).map_or("ts", |value| value.as_str());
        let code = captures.get(2).map_or("", |value| value.as_str());
        (code, language)
    } else {
        (trimmed, "ts")
    }
}

fn render_module_examples_markdown(examples: &[String]) -> String {
    let mut out = String::new();
    out.push_str("## ");
    out.push_str(if examples.len() == 1 { "Example" } else { "Examples" });
    out.push_str("\n\n");
    for example in examples {
        let (code, language) = parse_example_block(example);
        out.push_str("```");
        out.push_str(language);
        out.push('\n');
        out.push_str(code);
        out.push_str("\n```\n\n");
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
    symbol_map: Option<&HashMap<String, Vec<SymbolLocation>>>,
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
    doc_to_file: Option<&HashMap<String, String>>,
    symbol_map: Option<&HashMap<String, Vec<SymbolLocation>>>,
) -> String {
    let link_context = symbol_map.map(|symbol_map| MarkdownLinkContext {
        options,
        current_file_name: "index",
        current_module_name: "",
        symbol_map,
    });
    let mut markdown = "# API Documentation\n\n".to_string();
    markdown.push_str("Generated by [Ox Content](https://github.com/ubugeeei-prod/ox-content)\n\n");
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
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
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
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
) -> String {
    let link_context = MarkdownLinkContext {
        options,
        current_file_name: "index",
        current_module_name: "",
        symbol_map,
    };
    let mut markdown = "# API Documentation\n\n".to_string();
    markdown.push_str("Generated by [Ox Content](https://github.com/ubugeeei-prod/ox-content)\n\n");
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

fn convert_symbol_links(text: &str, context: &MarkdownLinkContext<'_>) -> String {
    static SYMBOL_RE: RegexCache = OnceLock::new();

    let Some(symbol_re) = cached_regex(&SYMBOL_RE, r"\[([A-Z_]\w*)\]") else {
        return text.to_string();
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
        return text.to_string();
    }

    result.push_str(&text[last_index..]);
    result
}

fn build_symbol_map(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
) -> HashMap<String, Vec<SymbolLocation>> {
    let mut map = HashMap::new();
    // In the TypeDoc strategy a re-exported symbol has a single canonical page;
    // resolve every reference to that owner module so cross-links never point at
    // a duplicate page that is no longer emitted.
    let canonical = (options.group_by == "file"
        && options.path_strategy == MarkdownPathStrategy::TypeDoc)
        .then(|| CanonicalOwners::compute(docs));

    for doc in docs {
        let module_name = module_file_name(&doc.file);
        for entry in &doc.entries {
            let (file_name, anchor) = match (options.group_by.as_str(), options.path_strategy) {
                ("file", MarkdownPathStrategy::TypeDoc) => {
                    let owner_module = canonical
                        .as_ref()
                        .and_then(|owners| owners.canonical_module(entry))
                        .unwrap_or(module_name.as_str());
                    (typedoc_entry_file_name(owner_module, entry), None)
                }
                ("category", _) => {
                    (plural_kind_file_name(&entry.kind), Some(entry_anchor(&entry.name)))
                }
                _ => (module_name.clone(), Some(entry_anchor(&entry.name))),
            };
            insert_symbol_location(
                &mut map,
                entry.name.clone(),
                SymbolLocation {
                    module_name: module_name.clone(),
                    file_name: file_name.clone(),
                    anchor,
                },
            );
            for member in &entry.members {
                insert_symbol_location(
                    &mut map,
                    member_symbol_name(&entry.name, &member.name),
                    SymbolLocation {
                        module_name: module_name.clone(),
                        file_name: file_name.clone(),
                        anchor: Some(member_anchor(&entry.name, member, options.path_strategy)),
                    },
                );
            }
        }
    }

    map
}

fn insert_symbol_location(
    map: &mut HashMap<String, Vec<SymbolLocation>>,
    symbol_name: String,
    location: SymbolLocation,
) {
    map.entry(symbol_name).or_default().push(location);
}

fn generate_source_href(
    file_path: &str,
    github_url: &str,
    line_number: Option<u32>,
    end_line_number: Option<u32>,
) -> String {
    let relative_path = normalize_doc_file_path(file_path);
    let fragment = if let Some(line_number) = line_number {
        let mut fragment = StringBuilder::with_capacity(24);
        fragment.push_str("#L");
        fragment.push_usize(line_number as usize);
        if let Some(end_line_number) =
            end_line_number.filter(|end_line_number| *end_line_number > line_number)
        {
            fragment.push_str("-L");
            fragment.push_usize(end_line_number as usize);
        }
        fragment.into_string()
    } else {
        String::new()
    };

    join4(github_url, "/blob/main/", &relative_path, &fragment)
}

fn generate_source_link(
    file_path: &str,
    github_url: &str,
    line_number: Option<u32>,
    end_line_number: Option<u32>,
) -> String {
    let href = generate_source_href(file_path, github_url, line_number, end_line_number);
    join3("**[Source](", &href, ")**")
}

fn file_name(file_path: &str) -> String {
    Path::new(file_path)
        .file_name()
        .and_then(|value| value.to_str())
        .map_or_else(|| file_path.to_string(), ToString::to_string)
}

fn file_stem(file_path: &str) -> String {
    Path::new(file_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .map_or_else(|| file_path.to_string(), ToString::to_string)
}

fn capitalize_ascii(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => {
            let rest = chars.as_str();
            let mut out = StringBuilder::with_capacity(first.len_utf8() + rest.len());
            out.push_char(first.to_ascii_uppercase());
            out.push_str(rest);
            out.into_string()
        }
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ApiDocTag, ApiParamDoc, ApiReturnDoc, ApiTypeParamDoc};

    fn test_entry(name: &str, kind: &str, file: &str, description: &str) -> ApiDocEntry {
        ApiDocEntry {
            name: name.to_string(),
            kind: kind.to_string(),
            description: description.to_string(),
            params: vec![],
            returns: None,
            examples: vec![],
            tags: vec![],
            private: false,
            file: file.to_string(),
            line: 1,
            end_line: 1,
            signature: Some(join3("export function ", name, "(): void")),
            has_body: false,
            members: vec![],
            type_parameters: vec![],
        }
    }

    fn link_test_docs() -> Vec<ApiDocModule> {
        vec![
            ApiDocModule {
                description: String::new(),
                file: "/repo/src/context.ts".to_string(),
                source_path: String::new(),
                examples: vec![],
                tags: vec![],
                entries: vec![test_entry(
                    "CommandContext",
                    "interface",
                    "/repo/src/context.ts",
                    "Command context.",
                )],
            },
            ApiDocModule {
                description: String::new(),
                file: "/repo/src/command.ts".to_string(),
                source_path: String::new(),
                examples: vec![],
                tags: vec![],
                entries: vec![test_entry(
                    "Command",
                    "function",
                    "/repo/src/command.ts",
                    "Runs with [CommandContext].",
                )],
            },
        ]
    }

    fn pure_test_docs() -> Vec<ApiDocModule> {
        vec![ApiDocModule {
            description: String::new(),
            file: "/repo/src/cli.ts".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                ApiDocEntry {
                    name: "cli".to_string(),
                    kind: "function".to_string(),
                    description: "Runs the CLI.".to_string(),
                    params: vec![ApiParamDoc {
                        name: "argv".to_string(),
                        type_annotation: "string[]".to_string(),
                        description: "Arguments.".to_string(),
                        optional: false,
                        default_value: None,
                    }],
                    returns: Some(ApiReturnDoc {
                        type_annotation: "void".to_string(),
                        description: "Nothing.".to_string(),
                    }),
                    examples: vec!["```ts\ncli([])\n```".to_string()],
                    tags: vec![ApiDocTag { tag: "since".to_string(), value: "1.0.0".to_string() }],
                    private: false,
                    file: "/repo/src/cli.ts".to_string(),
                    line: 1,
                    end_line: 3,
                    signature: Some("export function cli(argv: string[]): void".to_string()),
                    has_body: false,
                    members: vec![],
                    type_parameters: vec![],
                },
                ApiDocEntry {
                    name: "Command".to_string(),
                    kind: "interface".to_string(),
                    description: "A command.".to_string(),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    tags: vec![],
                    private: false,
                    file: "/repo/src/cli.ts".to_string(),
                    line: 5,
                    end_line: 8,
                    signature: Some("export interface Command".to_string()),
                    has_body: false,
                    members: vec![ApiDocMember {
                        name: "run".to_string(),
                        kind: "method".to_string(),
                        description: "Runs it.".to_string(),
                        signature: Some("run(): void".to_string()),
                        type_annotation: None,
                        params: vec![],
                        returns: None,
                        optional: false,
                        readonly: false,
                        r#static: false,
                        private: false,
                        tags: vec![],
                        line: 6,
                        end_line: 6,
                    }],
                    type_parameters: vec![],
                },
            ],
        }]
    }

    fn assert_no_api_html(markdown: &str) {
        assert!(!markdown.contains("<details"), "unexpected <details> in:\n{markdown}");
        assert!(!markdown.contains("class=\"ox-api"), "unexpected ox-api html in:\n{markdown}");
        assert!(!markdown.contains("<table"), "unexpected <table> in:\n{markdown}");
        assert!(!markdown.contains("ox-api-controls"), "unexpected controls in:\n{markdown}");
    }

    /// Asserts heading levels never increase by more than one (markdownlint
    /// MD001), ignoring `#` lines inside fenced code blocks.
    fn assert_no_heading_level_skips(markdown: &str) {
        let mut previous = 0usize;
        let mut in_fence = false;
        for line in markdown.lines() {
            if line.trim_start().starts_with("```") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence {
                continue;
            }
            let hashes = line.chars().take_while(|&ch| ch == '#').count();
            if hashes == 0 || line.as_bytes().get(hashes) != Some(&b' ') {
                continue;
            }
            if previous != 0 {
                assert!(
                    hashes <= previous + 1,
                    "heading level skip {previous} -> {hashes} at: {line}\nin:\n{markdown}"
                );
            }
            previous = hashes;
        }
    }

    #[test]
    fn render_style_markdown_flat_emits_pure_markdown() {
        let options = MarkdownDocsOptions {
            render_style: MarkdownRenderStyle::Markdown,
            github_url: Some("https://github.com/x/y".to_string()),
            ..MarkdownDocsOptions::default()
        };
        let out = generate_markdown(&pure_test_docs(), &options);

        let page = out.get("cli.md").unwrap();
        assert_no_api_html(page);
        assert!(page.contains("### cli"));
        // Flat entry heading is H3, so body sections render at H4.
        assert!(page.lines().any(|line| line == "#### Signature"));
        assert!(!page.contains("**Signature**"));
        assert!(page.contains("```ts"));
        assert!(page.contains("- `argv` (`string[]`) - Arguments."));
        assert!(!page.contains("| Name | Type | Description |"));
        // The interface member group is a real heading (no `**Members**` wrapper).
        assert!(page.lines().any(|line| line == "#### Methods"));
        assert!(!page.contains("**Members**"));
        assert!(page.contains("| Name | Kind | Type | Description |"));
        assert!(page.contains("[View source](https://github.com/x/y/blob/main/"));

        let index = out.get("index.md").unwrap();
        assert_no_api_html(index);
        assert!(index.contains("## Modules"));
    }

    #[test]
    fn render_style_markdown_typedoc_emits_pure_per_symbol_pages() {
        let options = MarkdownDocsOptions {
            render_style: MarkdownRenderStyle::Markdown,
            path_strategy: MarkdownPathStrategy::TypeDoc,
            base_path: Some("/api".to_string()),
            ..MarkdownDocsOptions::default()
        };
        let out = generate_markdown(&pure_test_docs(), &options);

        let key = out
            .keys()
            .find(|key| key.ends_with("functions/cli.md"))
            .expect("typedoc cli page should exist")
            .clone();
        let page = out.get(&key).unwrap();
        assert_no_api_html(page);
        assert!(page.starts_with("# Function: cli()"));
        assert!(page.contains("```ts"));
    }

    #[test]
    fn render_style_markdown_category_emits_pure_markdown() {
        let options = MarkdownDocsOptions {
            render_style: MarkdownRenderStyle::Markdown,
            group_by: "category".to_string(),
            ..MarkdownDocsOptions::default()
        };
        let out = generate_markdown(&pure_test_docs(), &options);

        let functions = out.get("functions.md").unwrap();
        assert_no_api_html(functions);
        assert!(functions.contains("### cli"));
    }

    fn typedoc_title_page(entry: ApiDocEntry) -> String {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "mod".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![entry],
        }];
        let out = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                render_style: MarkdownRenderStyle::Markdown,
                ..MarkdownDocsOptions::default()
            },
        );
        out.into_iter()
            .find(|(key, _)| {
                key.contains('/') && key.ends_with(".md") && !key.ends_with("index.md")
            })
            .map(|(_, page)| page)
            .expect("a per-symbol page")
    }

    #[test]
    fn typedoc_symbol_page_h1_includes_declaration_kind() {
        // Function: kind prefix + `()`, no type parameters in the title.
        let mut func =
            test_entry("args", "function", "/repo/src/combinators.ts", "Schema factory.");
        func.type_parameters = vec![ApiTypeParamDoc {
            name: "T".to_string(),
            constraint: None,
            default: None,
            description: String::new(),
        }];
        assert!(typedoc_title_page(func).starts_with("# Function: args()"));

        // Interface with a generic parameter (names only).
        let mut iface = test_entry("Command", "interface", "/repo/src/types.ts", "A command.");
        iface.type_parameters = vec![ApiTypeParamDoc {
            name: "G".to_string(),
            constraint: Some("GunshiParams".to_string()),
            default: None,
            description: String::new(),
        }];
        assert!(typedoc_title_page(iface).starts_with("# Interface: Command<G>"));

        // Type alias with a generic parameter.
        let mut alias = test_entry("Plugin", "type", "/repo/src/plugin.ts", "Plugin type.");
        alias.type_parameters = vec![ApiTypeParamDoc {
            name: "E".to_string(),
            constraint: None,
            default: None,
            description: String::new(),
        }];
        assert!(typedoc_title_page(alias).starts_with("# Type Alias: Plugin<E>"));

        // Class without type parameters: kind prefix only.
        let class = test_entry("DefaultTranslation", "class", "/repo/src/i18n.ts", "Translation.");
        assert!(typedoc_title_page(class).starts_with("# Class: DefaultTranslation\n"));

        // Variable: kind prefix only, no `()` or `<>`.
        let variable =
            test_entry("CLI_OPTIONS_DEFAULT", "variable", "/repo/src/constants.ts", "Defaults.");
        assert!(typedoc_title_page(variable).starts_with("# Variable: CLI_OPTIONS_DEFAULT\n"));
    }

    #[test]
    fn render_style_markdown_typedoc_sections_are_sequential_headings() {
        let options = MarkdownDocsOptions {
            render_style: MarkdownRenderStyle::Markdown,
            path_strategy: MarkdownPathStrategy::TypeDoc,
            github_url: Some("https://github.com/x/y".to_string()),
            ..MarkdownDocsOptions::default()
        };
        let out = generate_markdown(&pure_test_docs(), &options);

        // Function page: every section is a real H2 heading under the H1 title,
        // with no bold-as-header, no skipped levels.
        let fn_key =
            out.keys().find(|key| key.ends_with("functions/cli.md")).expect("cli page").clone();
        let page = out.get(&fn_key).unwrap();
        assert!(page.starts_with("# Function: cli()"));
        assert!(page.contains("## Signature"));
        assert!(page.contains("## Parameters"));
        assert!(page.contains("## Returns"));
        assert!(page.contains("## Examples"));
        // `@since` renders as a dedicated `## Since` section, not generic `## Tags`.
        assert!(page.contains("## Since"));
        assert!(!page.contains("## Tags"));
        assert!(!page.contains("**Signature**"));
        assert!(!page.contains("**Returns**"));
        assert!(!page.contains("#### "));
        assert_no_heading_level_skips(page);

        // Returns is its own heading with the value on the following line.
        let after_returns = page.split("## Returns\n\n").nth(1).expect("returns section");
        assert!(after_returns.starts_with("`void`"), "returns value on next line:\n{page}");

        // Interface page: member group is a real H2 heading (## Methods), not a
        // `#### Properties`/`**Members**` mix.
        let if_key = out
            .keys()
            .find(|key| key.ends_with("interfaces/Command.md"))
            .expect("Command page")
            .clone();
        let page = out.get(&if_key).unwrap();
        assert!(page.contains("## Methods"));
        assert!(!page.contains("#### "));
        assert!(!page.contains("**Members**"));
        assert_no_heading_level_skips(page);
    }

    #[test]
    fn render_style_markdown_flat_sections_render_at_h4() {
        let options = MarkdownDocsOptions {
            render_style: MarkdownRenderStyle::Markdown,
            ..MarkdownDocsOptions::default()
        };
        let out = generate_markdown(&pure_test_docs(), &options);
        let page = out.get("cli.md").unwrap();

        // Flat entry heading is H3, so its sections render at H4 (sequential).
        assert!(page.contains("### cli"));
        assert!(page.lines().any(|line| line == "#### Signature"));
        assert!(page.lines().any(|line| line == "#### Parameters"));
        assert!(page.lines().any(|line| line == "#### Returns"));
        assert!(!page.lines().any(|line| line == "## Signature"));
        assert_no_heading_level_skips(page);
    }

    #[test]
    fn render_style_defaults_to_html() {
        let out = generate_markdown(&pure_test_docs(), &MarkdownDocsOptions::default());
        let page = out.get("cli.md").unwrap();
        assert!(page.contains("<details"));
        assert!(page.contains("class=\"ox-api-entry\""));
    }

    #[test]
    fn file_group_index_links_default_to_markdown_extension() {
        let markdown = generate_markdown(&link_test_docs(), &MarkdownDocsOptions::default());
        let index = markdown.get("index.md").unwrap();

        assert!(index.contains("href=\"./context.md\""));
        assert!(index.contains("href=\"./context.md#commandcontext\""));
    }

    #[test]
    fn file_group_index_links_support_clean_urls() {
        let markdown = generate_markdown(
            &link_test_docs(),
            &MarkdownDocsOptions {
                link_style: MarkdownLinkStyle::Clean,
                ..MarkdownDocsOptions::default()
            },
        );
        let index = markdown.get("index.md").unwrap();

        assert!(index.contains("href=\"./context\""));
        assert!(index.contains("href=\"./context#commandcontext\""));
        assert!(!index.contains(".md#commandcontext"));
    }

    #[test]
    fn file_group_index_links_support_clean_urls_with_base_path() {
        let markdown = generate_markdown(
            &link_test_docs(),
            &MarkdownDocsOptions {
                link_style: MarkdownLinkStyle::Clean,
                base_path: Some("/api-ox".to_string()),
                ..MarkdownDocsOptions::default()
            },
        );
        let index = markdown.get("index.md").unwrap();

        assert!(index.contains("href=\"/api-ox/context\""));
        assert!(index.contains("href=\"/api-ox/context#commandcontext\""));
    }

    #[test]
    fn category_links_use_configured_link_policy() {
        let markdown = generate_markdown(
            &link_test_docs(),
            &MarkdownDocsOptions {
                group_by: "category".to_string(),
                link_style: MarkdownLinkStyle::Clean,
                base_path: Some("/api-ox".to_string()),
                ..MarkdownDocsOptions::default()
            },
        );
        let index = markdown.get("index.md").unwrap();

        assert!(index.contains("## [Functions](/api-ox/functions)"));
        assert!(index.contains("[`Command`](/api-ox/functions#command)"));
        assert!(!index.contains("functions.md"));
    }

    #[test]
    fn symbol_cross_file_links_use_configured_link_policy() {
        let markdown = generate_markdown(
            &link_test_docs(),
            &MarkdownDocsOptions {
                link_style: MarkdownLinkStyle::Clean,
                base_path: Some("/api-ox".to_string()),
                ..MarkdownDocsOptions::default()
            },
        );
        let page = markdown.get("command.md").unwrap();

        assert!(page.contains("<a href=\"/api-ox/context#commandcontext\">CommandContext</a>"));
    }

    #[test]
    fn jsdoc_inline_links_render_across_doc_fields() {
        let docs = vec![
            ApiDocModule {
                description: String::new(),
                file: "/repo/src/agent.ts".to_string(),
                source_path: String::new(),
                examples: vec![],
                tags: vec![],
                entries: vec![test_entry(
                    "AgentProfile",
                    "interface",
                    "/repo/src/agent.ts",
                    "Agent profile.",
                )],
            },
            ApiDocModule {
                description: String::new(),
                file: "/repo/src/command.ts".to_string(),
                source_path: String::new(),
                examples: vec![],
                tags: vec![],
                entries: vec![ApiDocEntry {
                    name: "Command".to_string(),
                    kind: "interface".to_string(),
                    description: "Runtime command.".to_string(),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    tags: vec![],
                    private: false,
                    file: "/repo/src/command.ts".to_string(),
                    line: 1,
                    end_line: 10,
                    signature: Some("export interface Command".to_string()),
                    has_body: false,
                    members: vec![ApiDocMember {
                        name: "args".to_string(),
                        kind: "property".to_string(),
                        description: "All {@linkcode Command.args} names use kebab-case."
                            .to_string(),
                        signature: None,
                        type_annotation: Some("Record<string, unknown>".to_string()),
                        params: vec![],
                        returns: None,
                        optional: false,
                        readonly: false,
                        r#static: false,
                        private: false,
                        tags: vec![],
                        line: 5,
                        end_line: 5,
                    }],
                    type_parameters: vec![],
                }],
            },
            ApiDocModule {
                description: String::new(),
                file: "/repo/src/build.ts".to_string(),
                source_path: String::new(),
                examples: vec![],
                tags: vec![],
                entries: vec![ApiDocEntry {
                    name: "buildCommand".to_string(),
                    kind: "function".to_string(),
                    description: "Builds {@linkcode Command | command} metadata.".to_string(),
                    params: vec![ApiParamDoc {
                        name: "entry".to_string(),
                        type_annotation: "Command".to_string(),
                        description: "A {@linkcode Command | entry command}".to_string(),
                        optional: false,
                        default_value: None,
                    }],
                    returns: Some(ApiReturnDoc {
                        type_annotation: "AgentProfile".to_string(),
                        description: "An {@link AgentProfile} result.".to_string(),
                    }),
                    examples: vec![],
                    tags: vec![
                        ApiDocTag {
                            tag: "see".to_string(),
                            value: "delegated to {@link https://github.com/unjs/std-env | std-env}"
                                .to_string(),
                        },
                        ApiDocTag {
                            tag: "remarks".to_string(),
                            value: "Falls back to {@link MissingSymbol | missing}.".to_string(),
                        },
                    ],
                    private: false,
                    file: "/repo/src/build.ts".to_string(),
                    line: 1,
                    end_line: 20,
                    signature: Some(
                        "export function buildCommand(entry: Command): AgentProfile".to_string(),
                    ),
                    has_body: false,
                    members: vec![],
                    type_parameters: vec![],
                }],
            },
        ];

        let markdown = generate_markdown(&docs, &MarkdownDocsOptions::default());
        let build_page = markdown.get("build.md").unwrap();
        let command_page = markdown.get("command.md").unwrap();
        let index = markdown.get("index.md").unwrap();

        assert!(!build_page.contains("{@link"));
        assert!(!command_page.contains("{@link"));
        assert!(!index.contains("{@link"));
        assert!(
            build_page.contains("<a href=\"./command.md#command\"><code>entry command</code></a>")
        );
        assert!(build_page.contains("<a href=\"./agent.md#agentprofile\">AgentProfile</a>"));
        assert!(build_page.contains("<a href=\"https://github.com/unjs/std-env\">std-env</a>"));
        assert!(build_page.contains("Falls back to missing."));
        assert!(command_page.contains("<tr id=\"command-args\">"));
        assert!(command_page.contains("<a href=\"#command-args\"><code>Command.args</code></a>"));
        assert!(index.contains("Builds command metadata."));
    }

    #[test]
    fn typedoc_path_strategy_emits_per_symbol_pages_and_links() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                ApiDocEntry {
                    name: "cli".to_string(),
                    kind: "function".to_string(),
                    description:
                        "Run with {@link CliOptions} and {@linkcode CliOptions.usageSilent}."
                            .to_string(),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    tags: vec![],
                    private: false,
                    file: "/repo/src/cli.ts".to_string(),
                    line: 1,
                    end_line: 10,
                    signature: Some("export function cli(options: CliOptions): void".to_string()),
                    has_body: false,
                    members: vec![],
                    type_parameters: vec![],
                },
                ApiDocEntry {
                    name: "CliOptions".to_string(),
                    kind: "interface".to_string(),
                    description: "CLI options.".to_string(),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    tags: vec![],
                    private: false,
                    file: "/repo/src/types.ts".to_string(),
                    line: 1,
                    end_line: 20,
                    signature: Some("export interface CliOptions".to_string()),
                    has_body: false,
                    members: vec![ApiDocMember {
                        name: "usageSilent".to_string(),
                        kind: "property".to_string(),
                        description: "Suppress usage output.".to_string(),
                        signature: None,
                        type_annotation: Some("boolean".to_string()),
                        params: vec![],
                        returns: None,
                        optional: true,
                        readonly: false,
                        r#static: false,
                        private: false,
                        tags: vec![],
                        line: 5,
                        end_line: 5,
                    }],
                    type_parameters: vec![],
                },
                test_entry("Plugin", "type", "/repo/src/plugin.ts", "Plugin type."),
                test_entry(
                    "CLI_OPTIONS_DEFAULT",
                    "variable",
                    "/repo/src/constants.ts",
                    "Default options.",
                ),
            ],
        }];

        let markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );
        let cli_page = markdown.get("default/functions/cli.md").unwrap();
        let options_page = markdown.get("default/interfaces/CliOptions.md").unwrap();
        let module_index = markdown.get("default/index.md").unwrap();

        assert!(markdown.contains_key("index.md"));
        assert!(markdown.contains_key("default/type-aliases/Plugin.md"));
        assert!(markdown.contains_key("default/variables/CLI_OPTIONS_DEFAULT.md"));
        // The module index lists members as a compact table, not bullets with
        // the full signature inlined.
        assert!(module_index.contains("| Function | Description |"));
        assert!(module_index.contains("| [cli](./functions/cli.md) |"));
        assert!(module_index.contains("| [CliOptions](./interfaces/CliOptions.md) |"));
        assert!(!module_index.contains("[`cli`]"));
        assert!(!module_index.contains("export function cli"));
        assert!(cli_page.contains("<a href=\"../interfaces/CliOptions.md\">CliOptions</a>"));
        assert!(cli_page.contains(
            "<a href=\"../interfaces/CliOptions.md#property-usagesilent\"><code>CliOptions.usageSilent</code></a>"
        ));
        assert!(options_page.contains("<tr id=\"property-usagesilent\">"));
    }

    #[test]
    fn typedoc_index_uses_module_description_not_symbol_description() {
        let docs = vec![
            ApiDocModule {
                description: "The entry for gunshi context.".to_string(),
                file: "context".to_string(),
                source_path: String::new(),
                examples: vec!["```ts\ncreateCommandContext()\n```".to_string()],
                tags: vec![],
                entries: vec![test_entry(
                    "CommandContextParams",
                    "interface",
                    "/repo/src/context.ts",
                    "Parameters of createCommandContext.",
                )],
            },
            ApiDocModule {
                description: String::new(),
                file: "plugin".to_string(),
                source_path: String::new(),
                examples: vec![],
                tags: vec![],
                entries: vec![test_entry(
                    "plugin",
                    "function",
                    "/repo/src/plugin.ts",
                    "Define a plugin.",
                )],
            },
        ];

        let markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                render_style: MarkdownRenderStyle::Markdown,
                ..MarkdownDocsOptions::default()
            },
        );

        // Root module list shows the module-level `@module` description, never a
        // symbol's description, and renders nothing for a module without one.
        let index = markdown.get("index.md").unwrap();
        assert!(index.contains("[context](./context/index.md)"));
        assert!(index.contains("[plugin](./plugin/index.md)"));
        assert!(!index.contains("[Context]"));
        assert!(!index.contains("[Plugin]"));
        assert!(index.contains("The entry for gunshi context."));
        assert!(!index.contains("Parameters of createCommandContext"));
        assert!(!index.contains("Define a plugin."));

        // The module index page renders its own description as a paragraph under
        // the heading (followed by the stats line, which starts with `_`); an
        // empty description emits no paragraph, so the heading is followed
        // directly by the stats line.
        let context_index = markdown.get("context/index.md").unwrap();
        assert!(context_index.starts_with(
            "# context\n\nThe entry for gunshi context.\n\n## Example\n\n```ts\ncreateCommandContext()\n```\n\n_"
        ));
        assert!(context_index.contains("_1 symbols · 1 interfaces · 1 examples_"));
        let plugin_index = markdown.get("plugin/index.md").unwrap();
        assert!(plugin_index.starts_with("# plugin\n\n_"));
    }

    #[test]
    fn typedoc_module_index_renders_module_examples_in_html_style() {
        let docs = vec![ApiDocModule {
            description: "Parser combinator entry point.".to_string(),
            file: "combinators".to_string(),
            source_path: String::new(),
            examples: vec!["```ts\nstring()\n```".to_string()],
            tags: vec![],
            entries: vec![],
        }];

        let markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );

        let module_index = markdown.get("combinators/index.md").unwrap();
        assert!(module_index.contains("<h2>Example</h2>"));
        assert!(module_index.contains("<pre><code class=\"language-ts\">string()</code></pre>"));
        assert!(module_index.contains(">1</strong>\n  <span>examples</span>"));
    }

    #[test]
    fn entries_without_file_omit_source_link() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "mod".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                test_entry("localSym", "function", "packages/x/src/a.ts", "Local symbol."),
                // Empty file = external-package source: no in-repo source location.
                test_entry("externalSym", "function", "", "External symbol."),
            ],
        }];

        for render_style in [MarkdownRenderStyle::Html, MarkdownRenderStyle::Markdown] {
            let markdown = generate_markdown(
                &docs,
                &MarkdownDocsOptions {
                    github_url: Some("https://github.com/o/r".to_string()),
                    path_strategy: MarkdownPathStrategy::TypeDoc,
                    render_style,
                    ..MarkdownDocsOptions::default()
                },
            );

            let local_page = markdown.get("mod/functions/localSym.md").unwrap();
            let external_page = markdown.get("mod/functions/externalSym.md").unwrap();

            // The local symbol links to its in-repo source.
            assert!(local_page.contains("https://github.com/o/r/blob/main/packages/x/src/a.ts"));
            // The external symbol emits no source link and leaks no path.
            assert!(!external_page.contains("blob/main"));
            assert!(!external_page.contains("View source"));
        }
    }

    #[test]
    fn type_parameters_render_as_a_section() {
        let mut entry = test_entry("make", "function", "src/make.ts", "Make a thing.");
        entry.type_parameters = vec![
            ApiTypeParamDoc {
                name: "G".to_string(),
                constraint: Some("Base".to_string()),
                default: Some("Default".to_string()),
                description: String::new(),
            },
            ApiTypeParamDoc {
                name: "T".to_string(),
                constraint: None,
                default: None,
                description: "The value type.".to_string(),
            },
        ];
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "mod".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![entry],
        }];

        let markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                render_style: MarkdownRenderStyle::Markdown,
                ..MarkdownDocsOptions::default()
            },
        );
        let page = markdown.get("mod/functions/make.md").unwrap();

        assert!(page.contains("## Type Parameters"));
        assert!(!page.contains("**Type Parameters**"));
        assert!(page.contains("`G` *extends* `Base` = `Default`"));
        assert!(page.contains("- `T` - The value type."));
    }

    #[test]
    fn markdown_display_format_options_render_tables() {
        let mut entry = test_entry("make", "function", "src/make.ts", "Make a thing.");
        entry.params = vec![ApiParamDoc {
            name: "value".to_string(),
            type_annotation: "string".to_string(),
            description: "Input value.".to_string(),
            optional: false,
            default_value: None,
        }];
        entry.type_parameters = vec![ApiTypeParamDoc {
            name: "T".to_string(),
            constraint: None,
            default: None,
            description: "Value type.".to_string(),
        }];
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "mod".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![entry],
        }];

        let markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                render_style: MarkdownRenderStyle::Markdown,
                index_format: MarkdownDisplayFormat::Table,
                parameters_format: MarkdownDisplayFormat::Table,
                ..MarkdownDocsOptions::default()
            },
        );
        let index = markdown.get("mod/index.md").unwrap();
        let page = markdown.get("mod/functions/make.md").unwrap();

        assert!(index.contains("| Function | Description |"));
        assert!(page.contains("| Name | Type | Description |"));
        assert!(page.contains("| `value` | `string` | Input value. |"));
        assert!(page.contains("| `T` | Value type. |"));
    }

    #[test]
    fn markdown_property_display_format_controls_property_groups() {
        let mut entry = test_entry("Command", "interface", "src/types.ts", "Command options.");
        entry.members = vec![ApiDocMember {
            name: "name".to_string(),
            kind: "property".to_string(),
            description: "Command name.".to_string(),
            signature: None,
            type_annotation: Some("string".to_string()),
            params: vec![],
            returns: None,
            optional: false,
            readonly: true,
            r#static: false,
            private: false,
            tags: vec![],
            line: 2,
            end_line: 2,
        }];
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "mod".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![entry],
        }];

        let list_markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                render_style: MarkdownRenderStyle::Markdown,
                ..MarkdownDocsOptions::default()
            },
        );
        let list_page = list_markdown.get("mod/interfaces/Command.md").unwrap();
        assert!(list_page.contains("- `name` _(readonly)_ `property` `string` - Command name."));
        assert!(!list_page.contains("| Name | Kind | Type | Description |"));

        let table_markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                render_style: MarkdownRenderStyle::Markdown,
                interface_properties_format: MarkdownDisplayFormat::Table,
                ..MarkdownDocsOptions::default()
            },
        );
        let table_page = table_markdown.get("mod/interfaces/Command.md").unwrap();
        assert!(table_page.contains("| Name | Kind | Type | Description |"));
        assert!(
            table_page.contains("| `name` _(readonly)_ | property | `string` | Command name. |")
        );
    }

    #[test]
    fn markdown_member_parameters_follow_parameters_format() {
        let mut entry = test_entry("Command", "interface", "src/types.ts", "Command options.");
        entry.members = vec![ApiDocMember {
            name: "run".to_string(),
            kind: "method".to_string(),
            description: "Runs the command.".to_string(),
            signature: Some("run(ctx: Context): Promise<void>".to_string()),
            type_annotation: None,
            params: vec![ApiParamDoc {
                name: "ctx".to_string(),
                type_annotation: "Context".to_string(),
                description: "Runtime context.".to_string(),
                optional: false,
                default_value: None,
            }],
            returns: None,
            optional: false,
            readonly: false,
            r#static: false,
            private: false,
            tags: vec![],
            line: 2,
            end_line: 2,
        }];
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "mod".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![entry],
        }];

        let list_markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                render_style: MarkdownRenderStyle::Markdown,
                ..MarkdownDocsOptions::default()
            },
        );
        let list_page = list_markdown.get("mod/interfaces/Command.md").unwrap();
        assert!(list_page.contains("### run Parameters"));
        assert!(list_page.contains("- `ctx` (`Context`) - Runtime context."));

        let table_markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                render_style: MarkdownRenderStyle::Markdown,
                parameters_format: MarkdownDisplayFormat::Table,
                ..MarkdownDocsOptions::default()
            },
        );
        let table_page = table_markdown.get("mod/interfaces/Command.md").unwrap();
        assert!(table_page.contains("### run Parameters"));
        assert!(table_page.contains("| Name | Type | Description |"));
        assert!(table_page.contains("| `ctx` | `Context` | Runtime context. |"));
    }

    #[test]
    fn html_display_format_options_switch_explicit_sections() {
        let mut make = test_entry("make", "function", "src/make.ts", "Make a thing.");
        make.params = vec![ApiParamDoc {
            name: "value".to_string(),
            type_annotation: "string".to_string(),
            description: "Input value.".to_string(),
            optional: false,
            default_value: None,
        }];
        make.type_parameters = vec![ApiTypeParamDoc {
            name: "T".to_string(),
            constraint: None,
            default: None,
            description: "Value type.".to_string(),
        }];

        let mut command = test_entry("Command", "interface", "src/types.ts", "Command options.");
        command.members = vec![
            ApiDocMember {
                name: "name".to_string(),
                kind: "property".to_string(),
                description: "Command name.".to_string(),
                signature: None,
                type_annotation: Some("string".to_string()),
                params: vec![],
                returns: None,
                optional: false,
                readonly: true,
                r#static: false,
                private: false,
                tags: vec![],
                line: 2,
                end_line: 2,
            },
            ApiDocMember {
                name: "run".to_string(),
                kind: "method".to_string(),
                description: "Runs the command.".to_string(),
                signature: Some("run(ctx: Context): Promise<void>".to_string()),
                type_annotation: None,
                params: vec![ApiParamDoc {
                    name: "ctx".to_string(),
                    type_annotation: "Context".to_string(),
                    description: "Runtime context.".to_string(),
                    optional: false,
                    default_value: None,
                }],
                returns: None,
                optional: false,
                readonly: false,
                r#static: false,
                private: false,
                tags: vec![],
                line: 3,
                end_line: 3,
            },
        ];

        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "mod".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![make, command],
        }];

        let table_markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                index_format: MarkdownDisplayFormat::Table,
                parameters_format: MarkdownDisplayFormat::Table,
                interface_properties_format: MarkdownDisplayFormat::List,
                ..MarkdownDocsOptions::default()
            },
        );
        let index = table_markdown.get("index.md").unwrap();
        assert!(index.contains("<table class=\"ox-api-modules-table\">"));
        assert!(index.contains("<th>Module</th><th>Symbols</th><th>Description</th>"));

        let page = table_markdown.get("mod.md").unwrap();
        assert!(page.contains("<table class=\"ox-api-entry__params-table\">"));
        assert!(page.contains("<table class=\"ox-api-entry__member-params-table\">"));
        assert!(!page.contains("<ul class=\"ox-api-entry__params\">"));
        assert!(page.contains("<ul class=\"ox-api-entry__members-list\">"));
        assert!(page.contains("<li id=\"command-name\" class=\"ox-api-entry__member\">"));

        let list_markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                index_format: MarkdownDisplayFormat::List,
                parameters_format: MarkdownDisplayFormat::List,
                ..MarkdownDocsOptions::default()
            },
        );
        let index = list_markdown.get("index.md").unwrap();
        assert!(index.contains("<ul class=\"ox-api-modules-list\">"));
        assert!(!index.contains("<details class=\"ox-api-module\">"));

        let page = list_markdown.get("mod.md").unwrap();
        assert!(page.contains("<ul class=\"ox-api-entry__type-parameters\">"));
        assert!(page.contains("<ul class=\"ox-api-entry__member-params\">"));
        assert!(!page.contains("<table class=\"ox-api-entry__type-parameters-table\">"));
    }

    #[test]
    fn typedoc_module_index_renders_member_tables() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                test_entry("cli", "function", "/repo/src/cli.ts", "Run the command."),
                test_entry("CliOptions", "interface", "/repo/src/types.ts", "CLI options."),
            ],
        }];

        let out = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );
        let index = out.get("default/index.md").unwrap();

        assert!(index.contains("## Functions"));
        assert!(index.contains("| Function | Description |\n| ------ | ------ |"));
        assert!(index.contains("| [cli](./functions/cli.md) | Run the command. |"));
        assert!(index.contains("## Interfaces"));
        assert!(index.contains("| Interface | Description |"));
        assert!(index.contains("| [CliOptions](./interfaces/CliOptions.md) | CLI options. |"));
        // No bullet list, no inlined kind label or signature.
        assert!(!index.contains("- [`cli`]"));
        assert!(!index.contains("`function`"));
        assert!(!index.contains("export function cli"));
    }

    #[test]
    fn typedoc_module_index_resolves_links_in_table_cells() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                test_entry(
                    "Args",
                    "interface",
                    "/repo/src/args.ts",
                    "An object that contains {@link ArgSchema | argument schema}.",
                ),
                test_entry("ArgSchema", "interface", "/repo/src/args.ts", "A schema."),
            ],
        }];

        let out = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );
        let index = out.get("default/index.md").unwrap();

        // The `{@link}` is resolved to a Markdown link inside the cell, not left raw.
        assert!(index.contains("[argument schema](./interfaces/ArgSchema.md)"));
        assert!(!index.contains("{@link"));
    }

    #[test]
    fn typedoc_module_index_collapses_overloads_to_one_row() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                test_entry("cli", "function", "/repo/src/cli.ts", "Run the command."),
                test_entry("cli", "function", "/repo/src/cli.ts", "Run the command."),
            ],
        }];

        let out = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );
        let index = out.get("default/index.md").unwrap();

        assert_eq!(index.matches("| [cli](./functions/cli.md) |").count(), 1);
    }

    #[test]
    fn typedoc_module_index_escapes_pipes_in_table_cells() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![test_entry("toUnion", "function", "/repo/src/u.ts", "Returns A | B.")],
        }];

        let out = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );
        let index = out.get("default/index.md").unwrap();

        assert!(index.contains("Returns A \\| B."));
    }

    #[test]
    fn typedoc_path_strategy_uses_clean_base_path_and_module_scope() {
        let docs = vec![
            ApiDocModule {
                description: String::new(),
                file: "default".to_string(),
                source_path: String::new(),
                examples: vec![],
                tags: vec![],
                entries: vec![
                    test_entry("Command", "interface", "/repo/src/default.ts", "Default command."),
                    test_entry(
                        "runDefault",
                        "function",
                        "/repo/src/default.ts",
                        "Runs {@link Command}.",
                    ),
                ],
            },
            ApiDocModule {
                description: String::new(),
                file: "plugin".to_string(),
                source_path: String::new(),
                examples: vec![],
                tags: vec![],
                entries: vec![
                    test_entry("Command", "interface", "/repo/src/plugin.ts", "Plugin command."),
                    test_entry(
                        "runPlugin",
                        "function",
                        "/repo/src/plugin.ts",
                        "Runs {@link Command}.",
                    ),
                ],
            },
        ];

        let markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                link_style: MarkdownLinkStyle::Clean,
                base_path: Some("/api".to_string()),
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );
        let default_page = markdown.get("default/functions/runDefault.md").unwrap();
        let plugin_page = markdown.get("plugin/functions/runPlugin.md").unwrap();
        let index = markdown.get("index.md").unwrap();

        assert!(index.contains("[default](/api/default)"));
        assert!(default_page.contains("<a href=\"/api/default/interfaces/Command\">Command</a>"));
        assert!(plugin_page.contains("<a href=\"/api/plugin/interfaces/Command\">Command</a>"));
        assert!(!default_page.contains(".md"));
    }

    fn lifecycle_module(entry: ApiDocEntry) -> Vec<ApiDocModule> {
        vec![ApiDocModule {
            description: String::new(),
            file: "combinators".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![entry],
        }]
    }

    fn markdown_typedoc_options() -> MarkdownDocsOptions {
        MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Markdown,
            ..MarkdownDocsOptions::default()
        }
    }

    fn group_order_docs() -> Vec<ApiDocModule> {
        vec![ApiDocModule {
            description: "Module description.".to_string(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                test_entry("alpha", "function", "/repo/src/a.ts", "A function."),
                test_entry("Config", "interface", "/repo/src/c.ts", "An interface."),
                test_entry("Engine", "class", "/repo/src/e.ts", "A class."),
                test_entry("VERSION", "variable", "/repo/src/v.ts", "A variable."),
            ],
        }]
    }

    #[test]
    fn order_by_group_title_none_preserves_order() {
        let sections = vec![("Functions".to_string(), 1), ("Variables".to_string(), 2)];
        assert_eq!(order_by_group_title(sections.clone(), None), sections);
    }

    #[test]
    fn order_by_group_title_orders_listed_then_alphabetical() {
        let sections = vec![
            ("Functions".to_string(), 1),
            ("Classes".to_string(), 2),
            ("Interfaces".to_string(), 3),
            ("References".to_string(), 4),
            ("Type Aliases".to_string(), 5),
            ("Variables".to_string(), 6),
        ];
        let group_order = ["Variables".to_string(), "Functions".to_string(), "Class".to_string()];
        let titles = order_by_group_title(sections, Some(&group_order))
            .into_iter()
            .map(|(title, _)| title)
            .collect::<Vec<_>>();

        // `Class` does not match the `Classes` group and is ignored; unlisted
        // groups (including References) follow alphabetically.
        assert_eq!(
            titles,
            vec!["Variables", "Functions", "Classes", "Interfaces", "References", "Type Aliases"]
        );
    }

    #[test]
    fn order_by_group_title_places_unspecified_at_star() {
        let sections = vec![
            ("Functions".to_string(), 1),
            ("Classes".to_string(), 2),
            ("Variables".to_string(), 3),
        ];
        let group_order = ["Variables".to_string(), "*".to_string(), "Functions".to_string()];
        let titles = order_by_group_title(sections, Some(&group_order))
            .into_iter()
            .map(|(title, _)| title)
            .collect::<Vec<_>>();

        assert_eq!(titles, vec!["Variables", "Classes", "Functions"]);
    }

    #[test]
    fn typedoc_group_order_defaults_to_fixed_kind_order() {
        let out = generate_markdown(&group_order_docs(), &markdown_typedoc_options());
        let index = out.get("default/index.md").unwrap();
        let functions = index.find("## Functions").unwrap();
        let classes = index.find("## Classes").unwrap();
        let interfaces = index.find("## Interfaces").unwrap();
        let variables = index.find("## Variables").unwrap();

        // Unchanged historical order (DOC_KIND_ORDER).
        assert!(functions < classes);
        assert!(classes < interfaces);
        assert!(interfaces < variables);
    }

    #[test]
    fn typedoc_group_order_reorders_module_index_sections() {
        let options = MarkdownDocsOptions {
            group_order: Some(vec!["Variables".to_string(), "Functions".to_string()]),
            ..markdown_typedoc_options()
        };
        let out = generate_markdown(&group_order_docs(), &options);
        let index = out.get("default/index.md").unwrap();
        let variables = index.find("## Variables").unwrap();
        let functions = index.find("## Functions").unwrap();
        let classes = index.find("## Classes").unwrap();
        let interfaces = index.find("## Interfaces").unwrap();

        // Listed groups lead in order; the rest follow alphabetically.
        assert!(variables < functions);
        assert!(functions < classes);
        assert!(classes < interfaces);
    }

    #[test]
    fn typedoc_group_order_supports_star_wildcard() {
        let options = MarkdownDocsOptions {
            group_order: Some(vec![
                "Variables".to_string(),
                "*".to_string(),
                "Functions".to_string(),
            ]),
            ..markdown_typedoc_options()
        };
        let out = generate_markdown(&group_order_docs(), &options);
        let index = out.get("default/index.md").unwrap();
        let variables = index.find("## Variables").unwrap();
        let classes = index.find("## Classes").unwrap();
        let interfaces = index.find("## Interfaces").unwrap();
        let functions = index.find("## Functions").unwrap();

        // Variables first, unspecified groups (alphabetical) in the middle, Functions last.
        assert!(variables < classes);
        assert!(classes < interfaces);
        assert!(interfaces < functions);
    }

    fn stats_docs() -> Vec<ApiDocModule> {
        vec![ApiDocModule {
            description: "Module description.".to_string(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                test_entry("cli", "function", "/repo/src/cli.ts", "Run the CLI."),
                test_entry("run", "function", "/repo/src/run.ts", "Run again."),
            ],
        }]
    }

    #[test]
    fn render_stats_keeps_stats_summary_by_default() {
        let out = generate_markdown(&stats_docs(), &markdown_typedoc_options());

        assert!(out.get("index.md").unwrap().contains("symbols ·"));
        assert!(out.get("default/index.md").unwrap().contains("symbols ·"));
    }

    #[test]
    fn render_stats_false_omits_stats_summary() {
        let options = MarkdownDocsOptions { render_stats: false, ..markdown_typedoc_options() };
        let out = generate_markdown(&stats_docs(), &options);
        let root = out.get("index.md").unwrap();
        let module_index = out.get("default/index.md").unwrap();

        assert!(!root.contains("symbols ·"));
        assert!(!module_index.contains("symbols ·"));
        // The gate also drops the trailing separator, so no stray blank line is
        // left where the stats summary used to be.
        assert!(!root.contains("\n\n\n"));
        assert!(!module_index.contains("\n\n\n"));
    }

    #[test]
    fn render_stats_false_omits_html_stats_block() {
        let options = MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Html,
            render_stats: false,
            ..MarkdownDocsOptions::default()
        };
        let out = generate_markdown(&stats_docs(), &options);

        // The option also gates the HTML render style's stats block.
        assert!(!out.get("index.md").unwrap().contains("ox-api-stats"));
        assert!(!out.get("default/index.md").unwrap().contains("ox-api-stats"));
    }

    fn overload_entry(
        name: &str,
        file: &str,
        description: &str,
        signature: &str,
        has_body: bool,
    ) -> ApiDocEntry {
        ApiDocEntry {
            name: name.to_string(),
            kind: "function".to_string(),
            description: description.to_string(),
            params: vec![],
            returns: None,
            examples: vec![],
            tags: vec![],
            private: false,
            file: file.to_string(),
            line: 1,
            end_line: 1,
            signature: Some(signature.to_string()),
            has_body,
            members: vec![],
            type_parameters: vec![],
        }
    }

    fn overload_module(entries: Vec<ApiDocEntry>) -> Vec<ApiDocModule> {
        vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries,
        }]
    }

    fn html_typedoc_options() -> MarkdownDocsOptions {
        MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            render_style: MarkdownRenderStyle::Html,
            ..MarkdownDocsOptions::default()
        }
    }

    #[test]
    fn typedoc_html_overloads_render_all_call_signatures() {
        let docs = overload_module(vec![
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin with extension.",
                "export function plugin<E>(options: WithExt): PluginWithExtension<E>",
                false,
            ),
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin without extension.",
                "export function plugin(options: WithoutExt): PluginWithoutExtension",
                false,
            ),
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin",
                "export function plugin(options: any = {}): any",
                true,
            ),
        ]);
        let out = generate_markdown(&docs, &html_typedoc_options());
        let page = out.get("default/functions/plugin.md").unwrap();

        // Both public overloads survive on one html page; the implementation is hidden.
        assert_eq!(page.matches("<h4>Call Signature</h4>").count(), 2);
        assert!(page.contains("PluginWithExtension"));
        assert!(page.contains("PluginWithoutExtension"));
        assert!(!page.contains("options: any = {}"));
    }

    #[test]
    fn typedoc_html_badges_include_experimental_and_version() {
        let mut entry = test_entry("widget", "function", "/repo/src/w.ts", "A widget.");
        entry.tags = vec![
            ApiDocTag { tag: "experimental".to_string(), value: String::new() },
            ApiDocTag { tag: "version".to_string(), value: "1.2.3".to_string() },
        ];
        let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
        let page = out.get("combinators/functions/widget.md").unwrap();

        assert!(page.contains(">experimental</span>"));
        assert!(page.contains("version 1.2.3"));
        // Every tag is structured, so no generic tag list is emitted.
        assert!(!page.contains("ox-api-entry__section--tags"));
    }

    #[test]
    fn typedoc_html_dedups_structured_tags_from_tag_list() {
        let mut entry = test_entry("run", "function", "/repo/src/run.ts", "Run.");
        entry.tags = vec![
            ApiDocTag { tag: "see".to_string(), value: "related".to_string() },
            ApiDocTag { tag: "deprecated".to_string(), value: "use other".to_string() },
            ApiDocTag { tag: "since".to_string(), value: "1.0.0".to_string() },
        ];
        let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
        let page = out.get("combinators/functions/run.md").unwrap();

        // Structured tags become badges (not duplicated in the tag list); `@see` stays.
        assert!(page.contains(">deprecated</span>"));
        assert!(page.contains("since 1.0.0"));
        assert!(page.contains("@see"));
        assert!(!page.contains("@deprecated"));
        assert!(!page.contains("@since"));
    }

    #[test]
    fn typedoc_html_member_table_shows_lifecycle_and_since_markers() {
        let mut entry = test_entry("Options", "interface", "/repo/src/o.ts", "Options.");
        entry.members = vec![ApiDocMember {
            name: "mode".to_string(),
            kind: "property".to_string(),
            description: "The mode.".to_string(),
            signature: None,
            type_annotation: Some("string".to_string()),
            params: vec![],
            returns: None,
            optional: false,
            readonly: false,
            r#static: false,
            private: false,
            tags: vec![
                ApiDocTag { tag: "deprecated".to_string(), value: String::new() },
                ApiDocTag { tag: "since".to_string(), value: "1.0.0".to_string() },
            ],
            line: 1,
            end_line: 1,
        }];
        let out = generate_markdown(&lifecycle_module(entry), &html_typedoc_options());
        let page = out.get("combinators/interfaces/Options.md").unwrap();

        assert!(page.contains(">deprecated</span>"));
        assert!(page.contains("since 1.0.0"));
    }

    #[test]
    fn typedoc_html_module_index_shows_lifecycle_badges() {
        let mut docs = lifecycle_module(test_entry("run", "function", "/repo/src/run.ts", "Run."));
        docs[0].tags = vec![ApiDocTag { tag: "experimental".to_string(), value: String::new() }];
        let out = generate_markdown(&docs, &html_typedoc_options());
        let module_index = out.get("combinators/index.md").unwrap();

        assert!(module_index.contains("ox-api-module__meta"));
        assert!(module_index.contains(">experimental</span>"));
    }

    #[test]
    fn typedoc_overloads_render_all_call_signatures() {
        let docs = overload_module(vec![
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin with extension.",
                "export function plugin<E>(options: WithExt): PluginWithExtension<E>",
                false,
            ),
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin without extension.",
                "export function plugin(options: WithoutExt): PluginWithoutExtension",
                false,
            ),
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin",
                "export function plugin(options: any = {}): any",
                true,
            ),
        ]);
        let out = generate_markdown(&docs, &markdown_typedoc_options());
        let page = out.get("default/functions/plugin.md").unwrap();

        assert!(page.contains("# Function: plugin()"));
        // Both public overloads survive on one page (not overwritten by the last).
        assert_eq!(page.matches("## Call Signature").count(), 2);
        assert!(page.contains("PluginWithExtension<E>"));
        assert!(page.contains("PluginWithoutExtension"));
    }

    #[test]
    fn typedoc_overloads_omit_implementation_signature() {
        let docs = overload_module(vec![
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin with extension.",
                "export function plugin<E>(options: WithExt): PluginWithExtension<E>",
                false,
            ),
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin without extension.",
                "export function plugin(options: WithoutExt): PluginWithoutExtension",
                false,
            ),
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin",
                "export function plugin(options: any = {}): any",
                true,
            ),
        ]);
        let out = generate_markdown(&docs, &markdown_typedoc_options());
        let page = out.get("default/functions/plugin.md").unwrap();

        // The implementation signature is hidden, not rendered as a call signature.
        assert!(!page.contains("options: any = {}"));
        assert!(!page.contains("## Signature"));
    }

    #[test]
    fn typedoc_overload_page_hoists_implementation_summary_and_since() {
        let mut implementation = overload_entry(
            "plugin",
            "/repo/src/plugin.ts",
            "Define a plugin",
            "export function plugin(options: any = {}): any",
            true,
        );
        implementation.tags =
            vec![ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() }];
        let docs = overload_module(vec![
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin with extension.",
                "export function plugin<E>(options: WithExt): PluginWithExtension<E>",
                false,
            ),
            overload_entry(
                "plugin",
                "/repo/src/plugin.ts",
                "Define a plugin without extension.",
                "export function plugin(options: WithoutExt): PluginWithoutExtension",
                false,
            ),
            implementation,
        ]);
        let out = generate_markdown(&docs, &markdown_typedoc_options());
        let page = out.get("default/functions/plugin.md").unwrap();

        // The implementation's summary and `## Since` are hoisted above the first
        // call signature (TypeDoc treats the implementation comment as the symbol
        // comment).
        assert!(page.contains("Define a plugin\n\n## Since\n\nv0.27.0"));
        let since = page.find("## Since").unwrap();
        let call = page.find("## Call Signature").unwrap();
        assert!(since < call);
    }

    #[test]
    fn typedoc_single_public_overload_renders_inline() {
        let docs = overload_module(vec![
            overload_entry(
                "define",
                "/repo/src/definition.ts",
                "Define a command.",
                "export function define<G>(definition: CommandDefinition<G>): CommandDefinitionResult<G>",
                false,
            ),
            overload_entry(
                "define",
                "/repo/src/definition.ts",
                "Define a command.",
                "export function define(definition: any): any",
                true,
            ),
        ]);
        let out = generate_markdown(&docs, &markdown_typedoc_options());
        let page = out.get("default/functions/define.md").unwrap();

        // A single public overload collapses to a normal symbol page (no
        // `## Call Signature` wrapper) showing the typed signature, not `any`.
        assert!(!page.contains("## Call Signature"));
        assert!(page.contains("## Signature"));
        assert!(page.contains("CommandDefinition<G>"));
        assert!(!page.contains("definition: any"));
    }

    #[test]
    fn typedoc_dts_overloads_without_implementation_render_all() {
        let docs = overload_module(vec![
            overload_entry(
                "merge",
                "/repo/src/merge.d.ts",
                "Merge one source.",
                "export function merge(a: A): A",
                false,
            ),
            overload_entry(
                "merge",
                "/repo/src/merge.d.ts",
                "Merge two sources.",
                "export function merge(a: A, b: B): A & B",
                false,
            ),
        ]);
        let out = generate_markdown(&docs, &markdown_typedoc_options());
        let page = out.get("default/functions/merge.md").unwrap();

        // No implementation exists (.d.ts); every call signature is preserved.
        assert_eq!(page.matches("## Call Signature").count(), 2);
        assert!(page.contains("merge(a: A): A"));
        assert!(page.contains("merge(a: A, b: B): A & B"));
    }

    #[test]
    fn typedoc_renders_experimental_tag_as_warning_alert() {
        let mut entry =
            test_entry("string", "function", "/repo/src/combinators.ts", "Create a string schema.");
        entry.tags = vec![ApiDocTag { tag: "experimental".to_string(), value: String::new() }];
        let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
        let page = out.get("combinators/functions/string.md").unwrap();

        assert!(page.contains(
            "> [!WARNING]\n> This API is experimental and may change in future versions."
        ));
        // Lifecycle tags move to the alert, not the generic Tags section.
        assert!(!page.contains("## Tags"));
        assert!(!page.contains("@experimental"));
        assert!(!page.contains("**Deprecated.**"));
    }

    #[test]
    fn typedoc_renders_deprecated_tag_as_caution_alert_with_body() {
        let mut entry =
            test_entry("oldDefine", "function", "/repo/src/definition.ts", "Old helper.");
        entry.tags = vec![ApiDocTag {
            tag: "deprecated".to_string(),
            value: "Use `define` instead.".to_string(),
        }];
        let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
        let page = out.get("combinators/functions/oldDefine.md").unwrap();

        assert!(page.contains("> [!CAUTION]\n> Use `define` instead."));
        assert!(!page.contains("## Tags"));
    }

    #[test]
    fn typedoc_keeps_non_structured_tags_in_tags_section() {
        // `@see` is neither a lifecycle tag nor a `@since`/`@version` tag, so it
        // stays in the generic `## Tags` list while structured tags move out.
        let mut entry = test_entry("run", "function", "/repo/src/run.ts", "Run it.");
        entry.tags = vec![
            ApiDocTag { tag: "see".to_string(), value: "related".to_string() },
            ApiDocTag { tag: "since".to_string(), value: "1.0.0".to_string() },
            ApiDocTag { tag: "experimental".to_string(), value: String::new() },
        ];
        let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
        let page = out.get("combinators/functions/run.md").unwrap();

        assert!(page.contains("> [!WARNING]"));
        assert!(page.contains("## Since"));
        assert!(page.contains("## Tags"));
        assert!(page.contains("`@see`"));
        assert!(!page.contains("`@since`"));
        assert!(!page.contains("`@experimental`"));
    }

    #[test]
    fn typedoc_marks_experimental_members_in_table() {
        let mut entry =
            test_entry("StringOptions", "interface", "/repo/src/combinators.ts", "String options.");
        entry.members = vec![ApiDocMember {
            name: "minLength".to_string(),
            kind: "property".to_string(),
            description: "Minimum string length.".to_string(),
            signature: None,
            type_annotation: Some("number".to_string()),
            params: vec![],
            returns: None,
            optional: true,
            readonly: false,
            r#static: false,
            private: false,
            tags: vec![ApiDocTag { tag: "experimental".to_string(), value: String::new() }],
            line: 1,
            end_line: 1,
        }];
        let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
        let page = out.get("combinators/interfaces/StringOptions.md").unwrap();

        assert!(page.contains("**Experimental.** Minimum string length."));
    }

    #[test]
    fn typedoc_renders_since_as_dedicated_section() {
        let mut entry = test_entry("Example", "interface", "/repo/src/example.ts", "Example API.");
        entry.tags = vec![ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() }];
        let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
        let page = out.get("combinators/interfaces/Example.md").unwrap();

        assert!(page.contains("## Since\n\nv0.27.0"));
        assert!(!page.contains("## Tags"));
        assert!(!page.contains("@since"));
    }

    #[test]
    fn typedoc_normalizes_version_into_since_section() {
        let mut entry = test_entry("Example", "interface", "/repo/src/example.ts", "Example API.");
        entry.tags = vec![ApiDocTag { tag: "version".to_string(), value: "1.2.3".to_string() }];
        let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
        let page = out.get("combinators/interfaces/Example.md").unwrap();

        assert!(page.contains("## Since\n\n1.2.3"));
        assert!(!page.contains("## Version"));
        assert!(!page.contains("## Tags"));
    }

    #[test]
    fn typedoc_combines_since_and_version_into_one_section() {
        let mut entry = test_entry("Example", "interface", "/repo/src/example.ts", "Example API.");
        entry.tags = vec![
            ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() },
            ApiDocTag { tag: "version".to_string(), value: "1.2.3".to_string() },
        ];
        let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
        let page = out.get("combinators/interfaces/Example.md").unwrap();

        assert!(page.contains("## Since\n\nv0.27.0\n\n1.2.3"));
        assert_eq!(page.matches("## Since").count(), 1);
    }

    #[test]
    fn typedoc_renders_member_since_inline() {
        let mut entry =
            test_entry("PluginOptions", "interface", "/repo/src/plugin.ts", "Plugin options.");
        entry.members = vec![ApiDocMember {
            name: "entry".to_string(),
            kind: "property".to_string(),
            description: "Whether this is an entry command.".to_string(),
            signature: None,
            type_annotation: Some("boolean".to_string()),
            params: vec![],
            returns: None,
            optional: true,
            readonly: false,
            r#static: false,
            private: false,
            tags: vec![ApiDocTag { tag: "since".to_string(), value: "v0.27.0".to_string() }],
            line: 1,
            end_line: 1,
        }];
        let out = generate_markdown(&lifecycle_module(entry), &markdown_typedoc_options());
        let page = out.get("combinators/interfaces/PluginOptions.md").unwrap();

        assert!(page.contains("Whether this is an entry command. **Since** v0.27.0"));
    }

    #[test]
    fn typedoc_renders_module_level_experimental_alert() {
        let docs = vec![ApiDocModule {
            description: "Parser combinator entry point.".to_string(),
            file: "combinators".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![ApiDocTag {
                tag: "experimental".to_string(),
                value: "This module is experimental and may change in future versions.".to_string(),
            }],
            entries: vec![test_entry("string", "function", "/repo/src/combinators.ts", "S.")],
        }];
        let out = generate_markdown(&docs, &markdown_typedoc_options());
        let index = out.get("combinators/index.md").unwrap();

        // Alert sits between the title and the description.
        assert!(index.contains(
            "# combinators\n\n> [!WARNING]\n> This module is experimental and may change in future versions.\n\nParser combinator entry point."
        ));
    }

    #[test]
    fn typedoc_resolves_links_in_lifecycle_alert_body() {
        let mut entry = test_entry("string", "function", "/repo/src/combinators.ts", "S.");
        entry.tags = vec![ApiDocTag {
            tag: "deprecated".to_string(),
            value: "Use {@link integer} instead.".to_string(),
        }];
        let mut docs = lifecycle_module(entry);
        docs[0].entries.push(test_entry("integer", "function", "/repo/src/combinators.ts", "I."));
        let out = generate_markdown(&docs, &markdown_typedoc_options());
        let page = out.get("combinators/functions/string.md").unwrap();

        assert!(page.contains("> [!CAUTION]"));
        assert!(page.contains("Use [integer](./integer.md) instead."));
        assert!(!page.contains("{@link"));
    }

    #[test]
    fn typedoc_dedupes_cross_entrypoint_reexports_to_canonical_page() {
        // `createCommandContext` is defined in context.ts and re-exported from
        // the default and plugin entry points. It must produce a single page
        // under its defining module (context), with the re-exporters linking to
        // it via a References section.
        let docs = vec![
            ApiDocModule {
                description: String::new(),
                file: "context".to_string(),
                source_path: "/repo/src/context.ts".to_string(),
                examples: vec![],
                tags: vec![],
                entries: vec![test_entry(
                    "createCommandContext",
                    "function",
                    "/repo/src/context.ts",
                    "Creates a command context.",
                )],
            },
            ApiDocModule {
                description: String::new(),
                file: "default".to_string(),
                source_path: "/repo/src/index.ts".to_string(),
                examples: vec![],
                tags: vec![],
                entries: vec![
                    test_entry(
                        "createCommandContext",
                        "function",
                        "/repo/src/context.ts",
                        "Creates a command context.",
                    ),
                    test_entry(
                        "runDefault",
                        "function",
                        "/repo/src/index.ts",
                        "Uses {@link createCommandContext}.",
                    ),
                ],
            },
            ApiDocModule {
                description: String::new(),
                file: "plugin".to_string(),
                source_path: "/repo/src/plugin.ts".to_string(),
                examples: vec![],
                tags: vec![],
                entries: vec![test_entry(
                    "createCommandContext",
                    "function",
                    "/repo/src/context.ts",
                    "Creates a command context.",
                )],
            },
        ];

        let out = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );

        // Exactly one canonical page, placed under the defining module.
        assert!(out.contains_key("context/functions/createCommandContext.md"));
        assert!(!out.contains_key("default/functions/createCommandContext.md"));
        assert!(!out.contains_key("plugin/functions/createCommandContext.md"));

        // The defining module lists it as a real entry; re-exporters reference it.
        let context_index = out.get("context/index.md").unwrap();
        assert!(context_index.contains("## Functions"));
        assert!(!context_index.contains("## References"));

        let default_index = out.get("default/index.md").unwrap();
        assert!(default_index.contains("## References"));
        // TypeDoc-style reference entry (heading + "Re-exports" link), not a bullet.
        assert!(default_index.contains("### createCommandContext"));
        assert!(default_index.contains("Re-exports [createCommandContext]("));
        assert!(!default_index.contains("- Re-exports"));
        // The re-export reference and any cross-link resolve to the canonical page.
        assert!(default_index.contains("context/functions/createCommandContext"));

        let run_default = out.get("default/functions/runDefault.md").unwrap();
        assert!(run_default.contains("context/functions/createCommandContext"));
    }

    #[test]
    fn typedoc_references_section_uses_typedoc_layout() {
        // Two symbols defined in `context` and re-exported from `default` produce
        // a TypeDoc-style References section: `### Name` headings, `Re-exports`
        // links, and a `***` separator between entries.
        let make = |module: &str, source: &str, entries: Vec<ApiDocEntry>| ApiDocModule {
            description: String::new(),
            file: module.to_string(),
            source_path: source.to_string(),
            examples: vec![],
            tags: vec![],
            entries,
        };
        let docs = vec![
            make(
                "context",
                "/repo/src/context.ts",
                vec![
                    test_entry("createCommandContext", "function", "/repo/src/context.ts", "Ctx."),
                    test_entry("CommandContextParams", "interface", "/repo/src/context.ts", "P."),
                ],
            ),
            make(
                "default",
                "/repo/src/index.ts",
                vec![
                    test_entry("createCommandContext", "function", "/repo/src/context.ts", "Ctx."),
                    test_entry("CommandContextParams", "interface", "/repo/src/context.ts", "P."),
                ],
            ),
        ];

        let out = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );
        let default_index = out.get("default/index.md").unwrap();

        assert!(default_index.contains("## References"));
        assert!(default_index.contains("### CommandContextParams"));
        // The link resolves to the canonical page under the owner module (context).
        assert!(default_index.contains(
            "Re-exports [CommandContextParams](../context/interfaces/CommandContextParams.md)"
        ));
        assert!(default_index.contains("### createCommandContext"));
        // Two references → exactly one thematic-break separator between them.
        assert_eq!(default_index.matches("\n***\n").count(), 1);
        assert!(!default_index.contains("- Re-exports"));
    }

    #[test]
    fn typedoc_references_collapse_overloads_to_one_entry() {
        // An overloaded function (two signatures) re-exported from another module
        // is referenced once, not once per overload.
        let docs = vec![
            ApiDocModule {
                description: String::new(),
                file: "definition".to_string(),
                source_path: "/repo/src/definition.ts".to_string(),
                examples: vec![],
                tags: vec![],
                entries: vec![
                    test_entry("define", "function", "/repo/src/definition.ts", "Define."),
                    test_entry("define", "function", "/repo/src/definition.ts", "Define."),
                ],
            },
            ApiDocModule {
                description: String::new(),
                file: "default".to_string(),
                source_path: "/repo/src/index.ts".to_string(),
                examples: vec![],
                tags: vec![],
                entries: vec![
                    test_entry("define", "function", "/repo/src/definition.ts", "Define."),
                    test_entry("define", "function", "/repo/src/definition.ts", "Define."),
                ],
            },
        ];

        let out = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );
        let default_index = out.get("default/index.md").unwrap();

        assert_eq!(default_index.matches("### define").count(), 1);
        assert_eq!(default_index.matches("Re-exports [define]").count(), 1);
    }

    #[test]
    fn typedoc_dedupe_without_source_path_uses_first_module() {
        // `Command` is defined in a non-entry-point file (command.ts), so no
        // module owns it via source_path; the canonical page falls back to the
        // first module (sorted) that exports it.
        let docs = vec![
            ApiDocModule {
                description: String::new(),
                file: "default".to_string(),
                source_path: "/repo/src/index.ts".to_string(),
                examples: vec![],
                tags: vec![],
                entries: vec![test_entry(
                    "Command",
                    "interface",
                    "/repo/src/command.ts",
                    "A command.",
                )],
            },
            ApiDocModule {
                description: String::new(),
                file: "plugin".to_string(),
                source_path: "/repo/src/plugin.ts".to_string(),
                examples: vec![],
                tags: vec![],
                entries: vec![test_entry(
                    "Command",
                    "interface",
                    "/repo/src/command.ts",
                    "A command.",
                )],
            },
        ];

        let out = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );

        assert!(out.contains_key("default/interfaces/Command.md"));
        assert!(!out.contains_key("plugin/interfaces/Command.md"));
        let plugin_index = out.get("plugin/index.md").unwrap();
        assert!(plugin_index.contains("### Command"));
        assert!(plugin_index.contains("Re-exports [Command]("));
    }

    #[test]
    fn category_group_ignores_typedoc_path_strategy() {
        let docs = link_test_docs();

        let category_flat = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                group_by: "category".to_string(),
                ..MarkdownDocsOptions::default()
            },
        );
        let category_typedoc = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                group_by: "category".to_string(),
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );

        let mut flat_keys = category_flat.keys().cloned().collect::<Vec<_>>();
        let mut typedoc_keys = category_typedoc.keys().cloned().collect::<Vec<_>>();
        flat_keys.sort();
        typedoc_keys.sort();
        assert_eq!(flat_keys, typedoc_keys);

        assert!(category_typedoc.contains_key("functions.md"));
        assert!(category_typedoc.contains_key("interfaces.md"));
        assert!(!category_typedoc.keys().any(|key| key.contains('/')));
    }

    #[test]
    fn typedoc_path_strategy_emits_enumerations_directory() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                ApiDocEntry {
                    name: "Mode".to_string(),
                    kind: "enum".to_string(),
                    description: "Execution mode.".to_string(),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    tags: vec![],
                    private: false,
                    file: "/repo/src/mode.ts".to_string(),
                    line: 1,
                    end_line: 5,
                    signature: Some("export enum Mode".to_string()),
                    has_body: false,
                    members: vec![ApiDocMember {
                        name: "Strict".to_string(),
                        kind: "enumMember".to_string(),
                        description: "Strict mode.".to_string(),
                        signature: None,
                        type_annotation: Some("\"strict\"".to_string()),
                        params: vec![],
                        returns: None,
                        optional: false,
                        readonly: false,
                        r#static: false,
                        private: false,
                        tags: vec![],
                        line: 2,
                        end_line: 2,
                    }],
                    type_parameters: vec![],
                },
                ApiDocEntry {
                    name: "run".to_string(),
                    kind: "function".to_string(),
                    description: "Runs in {@link Mode} or {@linkcode Mode.Strict}.".to_string(),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    tags: vec![],
                    private: false,
                    file: "/repo/src/run.ts".to_string(),
                    line: 1,
                    end_line: 5,
                    signature: Some("export function run(mode: Mode): void".to_string()),
                    has_body: false,
                    members: vec![],
                    type_parameters: vec![],
                },
            ],
        }];

        let markdown = generate_markdown(
            &docs,
            &MarkdownDocsOptions {
                path_strategy: MarkdownPathStrategy::TypeDoc,
                ..MarkdownDocsOptions::default()
            },
        );
        let mode_page = markdown.get("default/enumerations/Mode.md").unwrap();
        let run_page = markdown.get("default/functions/run.md").unwrap();
        let module_index = markdown.get("default/index.md").unwrap();

        assert!(module_index.contains("## Enumerations"));
        assert!(module_index.contains("| Enumeration | Description |"));
        assert!(module_index.contains("| [Mode](./enumerations/Mode.md) |"));
        assert!(mode_page.contains("<tr id=\"enumeration-member-strict\">"));
        assert!(run_page.contains("<a href=\"../enumerations/Mode.md\">Mode</a>"));
        assert!(run_page.contains(
            "<a href=\"../enumerations/Mode.md#enumeration-member-strict\"><code>Mode.Strict</code></a>"
        ));
    }

    #[test]
    fn renders_interface_members_table() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "/repo/src/command.ts".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![ApiDocEntry {
                name: "Command".to_string(),
                kind: "interface".to_string(),
                description: "Runtime command.".to_string(),
                params: vec![],
                returns: None,
                examples: vec![],
                tags: vec![],
                private: false,
                file: "/repo/src/command.ts".to_string(),
                line: 1,
                end_line: 10,
                signature: Some("export interface Command".to_string()),
                has_body: false,
                members: vec![
                    ApiDocMember {
                        name: "name".to_string(),
                        kind: "property".to_string(),
                        description: "Command name.".to_string(),
                        signature: None,
                        type_annotation: Some("string".to_string()),
                        params: vec![],
                        returns: None,
                        optional: false,
                        readonly: true,
                        r#static: false,
                        private: false,
                        tags: vec![],
                        line: 5,
                        end_line: 5,
                    },
                    ApiDocMember {
                        name: "run".to_string(),
                        kind: "method".to_string(),
                        description: "Runs the command.".to_string(),
                        signature: Some("run(ctx: Context): Promise<void>".to_string()),
                        type_annotation: None,
                        params: vec![ApiParamDoc {
                            name: "ctx".to_string(),
                            type_annotation: "Context".to_string(),
                            description: "Runtime context.".to_string(),
                            optional: false,
                            default_value: None,
                        }],
                        returns: Some(ApiReturnDoc {
                            type_annotation: "Promise".to_string(),
                            description: "Run result.".to_string(),
                        }),
                        optional: false,
                        readonly: false,
                        r#static: false,
                        private: false,
                        tags: vec![],
                        line: 7,
                        end_line: 7,
                    },
                ],
                type_parameters: vec![],
            }],
        }];

        let markdown = generate_markdown(&docs, &MarkdownDocsOptions::default());
        let page = markdown.get("command.md").unwrap();

        assert!(page.contains("<h4>Members</h4>"));
        assert!(page.contains("<h5>Properties</h5>"));
        assert!(page.contains("<code>name</code>"));
        assert!(page.contains("readonly"));
        assert!(page.contains("Command name."));
        assert!(page.contains("<h5>Methods</h5>"));
        assert!(page.contains("run(ctx: Context): Promise&lt;void&gt;"));
        assert!(page.contains("Runtime context."));
    }
}
