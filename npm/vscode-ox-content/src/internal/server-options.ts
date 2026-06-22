/**
 * Pure decision logic extracted from `config.ts`.
 *
 * Keeping the precedence rules (which binary wins) and the
 * initialization-options mapping (which settings get forwarded to the
 * LSP) free of `vscode` and `node:fs` lets them be unit-tested under
 * vitest without a VS Code Electron host — the same split that
 * `paths.ts` already follows.
 */

/** The minimal shape of a `vscode-languageclient` `Executable`. */
export interface ServerCommand {
  command: string;
  args: string[];
}

export interface ServerCommandInputs {
  /** `oxContent.server.path`, already resolved to an absolute path. */
  configuredPath?: string;
  /** `OX_CONTENT_LSP_PATH`, trimmed. */
  envBinary?: string;
  /** Ordered local-binary probe locations (debug, release, bundled). */
  localCandidates: string[];
  /** Existence predicate, injected so tests stay off the filesystem. */
  exists: (candidate: string) => boolean;
}

/**
 * Picks the server command by precedence: an explicitly configured path
 * wins, then `OX_CONTENT_LSP_PATH`, then the first local build that
 * exists, and finally `cargo run` as the develop-from-source fallback.
 * Every path-based option is gated on `exists` so a stale setting never
 * points the client at a missing binary.
 */
export function selectServerCommand(inputs: ServerCommandInputs): ServerCommand {
  const { configuredPath, envBinary, localCandidates, exists } = inputs;

  if (configuredPath && exists(configuredPath)) {
    return { command: configuredPath, args: [] };
  }
  if (envBinary && exists(envBinary)) {
    return { command: envBinary, args: [] };
  }
  const localBinary = localCandidates.find(exists);
  if (localBinary) {
    return { command: localBinary, args: [] };
  }
  return { command: "cargo", args: ["run", "-p", "ox_content_lsp", "--bin", "ox-content-lsp"] };
}

export interface InitializationInputs {
  /** `oxContent.frontmatter.schema`, trimmed. */
  schema?: string;
  /** `oxContent.textlint.enabled`. */
  textlintEnabled: boolean;
  /** `oxContent.textlint.command`, trimmed. */
  textlintCommand?: string;
  /** `oxContent.mdc.components`, trimmed. */
  mdcComponents?: string;
  /** `oxContent.spacing.betweenHalfAndFullWidth`. */
  spaceBetweenHalfAndFullWidth?: string;
  /** `oxContent.spacing.autoFixOnSave`. */
  spacingAutoFixOnSave: boolean;
  /** Resolves a possibly-relative path against the workspace root. */
  resolvePath: (value: string) => string;
}

/**
 * Builds the `initializationOptions` payload sent to the LSP. Only
 * non-empty / opted-in settings are forwarded so the handshake stays
 * minimal for users who haven't configured anything — textlint in
 * particular is omitted entirely unless explicitly enabled.
 */
export function buildInitializationOptions(
  inputs: InitializationInputs,
): Record<string, string | boolean> {
  const options: Record<string, string | boolean> = {};

  if (inputs.schema) {
    options.frontmatterSchema = inputs.resolvePath(inputs.schema);
  }
  if (inputs.textlintEnabled) {
    options.textlintEnabled = true;
  }
  if (inputs.textlintCommand) {
    options.textlintCommand = inputs.textlintCommand;
  }
  if (inputs.mdcComponents) {
    options.mdcComponents = inputs.resolvePath(inputs.mdcComponents);
  }
  if (inputs.spaceBetweenHalfAndFullWidth) {
    options.spaceBetweenHalfAndFullWidth = inputs.spaceBetweenHalfAndFullWidth;
  }
  if (inputs.spacingAutoFixOnSave) {
    options.spacingAutoFixOnSave = true;
  }

  return options;
}
