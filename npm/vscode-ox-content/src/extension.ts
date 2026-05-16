import * as vscode from "vscode";

import { COMMAND_OPEN_PREVIEW } from "./constants";
import { insertionCommands } from "./commands";
import { restartClient, startClient, stopClient } from "./client";
import { openPreview, refreshAllPreviews, schedulePreviewRefresh } from "./preview";

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  await startClient(context);

  context.subscriptions.push(
    ...insertionCommands().map(([command, handler]) =>
      vscode.commands.registerCommand(command, handler),
    ),
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
