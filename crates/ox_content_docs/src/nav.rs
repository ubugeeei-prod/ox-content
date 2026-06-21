//! Navigation metadata generation for API documentation.

use serde::{Deserialize, Serialize};

mod paths;
mod typedoc;

use paths::{get_doc_display_name, get_doc_file_name, normalize_base_path};

use crate::markdown::{MarkdownPathStrategy, MarkdownSingleEntryRoot};
use crate::model::ApiDocModule;
#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::{join3, StringBuilder};

const DEFAULT_BASE_PATH: &str = "/api";
const DEFAULT_EXPORT_NAME: &str = "apiNav";

/// Navigation item for generated documentation sidebars.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocsNavItem {
    /// Display title for the navigation item.
    pub title: String,
    /// Path to the documentation page.
    pub path: String,
    /// Child navigation items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DocsNavItem>>,
}

/// Options for generated navigation metadata from extracted docs.
#[derive(Debug, Clone, Copy)]
pub struct DocsNavMetadataOptions<'a> {
    /// Route prefix for generated navigation links.
    pub base_path: Option<&'a str>,
    /// Output path strategy the nav should mirror.
    pub path_strategy: MarkdownPathStrategy,
    /// TypeDoc-style group order for nav groups.
    pub group_order: Option<&'a [String]>,
    /// TypeDoc-style sort strategies for nav leaves.
    pub sort: Option<&'a [String]>,
    /// Whether to sort entry points alphabetically.
    pub sort_entry_points: bool,
    /// TypeDoc-style kind ranking for nav groups.
    pub kind_sort_order: Option<&'a [String]>,
    /// Single-entry root handling for TypeDoc-style nav.
    pub single_entry_root: MarkdownSingleEntryRoot,
}

impl Default for DocsNavMetadataOptions<'_> {
    fn default() -> Self {
        Self {
            base_path: None,
            path_strategy: MarkdownPathStrategy::Flat,
            group_order: None,
            sort: None,
            sort_entry_points: true,
            kind_sort_order: None,
            single_entry_root: MarkdownSingleEntryRoot::Preserve,
        }
    }
}

/// Generates sidebar navigation metadata from documentation file paths.
pub fn generate_nav_metadata(files: &[String], base_path: Option<&str>) -> Vec<DocsNavItem> {
    let base_path = normalize_base_path(base_path.unwrap_or(DEFAULT_BASE_PATH));
    let mut sorted_files = files.to_vec();

    // `sort_by_cached_key` derives each display name once instead of
    // recomputing it (a `file_stem` + title-format allocation) on every
    // comparison; the ordering is identical.
    sorted_files.sort_by_cached_key(|file| get_doc_display_name(file));

    sorted_files
        .into_iter()
        .map(|file| {
            let file_name = get_doc_file_name(&file);
            DocsNavItem {
                title: get_doc_display_name(&file),
                path: join3(&base_path, "/", &file_name),
                children: None,
            }
        })
        .collect()
}

/// Generates sidebar navigation metadata from extracted docs and the output path strategy.
///
/// `group_order` reorders the TypeDoc nav kind groups (matching the module index
/// section order). `sort` / `kind_sort_order` mirror the Markdown organization
/// options so the sidebar order never diverges from the generated pages, and
/// `sort_entry_points` preserves the caller-provided module order when `false`.
/// `None` / `true` keep the historical fixed order.
pub fn generate_nav_metadata_from_docs(
    docs: &[ApiDocModule],
    base_path: Option<&str>,
    path_strategy: MarkdownPathStrategy,
    group_order: Option<&[String]>,
    sort: Option<&[String]>,
    sort_entry_points: bool,
    kind_sort_order: Option<&[String]>,
) -> Vec<DocsNavItem> {
    generate_nav_metadata_from_docs_with_options(
        docs,
        &DocsNavMetadataOptions {
            base_path,
            path_strategy,
            group_order,
            sort,
            sort_entry_points,
            kind_sort_order,
            ..DocsNavMetadataOptions::default()
        },
    )
}

/// Generates sidebar navigation metadata with explicit nav options.
pub fn generate_nav_metadata_from_docs_with_options(
    docs: &[ApiDocModule],
    options: &DocsNavMetadataOptions<'_>,
) -> Vec<DocsNavItem> {
    profile_span!("docs::generate_nav");
    match options.path_strategy {
        MarkdownPathStrategy::Flat => {
            let files = docs.iter().map(|doc| doc.file.clone()).collect::<Vec<_>>();
            generate_nav_metadata(&files, options.base_path)
        }
        MarkdownPathStrategy::TypeDoc => {
            let nav = typedoc::generate_typedoc_nav_metadata(
                docs,
                options.base_path,
                options.group_order,
                options.sort,
                options.sort_entry_points,
                options.kind_sort_order,
            );
            if options.single_entry_root == MarkdownSingleEntryRoot::Flatten {
                typedoc::flatten_single_entry_typedoc_nav(nav)
            } else {
                nav
            }
        }
    }
}

/// Generates TypeScript source code for navigation metadata exports.
pub fn generate_nav_code(nav_items: &[DocsNavItem], export_name: Option<&str>) -> String {
    let export_name = export_name.unwrap_or(DEFAULT_EXPORT_NAME);
    let json = serde_json::to_string_pretty(nav_items).unwrap_or_else(|_| "[]".to_string());

    let mut out = StringBuilder::with_capacity(240 + export_name.len() + json.len());
    out.push_str(
        r"/**
 * Auto-generated API documentation navigation.
 * This file is automatically generated by the docs plugin.
 * Do not edit manually.
 */

export interface NavItem {
  title: string;
  path: string;
  children?: NavItem[];
}

export const ",
    );
    out.push_str(export_name);
    out.push_str(": NavItem[] = ");
    out.push_str(&json);
    out.push_str(" as const;\n");
    out.into_string()
}

#[cfg(test)]
mod tests;
