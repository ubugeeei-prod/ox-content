import * as fsSync from "node:fs";
import * as path from "node:path";
import type { DocumentAssetManifest } from "./document-assets";
import type {
  OxContentCustomHostStylesheet,
  OxContentCustomHostStylesheetDiagnostic,
  OxContentCustomHostStylesheetsInput,
  OxContentCustomHostStylesheetsResult,
} from "./custom-host-types";

export interface CustomHostDevModuleNode {
  id?: string | null;
  url?: string | null;
  file?: string | null;
  importedModules?: Iterable<CustomHostDevModuleNode>;
  ssrImportedModules?: Iterable<CustomHostDevModuleNode>;
}

export interface CustomHostDevModuleGraph {
  idToModuleMap?: Map<string, CustomHostDevModuleNode>;
  getModuleById(id: string): CustomHostDevModuleNode | undefined;
  getModulesByFile?(file: string): Set<CustomHostDevModuleNode> | undefined;
}

export interface ResolveCustomHostStylesheetsInput extends OxContentCustomHostStylesheetsInput {
  root?: string;
  manifest?: DocumentAssetManifest;
  moduleGraph?: CustomHostDevModuleGraph;
}

export function resolveCustomHostStylesheets(
  input: ResolveCustomHostStylesheetsInput,
): OxContentCustomHostStylesheetsResult {
  if (input.manifest) {
    return resolveBuildStylesheets(input.modules, input.manifest, input.base, input.root);
  }
  if (input.moduleGraph) {
    return resolveDevStylesheets(input.modules, input.moduleGraph, input.base, input.root);
  }
  return {
    stylesheets: [],
    dependencies: [],
    diagnostics: input.modules.map((moduleId) => ({
      code: "missing-resolver",
      moduleId,
      message: `No Vite manifest or development module graph was available for "${moduleId}".`,
    })),
  };
}

function resolveBuildStylesheets(
  moduleIds: readonly string[],
  manifest: DocumentAssetManifest,
  base: string | undefined,
  root: string | undefined,
): OxContentCustomHostStylesheetsResult {
  const stylesheets: OxContentCustomHostStylesheet[] = [];
  const diagnostics: OxContentCustomHostStylesheetDiagnostic[] = [];
  const seenCss = new Set<string>();
  const visiting = new Set<string>();
  const visited = new Set<string>();

  const visit = (key: string, requestedBy: string) => {
    if (visiting.has(key) || visited.has(key)) {
      return;
    }
    const chunk = manifest[key];
    if (!chunk) {
      diagnostics.push({
        code: "missing-module",
        moduleId: requestedBy,
        message: `Vite manifest entry "${key}" was not found for custom host module "${requestedBy}".`,
      });
      return;
    }
    visiting.add(key);
    for (const imported of chunk.imports ?? []) {
      visit(imported, requestedBy);
    }
    for (const css of chunk.css ?? []) {
      const href = joinBase(base, css);
      if (!seenCss.has(href)) {
        seenCss.add(href);
        stylesheets.push({ kind: "style", href, moduleId: requestedBy });
      }
    }
    visiting.delete(key);
    visited.add(key);
  };

  for (const moduleId of moduleIds) {
    const key = manifestKey(manifest, moduleId, root);
    if (!key) {
      diagnostics.push({
        code: "missing-module",
        moduleId,
        message: `Vite manifest entry was not found for custom host module "${moduleId}".`,
      });
      continue;
    }
    visit(key, moduleId);
  }

  return { stylesheets, diagnostics, dependencies: [] };
}

function resolveDevStylesheets(
  moduleIds: readonly string[],
  moduleGraph: CustomHostDevModuleGraph,
  base: string | undefined,
  root: string | undefined,
): OxContentCustomHostStylesheetsResult {
  const stylesheets: OxContentCustomHostStylesheet[] = [];
  const diagnostics: OxContentCustomHostStylesheetDiagnostic[] = [];
  const seenCss = new Set<string>();
  const dependencies = new Set<string>();

  for (const moduleId of moduleIds) {
    const entry = devEntry(moduleGraph, moduleId, root);
    if (!entry) {
      diagnostics.push({
        code: "missing-module",
        moduleId,
        message: `Vite module graph entry was not found for custom host module "${moduleId}".`,
      });
      continue;
    }
    visitDevModule(entry, moduleId, base, root, new Set(), seenCss, stylesheets, dependencies);
  }

  return { stylesheets, diagnostics, dependencies: [...dependencies] };
}

function visitDevModule(
  node: CustomHostDevModuleNode,
  requestedBy: string,
  base: string | undefined,
  root: string | undefined,
  seenNodes: Set<CustomHostDevModuleNode>,
  seenCss: Set<string>,
  stylesheets: OxContentCustomHostStylesheet[],
  dependencies: Set<string>,
): void {
  if (seenNodes.has(node)) {
    return;
  }
  seenNodes.add(node);

  const dependency = devDependency(node, root);
  if (dependency) {
    dependencies.add(dependency);
  }
  const visitImport = (imported: CustomHostDevModuleNode) =>
    visitDevModule(
      imported,
      requestedBy,
      base,
      root,
      seenNodes,
      seenCss,
      stylesheets,
      dependencies,
    );
  for (const imported of node.importedModules ?? []) {
    visitImport(imported);
  }
  for (const imported of node.ssrImportedModules ?? []) {
    visitImport(imported);
  }

  const href = devCssHref(node, base, root);
  if (href && !seenCss.has(href)) {
    seenCss.add(href);
    stylesheets.push({ kind: "style", href, moduleId: requestedBy });
  }
}

