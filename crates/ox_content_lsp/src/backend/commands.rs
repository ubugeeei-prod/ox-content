use std::collections::HashMap;

use tower_lsp::jsonrpc::{Error, ErrorCode, Result};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Position, Range, TextEdit, Url,
    WorkspaceEdit,
};

use crate::preview;

use super::Backend;

pub(super) const COMMAND_INSERT_TABLE: &str = "oxContent.insertTable";
pub(super) const COMMAND_INSERT_CODE_FENCE: &str = "oxContent.insertCodeFence";
pub(super) const COMMAND_INSERT_CALLOUT: &str = "oxContent.insertCallout";
pub(super) const COMMAND_PREVIEW_HTML: &str = "oxContent.previewHtml";

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
