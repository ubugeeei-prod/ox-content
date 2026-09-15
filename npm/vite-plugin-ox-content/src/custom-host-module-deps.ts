import * as fsSync from "node:fs";
import * as path from "node:path";
import type { CustomHostDevModuleGraph, CustomHostDevModuleNode } from "./custom-host-stylesheets";

export function collectDevModuleDependencies(
  moduleGraph: CustomHostDevModuleGraph,
  moduleId: string,
  root: string,
): string[] {
  const entry = devEntry(moduleGraph, moduleId, root);
  if (!entry) {
    return [];
  }
  const dependencies = new Set<string>();
  visitDevModule(entry, root, new Set(), dependencies);
  return [...dependencies];
}

function visitDevModule(
  node: CustomHostDevModuleNode,
  root: string,
  seen: Set<CustomHostDevModuleNode>,
  dependencies: Set<string>,
): void {
  if (seen.has(node)) {
    return;
  }
  seen.add(node);

  const dependency = devDependency(node, root);
  if (dependency) {
    dependencies.add(dependency);
  }
  for (const imported of node.importedModules ?? []) {
    visitDevModule(imported, root, seen, dependencies);
  }
  for (const imported of node.ssrImportedModules ?? []) {
    visitDevModule(imported, root, seen, dependencies);
  }
}

function devEntry(
  moduleGraph: CustomHostDevModuleGraph,
  moduleId: string,
  root: string,
): CustomHostDevModuleNode | undefined {
  for (const id of moduleIdCandidates(moduleId, root)) {
    const direct = moduleGraph.getModuleById(id);
    if (direct) {
      return direct;
    }
  }
  for (const file of moduleFileCandidates(moduleId, root)) {
    const direct = moduleGraph.getModuleById(file);
    if (direct) {
      return direct;
    }
  }
  for (const file of moduleFileCandidates(moduleId, root)) {
    const byFile = sourceModuleForFile(moduleGraph.getModulesByFile?.(file));
    if (byFile) {
      return byFile;
    }
  }

  const ids = new Set(moduleIdCandidates(moduleId, root).map(cleanModulePath));
  const files = new Set(moduleFileCandidates(moduleId, root).map(normalizeFilePath));
  let fileMatch: CustomHostDevModuleNode | undefined;
  for (const [id, node] of moduleGraph.idToModuleMap ?? []) {
    const cleanId = cleanModulePath(id);
    if (ids.has(cleanId)) {
      return node;
    }
    if (!node.file || !files.has(normalizeFilePath(node.file))) {
      continue;
    }
    if (!fileMatch || (isFrameworkStyleModule(fileMatch) && !isFrameworkStyleModule(node))) {
      fileMatch = node;
    }
  }
  return fileMatch;
}

function moduleIdCandidates(moduleId: string, root: string): string[] {
  const clean = cleanModulePath(moduleId).replace(/\\/g, "/");
  const withoutLeading = clean.replace(/^\/+/, "");
  const result = [moduleId, clean, withoutLeading];
  if (isWithinRoot(clean, root)) {
    result.push(path.posix.relative(normalizeFilePath(root), clean));
  } else if (clean.startsWith("/@fs/")) {
    result.push(path.posix.relative(normalizeFilePath(root), clean.slice("/@fs".length)));
  } else if (!clean.startsWith("/") && !clean.startsWith(".")) {
    result.push(`/${clean}`);
  }
  return unique(result.filter(Boolean));
}

function moduleFileCandidates(moduleId: string, root: string): string[] {
  const clean = cleanModulePath(moduleId);
  const result: string[] = [];
  if (clean.startsWith("/@fs/")) {
    result.push(clean.slice("/@fs".length));
  } else if (path.isAbsolute(clean) && !rootRelativeId(clean, root)) {
    result.push(clean);
  } else if (clean.startsWith("/")) {
    result.push(path.join(root, clean.slice(1)));
  } else if (clean) {
    result.push(path.join(root, clean));
  }
  return unique(result.map(normalizeFilePath));
}

function devDependency(node: CustomHostDevModuleNode, root: string): string | undefined {
  for (const raw of [node.file, node.id, node.url]) {
    const dependency = raw ? normalizeDevDependency(raw, root) : undefined;
    if (dependency) {
      return dependency;
    }
  }
  return undefined;
}

function normalizeDevDependency(raw: string, root: string): string | undefined {
  const clean = cleanModulePath(raw);
  if (!isFilesystemModulePath(clean)) {
    return undefined;
  }
  if (clean.startsWith("/@fs/")) {
    return normalizeFilePath(clean.slice("/@fs".length));
  }
  if (path.isAbsolute(clean) && !rootRelativeId(clean, root)) {
    return normalizeFilePath(clean);
  }
  if (clean.startsWith("/")) {
    return normalizeFilePath(path.join(root, clean.slice(1)));
  }
  return normalizeFilePath(path.join(root, clean));
}

function isFilesystemModulePath(value: string): boolean {
  if (
    value.includes("\0") ||
    value.startsWith("virtual:") ||
    value.startsWith("/@id/") ||
    value.startsWith("/@vite/")
  ) {
    return false;
  }
  if (value.startsWith("/@fs/") || path.isAbsolute(value) || value.startsWith(".")) {
    return true;
  }
  return value.includes("/") && !!path.extname(value);
}

function rootRelativeId(value: string, root: string): boolean {
  return value.startsWith("/") && !isWithinRoot(value, root) && !value.startsWith("/@fs/");
}

function isWithinRoot(file: string, root: string): boolean {
  const normalizedFile = normalizeFilePath(file);
  const normalizedRoot = normalizeFilePath(root);
  return normalizedFile === normalizedRoot || normalizedFile.startsWith(`${normalizedRoot}/`);
}

function cleanModulePath(moduleId: string): string {
  return splitModuleSuffix(moduleId)[0].replace(/\\/g, "/");
}

function splitModuleSuffix(moduleId: string): [string, string?] {
  const match = /[?#]/u.exec(moduleId);
  return match ? [moduleId.slice(0, match.index), moduleId.slice(match.index)] : [moduleId];
}

function normalizeFilePath(file: string): string {
  const resolved = path.resolve(file);
  try {
    return fsSync.realpathSync.native(resolved).replace(/\\/g, "/");
  } catch {
    return resolved.replace(/\\/g, "/");
  }
}

function unique(values: string[]): string[] {
  return values.filter((value, index) => values.indexOf(value) === index);
}

function first<T>(set: Set<T> | undefined): T | undefined {
  return set?.values().next().value;
}

function sourceModuleForFile(
  modules: Set<CustomHostDevModuleNode> | undefined,
): CustomHostDevModuleNode | undefined {
  if (!modules) {
    return undefined;
  }
  for (const module of modules) {
    if (!isFrameworkStyleModule(module)) {
      return module;
    }
  }
  return first(modules);
}

function isFrameworkStyleModule(node: CustomHostDevModuleNode): boolean {
  const raw = node.url ?? node.id;
  if (!raw) {
    return false;
  }
  const [pathname, suffix = ""] = splitModuleSuffix(raw);
  return pathname.endsWith(".svelte") && suffix.includes("type=style");
}
