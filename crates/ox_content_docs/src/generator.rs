//! Documentation site generator.

use std::path::Path;

use crate::config::DocsConfig;
use crate::extractor::{DocExtractor, DocItem, ExtractResult};
use crate::normalize::{normalize_doc_items, NormalizedDocEntry};

use thiserror::Error;

/// Result type for generation operations.
pub type GenerateResult<T> = Result<T, GenerateError>;

/// Errors during documentation generation.
#[derive(Debug, Error)]
pub enum GenerateError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Extraction error.
    #[error("Extraction error: {0}")]
    Extract(#[from] crate::extractor::ExtractError),

    /// Template error.
    #[error("Template error: {0}")]
    Template(String),
}

/// Documentation generator.
pub struct DocsGenerator {
    config: DocsConfig,
    extractor: DocExtractor,
}

/// Extracted documentation for one source module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedDocModule {
    /// Source file path.
    pub file: String,
    /// Normalized documentation entries for the source file.
    pub entries: Vec<NormalizedDocEntry>,
}

impl DocsGenerator {
    /// Creates a new documentation generator.
    #[must_use]
    pub fn new(config: DocsConfig) -> Self {
        let extractor = DocExtractor::with_private(config.document_private);
        Self { config, extractor }
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &DocsConfig {
        &self.config
    }

    /// Generates documentation for all source files.
    pub fn generate(&self) -> GenerateResult<()> {
        let items = self.extract_all()?;
        self.render(&items)?;
        Ok(())
    }

    /// Extracts documentation from all source files.
    pub fn extract_all(&self) -> ExtractResult<Vec<DocItem>> {
        let mut all_items = Vec::new();

        for src_dir in &self.config.src_dirs {
            let items = self.extract_dir(Path::new(src_dir))?;
            all_items.extend(items);
        }

        Ok(all_items)
    }

    /// Extracts documentation from a directory.
    fn extract_dir(&self, dir: &Path) -> ExtractResult<Vec<DocItem>> {
        let mut items = Vec::new();

        if !dir.is_dir() {
            return Ok(items);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                items.extend(self.extract_dir(&path)?);
            } else if self.should_include(&path) {
                if let Ok(file_items) = self.extractor.extract_file(&path) {
                    items.extend(file_items);
                }
            }
        }

        Ok(items)
    }

    /// Checks if a file should be included.
    fn should_include(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check excludes first
        for pattern in &self.config.exclude {
            if glob_match(pattern, &path_str) {
                return false;
            }
        }

        // Check includes
        for pattern in &self.config.include {
            if glob_match(pattern, &path_str) {
                return true;
            }
        }

        false
    }

    /// Renders documentation items to HTML.
    fn render(&self, items: &[DocItem]) -> GenerateResult<()> {
        let out_dir = Path::new(&self.config.out_dir);
        std::fs::create_dir_all(out_dir)?;

        if self.config.json {
            let json = serde_json::to_string_pretty(items)
                .map_err(|e| GenerateError::Template(e.to_string()))?;
            std::fs::write(out_dir.join("docs.json"), json)?;
        }

        // TODO: Generate HTML pages
        // For now, just create the output directory

        Ok(())
    }
}

/// Collects documentation source files under `src_dir`.
#[must_use]
pub fn collect_source_files(src_dir: &str, include: &[String], exclude: &[String]) -> Vec<String> {
    let mut files = Vec::new();
    collect_source_files_inner(Path::new(src_dir), include, exclude, &mut files);
    files.sort();
    files
}

/// Extracts normalized documentation from source directories.
pub fn extract_docs_from_directories(
    src_dirs: &[String],
    include: &[String],
    exclude: &[String],
    include_private: bool,
    include_internal: bool,
    type_parameters: bool,
) -> ExtractResult<Vec<ExtractedDocModule>> {
    let extractor = DocExtractor::with_visibility(include_private, include_internal);
    let mut modules = Vec::new();

    for src_dir in src_dirs {
        for file in collect_source_files(src_dir, include, exclude) {
            let entries =
                normalize_doc_items(extractor.extract_file(Path::new(&file))?, type_parameters);
            if !entries.is_empty() {
                modules.push(ExtractedDocModule { file, entries });
            }
        }
    }

    Ok(modules)
}

fn collect_source_files_inner(
    dir: &Path,
    include: &[String],
    exclude: &[String],
    files: &mut Vec<String>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let path_str = path.to_string_lossy();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_dir() {
            if !is_excluded_source_path(&path_str, exclude) {
                collect_source_files_inner(&path, include, exclude, files);
            }
        } else if file_type.is_file()
            && is_included_source_path(&path_str, include)
            && !is_excluded_source_path(&path_str, exclude)
        {
            files.push(path_str.into_owned());
        }
    }
}

fn is_included_source_path(path: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| glob_match(pattern, path))
}

fn is_excluded_source_path(path: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| {
        if pattern.contains("node_modules") {
            return path.contains("node_modules");
        }
        if pattern.contains(".test.") || pattern.contains(".spec.") {
            return path.contains(".test.") || path.contains(".spec.");
        }
        glob_match(pattern, path)
    })
}

/// Simple glob matching (** and * patterns).
fn glob_match(pattern: &str, path: &str) -> bool {
    // Very simplified glob matching
    // TODO: Use a proper glob library
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            // For **, we just check the suffix pattern
            if !suffix.is_empty() {
                // Handle *.ext suffix
                if let Some(ext) = suffix.strip_prefix('*') {
                    return path.ends_with(ext);
                }
                return path.ends_with(suffix);
            }
            if !prefix.is_empty() && !path.starts_with(prefix) {
                return false;
            }
            return true;
        }
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            return path.starts_with(parts[0]) && path.ends_with(parts[1]);
        }
    }

    pattern == path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        // ** with *.ext suffix (matches any path ending with .ts)
        assert!(glob_match("**/*.ts", "src/foo/bar.ts"));
        assert!(glob_match("**/*.ts", "bar.ts"));
        assert!(!glob_match("**/*.ts", "bar.js"));
        // Single * pattern (prefix + suffix matching)
        assert!(glob_match("*.ts", "foo.ts"));
        // Note: our simple glob treats *.ts as "starts with '' and ends with .ts"
        // so src/foo.ts also matches (this is a limitation of the simple implementation)
        assert!(glob_match("*.ts", "src/foo.ts"));
        // Exact match
        assert!(glob_match("foo.ts", "foo.ts"));
        assert!(!glob_match("foo.ts", "bar.ts"));
    }

    #[test]
    fn collects_source_files_with_doc_filters() {
        let root =
            std::env::temp_dir().join(format!("ox-content-docs-files-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src/nested")).unwrap();
        std::fs::create_dir_all(root.join("src/node_modules/pkg")).unwrap();
        std::fs::write(root.join("src/index.ts"), "").unwrap();
        std::fs::write(root.join("src/nested/view.tsx"), "").unwrap();
        std::fs::write(root.join("src/nested/view.test.ts"), "").unwrap();
        std::fs::write(root.join("src/node_modules/pkg/index.ts"), "").unwrap();

        let include = vec!["**/*.ts".to_string(), "**/*.tsx".to_string()];
        let exclude = vec!["**/*.test.*".to_string(), "node_modules".to_string()];
        let files =
            collect_source_files(root.join("src").to_string_lossy().as_ref(), &include, &exclude);

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|file| file.ends_with("src/index.ts")));
        assert!(files.iter().any(|file| file.ends_with("src/nested/view.tsx")));

        let _ = std::fs::remove_dir_all(&root);
    }
}
