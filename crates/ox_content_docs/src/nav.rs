//! Navigation metadata generation for API documentation.

use std::path::Path;

use phf::phf_map;
use serde::{Deserialize, Serialize};

use crate::markdown::{
    compare_entries, kind_order_slice, order_by_group_title, ordered_entry_kinds,
    parse_sort_strategies, CanonicalOwners, MarkdownPathStrategy,
};
use crate::model::ApiDocModule;
#[allow(unused_imports)]
use crate::profile_span;
use crate::string_builder::{join2, join3, join5, StringBuilder};

const DEFAULT_BASE_PATH: &str = "/api";
const DEFAULT_EXPORT_NAME: &str = "apiNav";
const OVERVIEW_TITLE: &str = "Overview";

/// Navigation item for generated documentation sidebars.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocsNavItem {
    /// Display title for the navigation item.
    pub title: String,
    /// Path to the documentation page.
    pub path: String,
    /// Child navigation items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DocsNavItem>>,
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
    profile_span!("docs::generate_nav");
    match path_strategy {
        MarkdownPathStrategy::Flat => {
            let files = docs.iter().map(|doc| doc.file.clone()).collect::<Vec<_>>();
            generate_nav_metadata(&files, base_path)
        }
        MarkdownPathStrategy::TypeDoc => generate_typedoc_nav_metadata(
            docs,
            base_path,
            group_order,
            sort,
            sort_entry_points,
            kind_sort_order,
        ),
    }
}

fn generate_typedoc_nav_metadata(
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
                let mut seen = std::collections::HashSet::new();
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

fn normalize_base_path(base_path: &str) -> String {
    let base_path = base_path.trim().trim_end_matches('/');

    if base_path.is_empty() || base_path == "/" {
        return String::new();
    }

    if base_path.starts_with('/') {
        base_path.to_string()
    } else {
        join2("/", base_path)
    }
}

fn nav_route_path(base_path: &str, file_name: &str) -> String {
    let file_name = file_name.strip_suffix("/index").unwrap_or(file_name);
    if base_path.is_empty() {
        join2("/", file_name)
    } else {
        join3(base_path, "/", file_name)
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

export interface NavItem {{
  title: string;
  path: string;
  children?: NavItem[];
}}

export const ",
    );
    out.push_str(export_name);
    out.push_str(": NavItem[] = ");
    out.push_str(&json);
    out.push_str(" as const;\n");
    out.into_string()
}

fn get_doc_display_name(file_path: &str) -> String {
    let file_name = file_stem(file_path);

    if file_name == "index" || file_name == "index-module" {
        return OVERVIEW_TITLE.to_string();
    }

    format_doc_title(&file_name)
}

fn get_doc_file_name(file_path: &str) -> String {
    file_stem(file_path)
}

fn module_file_name(file_path: &str) -> String {
    let mut file_name = file_stem(file_path);
    if file_name == "index" {
        file_name = "index-module".to_string();
    }
    sanitize_doc_path_segment(&file_name)
}

fn typedoc_module_route_name(doc: &ApiDocModule) -> String {
    module_file_name(&doc.file)
}

fn typedoc_module_display_name(doc: &ApiDocModule) -> String {
    if !doc.source_path.is_empty() {
        return doc.file.clone();
    }

    let display_name = file_stem(&doc.file);
    if display_name.is_empty() {
        doc.file.clone()
    } else {
        display_name
    }
}

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

fn typedoc_kind_segment(kind: &str) -> &'static str {
    TYPEDOC_KIND_SEGMENT.get(kind).copied().unwrap_or("symbols")
}

fn typedoc_kind_title(kind: &str) -> &'static str {
    TYPEDOC_KIND_TITLE.get(kind).copied().unwrap_or("Symbols")
}

fn sanitize_doc_path_segment(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | '?' | '#' | '[' | ']' | '<' | '>' | ':' | '"' | '|' | '*' => '-',
            _ => ch,
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "symbol".to_string()
    } else {
        sanitized
    }
}

fn file_stem(file_path: &str) -> String {
    Path::new(file_path).file_stem().and_then(|stem| stem.to_str()).unwrap_or_default().to_string()
}

