import * as fs from "node:fs";
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
  return selectServerCommand({
    configuredPath: configuredPath ? resolveFilePath(configuredPath, workspaceRoot) : undefined,
    envBinary: process.env.OX_CONTENT_LSP_PATH?.trim(),
    localCandidates: localServerBinaryCandidates({
      workspaceRoot,
      extensionPath: context.extensionPath,
      platform: process.platform,
    }),
    exists: fs.existsSync,
  });
}

export function resolveInitializationOptions(
  workspaceRoot?: string,
): Record<string, string | boolean> {
  return buildInitializationOptions({
    schema: getConfig().get<string>("frontmatter.schema", "").trim(),
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
