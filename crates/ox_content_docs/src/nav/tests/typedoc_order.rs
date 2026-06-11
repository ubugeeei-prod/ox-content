use super::super::*;
use super::nav_entry;

#[test]
fn typedoc_nav_respects_group_order() {
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![
            nav_entry("alpha", "function"),
            nav_entry("Engine", "class"),
            nav_entry("VERSION", "variable"),
        ],
        ..ApiDocModule::default()
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
        file: "default".to_string(),
        // Supplied out of order.
        entries: vec![
            nav_entry("plugin", "function"),
            nav_entry("cli", "function"),
            nav_entry("resolveArgs", "function"),
            nav_entry("parseArgs", "function"),
        ],
        ..ApiDocModule::default()
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
        file: file.to_string(),
        entries: vec![nav_entry(entry, "function")],
        ..ApiDocModule::default()
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
        file: "default".to_string(),
        entries: vec![
            nav_entry("alpha", "function"),
            nav_entry("Engine", "class"),
            nav_entry("VERSION", "variable"),
        ],
        ..ApiDocModule::default()
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
        file: "default".to_string(),
        entries: vec![zebra, alpha],
        ..ApiDocModule::default()
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
        file: "default".to_string(),
        // `cli` is overloaded (multiple same-name entries resolving to one page).
        entries: vec![
            nav_entry("cli", "function"),
            nav_entry("cli", "function"),
            nav_entry("cli", "function"),
            nav_entry("define", "function"),
        ],
        ..ApiDocModule::default()
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
