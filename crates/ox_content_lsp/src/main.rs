//! # ox-content-lsp
//!
//! Unified Language Server Protocol server for Ox Content authoring.
//!
//! Provides:
//! - schema-aware frontmatter completion and diagnostics
//! - fast Markdown snippet completions
//! - editor-triggered insertion commands
//! - preview HTML generation via `workspace/executeCommand`
//! - heading symbols for document outline navigation
//! - folding ranges for headings, code blocks, and frontmatter
//! - document links for Markdown links and images
//! - document highlights for matching link/image targets
//! - smart selection ranges for expand-selection
//! - half-width/full-width spacing diagnostics and fixes

#[tokio::main]
async fn main() {
    ox_content_lsp::run().await;
}
