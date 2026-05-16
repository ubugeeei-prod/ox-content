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

mod backend;
mod config;
mod document;
mod frontmatter;
mod i18n;
mod preview;
mod state;

use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(backend::Backend::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}
