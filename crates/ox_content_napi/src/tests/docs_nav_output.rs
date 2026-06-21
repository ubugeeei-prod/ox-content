use super::*;

#[test]
fn generate_docs_nav_metadata_from_docs_returns_typedoc_tree() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![
            JsDocsMarkdownEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                file: "/repo/src/cli.ts".to_string(),
                ..Default::default()
            },
            JsDocsMarkdownEntry {
                name: "Mode".to_string(),
                kind: "enum".to_string(),
                file: "/repo/src/mode.ts".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }];

    let nav = generate_docs_nav_metadata_from_docs_napi(
        docs,
        Some(JsDocsNavOptions {
            base_path: Some("/api".to_string()),
            path_strategy: Some("typedoc".to_string()),
            ..Default::default()
        }),
    );

    assert_eq!(nav[0].title, "default");
    assert_eq!(nav[0].path, "/api/default");
    let children = nav[0].children.as_ref().unwrap();
    assert_eq!(children[0].title, "Functions");
    assert_eq!(children[0].children.as_ref().unwrap()[0].path, "/api/default/functions/cli");
    assert_eq!(children[1].title, "Enumerations");
    assert_eq!(children[1].children.as_ref().unwrap()[0].path, "/api/default/enumerations/Mode");
}

#[test]
fn generate_docs_nav_metadata_from_docs_flattens_single_typedoc_entry() {
    let docs = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![
            JsDocsMarkdownEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                file: "/repo/src/cli.ts".to_string(),
                ..Default::default()
            },
            JsDocsMarkdownEntry {
                name: "Command".to_string(),
                kind: "interface".to_string(),
                file: "/repo/src/types.ts".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }];

    let nav = generate_docs_nav_metadata_from_docs_napi(
        docs,
        Some(JsDocsNavOptions {
            base_path: Some("/api".to_string()),
            path_strategy: Some("typedoc".to_string()),
            single_entry_root: Some("flatten".to_string()),
            ..Default::default()
        }),
    );

    assert_eq!(
        nav.iter().map(|item| item.title.as_str()).collect::<Vec<_>>(),
        vec!["Functions", "Interfaces"]
    );
    assert_eq!(nav[0].path, "/api/default/functions");
    assert_eq!(nav[0].children.as_ref().unwrap()[0].path, "/api/default/functions/cli");
    assert_eq!(nav[1].children.as_ref().unwrap()[0].path, "/api/default/interfaces/Command");
}

#[test]
fn generate_docs_nav_metadata_from_docs_defaults_to_flat() {
    let docs = vec![JsDocsMarkdownModule {
        file: "/repo/src/context.ts".to_string(),
        ..Default::default()
    }];

    let nav = generate_docs_nav_metadata_from_docs_napi(docs, None);

    assert_eq!(nav.len(), 1);
    assert_eq!(nav[0].path, "/api/context");
    assert!(nav[0].children.is_none());
}

#[test]
fn write_generated_docs_writes_typedoc_nested_files() {
    use crate::{write_generated_docs, JsDocsOutputOptions};

    let unique =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let out_dir = std::env::temp_dir()
        .join(format!("ox-content-napi-typedoc-write-{}-{unique}", std::process::id()));

    let extracted = vec![JsDocsMarkdownModule {
        file: "default".to_string(),
        entries: vec![
            JsDocsMarkdownEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                description: "Runs the CLI.".to_string(),
                file: "/repo/src/cli.ts".to_string(),
                signature: Some("export function cli(): void".to_string()),
                ..Default::default()
            },
            JsDocsMarkdownEntry {
                name: "version".to_string(),
                kind: "variable".to_string(),
                description: "Package version.".to_string(),
                file: "/repo/src/version.ts".to_string(),
                line: 2,
                end_line: 2,
                signature: Some("export const version = '1.0.0'".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }];

    let markdown = generate_docs_markdown(
        extracted.clone(),
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            link_style: Some("clean".to_string()),
            base_path: Some("/api".to_string()),
            path_strategy: Some("typedoc".to_string()),
            ..Default::default()
        }),
    );

    write_generated_docs(
        markdown,
        out_dir.to_string_lossy().to_string(),
        Some(extracted),
        Some(JsDocsOutputOptions {
            generate_nav: Some(true),
            group_by: Some("file".to_string()),
            generated_at: Some("2026-01-01T00:00:00.000Z".to_string()),
            base_path: Some("/api".to_string()),
            path_strategy: Some("typedoc".to_string()),
            group_order: Some(vec!["Variables".to_string(), "Functions".to_string()]),
            ..Default::default()
        }),
    )
    .unwrap();

    assert!(out_dir.join("default/index.md").exists());
    assert!(out_dir.join("default/functions/cli.md").exists());
    assert!(out_dir.join("default/variables/version.md").exists());

    let nav = fs::read_to_string(out_dir.join("nav.ts")).unwrap();
    insta::assert_snapshot!("write_generated_docs_writes_typedoc_nested_files__nav", nav);

    fs::remove_dir_all(&out_dir).unwrap();
}

#[test]
fn write_generated_docs_writes_flattened_single_entry_nav() {
    use crate::{write_generated_docs, JsDocsOutputOptions};

    let unique =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let out_dir = std::env::temp_dir()
        .join(format!("ox-content-napi-flatten-write-{}-{unique}", std::process::id()));

    let extracted = vec![JsDocsMarkdownModule {
        description: Some("Runtime API.".to_string()),
        file: "default".to_string(),
        entries: vec![JsDocsMarkdownEntry {
            name: "cli".to_string(),
            kind: "function".to_string(),
            description: "Runs the CLI.".to_string(),
            file: "/repo/src/cli.ts".to_string(),
            signature: Some("export function cli(): void".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    }];

    let markdown = generate_docs_markdown(
        extracted.clone(),
        Some(JsDocsMarkdownOptions {
            group_by: Some("file".to_string()),
            link_style: Some("clean".to_string()),
            base_path: Some("/api".to_string()),
            path_strategy: Some("typedoc".to_string()),
            render_style: Some("markdown".to_string()),
            single_entry_root: Some("flatten".to_string()),
            ..Default::default()
        }),
    );

    write_generated_docs(
        markdown,
        out_dir.to_string_lossy().to_string(),
        Some(extracted),
        Some(JsDocsOutputOptions {
            generate_nav: Some(true),
            group_by: Some("file".to_string()),
            generated_at: Some("2026-01-01T00:00:00.000Z".to_string()),
            base_path: Some("/api".to_string()),
            path_strategy: Some("typedoc".to_string()),
            single_entry_root: Some("flatten".to_string()),
            ..Default::default()
        }),
    )
    .unwrap();

    assert!(out_dir.join("index.md").exists());
    assert!(!out_dir.join("default/index.md").exists());
    assert!(out_dir.join("default/functions/cli.md").exists());

    let nav = fs::read_to_string(out_dir.join("nav.ts")).unwrap();
    insta::assert_snapshot!("write_generated_docs_writes_flattened_single_entry_nav__nav", nav);

    fs::remove_dir_all(&out_dir).unwrap();
}
