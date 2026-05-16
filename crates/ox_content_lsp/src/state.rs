use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp::lsp_types::Url;

use crate::config::InitializationOptions;
use crate::document::TextDocumentState;

#[derive(Clone)]
pub struct LspState {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default)]
struct Inner {
    documents: HashMap<Url, TextDocumentState>,
    root: Option<PathBuf>,
    init_options: InitializationOptions,
}

impl LspState {
    #[must_use]
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(Inner::default())) }
    }

    pub async fn set_root(&self, root: Option<PathBuf>) {
        let mut inner = self.inner.write().await;
        inner.root = root;
    }

    pub async fn set_init_options(&self, init_options: InitializationOptions) {
        let mut inner = self.inner.write().await;
        inner.init_options = init_options;
    }

    pub async fn root(&self) -> Option<PathBuf> {
        let inner = self.inner.read().await;
        inner.root.clone()
    }

    pub async fn init_options(&self) -> InitializationOptions {
        let inner = self.inner.read().await;
        inner.init_options.clone()
    }

    pub async fn upsert_document(&self, uri: Url, text: String) {
        let mut inner = self.inner.write().await;
        inner.documents.insert(uri, TextDocumentState::new(text));
    }

    pub async fn remove_document(&self, uri: &Url) {
        let mut inner = self.inner.write().await;
        inner.documents.remove(uri);
    }

    pub async fn document(&self, uri: &Url) -> Option<TextDocumentState> {
        let inner = self.inner.read().await;
        inner.documents.get(uri).cloned()
    }
}

impl Default for LspState {
    fn default() -> Self {
        Self::new()
    }
}
