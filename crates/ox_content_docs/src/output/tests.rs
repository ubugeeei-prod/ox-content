use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use crate::string_builder::StringBuilder;

fn temp_dir() -> std::path::PathBuf {
    // A timestamp alone is not unique enough: under parallel test execution the
    // system clock resolution can be coarse enough that two tests observe the same
    // nanosecond value and collide on the same directory. Combine it with a
    // process-wide atomic counter so every call gets a distinct path.
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut dirname = StringBuilder::with_capacity("ox-content-docs-output-".len() + 39 + 1 + 20);
    dirname.push_str("ox-content-docs-output-");
    dirname.push_u128(nonce);
    dirname.push_char('-');
    dirname.push_u128(u128::from(seq));
    std::env::temp_dir().join(dirname.into_string())
}

fn options() -> DocsOutputOptions {
    DocsOutputOptions {
        generate_nav: true,
        generated_at: "2026-01-01T00:00:00.000Z".to_string(),
        ..DocsOutputOptions::default()
    }
}

#[test]
fn removes_stale_manifest_files_without_touching_manual_files() {
    let out_dir = temp_dir();
    let mut docs = BTreeMap::new();
    docs.insert("alpha.md".to_string(), "# Alpha".to_string());
    docs.insert("beta.md".to_string(), "# Beta".to_string());

    fs::create_dir_all(&out_dir).unwrap();
    fs::write(out_dir.join("manual.md"), "# Manual").unwrap();
    write_docs_output(&docs, &out_dir, None, &options()).unwrap();

    docs.remove("alpha.md");
    docs.insert("beta.md".to_string(), "# Beta updated".to_string());
    write_docs_output(&docs, &out_dir, None, &options()).unwrap();

    assert!(!out_dir.join("alpha.md").exists());
    assert_eq!(fs::read_to_string(out_dir.join("beta.md")).unwrap(), "# Beta updated");
    assert_eq!(fs::read_to_string(out_dir.join("manual.md")).unwrap(), "# Manual");

    fs::remove_dir_all(out_dir).unwrap();
}

#[test]
fn generated_markdown_has_one_terminal_newline() {
    let out_dir = temp_dir();
    let docs = BTreeMap::from([("alpha.md".to_string(), "# Alpha\n\n\n".to_string())]);

    write_docs_output(&docs, &out_dir, None, &options()).unwrap();

    assert_eq!(fs::read_to_string(out_dir.join("alpha.md")).unwrap(), "# Alpha\n");
    fs::remove_dir_all(out_dir).unwrap();
}

#[test]
fn unchanged_docs_output_keeps_existing_mtime() {
    let out_dir = temp_dir();
    let docs = BTreeMap::from([("alpha.md".to_string(), "# Alpha".to_string())]);

    write_docs_output(&docs, &out_dir, None, &options()).unwrap();
    let path = out_dir.join("alpha.md");
    let first_modified = fs::metadata(&path).unwrap().modified().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(20));
    write_docs_output(&docs, &out_dir, None, &options()).unwrap();

    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), first_modified);
    fs::remove_dir_all(out_dir).unwrap();
}

#[test]
fn writes_and_removes_stale_nested_docs_output() {
    let out_dir = temp_dir();
    let mut docs = BTreeMap::new();
    docs.insert("default/functions/cli.md".to_string(), "# cli".to_string());
    docs.insert("default/interfaces/Command.md".to_string(), "# Command".to_string());

    write_docs_output(&docs, &out_dir, None, &options()).unwrap();

    assert!(out_dir.join("default/functions/cli.md").exists());
    assert!(out_dir.join("default/interfaces/Command.md").exists());

    docs.remove("default/functions/cli.md");
    write_docs_output(&docs, &out_dir, None, &options()).unwrap();

    assert!(!out_dir.join("default/functions/cli.md").exists());
    assert!(!out_dir.join("default/functions").exists());
    assert!(out_dir.join("default/interfaces/Command.md").exists());

    fs::remove_dir_all(out_dir).unwrap();
}

#[test]
fn writes_typedoc_docs_with_consistent_nav_and_data() {
    use crate::markdown::{MarkdownDocsOptions, generate_markdown};
    use crate::model::{ApiDocEntry, ApiDocMember};

    let out_dir = temp_dir();
    let extracted = vec![ApiDocModule {
        file: "default".to_string(),
        entries: vec![
            ApiDocEntry {
                name: "cli".to_string(),
                kind: "function".to_string(),
                description: "Runs the CLI.".to_string(),
                file: "/repo/src/cli.ts".to_string(),
                end_line: 5,
                signature: Some("export function cli(): void".to_string()),
                ..ApiDocEntry::default()
            },
            ApiDocEntry {
                name: "Mode".to_string(),
                kind: "enum".to_string(),
                description: "Run mode.".to_string(),
                file: "/repo/src/mode.ts".to_string(),
                end_line: 4,
                signature: Some("export enum Mode".to_string()),
                members: vec![ApiDocMember {
                    name: "Strict".to_string(),
                    kind: "enumMember".to_string(),
                    description: "Strict mode.".to_string(),
                    type_annotation: Some("\"strict\"".to_string()),
                    line: 2,
                    end_line: 2,
                    ..ApiDocMember::default()
                }],
                ..ApiDocEntry::default()
            },
        ],
        ..ApiDocModule::default()
    }];

    let markdown_options = MarkdownDocsOptions {
        path_strategy: MarkdownPathStrategy::TypeDoc,
        base_path: Some("/api".to_string()),
        ..MarkdownDocsOptions::default()
    };
    let docs = generate_markdown(&extracted, &markdown_options);

    let output_options = DocsOutputOptions {
        generate_nav: true,
        generated_at: "2026-01-01T00:00:00.000Z".to_string(),
        base_path: Some("/api".to_string()),
        path_strategy: MarkdownPathStrategy::TypeDoc,
        ..DocsOutputOptions::default()
    };
    write_docs_output(&docs, &out_dir, Some(&extracted), &output_options).unwrap();

    assert!(out_dir.join("default/index.md").exists());
    assert!(out_dir.join("default/functions/cli.md").exists());
    assert!(out_dir.join("default/enumerations/Mode.md").exists());

    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        let nav = fs::read_to_string(out_dir.join(DOCS_NAV_FILE)).unwrap();
        insta::assert_snapshot!("typedoc_docs_nav", nav);

        let data = fs::read_to_string(out_dir.join(DOCS_DATA_FILE)).unwrap();
        insta::assert_snapshot!("typedoc_docs_data", data);

        let manifest = fs::read_to_string(out_dir.join(DOCS_MANIFEST_FILE)).unwrap();
        insta::assert_snapshot!("typedoc_docs_manifest", manifest);
    });

    fs::remove_dir_all(out_dir).unwrap();
}
