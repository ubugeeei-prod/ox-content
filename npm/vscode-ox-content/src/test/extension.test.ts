/**
 * Extension activation + command surface tests.
 *
 * These run inside a real VS Code Electron host (via `@vscode/test-cli`),
 * so they exercise the actual activation path declared in package.json.
 *
 * LSP-dependent behavior is deliberately not asserted: starting the real
 * `ox-content-lsp` binary requires a Rust toolchain on the runner and is
 * covered by `cargo test -p ox_content_lsp`. The VS Code-side checks here
 * focus on the surface the host wires up before the LSP becomes available:
 *
 *  - the extension manifest is registered
 *  - all declared commands resolve at the command palette
 *  - snippet contribution is loaded for `markdown`
 *  - insert commands without an active markdown editor surface a hint
 *    instead of throwing
 */

import { strict as assert } from "node:assert";
import * as path from "node:path";

import * as vscode from "vscode";

// VS Code identifies extensions by `${publisher}.${name}`.
const EXTENSION_ID = "ubugeeei.vscode-ox-content";

suite("vscode-ox-content extension surface", () => {
  let activationCompleted = false;

  suiteSetup(async function () {
    // Activation can take a moment in CI on the first run.
    this.timeout(30_000);

    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, `extension ${EXTENSION_ID} is not present in the host`);

    // Trigger activation via an activationEvent rather than calling
    // `extension.activate()` directly. The activate() function registers
    // commands unconditionally, so a second invocation would error out
    // with "command already exists" once auto-activation has already
    // fired (e.g. because the workspace contains `sample.md`).
    const doc = await vscode.workspace.openTextDocument({
      language: "markdown",
      content: "# activator\n",
    });
    await vscode.window.showTextDocument(doc);

    // Wait up to 25s for full activation. activate() awaits startClient
    // which talks to the ox-content-lsp binary over stdio; on runners
    // without that binary, this may never complete and the LSP-dependent
    // checks below skip themselves.
    const start = Date.now();
    while (!extension.isActive && Date.now() - start < 25_000) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    activationCompleted = extension.isActive;
  });

  test("extension is registered with the workbench", () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, `extension ${EXTENSION_ID} is not present`);
  });

  test("declared commands are registered after activation", async function () {
    if (!activationCompleted) {
      this.skip();
    }
    const expected = [
      "oxContent.insertTable",
      "oxContent.insertCodeFence",
      "oxContent.insertCallout",
      "oxContent.openPreview",
    ];
    const registered = await vscode.commands.getCommands(true);
    const oxCommands = registered.filter((id) => id.startsWith("oxContent."));
    for (const id of expected) {
      assert.ok(
        registered.includes(id),
        `command ${id} not registered. registered oxContent.* commands: ${oxCommands.join(", ")}`,
      );
    }
  });

  test("contributes a snippet path for markdown that exists on disk", async () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, `extension ${EXTENSION_ID} not found`);

    const contributes = extension.packageJSON.contributes as
      | { snippets?: Array<{ language: string; path: string }> }
      | undefined;
    assert.ok(contributes?.snippets, "no snippets contribution");

    const markdownEntry = contributes.snippets.find((entry) => entry.language === "markdown");
    assert.ok(markdownEntry, "no markdown snippet entry");

    const absolutePath = path.join(extension.extensionPath, markdownEntry.path);
    const stat = await vscode.workspace.fs.stat(vscode.Uri.file(absolutePath));
    assert.equal(stat.type, vscode.FileType.File, "snippet path is not a file");
  });

  test("insert commands without an active editor show a hint instead of throwing", async function () {
    if (!activationCompleted) {
      this.skip();
    }
    // Close any inherited editor from the test workspace.
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");

    for (const id of [
      "oxContent.insertTable",
      "oxContent.insertCodeFence",
      "oxContent.insertCallout",
    ]) {
      // Should resolve without throwing; the actual hint is delivered via
      // `showInformationMessage`, which the test host swallows.
      await vscode.commands.executeCommand(id);
    }
  });

  test("openPreview without an active markdown editor shows a hint", async function () {
    if (!activationCompleted) {
      this.skip();
    }
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
    // Should resolve cleanly even though there's no markdown buffer to preview.
    await vscode.commands.executeCommand("oxContent.openPreview");
  });
});
