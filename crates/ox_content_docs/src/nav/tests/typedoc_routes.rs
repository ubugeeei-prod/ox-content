use super::super::*;
use super::nav_entry;
use crate::model::ApiDocEntry;

#[test]
fn generates_typedoc_nav_metadata_from_docs() {
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![
            ApiDocEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                file: "cli.ts".to_string(),
                ..ApiDocEntry::default()
            },
            ApiDocEntry {
                name: "Command".to_string(),
                kind: "interface".to_string(),
                file: "types.ts".to_string(),
                ..ApiDocEntry::default()
            },
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

    assert_eq!(nav[0].title, "default");
    assert_eq!(nav[0].path, "/api/default");
    let children = nav[0].children.as_ref().unwrap();
    assert_eq!(children[0].title, "Functions");
    assert_eq!(children[0].path, "/api/default/functions");
    assert_eq!(children[0].children.as_ref().unwrap()[0].path, "/api/default/functions/cli");
    assert_eq!(children[1].title, "Interfaces");
    assert_eq!(children[1].children.as_ref().unwrap()[0].path, "/api/default/interfaces/Command");
}

#[test]
fn typedoc_nav_single_entry_root_flatten_promotes_kind_groups() {
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![nav_entry("cli", "function"), nav_entry("Command", "interface")],
        ..ApiDocModule::default()
    }];

    let nav = generate_nav_metadata_from_docs_with_options(
        &docs,
        &DocsNavMetadataOptions {
            base_path: Some("/api"),
            path_strategy: MarkdownPathStrategy::TypeDoc,
            single_entry_root: MarkdownSingleEntryRoot::Flatten,
            ..DocsNavMetadataOptions::default()
        },
    );

    assert_eq!(
        nav.iter().map(|item| item.title.as_str()).collect::<Vec<_>>(),
        vec!["Functions", "Interfaces"]
    );
    assert_eq!(nav[0].path, "/api/default/functions");
    assert_eq!(nav[0].children.as_ref().unwrap()[0].path, "/api/default/functions/cli");
    assert_eq!(nav[1].path, "/api/default/interfaces");
    assert_eq!(nav[1].children.as_ref().unwrap()[0].path, "/api/default/interfaces/Command");
}

#[test]
fn generates_typedoc_nav_metadata_includes_enumerations() {
    let docs = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![ApiDocEntry {
            name: "Mode".to_string(),
            kind: "enum".to_string(),
            file: "mode.ts".to_string(),
            ..ApiDocEntry::default()
        }],
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
    let children = nav[0].children.as_ref().unwrap();

    assert_eq!(children[0].title, "Enumerations");
    assert_eq!(children[0].path, "/api/default/enumerations");
    assert_eq!(children[0].children.as_ref().unwrap()[0].path, "/api/default/enumerations/Mode");
}

#[test]
fn typedoc_nav_omits_reexports_from_non_owner_modules() {
    let make = |module: &str, source: &str| ApiDocModule {
        file: module.to_string(),
        source_path: source.to_string(),
        entries: vec![ApiDocEntry {
            name: "createCommandContext".to_string(),
            kind: "function".to_string(),
            file: "/repo/src/context.ts".to_string(),
            ..ApiDocEntry::default()
        }],
        ..ApiDocModule::default()
    };
    // `context` defines the symbol; `default` only re-exports it.
    let docs = vec![make("context", "/repo/src/context.ts"), make("default", "/repo/src/index.ts")];

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
    assert!(default.children.is_none(), "re-exporting module omits the symbol from the sidebar");
}
