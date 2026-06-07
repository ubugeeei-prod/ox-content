/**
 * Pure preview policy extracted from `preview.ts`: which server command
 * seeds a panel, and how a panel title is chosen. Free of `vscode` so it
 * can be unit-tested under plain Node, the same split `paths.ts` and
 * `server-options.ts` follow.
 */

import * as path from "node:path";

import { SERVER_COMMAND_PREVIEW_HTML, SERVER_COMMAND_PREVIEW_SUBSCRIBE } from "../constants";

/**
 * Picks the server command used to seed a freshly opened preview panel.
 * Auto-refresh subscribes for pushed updates; otherwise we do a one-shot
 * HTML fetch.
 */
export function previewSeedCommand(autoRefresh: boolean): string {
  return autoRefresh ? SERVER_COMMAND_PREVIEW_SUBSCRIBE : SERVER_COMMAND_PREVIEW_HTML;
}

/**
 * Title for a freshly rendered preview. The LSP-supplied title wins;
 * when it's empty we fall back to the document's basename (or `Untitled`
 * for a buffer with no filename) plus a ` Preview` suffix.
 */
export function previewPanelTitle(payloadTitle: string, fileName: string): string {
  if (payloadTitle) {
    return payloadTitle;
  }
  return `${path.basename(fileName) || "Untitled"} Preview`;
}

/**
 * Title for a pushed `previewDidChange` update. The server only re-titles
 * when it has something to say; an empty pushed title leaves the existing
 * panel title untouched.
 */
export function pushedPreviewTitle(pushedTitle: string, currentTitle: string): string {
  return pushedTitle || currentTitle;
}
