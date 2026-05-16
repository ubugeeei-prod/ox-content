use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

use crate::document::is_markdown_path;

use super::commands::{
    insert_actions, COMMAND_INSERT_CALLOUT, COMMAND_INSERT_CODE_FENCE, COMMAND_INSERT_TABLE,
    COMMAND_PREVIEW_HTML,
};
use super::Backend;

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.init_from_params(&params).await;
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "\"".into(),
                        "'".into(),
                        "#".into(),
                        "-".into(),
                        "[".into(),
                        "`".into(),
                        "!".into(),
                        ">".into(),
                        "|".into(),
                        ":".into(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        COMMAND_INSERT_TABLE.into(),
                        COMMAND_INSERT_CODE_FENCE.into(),
                        COMMAND_INSERT_CALLOUT.into(),
                        COMMAND_PREVIEW_HTML.into(),
                    ],
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "ox-content-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "ox-content LSP initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.open_document(&params.text_document.uri, params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.first() {
            self.on_change(&params.text_document.uri, change.text.clone()).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.close_document(&params.text_document.uri).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(self
            .completion_response(
                &params.text_document_position.text_document.uri,
                params.text_document_position.position,
            )
            .await)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        Ok(self
            .hover_response(
                &params.text_document_position_params.text_document.uri,
                params.text_document_position_params.position,
            )
            .await)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        Ok(self
            .goto_definition_response(
                &params.text_document_position_params.text_document.uri,
                params.text_document_position_params.position,
            )
            .await)
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        Ok(self.inlay_hints(&params.text_document.uri).await)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        Ok(self.document_symbols_response(&params.text_document.uri).await)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        if params.text_document.uri.to_file_path().ok().is_some_and(|path| !is_markdown_path(&path))
        {
            return Ok(None);
        }
        Ok(Some(insert_actions(&params.text_document.uri, params.range.start)))
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            COMMAND_INSERT_TABLE | COMMAND_INSERT_CODE_FENCE | COMMAND_INSERT_CALLOUT => {
                self.insert_template(&params.command, params.arguments).await
            }
            COMMAND_PREVIEW_HTML => self.preview_html(params.arguments).await,
            _ => Ok(None),
        }
    }
}
