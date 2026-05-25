import * as vscode from "vscode";

import { COMMAND_OPEN_PREVIEW } from "./constants";
import { restartClient, startClient, stopClient } from "./client";
import { openPreview, refreshAllPreviews, schedulePreviewRefresh } from "./preview";

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  await startClient(context);

  // The insertion commands (`oxContent.insertTable`, etc.) are registered
  // by `vscode-languageclient` from the server's
  // `execute_command_provider` capability. The active-editor guard runs in
  // the client middleware (see `client.ts`). The extension only adds the
  // commands the server does not advertise — notably `oxContent.openPreview`,
  // which is a webview-only operation.
  context.subscriptions.push(
    vscode.commands.registerCommand(COMMAND_OPEN_PREVIEW, async () => {
      await openPreview(context);
    }),
    vscode.workspace.onDidChangeTextDocument((event) => {
      schedulePreviewRefresh(event.document);
    }),
    vscode.workspace.onDidSaveTextDocument((document) => {
      schedulePreviewRefresh(document);
    }),
    vscode.workspace.onDidCloseTextDocument((document) => {
      schedulePreviewRefresh(document, true);
    }),
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (!event.affectsConfiguration("oxContent")) {
        return;
      }

      await restartClient(context);
      await refreshAllPreviews();
    }),
  );
}

export async function deactivate(): Promise<void> {
  await stopClient();
}
