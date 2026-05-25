//! Markdown rendering for generated API reference documentation.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

const DOC_KIND_ORDER: [&str; 6] = ["function", "class", "interface", "type", "variable", "module"];

/// Extracted docs for one source module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiDocModule {
    /// Source file path.
    pub file: String,
    /// Documented entries in the source file.
    pub entries: Vec<ApiDocEntry>,
}

/// A single normalized API documentation entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiDocEntry {
    /// Entry name.
    pub name: String,
    /// Entry kind.
    pub kind: String,
    /// Human-readable description.
    pub description: String,
    /// Parameters.
    pub params: Vec<ApiParamDoc>,
    /// Return documentation.
    pub returns: Option<ApiReturnDoc>,
    /// Example blocks.
    pub examples: Vec<String>,
    /// Custom JSDoc tags, kept in source insertion order.
    pub tags: Vec<ApiDocTag>,
    /// Whether the entry is private.
    pub private: bool,
    /// Source file path.
    pub file: String,
    /// Declaration start line.
    pub line: u32,
    /// Declaration end line.
    pub end_line: u32,
    /// Full source signature.
    pub signature: Option<String>,
    /// Members belonging to class/interface/type/enum entries.
    #[serde(default)]
    pub members: Vec<ApiDocMember>,
}

/// Documentation for a member of a class/interface/type/enum entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiDocMember {
    /// Member name.
    pub name: String,
    /// Member kind.
    pub kind: String,
    /// Human-readable description.
    pub description: String,
    /// Full member signature.
    pub signature: Option<String>,
    /// Property or enum member type/value annotation.
    pub type_annotation: Option<String>,
    /// Parameters.
    #[serde(default)]
    pub params: Vec<ApiParamDoc>,
    /// Return documentation.
    pub returns: Option<ApiReturnDoc>,
    /// Whether the member is optional.
    #[serde(default)]
    pub optional: bool,
    /// Whether the member is readonly.
    #[serde(default)]
    pub readonly: bool,
    /// Whether the member is static.
    #[serde(default)]
    pub r#static: bool,
    /// Whether the member is private.
    #[serde(default)]
    pub private: bool,
    /// Custom JSDoc tags, kept in source insertion order.
    #[serde(default)]
    pub tags: Vec<ApiDocTag>,
    /// Declaration start line.
    pub line: u32,
    /// Declaration end line.
    pub end_line: u32,
}

/// Parameter documentation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiParamDoc {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub type_annotation: String,
    /// Parameter description.
    pub description: String,
    /// Whether the parameter is optional.
    pub optional: bool,
    /// Default value.
    pub default_value: Option<String>,
}

/// Return type documentation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiReturnDoc {
    /// Return type.
    pub type_annotation: String,
    /// Return description.
    pub description: String,
}

/// Custom JSDoc tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiDocTag {
    /// Tag name without `@`.
    pub tag: String,
    /// Tag value.
    pub value: String,
}

/// Options for generated API Markdown.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkdownDocsOptions {
    /// Grouping mode: `file` or `category`.
    pub group_by: String,
    /// GitHub repository URL for source links.
    pub github_url: Option<String>,
}

impl Default for MarkdownDocsOptions {
    fn default() -> Self {
        Self { group_by: "file".to_string(), github_url: None }
    }
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
    file_name: String,
}

