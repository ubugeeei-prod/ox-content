use rustc_hash::FxHashMap;
use std::collections::BTreeMap;

use crate::model::ApiDocEntry;
use crate::string_builder::StringBuilder;

use super::{
    anchor_href, doc_page_href, effective_index_format, entries::generate_entry_markdown,
    entries::render_overview_line, entries::render_overview_table_row, entry_anchor, markdown_html,
    plural_kind_file_name, plural_kind_title, push_generated_by, push_stats, summarize_entries,
    MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkContext, MarkdownRenderStyle,
    SymbolLocation,
};

pub(super) fn generate_category_markdown(
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

pub(super) fn generate_category_index(
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
