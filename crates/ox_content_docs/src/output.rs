//! Generated API documentation output writing.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::data::generate_docs_data_json;
use crate::markdown::MarkdownPathStrategy;
use crate::model::ApiDocModule;
use crate::nav::{generate_nav_code, generate_nav_metadata_from_docs};

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
            let nav_items = generate_nav_metadata_from_docs(
                extracted_docs,
                Some(base_path),
                options.path_strategy,
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
        std::env::temp_dir().join(format!("ox-content-docs-output-{nonce}-{seq}"))
    }

    fn options() -> DocsOutputOptions {
        DocsOutputOptions {
            generate_nav: true,
            group_by: "file".to_string(),
            generated_at: "2026-01-01T00:00:00.000Z".to_string(),
            base_path: None,
            path_strategy: MarkdownPathStrategy::Flat,
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
        assert!(fs::read_to_string(out_dir.join("beta.md")).unwrap().contains("updated"));
        assert!(fs::read_to_string(out_dir.join("manual.md")).unwrap().contains("Manual"));

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
            description: String::new(),
            file: "default".to_string(),
            entries: vec![
                ApiDocEntry {
                    name: "cli".to_string(),
                    kind: "function".to_string(),
                    description: "Runs the CLI.".to_string(),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    tags: vec![],
                    private: false,
                    file: "/repo/src/cli.ts".to_string(),
                    line: 1,
                    end_line: 5,
                    signature: Some("export function cli(): void".to_string()),
                    members: vec![],
                },
                ApiDocEntry {
                    name: "Mode".to_string(),
                    kind: "enum".to_string(),
                    description: "Run mode.".to_string(),
                    params: vec![],
                    returns: None,
                    examples: vec![],
                    tags: vec![],
                    private: false,
                    file: "/repo/src/mode.ts".to_string(),
                    line: 1,
                    end_line: 4,
                    signature: Some("export enum Mode".to_string()),
                    members: vec![ApiDocMember {
                        name: "Strict".to_string(),
                        kind: "enumMember".to_string(),
                        description: "Strict mode.".to_string(),
                        signature: None,
                        type_annotation: Some("\"strict\"".to_string()),
                        params: vec![],
                        returns: None,
                        optional: false,
                        readonly: false,
                        r#static: false,
                        private: false,
                        tags: vec![],
                        line: 2,
                        end_line: 2,
                    }],
                },
            ],
        }];

        let markdown_options = MarkdownDocsOptions {
            path_strategy: MarkdownPathStrategy::TypeDoc,
            base_path: Some("/api".to_string()),
            ..MarkdownDocsOptions::default()
        };
        let docs = generate_markdown(&extracted, &markdown_options);

        let output_options = DocsOutputOptions {
            generate_nav: true,
            group_by: "file".to_string(),
            generated_at: "2026-01-01T00:00:00.000Z".to_string(),
            base_path: Some("/api".to_string()),
            path_strategy: MarkdownPathStrategy::TypeDoc,
        };
        write_docs_output(&docs, &out_dir, Some(&extracted), &output_options).unwrap();

        assert!(out_dir.join("default/index.md").exists());
        assert!(out_dir.join("default/functions/cli.md").exists());
        assert!(out_dir.join("default/enumerations/Mode.md").exists());

        let nav = fs::read_to_string(out_dir.join(DOCS_NAV_FILE)).unwrap();
        assert!(nav.contains("\"path\": \"/api/default\""));
        assert!(nav.contains("\"path\": \"/api/default/functions/cli\""));
        assert!(nav.contains("\"path\": \"/api/default/enumerations/Mode\""));
        assert!(nav.contains("\"title\": \"Enumerations\""));

        let data = fs::read_to_string(out_dir.join(DOCS_DATA_FILE)).unwrap();
        assert!(data.contains("\"enum\": 1"));

        let manifest = fs::read_to_string(out_dir.join(DOCS_MANIFEST_FILE)).unwrap();
        assert!(manifest.contains("default/functions/cli.md"));
        assert!(manifest.contains("default/enumerations/Mode.md"));
        assert!(manifest.contains(DOCS_NAV_FILE));
        assert!(manifest.contains(DOCS_DATA_FILE));

        fs::remove_dir_all(out_dir).unwrap();
    }
}
