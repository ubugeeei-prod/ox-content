#!/usr/bin/env node

// Driver for the vscode-ox-content integration suite.
//
// Resolves the absolute path to the freshly built `ox-content-lsp` binary
// at the repo root and exposes it to the extension via
// `OX_CONTENT_LSP_PATH`. The integration test fixture workspace lives
// under `npm/vscode-ox-content/src/test/fixtures/workspace/`, so the
// extension's local-binary probe would otherwise miss the binary we
// produced and fall through to `cargo run`, which serializes the LSP
// start behind a full debug build.

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..");
const extensionDir = join(repoRoot, "npm", "vscode-ox-content");

const binaryName = process.platform === "win32" ? "ox-content-lsp.exe" : "ox-content-lsp";
const lspBinary = join(repoRoot, "target", "release", binaryName);

if (!existsSync(lspBinary)) {
  console.error(
    `ox-content-lsp not found at ${lspBinary}.\n` +
      "Build it first: cargo build --release -p ox_content_lsp --bin ox-content-lsp",
  );
  process.exit(1);
}

const env = { ...process.env, OX_CONTENT_LSP_PATH: lspBinary };

const passthrough = process.argv.slice(2);

const result = spawnSync("pnpm", ["exec", "vscode-test", ...passthrough], {
  cwd: extensionDir,
  env,
  stdio: "inherit",
});

if (result.error) {
  throw result.error;
}

process.exit(result.status ?? 1);
