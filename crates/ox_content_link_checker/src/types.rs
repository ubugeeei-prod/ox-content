use std::path::PathBuf;

use serde::Serialize;

/// Kind of link target that resolution can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkKind {
    /// `[text](./foo.md)` or `<img src="foo.png">`.
    File,
    /// `[text](#anchor)`.
    Anchor,
    /// `[text](./foo.md#anchor)`.
    FileAnchor,
    /// `[text](https://...)` - passed through, never asserted unless
    /// HTTP checking is enabled (currently always disabled).
    External,
    /// `[text](mailto:a@b.example)` etc.
    Scheme,
    /// Could not be parsed (e.g. malformed `mailto:` payload).
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub kind: LinkKind,
    /// The raw URL or reference identifier as written in the source.
    pub target: String,
}

/// Inputs that change link resolution outcomes. Constructed by the
/// caller (CLI / LSP) before each `check_source` call.
#[derive(Debug, Clone)]
pub struct CheckOptions {
    /// Absolute path of the file being checked. Used to resolve
    /// relative paths and to skip self-anchor warnings.
    pub file_path: PathBuf,
    /// Treated as the root of paths that start with `/`. Defaults to
    /// the file's directory when `None`.
    pub src_dir: Option<PathBuf>,
    /// Patterns whose match short-circuits diagnostics. Each entry is
    /// matched against the raw target string via plain `contains`; this
    /// is intentionally simple so the LSP/CLI surface can layer
    /// glob/regex on top without changing the checker.
    pub ignore_patterns: Vec<String>,
}

impl CheckOptions {
    pub fn for_file(file_path: PathBuf) -> Self {
        Self { file_path, src_dir: None, ignore_patterns: Vec::new() }
    }
}
