use rustc_hash::{FxHashMap, FxHashSet};
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
    documents: FxHashMap<Url, TextDocumentState>,
    root: Option<PathBuf>,
    init_options: InitializationOptions,
    /// Set of document URIs the client has subscribed to preview updates
    /// for. When a subscribed document changes, the backend re-renders
    /// and pushes a `oxContent/previewDidChange` notification, replacing
    /// the polling-style refresh editors used to do client-side.
    preview_subscriptions: FxHashSet<Url>,
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
        // A closed document cannot push preview updates anymore. Drop
        // the subscription so the client doesn't leak state into the
        // next session for the same URI.
        inner.preview_subscriptions.remove(uri);
    }

    pub async fn document(&self, uri: &Url) -> Option<TextDocumentState> {
        let inner = self.inner.read().await;
        inner.documents.get(uri).cloned()
    }

    /// Mark `uri` as receiving preview push notifications. Returns the
    /// previous subscription state so callers can distinguish "newly
    /// subscribed" from "already subscribed".
    pub async fn subscribe_preview(&self, uri: Url) -> bool {
        let mut inner = self.inner.write().await;
        inner.preview_subscriptions.insert(uri)
    }

    /// Drop a preview subscription. Returns whether the URI was actually
    /// subscribed (so callers can no-op silently on double-unsubscribe).
    pub async fn unsubscribe_preview(&self, uri: &Url) -> bool {
        let mut inner = self.inner.write().await;
        inner.preview_subscriptions.remove(uri)
    }

    pub async fn is_preview_subscribed(&self, uri: &Url) -> bool {
        let inner = self.inner.read().await;
        inner.preview_subscriptions.contains(uri)
    }

    #[cfg(test)]
    pub async fn preview_subscription_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner.preview_subscriptions.len()
    }
}

impl Default for LspState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::Url;

    fn uri(path: &str) -> Url {
        Url::parse(&format!("file://{path}")).expect("valid file url")
    }

    #[tokio::test]
    async fn subscribe_preview_is_idempotent() {
        let state = LspState::new();
        let u = uri("/tmp/a.md");

        assert!(state.subscribe_preview(u.clone()).await, "first subscribe should report insert");
        assert!(
            !state.subscribe_preview(u.clone()).await,
            "second subscribe should report no-change",
        );
        assert!(state.is_preview_subscribed(&u).await);
        assert_eq!(state.preview_subscription_count().await, 1);
    }

    #[tokio::test]
    async fn unsubscribe_preview_returns_previous_state() {
        let state = LspState::new();
        let u = uri("/tmp/b.md");

        assert!(!state.unsubscribe_preview(&u).await, "unsubscribe on empty set is a no-op");
        state.subscribe_preview(u.clone()).await;
        assert!(state.unsubscribe_preview(&u).await);
        assert!(!state.is_preview_subscribed(&u).await);
        assert_eq!(state.preview_subscription_count().await, 0);
    }

    #[tokio::test]
    async fn close_document_drops_preview_subscription() {
        let state = LspState::new();
        let u = uri("/tmp/c.md");

        state.upsert_document(u.clone(), "# hi".into()).await;
        state.subscribe_preview(u.clone()).await;
        assert!(state.is_preview_subscribed(&u).await);

        state.remove_document(&u).await;
        assert!(
            !state.is_preview_subscribed(&u).await,
            "removing a document must also drop its preview subscription",
        );
    }
}
