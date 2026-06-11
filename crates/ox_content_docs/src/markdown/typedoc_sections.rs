use rustc_hash::FxHashSet;

use crate::model::ApiDocEntry;

use super::{
    clean_summary_text, doc_page_href_from, process_doc_text, typedoc_entry_file_name,
    typedoc_index_summary, typedoc_kind_singular, typedoc_kind_title, MarkdownDisplayFormat,
    MarkdownLinkContext,
};

/// A renderable section of a TypeDoc module index, kept title-tagged so
/// `group_order` can reorder kinds and References together.
pub(super) enum IndexSection<'a> {
    Kind { kind: String, entries: Vec<&'a ApiDocEntry> },
    References(Vec<(&'a ApiDocEntry, String)>),
}

/// Renders one `## {kind title}` section (table or list) for a module index.
pub(super) fn render_typedoc_kind_section(
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
    let mut seen = FxHashSet::default();
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
pub(super) fn render_typedoc_references_section(
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