fn format_doc_title(name: &str) -> String {
    let mut chars = name.chars().peekable();
    let mut result = String::new();

    while let Some(ch) = chars.next() {
        if matches!(ch, '-' | '_') {
            match chars.peek().copied() {
                Some(next) if next.is_ascii_lowercase() => {
                    result.push(' ');
                    result.push(next.to_ascii_uppercase());
                    chars.next();
                }
                _ => result.push(ch),
            }
        } else {
            result.push(ch);
        }
    }

    if let Some(first) = result.chars().next().filter(char::is_ascii_lowercase) {
        let uppercase = first.to_ascii_uppercase().to_string();
        result.replace_range(0..first.len_utf8(), &uppercase);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ApiDocEntry;

    fn nav_entry(name: &str, kind: &str) -> ApiDocEntry {
        ApiDocEntry {
            name: name.to_string(),
            kind: kind.to_string(),
            description: String::new(),
            params: vec![],
            returns: None,
            examples: vec![],
            tags: vec![],
            private: false,
            file: join3("/repo/src/", name, ".ts"),
            line: 1,
            end_line: 1,
            signature: None,
            extends: vec![],
            implements: vec![],
            has_body: false,
            members: vec![],
            type_parameters: vec![],
        }
    }

    #[test]
    fn typedoc_nav_respects_group_order() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                nav_entry("alpha", "function"),
                nav_entry("Engine", "class"),
                nav_entry("VERSION", "variable"),
            ],
        }];
        let group_order = ["Variables".to_string(), "Functions".to_string()];
        let nav = generate_nav_metadata_from_docs(
            &docs,
            Some("/api"),
            MarkdownPathStrategy::TypeDoc,
            Some(&group_order),
            None,
            true,
            None,
        );
        let children = nav[0].children.as_ref().unwrap();
        let titles = children.iter().map(|child| child.title.as_str()).collect::<Vec<_>>();

        // Listed groups lead in order; the rest follow alphabetically.
        assert_eq!(titles, vec!["Variables", "Functions", "Classes"]);
    }

    #[test]
    fn typedoc_nav_sorts_leaf_entries_alphabetically() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            // Supplied out of order.
            entries: vec![
                nav_entry("plugin", "function"),
                nav_entry("cli", "function"),
                nav_entry("resolveArgs", "function"),
                nav_entry("parseArgs", "function"),
            ],
        }];

        let nav = generate_nav_metadata_from_docs(
            &docs,
            Some("/api"),
            MarkdownPathStrategy::TypeDoc,
            None,
            None,
            true,
            None,
        );
        let functions = nav[0].children.as_ref().unwrap()[0].children.as_ref().unwrap();
        let leaves = functions.iter().map(|child| child.title.as_str()).collect::<Vec<_>>();

        // Leaf entries are sorted case-insensitively, matching TypeDoc and the
        // generated Markdown module index.
        assert_eq!(leaves, vec!["cli", "parseArgs", "plugin", "resolveArgs"]);
    }

    #[test]
    fn typedoc_nav_sort_entry_points_false_preserves_module_order() {
        let module = |file: &str, entry: &str| ApiDocModule {
            description: String::new(),
            file: file.to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![nav_entry(entry, "function")],
        };
        // Supplied in non-alphabetical order.
        let docs = vec![module("zebra", "z"), module("alpha", "a")];

        // Default sorts modules alphabetically.
        let sorted = generate_nav_metadata_from_docs(
            &docs,
            Some("/api"),
            MarkdownPathStrategy::TypeDoc,
            None,
            None,
            true,
            None,
        );
        assert_eq!(
            sorted.iter().map(|module| module.title.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "zebra"]
        );

        // `sortEntryPoints: false` preserves the caller-provided module order.
        let preserved = generate_nav_metadata_from_docs(
            &docs,
            Some("/api"),
            MarkdownPathStrategy::TypeDoc,
            None,
            None,
            false,
            None,
        );
        assert_eq!(
            preserved.iter().map(|module| module.title.as_str()).collect::<Vec<_>>(),
            vec!["zebra", "alpha"]
        );
    }

    #[test]
    fn typedoc_nav_kind_sort_order_reorders_groups() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                nav_entry("alpha", "function"),
                nav_entry("Engine", "class"),
                nav_entry("VERSION", "variable"),
            ],
        }];
        let kind_sort_order = ["variable".to_string(), "class".to_string(), "function".to_string()];
        let nav = generate_nav_metadata_from_docs(
            &docs,
            Some("/api"),
            MarkdownPathStrategy::TypeDoc,
            None,
            None,
            true,
            Some(&kind_sort_order),
        );
        let titles = nav[0]
            .children
            .as_ref()
            .unwrap()
            .iter()
            .map(|child| child.title.as_str())
            .collect::<Vec<_>>();

        // Nav groups follow the configured kind order.
        assert_eq!(titles, vec!["Variables", "Classes", "Functions"]);
    }

    #[test]
    fn typedoc_nav_sort_orders_leaf_entries() {
        let mut zebra = nav_entry("zebra", "function");
        zebra.line = 1;
        let mut alpha = nav_entry("alpha", "function");
        alpha.line = 2;
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![zebra, alpha],
        }];
        let sort = ["source-order".to_string()];
        let nav = generate_nav_metadata_from_docs(
            &docs,
            Some("/api"),
            MarkdownPathStrategy::TypeDoc,
            None,
            Some(&sort),
            true,
            None,
        );
        let leaves = nav[0].children.as_ref().unwrap()[0]
            .children
            .as_ref()
            .unwrap()
            .iter()
            .map(|child| child.title.as_str())
            .collect::<Vec<_>>();

        // Source order: `zebra` (line 1) before `alpha` (line 2).
        assert_eq!(leaves, vec!["zebra", "alpha"]);
    }

    #[test]
    fn typedoc_nav_collapses_overloads_to_one_leaf() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            // `cli` is overloaded (multiple same-name entries resolving to one page).
            entries: vec![
                nav_entry("cli", "function"),
                nav_entry("cli", "function"),
                nav_entry("cli", "function"),
                nav_entry("define", "function"),
            ],
        }];

        let nav = generate_nav_metadata_from_docs(
            &docs,
            Some("/api"),
            MarkdownPathStrategy::TypeDoc,
            None,
            None,
            true,
            None,
        );
        let functions = nav[0].children.as_ref().unwrap()[0].children.as_ref().unwrap();
        let leaves = functions.iter().map(|child| child.title.as_str()).collect::<Vec<_>>();

        // The overloaded `cli` appears once, like TypeDoc's sidebar.
        assert_eq!(leaves, vec!["cli", "define"]);
    }

    #[test]
    fn generates_nav_metadata_from_file_paths() {
        let nav = generate_nav_metadata(
            &[
                "/repo/src/types.ts".to_string(),
                "/repo/src/index.ts".to_string(),
                "/repo/src/nav-generator.ts".to_string(),
            ],
            Some("/api"),
        );

        assert_eq!(
            nav,
            vec![
                DocsNavItem {
                    title: "Nav Generator".to_string(),
                    path: "/api/nav-generator".to_string(),
                    children: None,
                },
                DocsNavItem {
                    title: "Overview".to_string(),
                    path: "/api/index".to_string(),
                    children: None,
                },
                DocsNavItem {
                    title: "Types".to_string(),
                    path: "/api/types".to_string(),
                    children: None,
                },
            ]
        );
    }

    #[test]
    fn normalizes_nav_base_path() {
        let nav = generate_nav_metadata(&["/repo/src/context.ts".to_string()], Some("api-ox/"));

        assert_eq!(nav[0].path, "/api-ox/context");
    }

    #[test]
    fn generates_typedoc_nav_metadata_from_docs() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![
                ApiDocEntry {
                    name: "cli".to_string(),
                    kind: "function".to_string(),
                    description: String::new(),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    tags: vec![],
                    private: false,
                    file: "cli.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: None,
                    extends: vec![],
                    implements: vec![],
                    has_body: false,
                    members: vec![],
                    type_parameters: vec![],
                },
                ApiDocEntry {
                    name: "Command".to_string(),
                    kind: "interface".to_string(),
                    description: String::new(),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    tags: vec![],
                    private: false,
                    file: "types.ts".to_string(),
                    line: 1,
                    end_line: 1,
                    signature: None,
                    extends: vec![],
                    implements: vec![],
                    has_body: false,
                    members: vec![],
                    type_parameters: vec![],
                },
            ],
        }];

        let nav = generate_nav_metadata_from_docs(
            &docs,
            Some("/api"),
            MarkdownPathStrategy::TypeDoc,
            None,
            None,
            true,
            None,
        );

        assert_eq!(nav[0].title, "default");
        assert_eq!(nav[0].path, "/api/default");
        let children = nav[0].children.as_ref().unwrap();
        assert_eq!(children[0].title, "Functions");
        assert_eq!(children[0].path, "/api/default/functions");
        assert_eq!(children[0].children.as_ref().unwrap()[0].path, "/api/default/functions/cli");
        assert_eq!(children[1].title, "Interfaces");
        assert_eq!(
            children[1].children.as_ref().unwrap()[0].path,
            "/api/default/interfaces/Command"
        );
    }

    #[test]
    fn generates_typedoc_nav_metadata_includes_enumerations() {
        let docs = vec![ApiDocModule {
            description: String::new(),
            file: "default".to_string(),
            source_path: String::new(),
            examples: vec![],
            tags: vec![],
            entries: vec![ApiDocEntry {
                name: "Mode".to_string(),
                kind: "enum".to_string(),
                description: String::new(),
                params: vec![],
                returns: None,
                examples: vec![],
                tags: vec![],
                private: false,
                file: "mode.ts".to_string(),
                line: 1,
                end_line: 1,
                signature: None,
                extends: vec![],
                implements: vec![],
                has_body: false,
                members: vec![],
                type_parameters: vec![],
            }],
        }];

        let nav = generate_nav_metadata_from_docs(
            &docs,
            Some("/api"),
            MarkdownPathStrategy::TypeDoc,
            None,
            None,
            true,
            None,
        );
        let children = nav[0].children.as_ref().unwrap();

        assert_eq!(children[0].title, "Enumerations");
        assert_eq!(children[0].path, "/api/default/enumerations");
        assert_eq!(
            children[0].children.as_ref().unwrap()[0].path,
            "/api/default/enumerations/Mode"
        );
    }

    #[test]
    fn typedoc_nav_omits_reexports_from_non_owner_modules() {
        let make = |module: &str, source: &str| ApiDocModule {
            description: String::new(),
            file: module.to_string(),
            source_path: source.to_string(),
            examples: vec![],
            tags: vec![],
            entries: vec![ApiDocEntry {
                name: "createCommandContext".to_string(),
                kind: "function".to_string(),
                description: String::new(),
                params: vec![],
                returns: None,
                examples: vec![],
                tags: vec![],
                private: false,
                file: "/repo/src/context.ts".to_string(),
                line: 1,
                end_line: 1,
                signature: None,
                extends: vec![],
                implements: vec![],
                has_body: false,
                members: vec![],
                type_parameters: vec![],
            }],
        };
        // `context` defines the symbol; `default` only re-exports it.
        let docs =
            vec![make("context", "/repo/src/context.ts"), make("default", "/repo/src/index.ts")];

        let nav = generate_nav_metadata_from_docs(
            &docs,
            Some("/api"),
            MarkdownPathStrategy::TypeDoc,
            None,
            None,
            true,
            None,
        );

        let context = nav.iter().find(|item| item.path == "/api/context").unwrap();
        assert!(context.children.is_some(), "owner module keeps the symbol in the sidebar");
        let default = nav.iter().find(|item| item.path == "/api/default").unwrap();
        assert!(
            default.children.is_none(),
            "re-exporting module omits the symbol from the sidebar"
        );
    }

    #[test]
    fn generates_nav_code() {
        let code = generate_nav_code(
            &[DocsNavItem {
                title: "Docs".to_string(),
                path: "/api/docs".to_string(),
                children: None,
            }],
            Some("apiNav"),
        );

        assert!(code.contains("export const apiNav: NavItem[] = ["));
        assert!(code.contains(r#""title": "Docs""#));
        assert!(code.ends_with(" as const;\n"));
    }
}
