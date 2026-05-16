import * as path from "node:path";
import * as vscode from "vscode";

import { getConfig } from "./config";
import { SERVER_COMMAND_PREVIEW_HTML } from "./constants";
import { sendServerCommand } from "./client";
import type { PreviewEntry, PreviewPayload } from "./types";

const previewEntries = new Map<string, PreviewEntry>();
const refreshTimers = new Map<string, NodeJS.Timeout>();

export async function openPreview(context: vscode.ExtensionContext): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "markdown") {
    void vscode.window.showInformationMessage("Open a Markdown or .mdc document first.");
    return;
  }

  const documentUri = editor.document.uri.toString();
  const existing = previewEntries.get(documentUri);
  if (existing) {
    existing.panel.reveal(vscode.ViewColumn.Beside, true);
    await updatePreview(existing.panel, editor.document);
    return;
  }

  const panel = vscode.window.createWebviewPanel(
    "oxContentPreview",
    "Ox Content Preview",
    { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true },
    { enableFindWidget: true, enableScripts: false, retainContextWhenHidden: true },
  );

  previewEntries.set(documentUri, { documentUri, panel });
  panel.onDidDispose(() => disposePreview(documentUri), null, context.subscriptions);
  await updatePreview(panel, editor.document);
}

export function schedulePreviewRefresh(document: vscode.TextDocument, dispose = false): void {
  const key = document.uri.toString();
  if (dispose) {
    disposePreview(key);
    return;
  }

  if (
    !getConfig().get<boolean>("preview.autoRefresh", true) ||
    document.languageId !== "markdown"
  ) {
    return;
  }

  const entry = previewEntries.get(key);
  if (!entry) {
    return;
  }

  clearTimer(key);
  refreshTimers.set(
    key,
    setTimeout(() => {
      void updatePreview(entry.panel, document);
    }, 150),
  );
}

export async function refreshAllPreviews(): Promise<void> {
  for (const entry of previewEntries.values()) {
    const document = await vscode.workspace.openTextDocument(vscode.Uri.parse(entry.documentUri));
    await updatePreview(entry.panel, document);
  }
}

async function updatePreview(
  panel: vscode.WebviewPanel,
  document: vscode.TextDocument,
): Promise<void> {
  try {
    const payload = await sendServerCommand<PreviewPayload>(SERVER_COMMAND_PREVIEW_HTML, [
      document.uri.toString(),
    ]);
    if (!payload) {
      panel.webview.html = errorHtml("Preview payload was empty.");
      return;
    }

    panel.title = payload.title || `${path.basename(document.fileName) || "Untitled"} Preview`;
    panel.webview.html = payload.html;
  } catch (error) {
    panel.webview.html = errorHtml(
      error instanceof Error ? error.message : "Failed to render preview.",
    );
  }
}

function disposePreview(key: string): void {
  previewEntries.delete(key);
  clearTimer(key);
}

function clearTimer(key: string): void {
  const timer = refreshTimers.get(key);
  if (timer) {
    clearTimeout(timer);
  }
  refreshTimers.delete(key);
}

function errorHtml(message: string): string {
  const escaped = message.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");

  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
      body { margin: 0; padding: 24px; font: 14px/1.6 ui-sans-serif, system-ui, sans-serif; background: #0f172a; color: #e2e8f0; }
      .card { max-width: 720px; margin: 0 auto; padding: 20px 22px; border-radius: 16px; border: 1px solid #334155; background: #111827; }
      h1 { margin-top: 0; font-size: 1rem; }
      code { color: #fda4af; }
    </style>
  </head>
  <body>
    <div class="card">
      <h1>Ox Content Preview</h1>
      <p>${escaped}</p>
      <p>Make sure <code>ox-content-lsp</code> is available or set <code>oxContent.server.path</code>.</p>
    </div>
  </body>
</html>`;
}
