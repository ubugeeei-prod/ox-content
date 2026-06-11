use rustc_hash::FxHashMap;
use std::collections::BTreeMap;

use crate::model::{ApiDocEntry, ApiDocModule};
#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::join2;

use super::{
    markdown_html, markdown_pure, module_file_name,
    owners::CanonicalOwners,
    push_typedoc_entry_page_title, typedoc_entry_file_name, typedoc_entry_page_title_len,
    typedoc_index::{
        generate_typedoc_module_index, generate_typedoc_module_index_for_file,
        generate_typedoc_root_index, TypedocModuleIndexPage,
    },
    typedoc_module_index_file_name, MarkdownDocsOptions, MarkdownLinkContext, MarkdownRenderStyle,
    MarkdownSingleEntryRoot, SymbolLocation,
};

pub(super) fn generate_typedoc_markdown(
    docs: &[ApiDocModule],
    options: &MarkdownDocsOptions,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
) -> BTreeMap<String, String> {
    profile_span!("docs::render_typedoc");
    let mut result = BTreeMap::new();
    let owners = CanonicalOwners::compute(docs);
    let flatten_single_entry =
        options.single_entry_root == MarkdownSingleEntryRoot::Flatten && docs.len() == 1;

    if flatten_single_entry {
        if let Some(doc) = docs.first() {
            let module_name = module_file_name(&doc.file);
            result.insert(
                "index.md".to_string(),
                generate_typedoc_module_index_for_file(
                    doc,
                    options,
                    &module_name,
                    TypedocModuleIndexPage {
                        current_file_name: "index",
                        title: "API Documentation",
                        include_generated_by: true,
                    },
                    symbol_map,
                    &owners,
                ),
            );
        }
    } else {
        result
            .insert("index.md".to_string(), generate_typedoc_root_index(docs, options, symbol_map));
    }

    for doc in docs {
        let module_name = module_file_name(&doc.file);
        if !flatten_single_entry {
            let module_index_file_name = typedoc_module_index_file_name(&module_name);
            result.insert(
                join2(&module_index_file_name, ".md"),
                generate_typedoc_module_index(doc, options, &module_name, symbol_map, &owners),
            );
        }

        // Both render styles group a symbol's overload signatures onto a single
        // page so every public call signature survives (TypeDoc parity); see
        // `generate_typedoc_entry_page_grouped`.
        for (entry_file_name, entries) in typedoc_canonical_groups(doc, &owners, &module_name) {
            result.insert(
                join2(&entry_file_name, ".md"),
                generate_typedoc_entry_page_grouped(
                    &entries,
                    options,
                    &module_name,
                    &entry_file_name,
                    symbol_map,
                ),
            );
        }
    }

    result
}

/// Groups a module's canonical entries by their TypeDoc page file name, preserving
/// first-seen (source) order. Entries that map to the same page — a symbol's
/// overload signatures plus its implementation — are collected together so the
/// renderer can emit one page per symbol instead of overwriting it per entry.
fn typedoc_canonical_groups<'a>(
    doc: &'a ApiDocModule,
    owners: &CanonicalOwners,
    module_name: &str,
) -> Vec<(String, Vec<&'a ApiDocEntry>)> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: FxHashMap<String, Vec<&ApiDocEntry>> = FxHashMap::default();
    for entry in &doc.entries {
        if !owners.is_canonical(doc, entry) {
            continue;
        }
        let file_name = typedoc_entry_file_name(module_name, entry);
        if !groups.contains_key(&file_name) {
            order.push(file_name.clone());
        }
        groups.entry(file_name).or_default().push(entry);
    }
    order
        .into_iter()
        .map(|file_name| {
            let entries = groups.remove(&file_name).unwrap_or_default();
            (file_name, entries)
        })
        .collect()
}

/// Renders one TypeDoc symbol page for a group of entries that share a page.
///
/// A single entry (the common case) renders exactly as before. When a symbol has
/// overloads, the implementation signature (the body-carrying entry) is hidden and
/// each public overload is rendered as a `## Call Signature` section; a lone public
/// overload collapses back to the normal single-signature page.
fn generate_typedoc_entry_page_grouped(
    entries: &[&ApiDocEntry],
    options: &MarkdownDocsOptions,
    module_name: &str,
    file_name: &str,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
) -> String {
    profile_span!("docs::render_entry_page");
    if entries.len() == 1 {
        return generate_typedoc_entry_page(
            entries[0],
            options,
            module_name,
            file_name,
            symbol_map,
        );
    }

    // Overload signatures have no body; the implementation does. Hide the
    // implementation and render the public signatures, matching TypeDoc.
    let implementation = entries.iter().copied().rfind(|entry| entry.has_body);
    let public: Vec<&ApiDocEntry> = {
        let signatures =
            entries.iter().copied().filter(|entry| !entry.has_body).collect::<Vec<_>>();
        if signatures.is_empty() {
            entries.to_vec()
        } else {
            signatures
        }
    };

    // A single public signature is just a normal symbol page (implementation
    // omitted) — no `## Call Signature` wrapper, like TypeDoc.
    if public.len() == 1 {
        return generate_typedoc_entry_page(public[0], options, module_name, file_name, symbol_map);
    }

    let link_context = MarkdownLinkContext {
        options,
        current_file_name: file_name,
        current_module_name: module_name,
        symbol_map,
    };
    // The H1 title is name + kind only (functions render `Function: name()` with no
    // generics), so any overload yields the same title. The body is rendered in the
    // configured style; both keep all public signatures and hide the implementation.
    let body = if options.render_style == MarkdownRenderStyle::Markdown {
        markdown_pure::render_overload_body_pure(
            &public,
            implementation,
            options,
            Some(&link_context),
            2,
        )
    } else {
        markdown_html::render_overload_body_html(
            &public,
            implementation,
            options,
            Some(&link_context),
        )
    };
    let mut markdown =
        String::with_capacity(typedoc_entry_page_title_len(public[0]) + 4 + body.len() + 1);
    markdown.push_str("# ");
    push_typedoc_entry_page_title(&mut markdown, public[0]);
    markdown.push_str("\n\n");
    if !body.is_empty() {
        markdown.push_str(&body);
        if options.render_style == MarkdownRenderStyle::Markdown {
            markdown.push('\n');
        }
    }
    markdown
}

fn generate_typedoc_entry_page(
    entry: &ApiDocEntry,
    options: &MarkdownDocsOptions,
    module_name: &str,
    current_file_name: &str,
    symbol_map: &FxHashMap<String, Vec<SymbolLocation>>,
) -> String {
    let link_context = MarkdownLinkContext {
        options,
        current_file_name,
        current_module_name: module_name,
        symbol_map,
    };
    // TypeDoc-style H1 includes the declaration kind (and generics / `()`),
    // e.g. `# Function: cli()`, `# Interface: Command<G>`.
    let title_len = typedoc_entry_page_title_len(entry);
    let body = if options.render_style == MarkdownRenderStyle::Markdown {
        // Per-symbol page: title is `# {title}` (H1), so sections render at H2.
        markdown_pure::render_entry_body_pure(entry, options, Some(&link_context), 2)
    } else {
        markdown_html::render_entry_page_html(entry, options, Some(&link_context))
    };
    let mut markdown = String::with_capacity(title_len + 4 + body.len() + 1);
    markdown.push_str("# ");
    push_typedoc_entry_page_title(&mut markdown, entry);
    markdown.push_str("\n\n");
    if !body.is_empty() {
        markdown.push_str(&body);
        if options.render_style == MarkdownRenderStyle::Markdown {
            markdown.push('\n');
        }
    }
    markdown
}
