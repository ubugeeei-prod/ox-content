use rustc_hash::{FxHashMap, FxHashSet};

use crate::model::ApiDocModule;
#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::StringBuilder;

use super::{
    clean_summary_text, doc_page_href_from, effective_index_format, generate_source_link,
    kind_order_slice, markdown_html, markdown_pure, module_display_name, module_route_name,
    order_by_group_title, ordered_entry_kinds,
    owners::CanonicalOwners,
    process_doc_text, push_generated_by, push_stats, render_module_examples_markdown,
    summarize_docs, summarize_module, typedoc_index_summary, typedoc_kind_title,
    typedoc_module_index_file_name,
    typedoc_sections::{
        render_typedoc_kind_section, render_typedoc_references_section, IndexSection,
    },
    MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkContext, MarkdownRenderStyle,
    SymbolLocation,
};

pub(super) fn generate_typedoc_root_index(
    docs: &[ApiDocModule],
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

pub(super) fn generate_typedoc_module_index(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    module_name: &str,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
    owners: &CanonicalOwners,
) -> String {
    let current_file_name = typedoc_module_index_file_name(module_name);
    let display_name = module_display_name(doc);
    generate_typedoc_module_index_for_file(
        doc,
        options,
        module_name,
        TypedocModuleIndexPage {
            current_file_name: &current_file_name,
            title: &display_name,
            include_generated_by: false,
        },
        symbol_map,
        owners,
    )
}

pub(super) struct TypedocModuleIndexPage<'a> {
    pub(super) current_file_name: &'a str,
    pub(super) title: &'a str,
    pub(super) include_generated_by: bool,
}

pub(super) fn generate_typedoc_module_index_for_file(
    doc: &ApiDocModule,
    options: &MarkdownDocsOptions,
    module_name: &str,
    page: TypedocModuleIndexPage<'_>,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
    owners: &CanonicalOwners,
) -> String {
    profile_span!("docs::render_module_index");
    let link_context = MarkdownLinkContext {
        options,
        current_file_name: page.current_file_name,
        current_module_name: module_name,
        symbol_map,
    };
    let mut builder = StringBuilder::with_capacity(page.title.len() + 4);
    builder.push_str("# ");
    builder.push_str(page.title);
    builder.push_str("\n\n");
    let mut markdown = builder.into_string();
    if page.include_generated_by {
        push_generated_by(&mut markdown, options);
    }

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
        // Link to the entry point's real source file. `doc.file` is the module
        // route name (e.g. `default`) under the TypeDoc strategy, which would
        // produce a dead link; `doc.source_path` holds the actual entry-point
        // path. Omit the link when no source path is available (matching the
        // per-symbol pages and TypeDoc, which has no module source line).
        if !doc.source_path.is_empty() {
            markdown.push_str(&generate_source_link(&doc.source_path, github_url, None, None));
            markdown.push_str("\n\n");
        }
    }

    push_stats(&mut markdown, options, &summarize_module(doc), None);

    let index_format = effective_index_format(options);

    // Collect the kind sections (in the historical order) plus the References
    // section, then order them by `group_order` before rendering. TypeDoc treats
    // References as just another group, so it participates in the ordering too.
    let mut sections: Vec<(String, IndexSection)> = Vec::new();
    let kind_order = kind_order_slice(options.kind_sort_order.as_deref());
    for kind in ordered_entry_kinds(&doc.entries, &kind_order) {
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
    let mut seen_references = FxHashSet::default();
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