/// Generates Markdown documentation pages from extracted API docs.
#[must_use]
pub fn generate_markdown(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    let sorted_docs = sort_extracted_docs(docs);
    let symbol_map = build_symbol_map(&sorted_docs);

    if options.group_by == "file" {
        let mut doc_to_file = HashMap::new();

        for doc in &sorted_docs {
            let mut file_name = file_stem(&doc.file);
            if file_name == "index" {
                file_name = "index-module".to_string();
            }
            doc_to_file.insert(doc.file.clone(), file_name.clone());

            let markdown = generate_file_markdown(doc, options, &file_name, &symbol_map);
            result.insert(format!("{file_name}.md"), markdown);
        }

        result.insert("index.md".to_string(), generate_index(&sorted_docs, Some(&doc_to_file)));
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

        result.insert("index.md".to_string(), generate_category_index(&by_kind));
    }

    result
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn entry_anchor(name: &str) -> String {
    name.to_lowercase()
}

fn clean_summary_text(text: &str, max_length: usize) -> String {
    static MARKDOWN_LINK_RE: OnceLock<Regex> = OnceLock::new();
    static BRACKET_LINK_RE: OnceLock<Regex> = OnceLock::new();
    static WHITESPACE_RE: OnceLock<Regex> = OnceLock::new();

    if text.is_empty() {
        return String::new();
    }

    let markdown_link_re =
        MARKDOWN_LINK_RE.get_or_init(|| Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap());
    let bracket_link_re = BRACKET_LINK_RE.get_or_init(|| Regex::new(r"\[([^\]]+)\]").unwrap());
    let whitespace_re = WHITESPACE_RE.get_or_init(|| Regex::new(r"\s+").unwrap());

    let collapsed = markdown_link_re.replace_all(text, "$1").to_string();
    let collapsed = bracket_link_re.replace_all(&collapsed, "$1").to_string();
    let collapsed = whitespace_re.replace_all(&collapsed, " ").trim().to_string();

    if collapsed.chars().count() <= max_length {
        return collapsed;
    }

    let truncated: String = collapsed.chars().take(max_length.saturating_sub(1)).collect();
    format!("{}…", truncated.trim_end())
}

fn render_inline_html(text: &str) -> String {
    static TOKEN_RE: OnceLock<Regex> = OnceLock::new();

    let token_re = TOKEN_RE.get_or_init(|| {
        Regex::new(
            r"`([^`]+)`|\[([^\]]+)\]\(([^)]+)\)|\*\*([^*]+)\*\*|__([^_]+)__|\*([^*]+)\*|_([^_]+)_",
        )
        .unwrap()
    });
    let mut html = String::new();
    let mut last_index = 0;

    for captures in token_re.captures_iter(text) {
        let mat = captures.get(0).expect("token match");
        html.push_str(&escape_html(&text[last_index..mat.start()]));

        if let Some(code) = captures.get(1) {
            html.push_str(&format!("<code>{}</code>", escape_html(code.as_str())));
        } else if let (Some(label), Some(href)) = (captures.get(2), captures.get(3)) {
            html.push_str(&format!(
                "<a href=\"{}\">{}</a>",
                escape_html(href.as_str()),
                render_inline_html(label.as_str())
            ));
        } else if let Some(strong) = captures.get(4).or_else(|| captures.get(5)) {
            html.push_str(&format!("<strong>{}</strong>", render_inline_html(strong.as_str())));
        } else if let Some(emphasis) = captures.get(6).or_else(|| captures.get(7)) {
            html.push_str(&format!("<em>{}</em>", render_inline_html(emphasis.as_str())));
        }

        last_index = mat.end();
    }

    html.push_str(&escape_html(&text[last_index..]));
    html.replace('\n', "<br>")
}

fn is_fence_start(line: &str) -> Option<String> {
    static FENCE_RE: OnceLock<Regex> = OnceLock::new();

    let fence_re = FENCE_RE.get_or_init(|| Regex::new(r"^```([\w-]+)?\s*$").unwrap());
    fence_re
        .captures(line.trim())
        .map(|captures| captures.get(1).map_or("text", |value| value.as_str()).to_string())
}

fn heading_match(line: &str) -> Option<(usize, String)> {
    static HEADING_RE: OnceLock<Regex> = OnceLock::new();

    let heading_re = HEADING_RE.get_or_init(|| Regex::new(r"^(#{1,6})\s+(.*)$").unwrap());
    heading_re.captures(line.trim()).map(|captures| {
        (
            captures.get(1).map_or(1, |value| value.as_str().len()).min(6),
            captures.get(2).map_or("", |value| value.as_str()).trim().to_string(),
        )
    })
}

fn ordered_list_item(line: &str) -> Option<String> {
    static ORDERED_RE: OnceLock<Regex> = OnceLock::new();

    let ordered_re = ORDERED_RE.get_or_init(|| Regex::new(r"^\d+\.\s+(.*)$").unwrap());
    ordered_re
        .captures(line.trim())
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn unordered_list_item(line: &str) -> Option<String> {
    static UNORDERED_RE: OnceLock<Regex> = OnceLock::new();

    let unordered_re = UNORDERED_RE.get_or_init(|| Regex::new(r"^[-*+]\s+(.*)$").unwrap());
    unordered_re
        .captures(line.trim())
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()))
}

fn is_markdown_block_start(line: &str) -> bool {
    is_fence_start(line).is_some()
        || heading_match(line).is_some()
        || ordered_list_item(line).is_some()
        || unordered_list_item(line).is_some()
}

