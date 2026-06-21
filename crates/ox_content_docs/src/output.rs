//! Generated API documentation output writing.

// BTreeMap keeps generated file output deterministic across runs.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::data::generate_docs_data_json;
use crate::markdown::{MarkdownPathStrategy, MarkdownSingleEntryRoot};
use crate::model::ApiDocModule;
use crate::nav::{
    generate_nav_code, generate_nav_metadata_from_docs_with_options, DocsNavMetadataOptions,
};
#[allow(unused_imports)]
use crate::profile_span;

const DOCS_MANIFEST_FILE: &str = ".ox-content-docs-manifest.json";
const DOCS_DATA_FILE: &str = "docs.json";
const DOCS_NAV_FILE: &str = "nav.ts";
const DOCS_NAV_BASE_PATH: &str = "/api";
const DOCS_NAV_EXPORT_NAME: &str = "apiNav";

/// Options for writing generated API documentation files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsOutputOptions {
    /// Whether to write `nav.ts` for file-grouped docs.
    pub generate_nav: bool,
    /// Documentation grouping mode.
    pub group_by: String,
    /// ISO timestamp included in `docs.json`.
    pub generated_at: String,
    /// Base path used for navigation links. Defaults to `/api` when `None`.
    pub base_path: Option<String>,
    /// Output path strategy used for navigation metadata.
    pub path_strategy: MarkdownPathStrategy,
    /// TypeDoc-style group order for nav groups.
    pub group_order: Option<Vec<String>>,
    /// TypeDoc-style sort strategies for nav leaf entries.
    pub sort: Option<Vec<String>>,
    /// Whether to sort entry points alphabetically.
    pub sort_entry_points: bool,
    /// TypeDoc-style kind ranking for nav groups.
    pub kind_sort_order: Option<Vec<String>>,
    /// Single-entry root handling for generated nav metadata.
    pub single_entry_root: MarkdownSingleEntryRoot,
}

impl Default for DocsOutputOptions {
    fn default() -> Self {
        Self {
            generate_nav: false,
            group_by: "file".to_string(),
            generated_at: String::new(),
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

/// Error returned while writing generated docs.
#[derive(Debug, Error)]
pub enum DocsOutputError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization or parsing error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for generated docs output writing.
pub type DocsOutputResult<T> = Result<T, DocsOutputError>;

/// Writes generated Markdown docs plus native sidecar files to an output directory.
pub fn write_docs_output(
    docs: &BTreeMap<String, String>,
    out_dir: &Path,
    extracted_docs: Option<&[ApiDocModule]>,
    options: &DocsOutputOptions,
) -> DocsOutputResult<()> {
    profile_span!("docs::write_output");
    fs::create_dir_all(out_dir)?;

    let mut generated_files = docs.keys().cloned().collect::<Vec<_>>();
    if extracted_docs.is_some() {
        generated_files.push(DOCS_DATA_FILE.to_string());
    }
    if extracted_docs.is_some() && options.generate_nav && options.group_by == "file" {
        generated_files.push(DOCS_NAV_FILE.to_string());
    }
    generated_files.sort();
    generated_files.dedup();

    remove_stale_files(out_dir, &generated_files)?;

    for (file_name, content) in docs {
        let output_path = out_dir.join(file_name);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output_path, content)?;
    }

    if let Some(extracted_docs) = extracted_docs {
        if options.generate_nav && options.group_by == "file" {
            let base_path = options.base_path.as_deref().unwrap_or(DOCS_NAV_BASE_PATH);
            let nav_items = generate_nav_metadata_from_docs_with_options(
                extracted_docs,
                &DocsNavMetadataOptions {
                    base_path: Some(base_path),
                    path_strategy: options.path_strategy,
                    group_order: options.group_order.as_deref(),
                    sort: options.sort.as_deref(),
                    sort_entry_points: options.sort_entry_points,
                    kind_sort_order: options.kind_sort_order.as_deref(),
                    single_entry_root: options.single_entry_root,
                },
            );
            fs::write(
                out_dir.join(DOCS_NAV_FILE),
                generate_nav_code(&nav_items, Some(DOCS_NAV_EXPORT_NAME)),
            )?;
        }

        fs::write(
            out_dir.join(DOCS_DATA_FILE),
            generate_docs_data_json(extracted_docs, &options.generated_at)?,
        )?;
    }

    fs::write(out_dir.join(DOCS_MANIFEST_FILE), serde_json::to_string_pretty(&generated_files)?)?;

    Ok(())
}

fn remove_stale_files(out_dir: &Path, generated_files: &[String]) -> DocsOutputResult<()> {
    let manifest_path = out_dir.join(DOCS_MANIFEST_FILE);
    let previous_files = match fs::read_to_string(&manifest_path) {
        Ok(content) => serde_json::from_str::<Vec<String>>(&content).unwrap_or_default(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error.into()),
    };

    // `generated_files` is sorted and deduped by the caller, so stale detection
    // is a binary search per previous file instead of repeatedly scanning the
    // new file list while cleaning nested TypeDoc output trees.
    for stale_file in previous_files {
        if generated_files.binary_search(&stale_file).is_ok() {
            continue;
        }

        let stale_path = out_dir.join(stale_file);
        match fs::remove_file(&stale_path) {
            Ok(()) => remove_empty_parent_dirs(out_dir, stale_path.parent())?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

fn remove_empty_parent_dirs(out_dir: &Path, parent: Option<&Path>) -> DocsOutputResult<()> {
    let Some(parent) = parent else {
        return Ok(());
    };
    if parent == out_dir || !parent.starts_with(out_dir) {
        return Ok(());
    }

    match fs::remove_dir(parent) {
        Ok(()) => remove_empty_parent_dirs(out_dir, parent.parent()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
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
        let mut dirname =
            StringBuilder::with_capacity("ox-content-docs-output-".len() + 39 + 1 + 20);
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
        use crate::markdown::{generate_markdown, MarkdownDocsOptions};
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

        let nav = fs::read_to_string(out_dir.join(DOCS_NAV_FILE)).unwrap();
        insta::assert_snapshot!("typedoc_docs_nav", nav);

        let data = fs::read_to_string(out_dir.join(DOCS_DATA_FILE)).unwrap();
        insta::assert_snapshot!("typedoc_docs_data", data);

        let manifest = fs::read_to_string(out_dir.join(DOCS_MANIFEST_FILE)).unwrap();
        insta::assert_snapshot!("typedoc_docs_manifest", manifest);

        fs::remove_dir_all(out_dir).unwrap();
    }
}
