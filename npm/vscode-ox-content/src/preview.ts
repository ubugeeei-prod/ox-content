import * as vscode from "vscode";

import { getConfig } from "./config";
import { NOTIFICATION_PREVIEW_DID_CHANGE, SERVER_COMMAND_PREVIEW_UNSUBSCRIBE } from "./constants";
import { onServerNotification, sendServerCommand } from "./client";
import { errorHtml } from "./internal/preview-html";
import {
  previewPanelTitle,
  previewSeedCommand,
  pushedPreviewTitle,
} from "./internal/preview-state";
import type { PreviewEntry, PreviewPayload } from "./types";

const previewEntries = new Map<string, PreviewEntry>();
let notificationSubscription: vscode.Disposable | undefined;

type PreviewDidChangeParams = PreviewPayload & { uri: string };

/**
 * Register the `oxContent/previewDidChange` notification handler against
 * the currently running LSP client. Safe to call after every
 * (re)startClient: the previous handler is disposed first so we never
 * stack listeners across restarts.
 */
export function registerPreviewListeners(context: vscode.ExtensionContext): void {
  notificationSubscription?.dispose();
  try {
    notificationSubscription = onServerNotification<PreviewDidChangeParams>(
      NOTIFICATION_PREVIEW_DID_CHANGE,
      (params) => applyPushedPreview(params),
    );
    context.subscriptions.push({
      dispose: () => notificationSubscription?.dispose(),
    });
  } catch {
    // The client may not be ready yet (e.g. tests that don't activate
    // the extension fully). Subscriptions will be re-registered on the
    // next startClient cycle.
  }
}

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
    await renderInitialPreview(existing.panel, editor.document);
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
  await renderInitialPreview(panel, editor.document);
}

/**
 * Force every open preview panel to re-fetch its HTML. Called after a
 * config change restarts the LSP — the previous subscriptions are gone
 * and the new server hasn't pushed anything yet, so we reseed each panel
 * by re-subscribing.
 */
export async function refreshAllPreviews(): Promise<void> {
  for (const entry of previewEntries.values()) {
    const document = await vscode.workspace.openTextDocument(vscode.Uri.parse(entry.documentUri));
    await renderInitialPreview(entry.panel, document);
  }
}

async function renderInitialPreview(
  panel: vscode.WebviewPanel,
  document: vscode.TextDocument,
): Promise<void> {
  const command = previewSeedCommand(previewAutoRefreshEnabled());

  try {
    const payload = await sendServerCommand<PreviewPayload>(command, [document.uri.toString()]);
    if (!payload) {
      panel.webview.html = errorHtml("Preview payload was empty.");
      return;
    }
    applyPayload(panel, document, payload);
  } catch (error) {
    panel.webview.html = errorHtml(
      error instanceof Error ? error.message : "Failed to render preview.",
    );
  }
}

function applyPushedPreview(params: PreviewDidChangeParams): void {
  const entry = previewEntries.get(params.uri);
  if (!entry) {
    return;
  }
  entry.panel.title = pushedPreviewTitle(params.title, entry.panel.title);
  entry.panel.webview.html = params.html;
}

function applyPayload(
  panel: vscode.WebviewPanel,
  document: vscode.TextDocument,
  payload: PreviewPayload,
): void {
  panel.title = previewPanelTitle(payload.title, document.fileName);
  panel.webview.html = payload.html;
}

function disposePreview(key: string): void {
  const entry = previewEntries.get(key);
  previewEntries.delete(key);
  if (entry && previewAutoRefreshEnabled()) {
    // Best-effort unsubscribe; the LSP also drops subscriptions on
    // `did_close`, so an error here is not user-visible.
    void sendServerCommand(SERVER_COMMAND_PREVIEW_UNSUBSCRIBE, [key]).catch(() => undefined);
  }
}

function previewAutoRefreshEnabled(): boolean {
  return getConfig().get<boolean>("preview.autoRefresh", true);
}
