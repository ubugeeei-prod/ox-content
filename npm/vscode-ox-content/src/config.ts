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

export function resolveInitializationOptions(workspaceRoot?: string): Record<string, string> {
  const schemaSetting = getConfig().get<string>("frontmatter.schema", "").trim();
  return schemaSetting ? { frontmatterSchema: resolveFilePath(schemaSetting, workspaceRoot) } : {};
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
