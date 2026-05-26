use std::collections::HashMap;

use tower_lsp::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, MessageType, Position, Range,
    TextEdit, Url, WorkspaceEdit,
};

use crate::preview;

use super::Backend;

pub(super) const COMMAND_INSERT_TABLE: &str = "oxContent.insertTable";
pub(super) const COMMAND_INSERT_CODE_FENCE: &str = "oxContent.insertCodeFence";
pub(super) const COMMAND_INSERT_CALLOUT: &str = "oxContent.insertCallout";
pub(super) const COMMAND_PREVIEW_HTML: &str = "oxContent.previewHtml";
pub(super) const COMMAND_PREVIEW_SUBSCRIBE: &str = "oxContent.previewSubscribe";
pub(super) const COMMAND_PREVIEW_UNSUBSCRIBE: &str = "oxContent.previewUnsubscribe";

/// Notification method pushed to the client whenever a subscribed
/// document's text changes. Payload matches `preview::PreviewPayload`
/// plus the originating URI so the client can route updates to the
/// right webview/panel.
pub(super) const NOTIFICATION_PREVIEW_DID_CHANGE: &str = "oxContent/previewDidChange";

#[derive(serde::Deserialize)]
struct EditCommandPayload {
    uri: String,
    position: Position,
}

impl Backend {
    pub(super) async fn insert_template(
        &self,
        command: &str,
        arguments: Vec<serde_json::Value>,
    ) -> Result<Option<serde_json::Value>> {
        let Some(payload) = arguments.first() else {
            return Ok(None);
        };
        let payload: EditCommandPayload = serde_json::from_value(payload.clone())
            .map_err(|_| Error::invalid_params("Invalid command payload"))?;
        let uri =
            Url::parse(&payload.uri).map_err(|_| Error::invalid_params("Invalid document URI"))?;
        let snippet = match command {
            COMMAND_INSERT_TABLE => "| Column | Column |\n| --- | --- |\n| Value | Value |\n",
            COMMAND_INSERT_CODE_FENCE => "```ts\nconst value = true;\n```\n",
            COMMAND_INSERT_CALLOUT => "> [!NOTE]\n> Add your note here.\n",
            _ => return Ok(None),
        };

        let mut changes = HashMap::new();
        changes.insert(
            uri,
            vec![TextEdit {
                range: Range { start: payload.position, end: payload.position },
                new_text: snippet.to_string(),
            }],
        );

        self.client
            .apply_edit(WorkspaceEdit { changes: Some(changes), ..Default::default() })
            .await?;
        Ok(None)
    }

    pub(super) async fn preview_html(
        &self,
        arguments: Vec<serde_json::Value>,
    ) -> Result<Option<serde_json::Value>> {
        let Some(argument) = arguments.first().and_then(serde_json::Value::as_str) else {
            return Err(Error::invalid_params("Missing preview URI"));
        };
        let uri = Url::parse(argument).map_err(|_| Error::invalid_params("Invalid preview URI"))?;
        let Some(document) = self.state.document(&uri).await else {
            return Ok(None);
        };

        let payload = preview::render_preview(document.text()).map_err(|error| Error {
            code: ErrorCode::InternalError,
            message: error.to_string().into(),
            data: None,
        })?;

        serde_json::to_value(payload).map(Some).map_err(|_| Error::internal_error())
    }

    /// Register the URI as a preview subscriber. Subsequent text
    /// changes will be pushed via
    /// [`NOTIFICATION_PREVIEW_DID_CHANGE`] until the client
    /// unsubscribes or closes the document. The current snapshot is
    /// returned so the client can paint the panel immediately without
    /// a follow-up `oxContent.previewHtml` round trip.
    pub(super) async fn preview_subscribe(
        &self,
        arguments: Vec<serde_json::Value>,
    ) -> Result<Option<serde_json::Value>> {
        let uri = parse_uri_argument(&arguments)?;
        self.state.subscribe_preview(uri.clone()).await;
        // Return the initial payload so the client can render before
        // the first push notification arrives.
        self.preview_html(arguments).await
    }

    pub(super) async fn preview_unsubscribe(
        &self,
        arguments: Vec<serde_json::Value>,
    ) -> Result<Option<serde_json::Value>> {
        let uri = parse_uri_argument(&arguments)?;
        self.state.unsubscribe_preview(&uri).await;
        Ok(None)
    }

    /// Re-render and push a preview update to any client that
    /// subscribed to this URI. No-op when no subscriber is listening,
    /// when the document is gone, or when the document is not
    /// renderable Markdown. Failures are logged through the client
    /// channel rather than surfaced — pushing diagnostics for an
    /// in-flight edit would only add noise.
    pub(super) async fn push_preview_update(&self, uri: &Url) {
        if !self.state.is_preview_subscribed(uri).await {
            return;
        }
        let Some(document) = self.state.document(uri).await else {
            return;
        };

        let payload = match preview::render_preview(document.text()) {
            Ok(payload) => payload,
            Err(error) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("ox-content preview render failed for {uri}: {error}"),
                    )
                    .await;
                return;
            }
        };

        let notification = PreviewDidChangeParams { uri: uri.to_string(), payload };
        self.client.send_notification::<PreviewDidChangeNotification>(notification).await;
    }
}

fn parse_uri_argument(arguments: &[serde_json::Value]) -> Result<Url> {
    let raw = arguments
        .first()
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| Error::invalid_params("Missing preview URI"))?;
    Url::parse(raw).map_err(|_| Error::invalid_params("Invalid preview URI"))
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct PreviewDidChangeParams {
    pub uri: String,
    #[serde(flatten)]
    pub payload: preview::PreviewPayload,
}

/// Strongly-typed `tower_lsp` notification handle. Splitting it out as
/// a dedicated zero-sized type keeps the method name centralized in
/// [`NOTIFICATION_PREVIEW_DID_CHANGE`] and makes it trivial to swap
/// transport details (e.g. payload version) later.
pub(super) enum PreviewDidChangeNotification {}

impl tower_lsp::lsp_types::notification::Notification for PreviewDidChangeNotification {
    type Params = PreviewDidChangeParams;
    const METHOD: &'static str = NOTIFICATION_PREVIEW_DID_CHANGE;
}

pub(super) fn insert_actions(uri: &Url, position: Position) -> Vec<CodeActionOrCommand> {
    [
        ("Insert table", COMMAND_INSERT_TABLE),
        ("Insert code fence", COMMAND_INSERT_CODE_FENCE),
        ("Insert callout", COMMAND_INSERT_CALLOUT),
    ]
    .into_iter()
    .map(|(title, command)| code_action(title, command, uri, position))
    .collect()
}

fn code_action(title: &str, command: &str, uri: &Url, position: Position) -> CodeActionOrCommand {
    CodeActionOrCommand::CodeAction(CodeAction {
        title: title.to_string(),
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        command: Some(Command {
            title: title.to_string(),
            command: command.to_string(),
            arguments: Some(vec![serde_json::json!({
                "uri": uri.to_string(),
                "position": position,
            })]),
        }),
        ..Default::default()
    })
}