function devEntry(
  moduleGraph: CustomHostDevModuleGraph,
  moduleId: string,
  root: string | undefined,
): CustomHostDevModuleNode | undefined {
  for (const id of moduleIdCandidates(moduleId, root)) {
    const direct = moduleGraph.getModuleById(id);
    if (direct) {
      return direct;
    }
  }
  for (const file of moduleFileCandidates(moduleId, root)) {
    const byFile = first(moduleGraph.getModulesByFile?.(file));
    if (byFile) {
      return byFile;
    }
  }

  const ids = new Set(moduleIdCandidates(moduleId, root).map(cleanModulePath));
  const files = new Set(moduleFileCandidates(moduleId, root).map(normalizeFilePath));
  for (const [id, node] of moduleGraph.idToModuleMap ?? []) {
    const cleanId = cleanModulePath(id);
    if (ids.has(cleanId) || (node.file && files.has(normalizeFilePath(node.file)))) {
      return node;
    }
  }
  return undefined;
}

function manifestKey(
  manifest: DocumentAssetManifest,
  moduleId: string,
  root: string | undefined,
): string | undefined {
  const candidates = new Set(moduleIdCandidates(moduleId, root).map(cleanModulePath));
  return Object.entries(manifest).find(([key, chunk]) => {
    const values = [key, chunk.src, chunk.file].filter((value): value is string => !!value);
    return values.some((value) => candidates.has(cleanModulePath(value)));
  })?.[0];
}

function moduleIdCandidates(moduleId: string, root: string | undefined): string[] {
  const clean = cleanModulePath(moduleId).replace(/\\/g, "/");
  const withoutLeading = clean.replace(/^\/+/, "");
  const result = [moduleId, clean, withoutLeading];
  if (root && isWithinRoot(clean, root)) {
    result.push(path.posix.relative(normalizeFilePath(root), clean));
  } else if (root && clean.startsWith("/@fs/")) {
    result.push(path.posix.relative(normalizeFilePath(root), clean.slice("/@fs".length)));
  } else if (!clean.startsWith("/") && !clean.startsWith(".")) {
    result.push(`/${clean}`);
  }
  return unique(result.filter(Boolean));
}

function moduleFileCandidates(moduleId: string, root: string | undefined): string[] {
  const clean = cleanModulePath(moduleId);
  const result: string[] = [];
  if (clean.startsWith("/@fs/")) {
    result.push(clean.slice("/@fs".length));
  } else if (path.isAbsolute(clean) && !rootRelativeId(clean, root)) {
    result.push(clean);
  } else if (root && clean.startsWith("/")) {
    result.push(path.join(root, clean.slice(1)));
  } else if (root && clean) {
    result.push(path.join(root, clean));
  }
  return unique(result.map(normalizeFilePath));
}

function devCssHref(
  node: CustomHostDevModuleNode,
  base: string | undefined,
  root: string | undefined,
): string | undefined {
  const raw = node.url ?? node.id ?? node.file;
  if (!raw) {
    return undefined;
  }
  const [pathname, suffix = ""] = splitModuleSuffix(raw);
  if (!pathname.endsWith(".css")) {
    return undefined;
  }
  if (pathname.startsWith("/@fs/")) {
    return joinBase(base, `${pathname}${suffix}`);
  }
  if (root && isWithinRoot(pathname, root)) {
    const relative = path.posix.relative(normalizeFilePath(root), normalizeFilePath(pathname));
    return joinBase(base, `/${relative}${suffix}`);
  }
  if (pathname.startsWith("/")) {
    return joinBase(base, `${pathname}${suffix}`);
  }
  return joinBase(base, `/${pathname}${suffix}`);
}

function devDependency(
  node: CustomHostDevModuleNode,
  root: string | undefined,
): string | undefined {
  const raw = node.file ?? node.id ?? node.url;
  if (!raw) {
    return undefined;
  }
  const clean = cleanModulePath(raw);
  if (clean.startsWith("/@fs/")) {
    return normalizeFilePath(clean.slice("/@fs".length));
  }
  if (path.isAbsolute(clean) && !rootRelativeId(clean, root)) {
    return normalizeFilePath(clean);
  }
  if (root && clean.startsWith("/")) {
    return normalizeFilePath(path.join(root, clean.slice(1)));
  }
  return root && clean ? normalizeFilePath(path.join(root, clean)) : undefined;
}

function rootRelativeId(value: string, root: string | undefined): boolean {
  if (!root || !value.startsWith("/")) {
    return false;
  }
  return !isWithinRoot(value, root) && !value.startsWith("/@fs/");
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

function joinBase(base: string | undefined, href: string): string {
  if (href.startsWith("#") || href.startsWith("//") || /^[a-z][a-z0-9+.-]*:/iu.test(href)) {
    return href;
  }
  const normalizedBase = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  if (normalizedBase !== "/" && href.startsWith(normalizedBase)) {
    return href;
  }
  return `${normalizedBase}${href.replace(/^\/+/u, "")}`;
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
