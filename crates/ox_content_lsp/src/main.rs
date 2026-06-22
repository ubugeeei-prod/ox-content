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

mod backend;
mod config;
mod document;
mod document_highlight;
mod document_link;
mod folding;
mod frontmatter;
mod i18n;
mod preview;
mod selection_range;
mod spacing;
mod state;
mod textlint;

use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(backend::Backend::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}
