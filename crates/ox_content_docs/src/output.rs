//! Generated API documentation output writing.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::data::generate_docs_data_json;
use crate::markdown::ApiDocModule;
use crate::nav::{generate_nav_code, generate_nav_metadata};

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
        fs::write(out_dir.join(file_name), content)?;
    }

    if let Some(extracted_docs) = extracted_docs {
        if options.generate_nav && options.group_by == "file" {
            let files = extracted_docs.iter().map(|doc| doc.file.clone()).collect::<Vec<_>>();
            let nav_items = generate_nav_metadata(&files, Some(DOCS_NAV_BASE_PATH));
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

        match fs::remove_file(out_dir.join(stale_file)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_dir() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("ox-content-docs-output-{nonce}"))
    }

    fn options() -> DocsOutputOptions {
        DocsOutputOptions {
            generate_nav: true,
            group_by: "file".to_string(),
            generated_at: "2026-01-01T00:00:00.000Z".to_string(),
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
}
