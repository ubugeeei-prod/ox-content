import * as vscode from "vscode";

import { COMMAND_OPEN_PREVIEW } from "./constants";
import { restartClient, startClient, stopClient } from "./client";
import { openPreview, refreshAllPreviews, registerPreviewListeners } from "./preview";

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  await startClient(context);
  // Wire the `oxContent/previewDidChange` push channel as soon as the
  // client is up. The handler is re-registered after every restart so
  // it survives `oxContent` configuration changes.
  registerPreviewListeners(context);

  // The insertion commands (`oxContent.insertTable`, etc.) are registered
  // by `vscode-languageclient` from the server's
  // `execute_command_provider` capability. The active-editor guard runs in
  // the client middleware (see `client.ts`). The extension only adds the
  // commands the server does not advertise — notably `oxContent.openPreview`,
  // which is a webview-only operation.
  //
  // Preview HMR is server-driven: opening the preview subscribes the URI
  // with the LSP, the LSP pushes `oxContent/previewDidChange` on every
  // text change, and the panel reapplies the HTML. The extension no
  // longer needs `onDidChangeTextDocument` debouncing.
  context.subscriptions.push(
    vscode.commands.registerCommand(COMMAND_OPEN_PREVIEW, async () => {
      await openPreview(context);
    }),
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (!event.affectsConfiguration("oxContent")) {
        return;
      }

      await restartClient(context);
      registerPreviewListeners(context);
      await refreshAllPreviews();
    }),
  );
}

export async function deactivate(): Promise<void> {
  await stopClient();
}
