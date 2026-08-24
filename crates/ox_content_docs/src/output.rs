//! Generated API documentation output writing.

// BTreeMap keeps generated file output deterministic across runs.
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::data::generate_docs_data_json;
use crate::markdown::{MarkdownPathStrategy, MarkdownSingleEntryRoot};
use crate::model::ApiDocModule;
use crate::nav::{
    DocsNavMetadataOptions, generate_nav_code, generate_nav_metadata_from_docs_with_options,
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
        fs::write(output_path, normalize_markdown_eof(content).as_bytes())?;
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

fn normalize_markdown_eof(content: &str) -> Cow<'_, str> {
    if !content.ends_with("\n\n") {
        return Cow::Borrowed(content);
    }

    let mut normalized = content.trim_end_matches('\n').to_string();
    normalized.push('\n');
    Cow::Owned(normalized)
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
mod tests;
