import {
  COMMAND_INSERT_CALLOUT,
  COMMAND_INSERT_CODE_FENCE,
  COMMAND_INSERT_TABLE,
} from "../constants";

/**
 * LSP-served commands that should only run while a Markdown buffer is
 * the active editor. The middleware in `client.ts` short-circuits with
 * a hint when the guard fails, before forwarding to the server.
 *
 * Pure data so it can be asserted from unit tests without spinning up
 * a `LanguageClient`.
 */
export const EDITOR_GUARDED_COMMANDS: ReadonlySet<string> = new Set([
  COMMAND_INSERT_TABLE,
  COMMAND_INSERT_CODE_FENCE,
  COMMAND_INSERT_CALLOUT,
]);

/**
 * True when this command must only be dispatched from a Markdown
 * editor. Centralized so we have one place to update if the LSP starts
 * advertising new editor-guarded commands.
 */
export function requiresMarkdownEditor(command: string): boolean {
  return EDITOR_GUARDED_COMMANDS.has(command);
}