fn render_markdown_blocks_html(text: &str) -> String {
    static ORDERED_CONTINUATION_RE: OnceLock<Regex> = OnceLock::new();
    static UNORDERED_CONTINUATION_RE: OnceLock<Regex> = OnceLock::new();

    let lines: Vec<&str> =
        text.split('\n').map(|line| line.strip_suffix('\r').unwrap_or(line)).collect();
    let mut blocks = Vec::new();
    let mut index = 0;
    let ordered_continuation_re =
        ORDERED_CONTINUATION_RE.get_or_init(|| Regex::new(r"^ {0,1}\d+\.\s+").unwrap());
    let unordered_continuation_re =
        UNORDERED_CONTINUATION_RE.get_or_init(|| Regex::new(r"^[-*+]\s+").unwrap());

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            index += 1;
            continue;
        }

        if let Some(language) = is_fence_start(line) {
            let mut code_lines = Vec::new();
            index += 1;

            while index < lines.len() && !lines[index].trim().starts_with("```") {
                code_lines.push(lines[index]);
                index += 1;
            }

            if index < lines.len() {
                index += 1;
            }

            blocks.push(render_code_block_html(&code_lines.join("\n"), &language));
            continue;
        }

        if let Some((level, content)) = heading_match(line) {
            blocks.push(format!("<h{level}>{}</h{level}>", render_inline_html(&content)));
            index += 1;
            continue;
        }

        if let Some(first_item) = ordered_list_item(line) {
            let mut items = Vec::new();
            let mut current = Some(first_item);

            while index < lines.len() {
                let Some(item_text) = current.take().or_else(|| ordered_list_item(lines[index]))
                else {
                    break;
                };

                let mut item_lines = vec![item_text.trim().to_string()];
                index += 1;

                while index < lines.len() {
                    let continuation = lines[index];
                    let continuation_trimmed = continuation.trim();

                    if continuation_trimmed.is_empty()
                        || is_markdown_block_start(continuation)
                        || ordered_continuation_re.is_match(continuation_trimmed)
                    {
                        break;
                    }

                    item_lines.push(continuation_trimmed.to_string());
                    index += 1;
                }

                items.push(format!("<li>{}</li>", render_inline_html(&item_lines.join(" "))));

                if index < lines.len() && lines[index].trim().is_empty() {
                    break;
                }
            }

            blocks.push(format!("<ol>\n{}\n</ol>", items.join("\n")));
            continue;
        }

        if let Some(first_item) = unordered_list_item(line) {
            let mut items = Vec::new();
            let mut current = Some(first_item);

            while index < lines.len() {
                let Some(item_text) = current.take().or_else(|| unordered_list_item(lines[index]))
                else {
                    break;
                };

                let mut item_lines = vec![item_text.trim().to_string()];
                index += 1;

                while index < lines.len() {
                    let continuation = lines[index];
                    let continuation_trimmed = continuation.trim();

                    if continuation_trimmed.is_empty()
                        || is_markdown_block_start(continuation)
                        || unordered_continuation_re.is_match(continuation_trimmed)
                    {
                        break;
                    }

                    item_lines.push(continuation_trimmed.to_string());
                    index += 1;
                }

                items.push(format!("<li>{}</li>", render_inline_html(&item_lines.join(" "))));

                if index < lines.len() && lines[index].trim().is_empty() {
                    break;
                }
            }

            blocks.push(format!("<ul>\n{}\n</ul>", items.join("\n")));
            continue;
        }

        let mut paragraph_lines = vec![trimmed.to_string()];
        index += 1;

        while index < lines.len() {
            let next_line = lines[index];
            let next_trimmed = next_line.trim();

            if next_trimmed.is_empty() || is_markdown_block_start(next_line) {
                break;
            }

            paragraph_lines.push(next_trimmed.to_string());
            index += 1;
        }

        blocks.push(format!("<p>{}</p>", render_inline_html(&paragraph_lines.join(" "))));
    }

    format!("<div class=\"ox-api-entry__prose\">\n{}\n</div>", blocks.join("\n"))
}

fn render_code_block_html(code: &str, language: &str) -> String {
    format!("<pre><code class=\"language-{language}\">{}</code></pre>", escape_html(code))
}

fn render_highlighted_inline_code_html(code: &str, class_name: &str, language: &str) -> String {
    format!(
        "<code class=\"{} language-{language}\">{}</code>",
        escape_html(class_name),
        escape_html(code)
    )
}

