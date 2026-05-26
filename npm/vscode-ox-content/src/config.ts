import * as fs from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import type { ServerOptions } from "vscode-languageclient/node";

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

  return options;
}

function findLocalServerBinary(
  context: vscode.ExtensionContext,
  workspaceRoot?: string,
): string | undefined {
  const binaryName = process.platform === "win32" ? "ox-content-lsp.exe" : "ox-content-lsp";
  const candidates = [
    workspaceRoot ? path.join(workspaceRoot, "target", "debug", binaryName) : undefined,
    workspaceRoot ? path.join(workspaceRoot, "target", "release", binaryName) : undefined,
    path.join(context.extensionPath, "bin", binaryName),
  ].filter((value): value is string => Boolean(value));

  return candidates.find((candidate) => fs.existsSync(candidate));
}

function resolveFilePath(value: string, workspaceRoot?: string): string {
  if (path.isAbsolute(value)) {
    return value;
  }

  return workspaceRoot ? path.join(workspaceRoot, value) : value;
}
