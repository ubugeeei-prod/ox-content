//! Markdown rendering for generated API reference documentation.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::model::{ApiDocEntry, ApiDocMember, ApiDocModule};

mod markdown_html;
mod markdown_pure;

const DOC_KIND_ORDER: [&str; 7] =
    ["function", "class", "interface", "type", "enum", "variable", "module"];

type RegexCache = OnceLock<Option<Regex>>;

fn cached_regex(cache: &'static RegexCache, pattern: &'static str) -> Option<&'static Regex> {
    cache.get_or_init(|| Regex::new(pattern).ok()).as_ref()
}

fn push_fmt(output: &mut String, args: std::fmt::Arguments<'_>) {
    if output.write_fmt(args).is_err() {
        output.push_str("[formatting failed]");
    }
}

fn fmt_args(args: std::fmt::Arguments<'_>) -> String {
    let mut output = String::new();
    push_fmt(&mut output, args);
    output
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

impl Default for MarkdownDocsOptions {
    fn default() -> Self {
        Self {
            group_by: default_group_by(),
            github_url: None,
            link_style: MarkdownLinkStyle::Markdown,
            base_path: None,
            path_strategy: MarkdownPathStrategy::Flat,
            render_style: MarkdownRenderStyle::Html,
        }
    }
}

fn default_group_by() -> String {
    "file".to_string()
}

#[derive(Debug, Clone, Default)]
struct EntryStats {
    entries: usize,
    by_kind: BTreeMap<String, usize>,
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
            result.insert(format!("{file_name}.md"), markdown);
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
            entries.sort_by(|left, right| compare_strings(&left.name, &right.name));
        }

        for (kind, entries) in &by_kind {
            result.insert(
                format!("{kind}s.md"),
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
            return format!("#{anchor}");
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
        format!("./{path}")
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
        format!("/{base_path}")
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
            format!("{}-{}", entry_anchor(entry_name), entry_anchor(&member.name))
        }
        MarkdownPathStrategy::TypeDoc => {
            let prefix = match member.kind.as_str() {
                "constructor" => return "constructor".to_string(),
                "method" => "method",
                "getter" | "setter" => "accessor",
                "enumMember" => "enumeration-member",
                _ => "property",
            };
            format!("{prefix}-{}", entry_anchor(&member.name))
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

fn typedoc_kind_segment(kind: &str) -> &'static str {
    match kind {
        "function" => "functions",
        "class" => "classes",
        "interface" => "interfaces",
        "type" => "type-aliases",
        "enum" => "enumerations",
        "variable" | "const" => "variables",
        "module" => "modules",
        _ => "symbols",
    }
}

fn typedoc_kind_title(kind: &str) -> &'static str {
    match kind {
        "function" => "Functions",
        "class" => "Classes",
        "interface" => "Interfaces",
        "type" => "Type Aliases",
        "enum" => "Enumerations",
        "variable" | "const" => "Variables",
        "module" => "Modules",
        _ => "Symbols",
    }
}

fn typedoc_entry_file_name(module_name: &str, entry: &ApiDocEntry) -> String {
    format!(
        "{}/{}/{}",
        module_name,
        typedoc_kind_segment(&entry.kind),
        sanitize_doc_path_segment(&entry.name)
    )
}

fn typedoc_module_index_file_name(module_name: &str) -> String {
    format!("{module_name}/index")
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
            format!("#{anchor}")
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
        format!("`{}`", label.trim_matches('`'))
    } else {
        label.to_string()
    };

    if let Some(href) = resolve_jsdoc_link_target(link.target, context) {
        format!("[{label}]({href})")
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

    let collapsed = markdown_link_re.replace_all(text, "$1").to_string();
    let collapsed = bracket_link_re.replace_all(&collapsed, "$1").to_string();
    let collapsed = inline_code_re.replace_all(&collapsed, "$1").to_string();
    let collapsed = whitespace_re.replace_all(&collapsed, " ").trim().to_string();

    truncate_summary_text(&collapsed, max_length)
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
        *stats.by_kind.entry(entry.kind.clone()).or_default() += 1;
        stats.members += entry.members.len();
        stats.params += entry.params.len();
        stats.returns += usize::from(entry.returns.is_some());
        stats.examples += entry.examples.len();
        stats.deprecated += usize::from(entry.tags.iter().any(|tag| tag.tag == "deprecated"));
    }

    stats
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

fn compare_strings(left: &str, right: &str) -> std::cmp::Ordering {
    left.to_lowercase().cmp(&right.to_lowercase()).then_with(|| left.cmp(right))
}

fn sort_extracted_docs(docs: &[ApiDocModule]) -> Vec<ApiDocModule> {
    let mut sorted = docs.to_vec();

    for doc in &mut sorted {
        doc.entries.sort_by(|left, right| compare_strings(&left.name, &right.name));
    }

    sorted.sort_by(|left, right| compare_strings(&file_name(&left.file), &file_name(&right.file)));
    sorted
}

/// Renders the per-page stats summary in the configured render style.
fn render_stats(
    options: &MarkdownDocsOptions,
    stats: &EntryStats,
    module_count: Option<usize>,
) -> String {
    match options.render_style {
        MarkdownRenderStyle::Html => markdown_html::render_stats_html(stats, module_count),
        MarkdownRenderStyle::Markdown => markdown_pure::render_stats_markdown(stats, module_count),
    }
}

fn generate_file_markdown(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    current_file_name: &str,
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
) -> String {
    let display_name = file_name(&doc.file);
    let mut markdown = String::new();
    push_fmt(&mut markdown, format_args!("# {display_name}\n\n"));

    if let Some(github_url) = &options.github_url {
        markdown.push_str(&generate_source_link(&doc.file, github_url, None, None));
        markdown.push_str("\n\n");
    }

    push_fmt(
        &mut markdown,
        format_args!(
            "> {} documented symbol{}. ",
            doc.entries.len(),
            if doc.entries.len() == 1 { "" } else { "s" }
        ),
    );
    markdown.push_str(
        "Read the signatures first, then expand each item for parameters, return types, and examples.\n\n",
    );

    markdown.push_str(&render_stats(options, &summarize_entries(&doc.entries), None));
    markdown.push_str("\n\n");

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

fn generate_typedoc_markdown(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();

    result.insert("index.md".to_string(), generate_typedoc_root_index(docs, options, symbol_map));

    for doc in docs {
        let module_name = module_file_name(&doc.file);
        let module_index_file_name = typedoc_module_index_file_name(&module_name);
        result.insert(
            format!("{module_index_file_name}.md"),
            generate_typedoc_module_index(doc, options, &module_name, symbol_map),
        );

        for entry in &doc.entries {
            let entry_file_name = typedoc_entry_file_name(&module_name, entry);
            result.insert(
                format!("{entry_file_name}.md"),
                generate_typedoc_entry_page(
                    entry,
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
    markdown.push_str("Generated by [Ox Content](https://github.com/ubugeeei/ox-content)\n\n");
    markdown.push_str(&render_stats(
        options,
        &summarize_entries(docs.iter().flat_map(|doc| doc.entries.iter())),
        Some(docs.len()),
    ));
    markdown.push_str("\n\n## Modules\n\n");

    for doc in docs {
        let module_name = module_file_name(&doc.file);
        let href = doc_page_href_from(
            options,
            "index",
            &typedoc_module_index_file_name(&module_name),
            None,
        );
        let summary = clean_summary_text(
            &doc.entries.first().map_or_else(String::new, |entry| {
                process_doc_text(&entry.description, Some(&link_context))
            }),
            88,
        );
        if summary.is_empty() {
            push_fmt(
                &mut markdown,
                format_args!("- [{}]({href})\n", capitalize_ascii(&module_name)),
            );
        } else {
            push_fmt(
                &mut markdown,
                format_args!("- [{}]({href}) - {summary}\n", capitalize_ascii(&module_name)),
            );
        }
    }

    markdown
}

fn generate_typedoc_module_index(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    module_name: &str,
    symbol_map: &HashMap<String, Vec<SymbolLocation>>,
) -> String {
    let current_file_name = typedoc_module_index_file_name(module_name);
    let link_context = MarkdownLinkContext {
        options,
        current_file_name: &current_file_name,
        current_module_name: module_name,
        symbol_map,
    };
    let mut markdown = fmt_args(format_args!("# {}\n\n", capitalize_ascii(module_name)));

    if let Some(github_url) = &options.github_url {
        markdown.push_str(&generate_source_link(&doc.file, github_url, None, None));
        markdown.push_str("\n\n");
    }

    markdown.push_str(&render_stats(options, &summarize_entries(&doc.entries), None));
    markdown.push_str("\n\n");

    for kind in ordered_entry_kinds(&doc.entries) {
        let entries = doc.entries.iter().filter(|entry| entry.kind == kind).collect::<Vec<_>>();
        if entries.is_empty() {
            continue;
        }

        push_fmt(&mut markdown, format_args!("## {}\n\n", typedoc_kind_title(&kind)));
        for entry in entries {
            let href = doc_page_href_from(
                options,
                &current_file_name,
                &typedoc_entry_file_name(module_name, entry),
                None,
            );
            markdown.push_str(&render_overview_line(entry, &href, Some(&link_context)));
        }
        markdown.push('\n');
    }

    markdown
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
    if options.render_style == MarkdownRenderStyle::Markdown {
        let body = markdown_pure::render_entry_body_pure(entry, options, Some(&link_context));
        let mut markdown = fmt_args(format_args!("# {}\n\n", entry.name));
        if !body.is_empty() {
            markdown.push_str(&body);
            markdown.push('\n');
        }
        return markdown;
    }

    let mut markdown = fmt_args(format_args!("# {}\n\n", entry.name));
    markdown.push_str(&markdown_html::render_entry_page_html(entry, options, Some(&link_context)));
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
    fmt_args(format_args!("{count} {label}"))
}

fn entry_tag_value<'a>(entry: &'a ApiDocEntry, tag_name: &str) -> Option<&'a str> {
    entry.tags.iter().find(|tag| tag.tag == tag_name).map(|tag| tag.value.as_str())
}

fn get_entry_badges(entry: &ApiDocEntry) -> Vec<EntryBadge> {
    let mut badges = Vec::new();

    if entry_tag_value(entry, "deprecated").is_some() {
        badges.push(EntryBadge { label: "deprecated".to_string(), tone: Some("warning") });
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
        badges.push(EntryBadge {
            label: fmt_args(format_args!("returns {}", returns.type_annotation)),
            tone: None,
        });
    }
    if !entry.examples.is_empty() {
        badges.push(EntryBadge {
            label: format_count_label(entry.examples.len(), "example", Some("examples")),
            tone: None,
        });
    }
    if let Some(since) = entry_tag_value(entry, "since") {
        badges.push(EntryBadge { label: fmt_args(format_args!("since {since}")), tone: None });
    }
    if entry.private {
        badges.push(EntryBadge { label: "private".to_string(), tone: Some("warning") });
    }

    badges
}

fn parse_example_block(example: &str) -> (String, String) {
    static FENCE_RE: RegexCache = OnceLock::new();

    let trimmed = example.trim();
    let Some(fence_re) = cached_regex(&FENCE_RE, r"(?s)^```([\w-]+)?[^\n]*\n(.*?)\n?```$") else {
        return (trimmed.to_string(), "ts".to_string());
    };

    if let Some(captures) = fence_re.captures(trimmed) {
        let language = captures.get(1).map_or("ts", |value| value.as_str()).to_string();
        let code = captures.get(2).map_or("", |value| value.as_str()).to_string();
        (code, language)
    } else {
        (trimmed.to_string(), "ts".to_string())
    }
}

fn render_overview_line(
    entry: &ApiDocEntry,
    href: &str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let signature = normalize_signature(entry.signature.as_deref());
    let summary = clean_summary_text(&process_doc_text(&entry.description, context), 88);
    let mut parts = vec![format!("- [`{}`]({href})", entry.name), format!("`{}`", entry.kind)];

    if let Some(signature) = signature {
        parts.push(format!("`{signature}`"));
    }

    if !summary.is_empty() {
        parts.push(format!("- {summary}"));
    }

    format!("{}\n", parts.join(" "))
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
        let body = markdown_pure::render_entry_body_pure(entry, options, link_context);
        let mut markdown = fmt_args(format_args!("### {}\n\n", entry.name));
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
    markdown.push_str("Generated by [Ox Content](https://github.com/ubugeeei/ox-content)\n\n");
    markdown.push_str(
        "> Use search scopes like `@api transform` to limit results to the generated API reference.\n\n",
    );
    markdown.push_str(&render_stats(
        options,
        &summarize_entries(docs.iter().flat_map(|doc| doc.entries.iter())),
        Some(docs.len()),
    ));
    markdown.push_str("\n\n");

    markdown.push_str("## Modules\n\n");
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

        let count_label = fmt_args(format_args!(
            "{} symbol{}",
            doc.entries.len(),
            if doc.entries.len() == 1 { "" } else { "s" }
        ));

        if options.render_style == MarkdownRenderStyle::Markdown {
            push_fmt(
                &mut markdown,
                format_args!(
                    "### [{}]({}) — {count_label}\n\n",
                    display_name,
                    doc_page_href(options, &file_name, None)
                ),
            );
            for entry in &doc.entries {
                let href = doc_page_href(options, &file_name, Some(&entry_anchor(&entry.name)));
                markdown.push_str(&render_overview_line(entry, &href, link_context.as_ref()));
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
    let category_file_name = fmt_args(format_args!("{kind}s"));
    let link_context = MarkdownLinkContext {
        options,
        current_file_name: &category_file_name,
        current_module_name: "",
        symbol_map,
    };
    let mut markdown = fmt_args(format_args!("# {}s\n\n", capitalize_ascii(kind)));
    push_fmt(
        &mut markdown,
        format_args!(
            "> {} documented {kind}{} collected across modules.\n\n",
            entries.len(),
            if entries.len() == 1 { "" } else { "s" }
        ),
    );
    markdown.push_str(&render_stats(options, &summarize_entries(entries), None));
    markdown.push_str("\n\n");

    markdown.push_str("## Overview\n\n");
    for entry in entries {
        markdown.push_str(&render_overview_line(
            entry,
            &fmt_args(format_args!("#{}", entry_anchor(&entry.name))),
            Some(&link_context),
        ));
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
    markdown.push_str("Generated by [Ox Content](https://github.com/ubugeeei/ox-content)\n\n");
    markdown.push_str(&render_stats(
        options,
        &summarize_entries(by_kind.values().flat_map(|entries| entries.iter())),
        None,
    ));
    markdown.push_str("\n\n");

    for (kind, entries) in by_kind {
        let kind_title = fmt_args(format_args!("{}s", capitalize_ascii(kind)));
        let category_file_name = fmt_args(format_args!("{kind}s"));
        push_fmt(
            &mut markdown,
            format_args!(
                "## [{kind_title}]({})\n\n",
                doc_page_href(options, &category_file_name, None)
            ),
        );
        push_fmt(
            &mut markdown,
            format_args!(
                "> {} item{}.\n\n",
                entries.len(),
                if entries.len() == 1 { "" } else { "s" }
            ),
        );

        for entry in entries {
            let href =
                doc_page_href(options, &category_file_name, Some(&entry_anchor(&entry.name)));
            markdown.push_str(&render_overview_line(entry, &href, Some(&link_context)));
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
        push_fmt(
            &mut result,
            format_args!("[{symbol_name}]({})", format_symbol_href(context, location)),
        );
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

    for doc in docs {
        let module_name = module_file_name(&doc.file);
        for entry in &doc.entries {
            let (file_name, anchor) = match (options.group_by.as_str(), options.path_strategy) {
                ("file", MarkdownPathStrategy::TypeDoc) => {
                    (typedoc_entry_file_name(&module_name, entry), None)
                }
                ("category", _) => {
                    (fmt_args(format_args!("{}s", entry.kind)), Some(entry_anchor(&entry.name)))
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
                    fmt_args(format_args!("{}.{}", entry.name, member.name)),
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
        match end_line_number.filter(|end_line_number| *end_line_number > line_number) {
            Some(end_line_number) => format!("#L{line_number}-L{end_line_number}"),
            None => format!("#L{line_number}"),
        }
    } else {
        String::new()
    };

    format!("{github_url}/blob/main/{relative_path}{fragment}")
}

fn generate_source_link(
    file_path: &str,
    github_url: &str,
    line_number: Option<u32>,
    end_line_number: Option<u32>,
) -> String {
    format!(
        "**[Source]({})**",
        generate_source_href(file_path, github_url, line_number, end_line_number)
    )
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
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ApiDocTag, ApiParamDoc, ApiReturnDoc};

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
            signature: Some(format!("export function {name}(): void")),
            members: vec![],
        }
    }

    fn link_test_docs() -> Vec<ApiDocModule> {
        vec![
            ApiDocModule {
                file: "/repo/src/context.ts".to_string(),
                entries: vec![test_entry(
                    "CommandContext",
                    "interface",
                    "/repo/src/context.ts",
                    "Command context.",
                )],
            },
            ApiDocModule {
                file: "/repo/src/command.ts".to_string(),
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
            file: "/repo/src/cli.ts".to_string(),
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
                    members: vec![],
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
        assert!(page.contains("**Signature**"));
        assert!(page.contains("```ts"));
        assert!(page.contains("| Name | Type | Description |"));
        assert!(page.contains("**Members**"));
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
        assert!(page.starts_with("# cli"));
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
                file: "/repo/src/agent.ts".to_string(),
                entries: vec![test_entry(
                    "AgentProfile",
                    "interface",
                    "/repo/src/agent.ts",
                    "Agent profile.",
                )],
            },
            ApiDocModule {
                file: "/repo/src/command.ts".to_string(),
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
                }],
            },
            ApiDocModule {
                file: "/repo/src/build.ts".to_string(),
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
                    members: vec![],
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
            file: "default".to_string(),
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
                    members: vec![],
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
        assert!(module_index.contains("[`cli`](./functions/cli.md)"));
        assert!(module_index.contains("[`CliOptions`](./interfaces/CliOptions.md)"));
        assert!(cli_page.contains("<a href=\"../interfaces/CliOptions.md\">CliOptions</a>"));
        assert!(cli_page.contains(
            "<a href=\"../interfaces/CliOptions.md#property-usagesilent\"><code>CliOptions.usageSilent</code></a>"
        ));
        assert!(options_page.contains("<tr id=\"property-usagesilent\">"));
    }

    #[test]
    fn typedoc_path_strategy_uses_clean_base_path_and_module_scope() {
        let docs = vec![
            ApiDocModule {
                file: "default".to_string(),
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
                file: "plugin".to_string(),
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

        assert!(index.contains("[Default](/api/default)"));
        assert!(default_page.contains("<a href=\"/api/default/interfaces/Command\">Command</a>"));
        assert!(plugin_page.contains("<a href=\"/api/plugin/interfaces/Command\">Command</a>"));
        assert!(!default_page.contains(".md"));
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
            file: "default".to_string(),
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
                    members: vec![],
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
        assert!(module_index.contains("[`Mode`](./enumerations/Mode.md)"));
        assert!(mode_page.contains("<tr id=\"enumeration-member-strict\">"));
        assert!(run_page.contains("<a href=\"../enumerations/Mode.md\">Mode</a>"));
        assert!(run_page.contains(
            "<a href=\"../enumerations/Mode.md#enumeration-member-strict\"><code>Mode.Strict</code></a>"
        ));
    }

    #[test]
    fn renders_interface_members_table() {
        let docs = vec![ApiDocModule {
            file: "/repo/src/command.ts".to_string(),
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
