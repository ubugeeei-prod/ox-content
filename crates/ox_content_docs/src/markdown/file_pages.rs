use rustc_hash::FxHashMap;

use crate::model::ApiDocModule;
#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::StringBuilder;

use super::{
    doc_page_href, effective_index_format, entries::generate_entry_markdown,
    entries::render_overview_line, entries::render_overview_table_row, entry_anchor, file_name,
    file_stem, format_count_label, generate_source_link, markdown_html, push_generated_by,
    push_stats, summarize_entries, MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkContext,
    MarkdownRenderStyle, SymbolLocation,
};

pub(super) fn generate_file_markdown(
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

pub(super) fn generate_index(
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
