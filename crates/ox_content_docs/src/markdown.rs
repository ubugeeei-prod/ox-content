//! Markdown rendering for generated API reference documentation.

use rustc_hash::FxHashMap;
// BTreeMap keeps generated API section and tag output deterministic.
use std::collections::BTreeMap;

use crate::model::{ApiDocEntry, ApiDocModule};
#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::{join2, join3};

mod category_pages;
mod entries;
mod examples;
mod file_pages;
mod group_order;
mod implementation;
mod labels;
mod links;
mod markdown_html;
mod markdown_pure;
mod options;
mod owners;
mod paths;
mod regex_cache;
mod sort;
mod stats;
mod summary;
mod symbol_map;
mod tags;
mod type_links;
mod typedoc;
mod typedoc_index;
mod typedoc_pages;
mod typedoc_sections;

use category_pages::{generate_category_index, generate_category_markdown};
use examples::{parse_example_block, render_module_examples_markdown, ExampleBlock};
use file_pages::{generate_file_markdown, generate_index};
pub use group_order::{order_by_group_title, ordered_entry_kinds};
use implementation::annotate_implementation_relationships;
use labels::{format_count_label, format_kind_label, normalize_signature};
use links::{process_doc_text, MarkdownLinkContext, SymbolLocation};
pub use options::{
    MarkdownDisplayFormat, MarkdownDocsOptions, MarkdownLinkStyle, MarkdownPathStrategy,
    MarkdownRenderStyle, MarkdownSingleEntryRoot, DOC_KIND_ORDER,
};
use paths::{
    capitalize_ascii, doc_page_href, doc_page_href_from, entry_anchor, file_name, file_stem,
    generate_source_href, generate_source_link, member_anchor, module_display_name,
    module_file_name, module_route_name,
};
use regex_cache::{cached_regex, RegexCache};
use sort::sort_extracted_docs;
#[allow(unused_imports)]
pub use sort::SortStrategy;
pub use sort::{compare_entries, kind_order_slice, parse_sort_strategies};
use stats::{
    doc_kind_plural, effective_index_format, effective_members_format, effective_parameters_format,
    member_table_includes_kind, push_generated_by, push_stats, summarize_docs, summarize_entries,
    summarize_module, EntryStats,
};
use summary::{
    clean_summary_text, collapse_inline_whitespace, collapse_type_annotation_whitespace,
    markdown_index_summary, typedoc_index_summary,
};
use symbol_map::build_symbol_map;
use tags::{get_entry_badges, is_structured_tag, is_throws_tag, rendered_throws, SINCE_TAGS};
use type_links::{resolve_type_fragments, TypeFragment};
use typedoc::{
    anchor_href, plural_kind_file_name, plural_kind_title, push_typedoc_entry_page_title,
    sanitize_doc_path_segment, typedoc_entry_file_name, typedoc_entry_page_title_len,
    typedoc_kind_singular, typedoc_kind_title, typedoc_module_index_file_name,
};
use typedoc_pages::generate_typedoc_markdown;

use entries::member_symbol_name;
pub use owners::CanonicalOwners;

/// Generates Markdown documentation pages from extracted API docs.
#[must_use]
pub fn generate_markdown(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
) -> BTreeMap<String, String> {
    profile_span!("docs::generate_markdown");
    let mut result = BTreeMap::new();
    let mut sorted_docs = sort_extracted_docs(docs, options);
    annotate_implementation_relationships(&mut sorted_docs);
    let symbol_map = build_symbol_map(&sorted_docs, options);

    if options.group_by == "file" {
        if options.path_strategy == MarkdownPathStrategy::TypeDoc {
            return generate_typedoc_markdown(&sorted_docs, options, &symbol_map);
        }

        let mut doc_to_file = FxHashMap::default();

        for doc in &sorted_docs {
            let file_name = module_file_name(&doc.file);
            doc_to_file.insert(doc.file.clone(), file_name.clone());

            let markdown = generate_file_markdown(doc, options, &file_name, &symbol_map);
            result.insert(join2(&file_name, ".md"), markdown);
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

        let strategies = options.sort.as_deref().map(parse_sort_strategies);
        let kind_order = kind_order_slice(options.kind_sort_order.as_deref());
        for entries in by_kind.values_mut() {
            if let Some(strategies) = &strategies {
                entries.sort_by(|a, b| compare_entries(a, b, strategies, &kind_order));
            } else {
                // Case-insensitive sort with a case-sensitive tiebreak. Caching the
                // (lowercase, original) key computes each side's lowercase form once
                // per entry instead of on every comparison (O(n) vs O(n log n)
                // allocations); the tuple's lexicographic order reproduces the
                // previous "lowercase, then original" ordering exactly.
                entries.sort_by_cached_key(|entry| (entry.name.to_lowercase(), entry.name.clone()));
            }
        }

        for (kind, entries) in &by_kind {
            result.insert(
                join3(kind, "s", ".md"),
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

#[cfg(test)]
mod tests;
