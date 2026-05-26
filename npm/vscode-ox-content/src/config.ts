import * as fs from "node:fs";
import * as vscode from "vscode";
import type { ServerOptions } from "vscode-languageclient/node";

import { localServerBinaryCandidates, resolveFilePath } from "./internal/paths";

export function getConfig(): vscode.WorkspaceConfiguration {
  return vscode.workspace.getConfiguration("oxContent");
}

export function resolveServerOptions(
  context: vscode.ExtensionContext,
  workspaceRoot?: string,
): ServerOptions {
  const configuredPath = getConfig().get<string>("server.path", "").trim();
  const resolvedConfiguredPath = configuredPath
    ? resolveFilePath(configuredPath, workspaceRoot)
    : undefined;

  if (resolvedConfiguredPath && fs.existsSync(resolvedConfiguredPath)) {
    return { command: resolvedConfiguredPath, args: [] };
  }

  // Escape hatch for CI and the integration test runner: an absolute path
  // in `OX_CONTENT_LSP_PATH` wins over the local-binary probe so the test
  // host can use a freshly built `target/release/ox-content-lsp` without
  // synthesizing a workspace `.vscode/settings.json`.
  const envBinary = process.env.OX_CONTENT_LSP_PATH?.trim();
  if (envBinary && fs.existsSync(envBinary)) {
    return { command: envBinary, args: [] };
  }

  const localBinary = findLocalServerBinary(context, workspaceRoot);
  if (localBinary) {
    return { command: localBinary, args: [] };
  }

  return {
    command: "cargo",
    args: ["run", "-p", "ox_content_lsp", "--bin", "ox-content-lsp"],
  };
}

export function resolveInitializationOptions(
  workspaceRoot?: string,
): Record<string, string | boolean> {
  const options: Record<string, string | boolean> = {};

  const schemaSetting = getConfig().get<string>("frontmatter.schema", "").trim();
  if (schemaSetting) {
    options.frontmatterSchema = resolveFilePath(schemaSetting, workspaceRoot);
  }

  // textlint defaults to off; only forward the flag when explicitly
  // enabled to keep the init payload small for users who haven't
  // opted in.
  const textlintEnabled = getConfig().get<boolean>("textlint.enabled", false);
  if (textlintEnabled) {
    options.textlintEnabled = true;
  }
  const textlintCommand = getConfig().get<string>("textlint.command", "").trim();
  if (textlintCommand) {
    options.textlintCommand = textlintCommand;
  }

  const mdcComponents = getConfig().get<string>("mdc.components", "").trim();
  if (mdcComponents) {
    options.mdcComponents = resolveFilePath(mdcComponents, workspaceRoot);
  }

  return options;
}

function findLocalServerBinary(
  context: vscode.ExtensionContext,
  workspaceRoot?: string,
): string | undefined {
  const candidates = localServerBinaryCandidates({
    workspaceRoot,
    extensionPath: context.extensionPath,
    platform: process.platform,
  });

  return candidates.find((candidate) => fs.existsSync(candidate));
}
