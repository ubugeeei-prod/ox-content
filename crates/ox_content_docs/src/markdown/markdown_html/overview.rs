use super::super::{
    clean_summary_text, format_kind_label, get_entry_badges, normalize_signature, process_doc_text,
    MarkdownLinkContext,
};
use super::inline::{escape_html, render_highlighted_inline_code_html, render_inline_html};
use crate::model::ApiDocEntry;
use crate::string_builder::{join3, StringBuilder};

pub(super) fn render_entry_badges_html(entry: &ApiDocEntry, class_name: &str) -> String {
    let badges = get_entry_badges(entry);
    if badges.is_empty() {
        return String::new();
    }

    let mut rendered = StringBuilder::new();
    for badge in badges {
        rendered.push_str("<span class=\"ox-api-badge");
        if let Some(tone) = badge.tone {
            rendered.push_str(" ox-api-badge--");
            rendered.push_str(tone);
        }
        rendered.push_str("\">");
        rendered.push_str(&escape_html(&badge.label));
        rendered.push_str("</span>");
    }

    let rendered = rendered.into_string();
    let mut out = StringBuilder::with_capacity(class_name.len() + rendered.len() + 23);
    out.push_str("<span class=\"");
    out.push_str(class_name);
    out.push_str("\">");
    out.push_str(&rendered);
    out.push_str("</span>");
    out.into_string()
}

pub(super) fn render_overview_html_item(
    entry: &ApiDocEntry,
    href: &str,
    context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let signature = normalize_signature(entry.signature.as_deref());
    let summary = clean_summary_text(&process_doc_text(&entry.description, context), 88);
    let meta = render_entry_badges_html(entry, "ox-api-module__meta");
    let href = escape_html(href);
    let heading = if let Some(signature) = signature {
        let content = render_highlighted_inline_code_html(
            &signature,
            "ox-api-module__signature ox-api-module__signature--highlighted",
            "typescript",
        );
        let mut heading = StringBuilder::with_capacity(href.len() + content.len() + 43);
        heading.push_str("<a href=\"");
        heading.push_str(&href);
        heading.push_str("\" class=\"ox-api-module__link\">");
        heading.push_str(&content);
        heading.push_str("</a>");
        heading.into_string()
    } else {
        let name = escape_html(&entry.name);
        let mut heading = StringBuilder::with_capacity(href.len() + name.len() + 80);
        heading.push_str("<a href=\"");
        heading.push_str(&href);
        heading.push_str("\" class=\"ox-api-module__link\"><code class=\"ox-api-module__name\">");
        heading.push_str(&name);
        heading.push_str("</code></a>");
        heading.into_string()
    };

    let summary_html = if summary.is_empty() {
        String::new()
    } else {
        join3("<span class=\"ox-api-module__summary\">", &render_inline_html(&summary), "</span>")
    };
    let kind = escape_html(format_kind_label(&entry.kind));
    let mut item = StringBuilder::with_capacity(
        kind.len() + heading.len() + summary_html.len() + meta.len() + 92,
    );
    item.push_str("<li><span class=\"ox-api-module__kind\">");
    item.push_str(&kind);
    item.push_str("</span><div class=\"ox-api-module__item\">");
    item.push_str(&heading);
    item.push_str(&summary_html);
    item.push_str(&meta);
    item.push_str("</div></li>");
    item.into_string()
}
