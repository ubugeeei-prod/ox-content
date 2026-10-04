import * as fs from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import * as vscode from "vscode";
import type { ServerOptions } from "vscode-languageclient/node";

import { localServerBinaryCandidates, resolveFilePath } from "./internal/paths";
import { buildInitializationOptions, selectServerCommand } from "./internal/server-options";

export function getConfig(): vscode.WorkspaceConfiguration {
  return vscode.workspace.getConfiguration("oxContent");
}

export function resolveServerOptions(
  context: vscode.ExtensionContext,
  workspaceRoot?: string,
): ServerOptions {
  const configuredPath = getConfig().get<string>("server.path", "").trim();

  // The `OX_CONTENT_LSP_PATH` escape hatch lets CI and the integration
  // test runner point at a freshly built `target/release/ox-content-lsp`
  // without synthesizing a workspace `.vscode/settings.json`.
  let projectCli: string | undefined;
  if (
    workspaceRoot &&
    vscode.workspace.isTrusted &&
    getConfig().get<boolean>("frontmatter.projectValidation", true)
  ) {
    try {
      const entry = createRequire(resolve(workspaceRoot, "package.json")).resolve(
        "@ox-content/vite-plugin/cli",
      );
      projectCli = resolve(dirname(entry), "oxct.mjs");
    } catch {
      /* Projects without the plugin use the bundled Rust server. */
    }
  }
  const selected = selectServerCommand({
    configuredPath: configuredPath ? resolveFilePath(configuredPath, workspaceRoot) : undefined,
    envBinary: process.env.OX_CONTENT_LSP_PATH?.trim(),
    projectCli,
    workspaceTrusted: vscode.workspace.isTrusted,
    localCandidates: localServerBinaryCandidates({
      workspaceRoot,
      extensionPath: context.extensionPath,
      platform: process.platform,
    }),
    exists: fs.existsSync,
  });
  if (selected.args[0] === projectCli && projectCli) {
    return {
      ...selected,
      command: process.execPath,
      options: {
        cwd: workspaceRoot,
        env: { ...process.env, ELECTRON_RUN_AS_NODE: "1" },
      },
    };
  }
  return selected;
}

export function resolveInitializationOptions(
  workspaceRoot?: string,
): Record<string, string | boolean> {
  return buildInitializationOptions({
    schema: getConfig().get<string>("frontmatter.schema", "").trim(),
    markdownLintEnabled: getConfig().get<boolean | null>("markdownLint.enabled") ?? undefined,
    textlintEnabled: getConfig().get<boolean>("textlint.enabled", false),
    textlintCommand: getConfig().get<string>("textlint.command", "").trim(),
    mdcComponents: getConfig().get<string>("mdc.components", "").trim(),
    spaceBetweenHalfAndFullWidth: getConfig().get<string>(
      "spacing.betweenHalfAndFullWidth",
      "forbid",
    ),
    spacingAutoFixOnSave: getConfig().get<boolean>("spacing.autoFixOnSave", false),
    resolvePath: (value) => resolveFilePath(value, workspaceRoot),
  });
}
