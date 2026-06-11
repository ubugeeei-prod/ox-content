//! # ox_content_i18n_checker
//!
//! Static checker for i18n translation keys in Ox Content projects.
//!
//! Collects `t("key")` / `$t("key")` calls from TypeScript/JavaScript source files using
//! the OXC parser, and `{{t("key")}}` patterns from Markdown files. Then runs the collected
//! keys against loaded dictionaries to detect missing keys, unused keys, type mismatches,
//! and MF2 syntax errors.
//!
//! ## Usage
//!
//! ```no_run
//! use ox_content_i18n_checker::{check, CheckConfig};
//!
//! let config = CheckConfig::default();
//! let result = check(&config).expect("check failed");
//! println!("{} errors, {} warnings", result.error_count, result.warning_count);
//! ```

pub mod diagnostic;
pub mod key_collector;
pub mod md_key_collector;

use rustc_hash::FxHashSet;
use std::path::Path;

use ox_content_i18n::checker::{self, Diagnostic};
use ox_content_i18n::dictionary;

use key_collector::KeyCollector;

/// Configuration for the checker.
pub struct CheckConfig {
    /// Path to the i18n dictionary directory.
    pub dict_dir: String,
    /// Source directories to scan.
    pub src_dirs: Vec<String>,
    /// File extensions to scan.
    pub extensions: Vec<String>,
    /// Translation function names to look for.
    pub function_names: Vec<String>,
    /// Default locale.
    pub default_locale: Option<String>,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            dict_dir: "content/i18n".to_string(),
            src_dirs: vec!["src".to_string()],
            extensions: vec![
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
                "md".to_string(),
                "mdx".to_string(),
            ],
            function_names: vec!["t".to_string(), "$t".to_string()],
            default_locale: Some("en".to_string()),
        }
    }
}

/// Result of running the i18n check.
pub struct CheckResult {
    pub diagnostics: Vec<Diagnostic>,
    pub used_keys: FxHashSet<String>,
    pub error_count: usize,
    pub warning_count: usize,
}

/// Runs the full i18n check: collects keys from source, loads dictionaries, runs all rules.
pub fn check(config: &CheckConfig) -> Result<CheckResult, String> {
    // Load dictionaries
    let dict_path = Path::new(&config.dict_dir);
    let mut dict_set = dictionary::load_from_dir(dict_path)
        .map_err(|e| format!("failed to load dictionaries: {e}"))?;

    if let Some(ref locale_str) = config.default_locale {
        if let Ok(locale) = ox_content_i18n::Locale::new(locale_str) {
            dict_set.set_default_locale(locale);
        }
    }

    // Collect keys from source files
    let collector = if config.function_names.is_empty() {
        KeyCollector::new()
    } else {
        KeyCollector::with_function_names(config.function_names.clone())
    };

    let mut used_keys = FxHashSet::default();

    for src_dir in &config.src_dirs {
        collect_keys_recursive(Path::new(src_dir), &collector, &config.extensions, &mut used_keys)?;
    }

    // Run all checks
    let diagnostics = checker::check_all(&used_keys, &dict_set);

    let error_count = diagnostics.iter().filter(|d| d.severity == checker::Severity::Error).count();
    let warning_count =
        diagnostics.iter().filter(|d| d.severity == checker::Severity::Warning).count();

    Ok(CheckResult { diagnostics, used_keys, error_count, warning_count })
}

/// Recursively collects translation keys from files in a directory.
fn collect_keys_recursive(
    dir: &Path,
    collector: &KeyCollector,
    extensions: &[String],
    keys: &mut FxHashSet<String>,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("failed to read directory {}: {e}", dir.display()))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("directory entry error: {e}"))?;
        let path = entry.path();

        if path.is_dir() {
            // Skip node_modules and hidden directories
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if dir_name.starts_with('.') || dir_name == "node_modules" {
                continue;
            }
            collect_keys_recursive(&path, collector, extensions, keys)?;
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            if !extensions.iter().any(|e| e == ext) {
                continue;
            }

            // Markdown files: use md_key_collector
            if ext == "md" || ext == "mdx" {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let file_path = path.to_string_lossy().to_string();
                    let usages = md_key_collector::collect_md_keys(&content, &file_path);
                    for usage in usages {
                        keys.insert(usage.key);
                    }
                }
            } else {
                // TS/JS files: use OXC key collector
                if let Ok(usages) = collector.collect_file(&path) {
                    for usage in usages {
                        keys.insert(usage.key);
                    }
                } else {
                    // Skip files that fail to parse
                }
            }
        }
    }

    Ok(())
}
