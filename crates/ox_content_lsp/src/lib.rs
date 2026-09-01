//! Unified Language Server Protocol server for Ox Content authoring.

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
mod session;
mod spacing;
mod state;
mod textlint;

use tower_lsp::{LspService, Server};

/// Runs the Ox Content language server over standard input and output.
pub async fn run() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(backend::Backend::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}
