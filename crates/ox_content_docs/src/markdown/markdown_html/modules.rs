use rustc_hash::FxHashMap;

use super::super::{
    clean_summary_text, doc_page_href, entry_anchor, file_stem, format_count_label,
    process_doc_text, MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkContext,
};
use super::inline::{escape_html, render_inline_html};
use super::overview::render_overview_html_item;
use crate::model::{ApiDocModule, ApiDocTag};
use crate::string_builder::{join3, StringBuilder};

/// Renders module-level lifecycle tags (`@deprecated` / `@experimental`) as a badge
/// row for the HTML module index (the markdown renderer uses GitHub alerts here).
/// Returns "" when no lifecycle tags are present.
pub(in crate::markdown) fn render_module_lifecycle_badges_html(tags: &[ApiDocTag]) -> String {
    let mut markers = String::new();
    if tags.iter().any(|tag| tag.tag == "deprecated") {
        markers.push_str("<span class=\"ox-api-badge ox-api-badge--warning\">deprecated</span>");
    }
    if tags.iter().any(|tag| tag.tag == "experimental") {
        markers.push_str("<span class=\"ox-api-badge ox-api-badge--warning\">experimental</span>");
    }
    if markers.is_empty() {
        return String::new();
    }
    join3("<p class=\"ox-api-module__meta\">", &markers, "</p>\n\n")
}

pub(in crate::markdown) fn render_module_section_html(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    file_name: &str,
    display_name: &str,
    count_label: &str,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let mut markdown = String::new();
    markdown.push_str(
        "<details class=\"ox-api-module\">
  <summary>
    <span class=\"ox-api-module__title\"><a href=\"",
    );
    markdown.push_str(&escape_html(&doc_page_href(options, file_name, None)));
    markdown.push_str("\">");
    markdown.push_str(&escape_html(display_name));
    markdown.push_str(
        "</a></span>
    <span class=\"ox-api-module__count\">",
    );
    markdown.push_str(count_label);
    markdown.push_str(
        "</span>
  </summary>
  <div class=\"ox-api-module__body\">
    <ul class=\"ox-api-module__list\">
",
    );

    for entry in &doc.entries {
        let href = doc_page_href(options, file_name, Some(&entry_anchor(&entry.name)));
        markdown.push_str("      ");
        markdown.push_str(&render_overview_html_item(entry, &href, link_context));
        markdown.push('\n');
    }

    markdown.push_str(
        "    </ul>
  </div>
</details>

",
    );

    markdown
}

pub(in crate::markdown) fn render_module_index_html(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
    doc_to_file: Option<&FxHashMap<String, String>>,
    display_format: MarkdownDisplayFormat,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let items = docs
        .iter()
        .map(|doc| {
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
            let href = doc_page_href(options, &file_name, None);
            let summary = clean_summary_text(&process_doc_text(&doc.description, link_context), 88);
            (display_name, href, count_label, summary)
        })
        .collect::<Vec<_>>();

    if display_format == MarkdownDisplayFormat::Table {
        let mut rows = StringBuilder::new();
        for (display_name, href, count_label, summary) in &items {
            if !rows.is_empty() {
                rows.push_char('\n');
            }
            rows.push_str("<tr><td><a href=\"");
            rows.push_str(&escape_html(href));
            rows.push_str("\">");
            rows.push_str(&escape_html(display_name));
            rows.push_str("</a></td><td>");
            rows.push_str(&escape_html(count_label));
            rows.push_str("</td><td>");
            rows.push_str(&render_inline_html(summary));
            rows.push_str("</td></tr>");
        }
        let rows = rows.into_string();

        let mut out = StringBuilder::with_capacity(rows.len() + 150);
        out.push_str(
            "<table class=\"ox-api-modules-table\">
<thead><tr><th>Module</th><th>Symbols</th><th>Description</th></tr></thead>
<tbody>
",
        );
        out.push_str(&rows);
        out.push_str(
            "
</tbody>
</table>

",
        );
        return out.into_string();
    }

    let mut rows = StringBuilder::new();
    for (display_name, href, count_label, summary) in &items {
        if !rows.is_empty() {
            rows.push_char('\n');
        }
        rows.push_str("<li><a href=\"");
        rows.push_str(&escape_html(href));
        rows.push_str("\">");
        rows.push_str(&escape_html(display_name));
        rows.push_str("</a><span class=\"ox-api-module__count\">");
        rows.push_str(&escape_html(count_label));
        rows.push_str("</span>");
        if !summary.is_empty() {
            rows.push_str("<span class=\"ox-api-module__summary\">");
            rows.push_str(&render_inline_html(summary));
            rows.push_str("</span>");
        }
        rows.push_str("</li>");
    }
    let rows = rows.into_string();

    let mut out = StringBuilder::with_capacity(rows.len() + 40);
    out.push_str(
        "<ul class=\"ox-api-modules-list\">
",
    );
    out.push_str(&rows);
    out.push_str(
        "
</ul>

",
    );
    out.into_string()
}
