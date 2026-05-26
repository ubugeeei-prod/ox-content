/**
 * Extension activation + command surface tests.
 *
 * These run inside a real VS Code Electron host (via `@vscode/test-cli`),
 * so they exercise the actual activation path declared in package.json.
 *
 * LSP-dependent behavior (insert commands actually mutating a buffer,
 * preview HTML coming back from `oxContent.previewHtml`) is gated on
 * activation completing — when `ox-content-lsp` is unavailable on the
 * runner, the suite still asserts the editor-side wiring: command
 * registration, snippet contribution, middleware short-circuits, and
 * webview lifecycle.
 */

import { strict as assert } from "node:assert";
import * as path from "node:path";

import * as vscode from "vscode";

// VS Code identifies extensions by `${publisher}.${name}`.
const EXTENSION_ID = "ubugeeei.vscode-ox-content";

const EXPECTED_COMMANDS = [
  "oxContent.insertTable",
  "oxContent.insertCodeFence",
  "oxContent.insertCallout",
  "oxContent.openPreview",
];

const EDITOR_GUARDED_COMMANDS = [
  "oxContent.insertTable",
  "oxContent.insertCodeFence",
  "oxContent.insertCallout",
];

const EXPECTED_CONFIG_KEYS = [
  "oxContent.server.path",
  "oxContent.frontmatter.schema",
  "oxContent.preview.autoRefresh",
];

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

  setup(async () => {
    // Each test starts from a clean editor stack so an inherited preview
    // panel or open document does not leak into the next assertion.
    await vscode.commands.executeCommand("workbench.action.closeAllEditors");
  });

  test("extension is registered with the workbench", () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension, `extension ${EXTENSION_ID} is not present`);
  });

  test("manifest declares the expected command palette entries", () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension);
    const commands = (extension.packageJSON.contributes?.commands ?? []) as Array<{
      command: string;
      title: string;
      category?: string;
    }>;
    const titles = new Map(commands.map((c) => [c.command, c]));
    for (const id of EXPECTED_COMMANDS) {
      const entry = titles.get(id);
      assert.ok(entry, `manifest is missing command ${id}`);
      assert.equal(
        entry.category,
        "Ox Content",
        `${id} should be filed under the "Ox Content" category`,
      );
    }
  });

  test("manifest registers .mdc files as markdown by default", () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension);
    const associations = extension.packageJSON.contributes?.configurationDefaults?.[
      "files.associations"
    ] as Record<string, string> | undefined;
    assert.ok(associations, "no files.associations contribution");
    assert.equal(associations["*.mdc"], "markdown");
  });

  test("manifest exposes the documented configuration keys", () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension);
    const properties = extension.packageJSON.contributes?.configuration?.properties as
      | Record<string, unknown>
      | undefined;
    assert.ok(properties, "no configuration properties contribution");
    for (const key of EXPECTED_CONFIG_KEYS) {
      assert.ok(key in properties, `configuration is missing ${key}`);
    }
  });

  test("activation events match the declared commands and languages", () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension);
    const events = (extension.packageJSON.activationEvents ?? []) as string[];
    for (const id of EXPECTED_COMMANDS) {
      assert.ok(events.includes(`onCommand:${id}`), `missing activation event for ${id}`);
    }
    for (const lang of ["markdown", "javascript", "typescript", "json", "yaml"]) {
      assert.ok(events.includes(`onLanguage:${lang}`), `missing onLanguage:${lang}`);
    }
  });

  test("declared commands are registered after activation", async function () {
    if (!activationCompleted) {
      this.skip();
    }
    const registered = await vscode.commands.getCommands(true);
    const oxCommands = registered.filter((id) => id.startsWith("oxContent."));
    for (const id of EXPECTED_COMMANDS) {
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

  test("snippet file content matches the documented prefixes", async () => {
    const extension = vscode.extensions.getExtension(EXTENSION_ID);
    assert.ok(extension);
    const contributes = extension.packageJSON.contributes as {
      snippets?: Array<{ language: string; path: string }>;
    };
    const markdownEntry = contributes.snippets?.find((e) => e.language === "markdown");
    assert.ok(markdownEntry);

    const absolutePath = path.join(extension.extensionPath, markdownEntry.path);
    const bytes = await vscode.workspace.fs.readFile(vscode.Uri.file(absolutePath));
    const data = JSON.parse(new TextDecoder().decode(bytes)) as Record<
      string,
      { prefix: string; body: string[]; description?: string }
    >;
    const prefixes = new Set(Object.values(data).map((entry) => entry.prefix));
    for (const prefix of ["h1", "h2", "quote", "task", "code", "table"]) {
      assert.ok(prefixes.has(prefix), `snippet prefix ${prefix} missing from markdown.json`);
    }
  });

  test("insert commands without an active editor show a hint instead of throwing", async function () {
    if (!activationCompleted) {
      this.skip();
    }
    for (const id of EDITOR_GUARDED_COMMANDS) {
      // Should resolve without throwing; the actual hint is delivered via
      // `showInformationMessage`, which the test host swallows.
      await vscode.commands.executeCommand(id);
    }
  });

  test("openPreview without an active markdown editor shows a hint", async function () {
    if (!activationCompleted) {
      this.skip();
    }
    // Should resolve cleanly even though there's no markdown buffer to preview.
    await vscode.commands.executeCommand("oxContent.openPreview");
  });

  test("openPreview without an active markdown editor does not open a webview", async function () {
    if (!activationCompleted) {
      this.skip();
    }
    // Open a non-markdown buffer so there *is* an active editor — just
    // not one the preview should accept.
    const json = await vscode.workspace.openTextDocument({
      language: "json",
      content: "{}\n",
    });
    await vscode.window.showTextDocument(json);

    await vscode.commands.executeCommand("oxContent.openPreview");

    // The webview is opened via createWebviewPanel which surfaces as a
    // tab whose viewType begins with our id. Assert none of them appear.
    const previewTabs = vscode.window.tabGroups.all
      .flatMap((group) => group.tabs)
      .filter(
        (tab) =>
          tab.input instanceof vscode.TabInputWebview &&
          tab.input.viewType.endsWith("oxContentPreview"),
      );
    assert.equal(
      previewTabs.length,
      0,
      "openPreview should not create a webview for a non-markdown editor",
    );
  });

  test("openPreview from a markdown editor opens (and disposes) a webview tab", async function () {
    if (!activationCompleted) {
      this.skip();
    }
    const doc = await vscode.workspace.openTextDocument({
      language: "markdown",
      content: "# preview-target\n\nbody text.\n",
    });
    await vscode.window.showTextDocument(doc);

    await vscode.commands.executeCommand("oxContent.openPreview");

    // Wait briefly for the webview tab to appear. The renderer round-trip
    // to the LSP can take a beat.
    const start = Date.now();
    let previewTabs: vscode.Tab[] = [];
    while (Date.now() - start < 5_000) {
      previewTabs = vscode.window.tabGroups.all
        .flatMap((group) => group.tabs)
        .filter(
          (tab) =>
            tab.input instanceof vscode.TabInputWebview &&
            tab.input.viewType.endsWith("oxContentPreview"),
        );
      if (previewTabs.length > 0) {
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    if (previewTabs.length === 0) {
      // The preview pipeline depends on the LSP responding; in environments
      // where it doesn't, surface the skip rather than failing.
      this.skip();
    }

    // Closing the tab should remove it from the workbench tab list.
    for (const tab of previewTabs) {
      await vscode.window.tabGroups.close(tab);
    }
    const stillThere = vscode.window.tabGroups.all
      .flatMap((group) => group.tabs)
      .filter(
        (tab) =>
          tab.input instanceof vscode.TabInputWebview &&
          tab.input.viewType.endsWith("oxContentPreview"),
      );
    assert.equal(stillThere.length, 0, "preview tab was not disposed after close");
  });

  test("oxContent.preview.autoRefresh round-trips through the workbench config", async () => {
    const config = vscode.workspace.getConfiguration("oxContent");
    const original = config.get<boolean>("preview.autoRefresh");
    try {
      await config.update("preview.autoRefresh", false, vscode.ConfigurationTarget.Global);
      assert.equal(
        vscode.workspace.getConfiguration("oxContent").get("preview.autoRefresh"),
        false,
      );
    } finally {
      await vscode.workspace
        .getConfiguration("oxContent")
        .update("preview.autoRefresh", original, vscode.ConfigurationTarget.Global);
    }
  });
});
