import * as vscode from "vscode";

import {
  COMMAND_INSERT_CALLOUT,
  COMMAND_INSERT_CODE_FENCE,
  COMMAND_INSERT_TABLE,
} from "./constants";
import { sendServerCommand } from "./client";

export async function runInsertCommand(command: string): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "markdown") {
    void vscode.window.showInformationMessage("Open a Markdown or .mdc document first.");
    return;
  }

  await sendServerCommand(command, [
    {
      uri: editor.document.uri.toString(),
      position: editor.selection.active,
    },
  ]);
}

export function insertionCommands(): Array<[string, () => Promise<void>]> {
  return [
    [COMMAND_INSERT_TABLE, () => runInsertCommand(COMMAND_INSERT_TABLE)],
    [COMMAND_INSERT_CODE_FENCE, () => runInsertCommand(COMMAND_INSERT_CODE_FENCE)],
    [COMMAND_INSERT_CALLOUT, () => runInsertCommand(COMMAND_INSERT_CALLOUT)],
  ];
}
