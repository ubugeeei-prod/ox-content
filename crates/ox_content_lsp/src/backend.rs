mod assets;
mod commands;
mod diagnostics;
mod features;
mod handlers;
mod mdc;
mod snippets;

use tower_lsp::Client;
use tower_lsp::lsp_types::{Diagnostic, TextDocumentContentChangeEvent, Url};

use crate::config::{InitializationOptions, ResolvedConfig};
use crate::document::is_markdown_path;
use crate::frontmatter::{self, FrontmatterSchema};
use crate::i18n::{self, I18nState};
use crate::state::LspState;

pub struct Backend {
    pub(super) client: Client,
    pub(super) i18n_state: I18nState,
    pub(super) state: LspState,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self { client, i18n_state: I18nState::new(), state: LspState::new() }
    }

    pub(super) async fn open_document(&self, uri: &Url, text: String, version: i32) {
        if uri.to_file_path().is_ok_and(|path| i18n::is_i18n_source_path(&path)) {
            self.i18n_state.add_open_uri(uri.clone()).await;
        }
        self.state.upsert_document(uri.clone(), text.clone(), version).await;
        self.after_document_edit(uri, &text).await;
    }

    pub(super) async fn change_document(
        &self,
        uri: &Url,
        version: i32,
        changes: &[TextDocumentContentChangeEvent],
    ) {
        let Some(document) = self.state.apply_changes(uri.clone(), version, changes).await else {
            return;
        };
        self.after_document_edit(uri, document.text()).await;
    }

    async fn after_document_edit(&self, uri: &Url, text: &str) {
        let Some(path) = uri.to_file_path().ok() else {
            return;
        };

        if is_markdown_path(&path) {
            self.publish_diagnostics_for(uri).await;
            // HMR: any subscribed preview panel needs the rendered HTML
            // pushed to it; this replaces the polling
            // `onDidChangeTextDocument` refresh editors used to do. The
            // helper short-circuits silently when no client is listening.
            self.push_preview_update(uri).await;
        }

        let path_str = path.to_string_lossy().to_string();
        if i18n::is_i18n_source_path(&path) {
            self.i18n_state.update_file_keys(&path_str, text).await;
            self.publish_i18n_diagnostics().await;
        } else if i18n::is_i18n_dictionary_path(&path) {
            self.i18n_state.reload_dictionaries().await;
            self.publish_i18n_diagnostics().await;
        }
    }

    pub(super) async fn close_document(&self, uri: &Url) {
        self.state.remove_document(uri).await;

        if let Ok(path) = uri.to_file_path() {
            let path_str = path.to_string_lossy().to_string();
            if i18n::is_i18n_source_path(&path) {
                self.i18n_state.remove_file(&path_str).await;
                self.publish_i18n_diagnostics().await;
            }
        }

        self.client.publish_diagnostics(uri.clone(), Vec::new(), None).await;
    }

    pub(super) async fn spacing_formatting_edits(
        &self,
        uri: &Url,
    ) -> Option<Vec<tower_lsp::lsp_types::TextEdit>> {
        let path = uri.to_file_path().ok()?;
        if !is_markdown_path(&path) {
            return None;
        }
        let document = self.state.document(uri).await?;
        let config = self.resolved_config().await;
        let frontmatter = frontmatter::parse_frontmatter(&document);
        let edits =
            crate::spacing::formatting_edits(&document, frontmatter.block.as_ref(), config.spacing);
        (!edits.is_empty()).then_some(edits)
    }

    pub(super) async fn publish_i18n_diagnostics(&self) {
        let checker_diags = self.i18n_state.check_diagnostics().await;
        for uri in &self.i18n_state.get_open_uris().await {
            let Ok(path) = uri.to_file_path() else {
                continue;
            };
            let path_str = path.to_string_lossy().to_string();
            let usages = self.i18n_state.get_file_key_usages(&path_str).await;
            let mut diagnostics = Vec::new();

            for usage in &usages {
                for diag in &checker_diags {
                    if diag.key.as_deref() == Some(&usage.key) {
                        diagnostics.push(Diagnostic {
                            range: tower_lsp::lsp_types::Range {
                                start: tower_lsp::lsp_types::Position {
                                    line: usage.line - 1,
                                    character: usage.column - 1,
                                },
                                end: tower_lsp::lsp_types::Position {
                                    line: usage.line - 1,
                                    character: usage.end_column - 1,
                                },
                            },
                            severity: Some(match diag.severity {
                                ox_content_i18n::checker::Severity::Error => {
                                    tower_lsp::lsp_types::DiagnosticSeverity::ERROR
                                }
                                ox_content_i18n::checker::Severity::Warning => {
                                    tower_lsp::lsp_types::DiagnosticSeverity::WARNING
                                }
                                ox_content_i18n::checker::Severity::Info => {
                                    tower_lsp::lsp_types::DiagnosticSeverity::INFORMATION
                                }
                            }),
                            source: Some("ox-content-i18n".to_string()),
                            message: diag.message.clone(),
                            ..Default::default()
                        });
                    }
                }
            }

            self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
        }
    }

    pub(super) async fn resolved_config(&self) -> ResolvedConfig {
        let root = self.state.root().await;
        let init = self.state.init_options().await;
        ResolvedConfig::load(root.as_deref(), &init)
    }

    pub(super) fn load_schema(
        config: &ResolvedConfig,
    ) -> std::result::Result<Option<FrontmatterSchema>, String> {
        let Some(path) = &config.frontmatter_schema else {
            return Ok(Some(frontmatter::builtin_schema()));
        };
        if !path.exists() {
            return Err(format!(
                "Configured frontmatter schema does not exist: {}",
                path.display()
            ));
        }
        frontmatter::load_schema(path).map(Some)
    }

    pub(super) async fn init_from_params(&self, params: &tower_lsp::lsp_types::InitializeParams) {
        let root = params.root_uri.as_ref().and_then(|uri| uri.to_file_path().ok()).or_else(|| {
            params.workspace_folders.as_ref().and_then(|folders| {
                folders.first().and_then(|folder| folder.uri.to_file_path().ok())
            })
        });
        let init_options = params
            .initialization_options
            .clone()
            .and_then(|value| serde_json::from_value::<InitializationOptions>(value).ok())
            .unwrap_or_default();

        self.state.set_root(root.clone()).await;
        self.i18n_state.set_root(root).await;
        self.state.set_init_options(init_options).await;
    }
}
