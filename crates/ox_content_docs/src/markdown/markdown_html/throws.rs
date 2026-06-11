use rustc_hash::FxHashSet;

use super::super::MarkdownLinkContext;
use super::inline::{render_doc_inline_html, render_type_inner_html};
use crate::model::ApiThrowsDoc;
use crate::string_builder::StringBuilder;

pub(super) fn render_throws_list_html(
    throws: &[ApiThrowsDoc],
    link_context: Option<&MarkdownLinkContext<'_>>,
    class_name: &str,
) -> String {
    let mut items = StringBuilder::new();
    for throws_doc in throws {
        let item = render_throws_item_html(throws_doc, link_context);
        if item.is_empty() {
            continue;
        }
        items.push_str("<li>");
        items.push_str(&item);
        items.push_str("</li>");
    }
    if items.is_empty() {
        return String::new();
    }
    let mut out = StringBuilder::new();
    out.push_str("<ul class=\"");
    out.push_str(class_name);
    out.push_str("\">");
    out.push_str(&items.into_string());
    out.push_str("</ul>");
    out.into_string()
}

pub(super) fn render_throws_inline_html(
    throws: &[ApiThrowsDoc],
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    throws
        .iter()
        .map(|throws_doc| render_throws_item_html(throws_doc, link_context))
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

fn render_throws_item_html(
    throws_doc: &ApiThrowsDoc,
    link_context: Option<&MarkdownLinkContext<'_>>,
) -> String {
    let type_annotation =
        throws_doc.type_annotation.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let description = throws_doc.description.trim();
    if type_annotation.is_none() && description.is_empty() {
        return String::new();
    }

    let mut out = StringBuilder::new();
    if let Some(type_annotation) = type_annotation {
        out.push_str("<code class=\"ox-api-entry__throws-type\">");
        out.push_str(&render_type_inner_html(type_annotation, link_context, &FxHashSet::default()));
        out.push_str("</code>");
        if !description.is_empty() {
            out.push_str(" <span class=\"ox-api-entry__throws-description\">");
            out.push_str(&render_doc_inline_html(description, link_context));
            out.push_str("</span>");
        }
    } else {
        out.push_str("<span class=\"ox-api-entry__throws-description\">");
        out.push_str(&render_doc_inline_html(description, link_context));
        out.push_str("</span>");
    }
    out.into_string()
}
