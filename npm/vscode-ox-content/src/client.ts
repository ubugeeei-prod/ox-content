import * as vscode from "vscode";
import { LanguageClient, type LanguageClientOptions } from "vscode-languageclient/node";

import { resolveInitializationOptions, resolveServerOptions } from "./config";

let client: LanguageClient | undefined;

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
