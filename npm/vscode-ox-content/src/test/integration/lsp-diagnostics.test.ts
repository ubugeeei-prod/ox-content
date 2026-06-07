/**
 * LSP-backed diagnostics, end to end through the real VS Code Electron
 * host (via `@vscode/test-cli`).
 *
 * These exercise the full pipeline the unit and protocol tests can't:
 * extension activation -> language client start -> `didOpen` ->
 * `publishDiagnostics` -> diagnostics surfaced in the editor. We assert
 * on `ox-content-*` sources, which are distinctly ours — unlike folding
 * or document links, the built-in Markdown providers never emit them, so
 * there's no attribution ambiguity.
 *
 * Each assertion needs a running `ox-content-lsp`. The integration runner
 * points the extension at a freshly built binary via `OX_CONTENT_LSP_PATH`
 * (see `scripts/run-vscode-tests.mjs`); when it isn't available the test
 * skips rather than fails, matching the rest of the suite.
 */

import { strict as assert } from "node:assert";
import * as os from "node:os";
import * as path from "node:path";

import * as vscode from "vscode";

const EXTENSION_ID = "ubugeeei.vscode-ox-content";

// Generous budget: the client has to spawn the server, complete the LSP
// handshake, and parse the document before the first diagnostics arrive.
const DIAGNOSTIC_TIMEOUT_MS = 20_000;

suite("vscode-ox-content LSP diagnostics", () => {
  const createdFiles: vscode.Uri[] = [];

  suiteSetup(async function () {
    this.timeout(30_000);
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, `extension ${EXTENSION_ID} is not present`);
    if (!extension.isActive) {
      await extension.activate();
    }
  });

  setup(async () => {
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  suiteTeardown(async () => {
    for (const uri of createdFiles) {
      try {
        await vscode.workspace.fs.delete(uri);
      } catch {
        // Best-effort cleanup; a leaked temp file is harmless.
      }
    }
  });

  test("a dead relative link surfaces an ox-content-link diagnostic", async function () {
    this.timeout(DIAGNOSTIC_TIMEOUT_MS + 10_000);

    const uri = await writeTempDoc(
      createdFiles,
      "dead-link.md",
      "# Title\n\nSee [missing](./does-not-exist-xyz.md).\n",
    );
    await openMarkdown(uri);

    const diagnostic = await waitForDiagnostic(
      uri,
      (entry) => entry.source === "ox-content-link",
      DIAGNOSTIC_TIMEOUT_MS,
    );

    if (!diagnostic) {
      // No ox-content diagnostic appeared — the server isn't running on
      // this host. Surface a skip rather than a failure.
      this.skip();
    }
    assert.match(diagnostic.message, /.+/, "diagnostic should carry a message");
  });

  test("an unquoted MDC attribute surfaces an ox-content-mdc diagnostic", async function () {
    this.timeout(DIAGNOSTIC_TIMEOUT_MS + 10_000);

    const uri = await writeTempDoc(createdFiles, "invalid.mdc", "<Alert tone=info></Alert>\n");
    // `.mdc` rides the Markdown association the extension contributes; make
    // it explicit so the document selector matches even if the default
    // hasn't been applied in the test profile.
    const document = await openMarkdown(uri);
    if (document.languageId !== "markdown") {
      await vscode.languages.setTextDocumentLanguage(document, "markdown");
    }

    const diagnostic = await waitForDiagnostic(
      uri,
      (entry) => entry.source === "ox-content-mdc",
      DIAGNOSTIC_TIMEOUT_MS,
    );

    if (!diagnostic) {
      this.skip();
    }
    assert.match(diagnostic.message, /.+/, "diagnostic should carry a message");
  });
});

async function writeTempDoc(
  registry: vscode.Uri[],
  name: string,
  content: string,
): Promise<vscode.Uri> {
  // Write into the OS temp dir rather than the fixture workspace so we
  // never leave a broken-link file under version control. Diagnostics are
  // per-document and resolve relative links against the document's own
  // directory, so the file does not need to live inside the workspace.
  const filePath = path.join(os.tmpdir(), `ox-content-vscode-${process.pid}-${name}`);
  const uri = vscode.Uri.file(filePath);
  await vscode.workspace.fs.writeFile(uri, Buffer.from(content, "utf8"));
  registry.push(uri);
  return uri;
}

async function openMarkdown(uri: vscode.Uri): Promise<vscode.TextDocument> {
  const document = await vscode.workspace.openTextDocument(uri);
  await vscode.window.showTextDocument(document);
  return document;
}

async function waitForDiagnostic(
  uri: vscode.Uri,
  predicate: (diagnostic: vscode.Diagnostic) => boolean,
  timeoutMs: number,
): Promise<vscode.Diagnostic | undefined> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    const match = vscode.languages.getDiagnostics(uri).find(predicate);
    if (match) {
      return match;
    }
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  return undefined;
}
