use std::fmt::Write as _;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document;
use crate::state::LspState;

pub struct Backend {
    pub client: Client,
    pub state: LspState,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self { client, state: LspState::new() }
    }

    async fn on_change(&self, uri: &Url, text: &str) {
        if let Ok(path) = uri.to_file_path() {
            let path_str = path.to_string_lossy().to_string();

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            match ext {
                "ts" | "tsx" | "js" | "jsx" => {
                    self.state.update_file_keys(&path_str, text).await;
                }
                "json" | "yaml" | "yml" => {
                    // Dictionary file changed — reload dictionaries
                    self.state.reload_dictionaries().await;
                }
                _ => {}
            }
        }

        self.publish_diagnostics().await;
    }

    async fn publish_diagnostics(&self) {
        let checker_diags = self.state.check_diagnostics().await;
        let open_uris = self.state.get_open_uris().await;

        for uri in &open_uris {
            let Ok(path) = uri.to_file_path() else {
                continue;
            };
            let path_str = path.to_string_lossy().to_string();
            let usages = self.state.get_file_key_usages(&path_str).await;

            let mut lsp_diags = Vec::new();

            for usage in &usages {
                // Check if this key has a "missing" diagnostic
                for diag in &checker_diags {
                    if diag.key.as_deref() == Some(&usage.key) {
                        let severity = match diag.severity {
                            ox_content_i18n::checker::Severity::Error => DiagnosticSeverity::ERROR,
                            ox_content_i18n::checker::Severity::Warning => {
                                DiagnosticSeverity::WARNING
                            }
                            ox_content_i18n::checker::Severity::Info => {
                                DiagnosticSeverity::INFORMATION
                            }
                        };

                        lsp_diags.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: usage.line - 1,
                                    character: usage.column - 1,
                                },
                                end: Position {
                                    line: usage.line - 1,
                                    character: usage.end_column - 1,
                                },
                            },
                            severity: Some(severity),
                            source: Some("ox-content-i18n".to_string()),
                            message: diag.message.clone(),
                            ..Default::default()
                        });
                    }
                }
            }

            self.client.publish_diagnostics(uri.clone(), lsp_diags, None).await;
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Set workspace root
        if let Some(root_uri) = params.root_uri {
            if let Ok(root_path) = root_uri.to_file_path() {
                self.state.set_root(root_path).await;
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["'".to_string(), "\"".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "ox-content-i18n LSP initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.state.add_open_uri(params.text_document.uri.clone()).await;
        self.on_change(&params.text_document.uri, &params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.first() {
            self.on_change(&params.text_document.uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.state.remove_open_uri(&params.text_document.uri).await;
        if let Ok(path) = params.text_document.uri.to_file_path() {
            self.state.remove_file(&path.to_string_lossy()).await;
        }
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let keys = self.state.all_dictionary_keys().await;

        let items: Vec<CompletionItem> = keys
            .into_iter()
            .map(|key| CompletionItem {
                label: key,
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("i18n translation key".to_string()),
                ..Default::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Ok(path) = uri.to_file_path() else {
            return Ok(None);
        };
        let path_str = path.to_string_lossy().to_string();

        let usages = self.state.get_file_key_usages(&path_str).await;
        let Some(key) = document::key_at_position(&usages, position) else {
            return Ok(None);
        };

        let translations = self.state.translations_for_key(&key).await;
        if translations.is_empty() {
            return Ok(None);
        }

        let mut md = String::new();
        push_fmt(
            &mut md,
            format_args!("**`{key}`**\n\n| Locale | Translation |\n|--------|-------------|\n"),
        );
        for (locale, value) in &translations {
            push_fmt(&mut md, format_args!("| `{locale}` | {value} |\n"));
        }

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: md,
            }),
            range: None,
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Ok(path) = uri.to_file_path() else {
            return Ok(None);
        };
        let path_str = path.to_string_lossy().to_string();

        let usages = self.state.get_file_key_usages(&path_str).await;
        let Some(key) = document::key_at_position(&usages, position) else {
            return Ok(None);
        };

        let Some((dict_file, _locale)) = self.state.find_key_definition(&key).await else {
            return Ok(None);
        };

        let line = document::find_key_line_in_file(&dict_file, &key).unwrap_or(0);

        let target_uri = Url::from_file_path(&dict_file)
            .map_err(|()| tower_lsp::jsonrpc::Error::invalid_params("Invalid file path"))?;

        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri: target_uri,
            range: Range {
                start: Position { line, character: 0 },
                end: Position { line, character: 0 },
            },
        })))
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = &params.text_document.uri;

        let Ok(path) = uri.to_file_path() else {
            return Ok(None);
        };
        let path_str = path.to_string_lossy().to_string();

        let usages = self.state.get_file_key_usages(&path_str).await;
        if usages.is_empty() {
            return Ok(None);
        }

        let mut hints = Vec::new();
        for usage in &usages {
            if let Some(translation) = self.state.default_translation(&usage.key).await {
                let label = if translation.len() > 40 {
                    format!(" {}...", &translation[..37])
                } else {
                    format!(" {translation}")
                };

                hints.push(InlayHint {
                    position: Position {
                        line: usage.line - 1, // KeyUsage is 1-based, LSP is 0-based
                        character: usage.end_column - 1,
                    },
                    label: InlayHintLabel::String(label),
                    kind: Some(InlayHintKind::PARAMETER),
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(true),
                    padding_right: None,
                    data: None,
                });
            }
        }

        Ok(Some(hints))
    }
}

fn push_fmt(output: &mut String, args: std::fmt::Arguments<'_>) {
    if output.write_fmt(args).is_err() {
        output.push_str("[formatting failed]");
    }
}
