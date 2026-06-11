use phf::phf_map;

use super::paths::{
    nav_route_path, normalize_base_path, sanitize_doc_path_segment, typedoc_module_display_name,
    typedoc_module_route_name,
};
use super::{DocsNavItem, DEFAULT_BASE_PATH};
use crate::markdown::{
    compare_entries, kind_order_slice, order_by_group_title, ordered_entry_kinds,
    parse_sort_strategies, CanonicalOwners,
};
use crate::model::ApiDocModule;
use crate::string_builder::{join3, join5};

/// Directory segment for each documentation kind under the TypeDoc path strategy.
static TYPEDOC_KIND_SEGMENT: phf::Map<&'static str, &'static str> = phf_map! {
    "function" => "functions",
    "class" => "classes",
    "interface" => "interfaces",
    "type" => "type-aliases",
    "enum" => "enumerations",
    "variable" => "variables",
    "const" => "variables",
    "module" => "modules",
};

/// Plural category heading for each documentation kind.
static TYPEDOC_KIND_TITLE: phf::Map<&'static str, &'static str> = phf_map! {
    "function" => "Functions",
    "class" => "Classes",
    "interface" => "Interfaces",
    "type" => "Type Aliases",
    "enum" => "Enumerations",
    "variable" => "Variables",
    "const" => "Variables",
    "module" => "Modules",
};

pub(super) fn flatten_single_entry_typedoc_nav(mut nav: Vec<DocsNavItem>) -> Vec<DocsNavItem> {
    if nav.len() != 1 {
        return nav;
    }

    let children = nav[0].children.take();
    if let Some(children) = children.filter(|children| !children.is_empty()) {
        children
    } else {
        nav
    }
}

pub(super) fn generate_typedoc_nav_metadata(
    docs: &[ApiDocModule],
    base_path: Option<&str>,
    group_order: Option<&[String]>,
    sort: Option<&[String]>,
    sort_entry_points: bool,
    kind_sort_order: Option<&[String]>,
) -> Vec<DocsNavItem> {
    let base_path = normalize_base_path(base_path.unwrap_or(DEFAULT_BASE_PATH));
    let strategies = sort.map(parse_sort_strategies);
    let kind_order = kind_order_slice(kind_sort_order);
    let mut docs = docs.to_vec();
    // `sortEntryPoints: false` preserves the caller-provided module order.
    if sort_entry_points {
        docs.sort_by_cached_key(typedoc_module_route_name);
    }
    // A re-exported symbol appears in the sidebar only under the module that owns
    // its canonical page (matching TypeDoc's single-location listing).
    let owners = CanonicalOwners::compute(&docs);

    docs.into_iter()
        .map(|doc| {
            let module_name = typedoc_module_route_name(&doc);
            let module_title = typedoc_module_display_name(&doc);
            let mut children = Vec::new();
            // Collect the present kind groups (title-tagged) in the base kind order
            // (`kindSortOrder` or the historical order), then reorder them by
            // `group_order` so the sidebar matches the module index section order.
            let kind_groups = ordered_entry_kinds(&doc.entries, &kind_order)
                .into_iter()
                .filter(|kind| {
                    doc.entries
                        .iter()
                        .any(|entry| entry.kind == *kind && owners.is_canonical(&doc, entry))
                })
                .map(|kind| (typedoc_kind_title(&kind).to_string(), kind))
                .collect::<Vec<_>>();
            for (_title, kind) in order_by_group_title(kind_groups, group_order) {
                let mut entries = doc
                    .entries
                    .iter()
                    .filter(|entry| entry.kind == kind && owners.is_canonical(&doc, entry))
                    .collect::<Vec<_>>();
                if entries.is_empty() {
                    continue;
                }
                // Sort leaf entries to match ox-content's generated Markdown module
                // index: by the configured `sort` strategies when set, otherwise
                // alphabetically (case-insensitive).
                if let Some(strategies) = &strategies {
                    entries.sort_by(|a, b| compare_entries(a, b, strategies, &kind_order));
                } else {
                    entries.sort_by_cached_key(|entry| {
                        (entry.name.to_lowercase(), entry.name.clone())
                    });
                }

                let kind_segment = typedoc_kind_segment(&kind);
                let kind_file_name = join3(&module_name, "/", kind_segment);
                let kind_path = nav_route_path(&base_path, &kind_file_name);
                // Overloads share a name and resolve to one typedoc page, so collapse
                // them to a single sidebar leaf (matching the module index).
                let mut seen = rustc_hash::FxHashSet::default();
                let entry_children = entries
                    .into_iter()
                    .filter(|entry| seen.insert(entry.name.as_str()))
                    .map(|entry| {
                        let entry_file_name = join5(
                            &module_name,
                            "/",
                            kind_segment,
                            "/",
                            &sanitize_doc_path_segment(&entry.name),
                        );
                        DocsNavItem {
                            title: entry.name.clone(),
                            path: nav_route_path(&base_path, &entry_file_name),
                            children: None,
                        }
                    })
                    .collect::<Vec<_>>();

                children.push(DocsNavItem {
                    title: typedoc_kind_title(&kind).to_string(),
                    path: kind_path,
                    children: Some(entry_children),
                });
            }

            DocsNavItem {
                title: module_title,
                path: nav_route_path(&base_path, &join3(&module_name, "/", "index")),
                children: if children.is_empty() { None } else { Some(children) },
            }
        })
        .collect()
}

fn typedoc_kind_segment(kind: &str) -> &'static str {
    TYPEDOC_KIND_SEGMENT.get(kind).copied().unwrap_or("symbols")
}

fn typedoc_kind_title(kind: &str) -> &'static str {
    TYPEDOC_KIND_TITLE.get(kind).copied().unwrap_or("Symbols")
}