fn render_details_controls_html(target_selector: &str) -> String {
    format!(
        "<div class=\"ox-api-controls\" data-ox-api-target=\"{target_selector}\" role=\"toolbar\" aria-label=\"Reference display controls\">
<button type=\"button\" class=\"ox-api-controls__button\" data-ox-api-toggle=\"expand\">Open all</button>
<button type=\"button\" class=\"ox-api-controls__button\" data-ox-api-toggle=\"collapse\">Close all</button>
</div>"
    )
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

fn render_stats_html(stats: &EntryStats, module_count: Option<usize>) -> String {
    let mut items = Vec::new();

    if let Some(module_count) = module_count {
        items.push(("modules".to_string(), module_count, None));
    }

    items.push(("symbols".to_string(), stats.entries, None));

    for kind in DOC_KIND_ORDER {
        if let Some(count) = stats.by_kind.get(kind).copied().filter(|count| *count > 0) {
            items.push((doc_kind_plural(kind).to_string(), count, None));
        }
    }

    if stats.params > 0 {
        items.push(("parameters".to_string(), stats.params, None));
    }
    if stats.members > 0 {
        items.push(("members".to_string(), stats.members, None));
    }
    if stats.returns > 0 {
        items.push(("returns".to_string(), stats.returns, None));
    }
    if stats.examples > 0 {
        items.push(("examples".to_string(), stats.examples, None));
    }
    if stats.deprecated > 0 {
        items.push(("deprecated".to_string(), stats.deprecated, Some("warning")));
    }

    let rendered_items = items
        .into_iter()
        .map(|(label, value, tone)| {
            format!(
                "<span class=\"ox-api-stat{}\">
  <strong>{value}</strong>
  <span>{}</span>
</span>",
                tone.map_or(String::new(), |tone| format!(" ox-api-stat--{tone}")),
                escape_html(&label)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("<div class=\"ox-api-stats\" aria-label=\"API reference summary\">\n{rendered_items}\n</div>")
}

fn doc_kind_plural(kind: &str) -> &'static str {
    match kind {
        "function" => "functions",
        "class" => "classes",
        "interface" => "interfaces",
        "type" => "types",
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

fn generate_file_markdown(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    current_file_name: &str,
    symbol_map: &HashMap<String, SymbolLocation>,
) -> String {
    let display_name = file_name(&doc.file);
    let mut markdown = format!("# {display_name}\n\n");

    if let Some(github_url) = &options.github_url {
        markdown.push_str(&generate_source_link(&doc.file, github_url, None, None));
        markdown.push_str("\n\n");
    }

    markdown.push_str(&format!(
        "> {} documented symbol{}. ",
        doc.entries.len(),
        if doc.entries.len() == 1 { "" } else { "s" }
    ));
    markdown.push_str(
        "Read the signatures first, then expand each item for parameters, return types, and examples.\n\n",
    );

    markdown.push_str(&render_stats_html(&summarize_entries(&doc.entries), None));
    markdown.push_str("\n\n");

    markdown.push_str("## Reference\n\n");
    if doc.entries.len() > 1 {
        markdown.push_str(&render_details_controls_html(".ox-api-entry"));
        markdown.push_str("\n\n");
    }

    for entry in &doc.entries {
        markdown.push_str(&generate_entry_markdown(
            entry,
            options,
            Some(current_file_name),
            Some(symbol_map),
        ));
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
    format!("{count} {label}")
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
        badges
            .push(EntryBadge { label: format!("returns {}", returns.type_annotation), tone: None });
    }
    if !entry.examples.is_empty() {
        badges.push(EntryBadge {
            label: format_count_label(entry.examples.len(), "example", Some("examples")),
            tone: None,
        });
    }
    if let Some(since) = entry_tag_value(entry, "since") {
        badges.push(EntryBadge { label: format!("since {since}"), tone: None });
    }
    if entry.private {
        badges.push(EntryBadge { label: "private".to_string(), tone: Some("warning") });
    }

    badges
}

fn render_entry_badges_html(entry: &ApiDocEntry, class_name: &str) -> String {
    let badges = get_entry_badges(entry);
    if badges.is_empty() {
        return String::new();
    }

    let mut rendered = String::new();
    for badge in badges {
        let tone_class = badge.tone.map_or(String::new(), |tone| format!(" ox-api-badge--{tone}"));
        write!(
            rendered,
            "<span class=\"ox-api-badge{}\">{}</span>",
            tone_class,
            escape_html(&badge.label)
        )
        .unwrap();
    }

    format!("<span class=\"{class_name}\">{rendered}</span>")
}

fn parse_example_block(example: &str) -> (String, String) {
    static FENCE_RE: OnceLock<Regex> = OnceLock::new();

    let trimmed = example.trim();
    let fence_re =
        FENCE_RE.get_or_init(|| Regex::new(r"(?s)^```([\w-]+)?[^\n]*\n(.*?)\n?```$").unwrap());

    if let Some(captures) = fence_re.captures(trimmed) {
        let language = captures.get(1).map_or("ts", |value| value.as_str()).to_string();
        let code = captures.get(2).map_or("", |value| value.as_str()).to_string();
        (code, language)
    } else {
        (trimmed.to_string(), "ts".to_string())
    }
}

fn render_overview_line(entry: &ApiDocEntry, href: &str) -> String {
    let signature = normalize_signature(entry.signature.as_deref());
    let summary = clean_summary_text(&entry.description, 88);
    let mut parts = vec![format!("- [`{}`]({href})", entry.name), format!("`{}`", entry.kind)];

    if let Some(signature) = signature {
        parts.push(format!("`{signature}`"));
    }

    if !summary.is_empty() {
        parts.push(format!("- {summary}"));
    }

    format!("{}\n", parts.join(" "))
}

fn render_overview_html_item(entry: &ApiDocEntry, href: &str) -> String {
    let signature = normalize_signature(entry.signature.as_deref());
    let summary = clean_summary_text(&entry.description, 88);
    let meta = render_entry_badges_html(entry, "ox-api-module__meta");
    let heading = if let Some(signature) = signature {
        format!(
            "<a href=\"{}\" class=\"ox-api-module__link\">{}</a>",
            escape_html(href),
            render_highlighted_inline_code_html(
                &signature,
                "ox-api-module__signature ox-api-module__signature--highlighted",
                "typescript",
            )
        )
    } else {
        format!(
            "<a href=\"{}\" class=\"ox-api-module__link\"><code class=\"ox-api-module__name\">{}</code></a>",
            escape_html(href),
            escape_html(&entry.name)
        )
    };

    format!(
        "<li><span class=\"ox-api-module__kind\">{}</span><div class=\"ox-api-module__item\">{}{summary_html}{meta}</div></li>",
        escape_html(format_kind_label(&entry.kind)),
        heading,
        summary_html = if summary.is_empty() {
            String::new()
        } else {
            format!("<span class=\"ox-api-module__summary\">{}</span>", render_inline_html(&summary))
        }
    )
}

fn render_params_list_html(params: &[ApiParamDoc]) -> String {
    let rows = params
        .iter()
        .map(|param| {
            let mut flags = Vec::new();
            if param.optional {
                flags.push("optional".to_string());
            }
            if let Some(default_value) = &param.default_value {
                flags.push(format!("default: {default_value}"));
            }
            let flag_text = flags.join(" · ");
            let description = [param.description.as_str(), flag_text.as_str()]
                .into_iter()
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" — ");

            format!(
                "<li class=\"ox-api-entry__param\">
  <div class=\"ox-api-entry__param-heading\">
    <code class=\"ox-api-entry__param-name\">{}</code>
    <code class=\"ox-api-entry__param-type\">{}</code>
  </div>
  {}
</li>",
                escape_html(&param.name),
                escape_html(&param.type_annotation),
                if description.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<p class=\"ox-api-entry__param-description\">{}</p>",
                        render_inline_html(&description)
                    )
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<div class=\"ox-api-entry__section ox-api-entry__section--params\">
<h4>Parameters</h4>
<ul class=\"ox-api-entry__params\">
{rows}
</ul>
</div>"
    )
}

fn render_tag_list_html(tags: &[ApiDocTag]) -> String {
    let mut items = String::new();
    for tag in tags {
        write!(
            items,
            "<li><span class=\"ox-api-entry__tag-name\">@{}</span><span class=\"ox-api-entry__tag-value\">{}</span></li>",
            escape_html(&tag.tag),
            render_inline_html(&tag.value)
        )
        .unwrap();
    }

    format!(
        "<div class=\"ox-api-entry__section ox-api-entry__section--tags\">
<h4>Tags</h4>
<ul class=\"ox-api-entry__tags\">{items}</ul>
</div>"
    )
}

fn render_member_flags(member: &ApiDocMember) -> String {
    let mut flags = Vec::new();
    if member.optional {
        flags.push("optional");
    }
    if member.readonly {
        flags.push("readonly");
    }
    if member.r#static {
        flags.push("static");
    }
    if member.private {
        flags.push("private");
    }

    let mut html = String::new();
    for flag in flags {
        write!(html, "<span class=\"ox-api-badge\">{flag}</span>").unwrap();
    }
    html
}

fn render_member_type_html(member: &ApiDocMember) -> String {
    let value = member
        .signature
        .as_deref()
        .or(member.type_annotation.as_deref())
        .or_else(|| member.returns.as_ref().map(|returns| returns.type_annotation.as_str()));

    value.map_or_else(String::new, |value| {
        render_highlighted_inline_code_html(value, "ox-api-entry__member-type", "typescript")
    })
}

fn render_member_description_html(member: &ApiDocMember) -> String {
    let mut blocks = Vec::new();

    if !member.description.is_empty() {
        blocks.push(format!(
            "<div class=\"ox-api-entry__member-description\">{}</div>",
            render_inline_html(&member.description)
        ));
    }

    if !member.params.is_empty() {
        let mut params = String::new();
        for param in &member.params {
            let mut description = param.description.clone();
            if param.optional {
                if description.is_empty() {
                    description.push_str("optional");
                } else {
                    description.push_str(" - optional");
                }
            }
            write!(
                params,
                "<li><code>{}</code>{}</li>",
                escape_html(&param.name),
                if description.is_empty() {
                    String::new()
                } else {
                    format!(" {}", render_inline_html(&description))
                }
            )
            .unwrap();
        }
        blocks.push(format!("<ul class=\"ox-api-entry__member-params\">{params}</ul>"));
    }

    if let Some(returns) = &member.returns {
        if !returns.description.is_empty() {
            blocks.push(format!(
                "<div class=\"ox-api-entry__member-return\"><span>Returns</span> {}</div>",
                render_inline_html(&returns.description)
            ));
        }
    }

    blocks.join("")
}

fn render_member_table_html(title: &str, members: &[&ApiDocMember]) -> String {
    if members.is_empty() {
        return String::new();
    }

    let rows = members
        .iter()
        .map(|member| {
            format!(
                "<tr>
  <td><code>{}</code>{}</td>
  <td><span class=\"ox-api-entry__member-kind\">{}</span></td>
  <td>{}</td>
  <td>{}</td>
</tr>",
                escape_html(&member.name),
                render_member_flags(member),
                escape_html(&member.kind),
                render_member_type_html(member),
                render_member_description_html(member)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "<div class=\"ox-api-entry__member-group\">
<h5>{}</h5>
<table class=\"ox-api-entry__members-table\">
<thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
{rows}
</tbody>
</table>
</div>",
        escape_html(title)
    )
}

fn render_members_table_html(members: &[ApiDocMember], entry_kind: &str) -> String {
    if members.is_empty() {
        return String::new();
    }

    let constructors =
        members.iter().filter(|member| member.kind == "constructor").collect::<Vec<_>>();
    let static_methods = members
        .iter()
        .filter(|member| {
            member.r#static && matches!(member.kind.as_str(), "method" | "getter" | "setter")
        })
        .collect::<Vec<_>>();
    let methods = members
        .iter()
        .filter(|member| {
            !member.r#static && matches!(member.kind.as_str(), "method" | "getter" | "setter")
        })
        .collect::<Vec<_>>();
    let static_properties = members
        .iter()
        .filter(|member| member.r#static && member.kind == "property")
        .collect::<Vec<_>>();
    let properties = members
        .iter()
        .filter(|member| !member.r#static && member.kind == "property")
        .collect::<Vec<_>>();
    let enum_members =
        members.iter().filter(|member| member.kind == "enumMember").collect::<Vec<_>>();

    let mut groups = Vec::new();
    match entry_kind {
        "class" => {
            groups.push(render_member_table_html("Constructors", &constructors));
            groups.push(render_member_table_html("Static Methods", &static_methods));
            groups.push(render_member_table_html("Methods", &methods));
            groups.push(render_member_table_html("Static Properties", &static_properties));
            groups.push(render_member_table_html("Properties", &properties));
        }
        "interface" => {
            groups.push(render_member_table_html("Properties", &properties));
            groups.push(render_member_table_html("Methods", &methods));
        }
        "type" => {
            groups.push(render_member_table_html("Properties", &properties));
            groups.push(render_member_table_html("Methods", &methods));
            groups.push(render_member_table_html("Enum Members", &enum_members));
        }
        _ => groups.push(render_member_table_html("Members", &members.iter().collect::<Vec<_>>())),
    }

    let groups = groups.into_iter().filter(|group| !group.is_empty()).collect::<Vec<_>>();
    if groups.is_empty() {
        return String::new();
    }

    format!(
        "<div class=\"ox-api-entry__section ox-api-entry__section--members\">
<h4>Members</h4>
{}
</div>",
        groups.join("\n")
    )
}

fn generate_entry_markdown(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    current_file_name: Option<&str>,
    symbol_map: Option<&HashMap<String, SymbolLocation>>,
) -> String {
    let processed_description = match (current_file_name, symbol_map) {
        (Some(current_file_name), Some(symbol_map)) if !entry.description.is_empty() => {
            convert_symbol_links(&entry.description, current_file_name, symbol_map)
        }
        _ => entry.description.clone(),
    };
    let summary_signature = normalize_signature(entry.signature.as_deref());
    let source_href = options.github_url.as_ref().map(|github_url| {
        generate_source_href(&entry.file, github_url, Some(entry.line), Some(entry.end_line))
    });
    let mut body = String::new();

    if !processed_description.is_empty() {
        body.push_str(&render_markdown_blocks_html(&processed_description));
        body.push('\n');
    }

    if let Some(signature) = &entry.signature {
        body.push_str(&format!(
            "<div class=\"ox-api-entry__section ox-api-entry__section--signature\">
<h4>Signature</h4>
{}
</div>\n",
            render_code_block_html(signature, "typescript")
        ));
    }

    if let Some(source_href) = source_href {
        body.push_str(&format!(
            "<p class=\"ox-api-entry__source\"><a href=\"{}\">View source</a></p>\n",
            escape_html(&source_href)
        ));
    }

    if !entry.members.is_empty() {
        body.push_str(&render_members_table_html(&entry.members, &entry.kind));
        body.push('\n');
    }

    if !entry.params.is_empty() {
        body.push_str(&render_params_list_html(&entry.params));
        body.push('\n');
    }

    if let Some(returns) = &entry.returns {
        body.push_str(&format!(
            "<div class=\"ox-api-entry__section ox-api-entry__section--returns\">
<h4>Returns</h4>
<div class=\"ox-api-entry__return\">
  <code class=\"ox-api-entry__return-type\">{}</code>
  {}
</div>
</div>\n",
            escape_html(&returns.type_annotation),
            if returns.description.is_empty() {
                String::new()
            } else {
                format!(
                    "<p class=\"ox-api-entry__return-description\">{}</p>",
                    render_inline_html(&returns.description)
                )
            }
        ));
    }

    if !entry.examples.is_empty() {
        let examples_html = entry
            .examples
            .iter()
            .enumerate()
            .map(|(index, example)| {
                let (code, language) = parse_example_block(example);
                format!(
                    "<div class=\"ox-api-entry__example\">
<div class=\"ox-api-entry__example-heading\">Example {}</div>
{}
</div>",
                    index + 1,
                    render_code_block_html(&code, &language)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        body.push_str(&format!(
            "<div class=\"ox-api-entry__section ox-api-entry__section--examples\">
<h4>Examples</h4>
{examples_html}
</div>\n"
        ));
    }

    if !entry.tags.is_empty() {
        body.push_str(&render_tag_list_html(&entry.tags));
        body.push('\n');
    }

    let summary_description = clean_summary_text(
        &processed_description,
        if summary_signature.is_some() { 80 } else { 120 },
    );
    let summary_heading = if let Some(summary_signature) = summary_signature {
        render_highlighted_inline_code_html(
            &summary_signature,
            "ox-api-entry__signature ox-api-entry__signature--highlighted",
            "typescript",
        )
    } else {
        format!("<code class=\"ox-api-entry__name\">{}</code>", escape_html(&entry.name))
    };
    let summary_parts = [
        format!(
            "<span class=\"ox-api-entry__kind\">{}</span>",
            escape_html(format_kind_label(&entry.kind))
        ),
        format!(
            "<span class=\"ox-api-entry__summary-main\">{}{}{}</span>",
            summary_heading,
            if summary_description.is_empty() {
                String::new()
            } else {
                format!(
                    "<span class=\"ox-api-entry__description\">{}</span>",
                    render_inline_html(&summary_description)
                )
            },
            render_entry_badges_html(entry, "ox-api-entry__meta")
        ),
    ];

    format!(
        "<details id=\"{}\" class=\"ox-api-entry\">
  <summary>{}</summary>
  <div class=\"ox-api-entry__body\">
{}
  </div>
</details>

",
        entry_anchor(&entry.name),
        summary_parts.join(""),
        body.trim()
    )
}

fn generate_index(docs: &[ApiDocModule], doc_to_file: Option<&HashMap<String, String>>) -> String {
    let mut markdown = "# API Documentation\n\n".to_string();
    markdown.push_str("Generated by [Ox Content](https://github.com/ubugeeei/ox-content)\n\n");
    markdown.push_str(
        "> Use search scopes like `@api transform` to limit results to the generated API reference.\n\n",
    );
    markdown.push_str(&render_stats_html(
        &summarize_entries(docs.iter().flat_map(|doc| doc.entries.iter())),
        Some(docs.len()),
    ));
    markdown.push_str("\n\n");

    markdown.push_str("## Modules\n\n");
    if docs.len() > 1 {
        markdown.push_str(&render_details_controls_html(".ox-api-module"));
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

        let count_label = format!(
            "{} symbol{}",
            doc.entries.len(),
            if doc.entries.len() == 1 { "" } else { "s" }
        );
        markdown.push_str(&format!(
            "<details class=\"ox-api-module\">
  <summary>
    <span class=\"ox-api-module__title\"><a href=\"./{file_name}.md\">{}</a></span>
    <span class=\"ox-api-module__count\">{count_label}</span>
  </summary>
  <div class=\"ox-api-module__body\">
    <ul class=\"ox-api-module__list\">
",
            escape_html(&display_name)
        ));

        for entry in &doc.entries {
            markdown.push_str(&format!(
                "      {}\n",
                render_overview_html_item(
                    entry,
                    &format!("./{file_name}.md#{}", entry_anchor(&entry.name))
                )
            ));
        }

        markdown.push_str(
            "    </ul>
  </div>
</details>

",
        );
    }

    markdown
}

fn generate_category_markdown(
    kind: &str,
    entries: &[ApiDocEntry],
    options: &MarkdownDocsOptions,
    symbol_map: &HashMap<String, SymbolLocation>,
) -> String {
    let category_file_name = format!("{kind}s");
    let mut markdown = format!("# {}s\n\n", capitalize_ascii(kind));
    markdown.push_str(&format!(
        "> {} documented {kind}{} collected across modules.\n\n",
        entries.len(),
        if entries.len() == 1 { "" } else { "s" }
    ));
    markdown.push_str(&render_stats_html(&summarize_entries(entries), None));
    markdown.push_str("\n\n");

    markdown.push_str("## Overview\n\n");
    for entry in entries {
        markdown.push_str(&render_overview_line(entry, &format!("#{}", entry_anchor(&entry.name))));
    }
    markdown.push_str("\n## Reference\n\n");
    if entries.len() > 1 {
        markdown.push_str(&render_details_controls_html(".ox-api-entry"));
        markdown.push_str("\n\n");
    }

    for entry in entries {
        markdown.push_str(&generate_entry_markdown(
            entry,
            options,
            Some(&category_file_name),
            Some(symbol_map),
        ));
    }

    markdown
}

fn generate_category_index(by_kind: &BTreeMap<String, Vec<ApiDocEntry>>) -> String {
    let mut markdown = "# API Documentation\n\n".to_string();
    markdown.push_str("Generated by [Ox Content](https://github.com/ubugeeei/ox-content)\n\n");
    markdown.push_str(&render_stats_html(
        &summarize_entries(by_kind.values().flat_map(|entries| entries.iter())),
        None,
    ));
    markdown.push_str("\n\n");

    for (kind, entries) in by_kind {
        let kind_title = format!("{}s", capitalize_ascii(kind));
        markdown.push_str(&format!("## [{kind_title}](./{kind}s.md)\n\n"));
        markdown.push_str(&format!(
            "> {} item{}.\n\n",
            entries.len(),
            if entries.len() == 1 { "" } else { "s" }
        ));

        for entry in entries {
            markdown.push_str(&render_overview_line(
                entry,
                &format!("./{kind}s.md#{}", entry_anchor(&entry.name)),
            ));
        }
        markdown.push('\n');
    }

    markdown
}

fn convert_symbol_links(
    text: &str,
    current_file_name: &str,
    symbol_map: &HashMap<String, SymbolLocation>,
) -> String {
    static SYMBOL_RE: OnceLock<Regex> = OnceLock::new();

    let symbol_re = SYMBOL_RE.get_or_init(|| Regex::new(r"\[([A-Z_]\w*)\]").unwrap());
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
        let Some(location) = symbol_map.get(symbol_name) else {
            continue;
        };

        result.push_str(&text[last_index..mat.start()]);
        if location.file_name == current_file_name {
            result.push_str(&format!("[{symbol_name}](#{})", symbol_name.to_lowercase()));
        } else {
            result.push_str(&format!(
                "[{symbol_name}](./{}.md#{})",
                location.file_name,
                symbol_name.to_lowercase()
            ));
        }
        last_index = mat.end();
    }

    if last_index == 0 {
        return text.to_string();
    }

    result.push_str(&text[last_index..]);
    result
}

fn build_symbol_map(docs: &[ApiDocModule]) -> HashMap<String, SymbolLocation> {
    let mut map = HashMap::new();

    for doc in docs {
        let mut file_name = file_stem(&doc.file);
        if file_name == "index" {
            file_name = "index-module".to_string();
        }

        for entry in &doc.entries {
            map.insert(entry.name.clone(), SymbolLocation { file_name: file_name.clone() });
        }
    }

    map
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
