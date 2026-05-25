import * as vscode from "vscode";
import { LanguageClient, type LanguageClientOptions } from "vscode-languageclient/node";

import { resolveInitializationOptions, resolveServerOptions } from "./config";
import {
  COMMAND_INSERT_CALLOUT,
  COMMAND_INSERT_CODE_FENCE,
  COMMAND_INSERT_TABLE,
} from "./constants";

let client: LanguageClient | undefined;

// LSP-served commands that should only run while a Markdown buffer is the
// active editor. The middleware below short-circuits with a hint when the
// guard fails, before forwarding to the server.
const EDITOR_GUARDED_COMMANDS = new Set<string>([
  COMMAND_INSERT_TABLE,
  COMMAND_INSERT_CODE_FENCE,
  COMMAND_INSERT_CALLOUT,
]);

export async function startClient(context: vscode.ExtensionContext): Promise<LanguageClient> {
  if (client) {
    return client;
  }

  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { language: "markdown", scheme: "file" },
      { language: "markdown", scheme: "untitled" },
      { language: "javascript", scheme: "file" },
      { language: "javascriptreact", scheme: "file" },
      { language: "typescript", scheme: "file" },
      { language: "typescriptreact", scheme: "file" },
      { language: "json", scheme: "file" },
      { language: "yaml", scheme: "file" },
    ],
    initializationOptions: resolveInitializationOptions(workspaceRoot),
    synchronize: { configurationSection: "oxContent" },
    // The LSP advertises insertTable / insertCodeFence / insertCallout in
    // its `execute_command_provider` capability, which makes
    // vscode-languageclient register them as VS Code commands. Without
    // this middleware the command would be invoked with empty arguments
    // when no Markdown editor is active. Wrap it so we can attach the
    // editor's URI + cursor position (matching what `commands.ts` used
    // to do via a redundant `registerCommand`) and surface a friendly
    // hint when the user has no Markdown buffer open.
    middleware: {
      executeCommand: async (command, args, next) => {
        if (EDITOR_GUARDED_COMMANDS.has(command)) {
          const editor = vscode.window.activeTextEditor;
          if (!editor || editor.document.languageId !== "markdown") {
            void vscode.window.showInformationMessage(
              "Open a Markdown or .mdc document first.",
            );
            return undefined;
          }
          return next(command, [
            {
              uri: editor.document.uri.toString(),
              position: editor.selection.active,
            },
          ]);
        }
        return next(command, args);
      },
    },
  };

  client = new LanguageClient(
    "oxContent",
    "Ox Content",
    resolveServerOptions(context, workspaceRoot),
    clientOptions,
  );

  await client.start();
  context.subscriptions.push({ dispose: () => void client?.stop() });
  return client;
}

export async function restartClient(context: vscode.ExtensionContext): Promise<LanguageClient> {
  await stopClient();
  return startClient(context);
}

export async function stopClient(): Promise<void> {
  const current = client;
  client = undefined;
  if (current) {
    await current.stop();
  }
}

export async function sendServerCommand<T = unknown>(
  command: string,
  args: unknown[],
): Promise<T | undefined> {
  if (!client) {
    throw new Error("Ox Content language client is not running.");
  }

  return client.sendRequest<T | undefined>("workspace/executeCommand", {
    command,
    arguments: args,
  });
}
