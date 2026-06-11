use rustc_hash::FxHashMap;

use crate::model::ApiDocEntry;
use crate::string_builder::StringBuilder;

use super::{
    clean_summary_text, markdown_html, markdown_index_summary, markdown_pure, normalize_signature,
    process_doc_text, MarkdownDocsOptions, MarkdownLinkContext, MarkdownRenderStyle,
    SymbolLocation,
};

pub(super) fn member_symbol_name(entry_name: &str, member_name: &str) -> String {
    let mut symbol_name = StringBuilder::with_capacity(entry_name.len() + member_name.len() + 1);
    symbol_name.push_str(entry_name);
    symbol_name.push_char('.');
    symbol_name.push_str(member_name);
    symbol_name.into_string()
}

pub(super) fn render_overview_line(
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

pub(super) fn render_overview_table_row(
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

pub(super) fn generate_entry_markdown(
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
