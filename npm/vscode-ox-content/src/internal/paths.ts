import * as path from "node:path";

/**
 * Pure path helpers shared by the extension activation code and the unit
 * tests. They do not import `vscode`, so they can run under plain Node
 * (vitest) without spinning up an Electron host.
 */

export type Platform = NodeJS.Platform;

/**
 * Resolve a user-supplied path to an absolute path. Absolute paths pass
 * through unchanged; relative paths join against the workspace root when
 * one is provided, otherwise they are returned untouched (so the caller
 * can still `fs.existsSync` them against `process.cwd()` if it wants).
 */
export function resolveFilePath(value: string, workspaceRoot?: string): string {
  if (path.isAbsolute(value)) {
    return value;
  }

  return workspaceRoot ? path.join(workspaceRoot, value) : value;
}

/**
 * The expected `ox-content-lsp` binary name on the given platform.
 * Split out so tests can pin both branches without monkey-patching
 * `process.platform`.
 */
export function serverBinaryName(platform: Platform): string {
  return platform === "win32" ? "ox-content-lsp.exe" : "ox-content-lsp";
}

/**
 * The ordered candidate list the extension probes for a locally built
 * `ox-content-lsp`. Matches the resolution order documented in
 * `oxContent.server.path`: local workspace builds first, then the
 * extension's bundled binary.
 *
 * Pure — no `fs` calls. The caller checks each entry with `fs.existsSync`.
 */
export function localServerBinaryCandidates(options: {
  workspaceRoot?: string;
  extensionPath: string;
  platform: Platform;
}): string[] {
  const binaryName = serverBinaryName(options.platform);
  const candidates: string[] = [];

  if (options.workspaceRoot) {
    candidates.push(path.join(options.workspaceRoot, "target", "debug", binaryName));
    candidates.push(path.join(options.workspaceRoot, "target", "release", binaryName));
  }
  candidates.push(path.join(options.extensionPath, "bin", binaryName));

  return candidates;
}
