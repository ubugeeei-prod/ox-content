mod commands;
mod diagnostics;
mod features;
mod handlers;
mod snippets;

use tower_lsp::lsp_types::{Diagnostic, Url};
use tower_lsp::Client;

use crate::config::{InitializationOptions, ResolvedConfig};
use crate::document::{is_markdown_path, TextDocumentState};
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

    pub(super) async fn on_change(&self, uri: &Url, text: String) {
        let Some(path) = uri.to_file_path().ok() else {
            return;
        };

        if is_markdown_path(&path) {
            self.state.upsert_document(uri.clone(), text.clone()).await;
            self.publish_diagnostics_for(uri).await;
        }

        let path_str = path.to_string_lossy().to_string();
        if i18n::is_i18n_source_path(&path) {
            self.i18n_state.update_file_keys(&path_str, &text).await;
            self.publish_i18n_diagnostics().await;
        } else if i18n::is_i18n_dictionary_path(&path) {
            self.i18n_state.reload_dictionaries().await;
            self.publish_i18n_diagnostics().await;
        }
    }

    pub(super) async fn open_document(&self, uri: &Url, text: String) {
        if uri.to_file_path().ok().is_some_and(|path| i18n::is_i18n_source_path(&path)) {
            self.i18n_state.add_open_uri(uri.clone()).await;
        }
        self.on_change(uri, text).await;
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

    pub(super) async fn publish_diagnostics_for(&self, uri: &Url) {
        let Some(document) = self.state.document(uri).await else {
            return;
        };

        let diagnostics = self.diagnostics(&document).await;
        self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
    }

    async fn diagnostics(&self, document: &TextDocumentState) -> Vec<Diagnostic> {
        let config = self.resolved_config().await;
        let frontmatter = frontmatter::parse_frontmatter(document);
        let mut diagnostics = frontmatter
            .block
            .as_ref()
            .map_or_else(Vec::new, |block| Self::frontmatter_diagnostics(block, &config));
        diagnostics
            .extend(diagnostics::markdown_parse_diagnostics(document, frontmatter.block.as_ref()));
        diagnostics
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

    fn frontmatter_diagnostics(
        block: &frontmatter::FrontmatterBlock,
        config: &ResolvedConfig,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = block.diagnostics.clone();
        match Self::load_schema(config) {
            Ok(Some(schema)) => {
                diagnostics.extend(frontmatter::validate_frontmatter(block, &schema));
            }
            Ok(None) => {}
            Err(message) => diagnostics.push(Diagnostic {
                range: block.block_range,
                severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
                source: Some("ox-content".to_string()),
                message,
                ..Default::default()
            }),
        }
        diagnostics
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
            return Ok(None);
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
