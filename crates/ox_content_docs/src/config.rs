//! Configuration for documentation generation.

use serde::{Deserialize, Serialize};

/// Configuration for documentation generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsConfig {
    /// Source directories to scan.
    pub src_dirs: Vec<String>,

    /// Output directory.
    pub out_dir: String,

    /// File patterns to include.
    pub include: Vec<String>,

    /// File patterns to exclude.
    pub exclude: Vec<String>,

    /// Whether to generate JSON output.
    pub json: bool,

    /// Whether to include private items.
    pub document_private: bool,

    /// Theme for the generated docs.
    pub theme: Option<String>,
}

impl Default for DocsConfig {
    fn default() -> Self {
        Self {
            src_dirs: Vec::from([String::from("src")]),
            out_dir: "docs".to_string(),
            include: Vec::from([
                String::from("**/*.ts"),
                String::from("**/*.tsx"),
                String::from("**/*.js"),
                String::from("**/*.jsx"),
            ]),
            exclude: Vec::from([
                String::from("**/node_modules/**"),
                String::from("**/dist/**"),
                String::from("**/*.test.*"),
                String::from("**/*.spec.*"),
            ]),
            json: false,
            document_private: false,
            theme: None,
        }
    }
}
