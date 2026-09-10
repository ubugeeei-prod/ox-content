import * as path from "node:path";
import {
  cleanModulePath,
  isWithinRoot,
  normalizeFilePath,
  unique,
} from "./custom-host-ssr-imports";
import type {
  OxContentCustomHostStylesheet,
  OxContentCustomHostStylesheetDiagnostic,
  OxContentCustomHostStylesheetsResult,
} from "./custom-host-types";
import { withBase } from "./custom-host-utils";

export interface CustomHostDevModuleNode {
  id?: string | null;
  url?: string | null;
  file?: string | null;
  meta?: Record<string, unknown> | null;
  transformResult?: { meta?: Record<string, unknown> | null } | null;
  ssrTransformResult?: { meta?: Record<string, unknown> | null } | null;
  importedModules?: Iterable<CustomHostDevModuleNode>;
  ssrImportedModules?: Iterable<CustomHostDevModuleNode>;
}

export interface CustomHostDevModuleGraph {
  idToModuleMap?: Map<string, CustomHostDevModuleNode>;
  getModuleById(id: string): CustomHostDevModuleNode | undefined;
  getModulesByFile?(file: string): Set<CustomHostDevModuleNode> | undefined;
}

export function resolveDevStylesheets(
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

  const frameworkCss = devFrameworkCssContent(node);
  if (!frameworkCss && isDevFrameworkStyleModule(node)) {
    return;
  }
  const href = frameworkCss ? devFrameworkCssHref(node, base, root) : devCssHref(node, base, root);
  if (href) {
    addDevStylesheet(seenCss, stylesheets, {
      kind: "style",
      href,
      moduleId: requestedBy,
      content: frameworkCss,
    });
  }
}

function addDevStylesheet(
  seenCss: Set<string>,
  stylesheets: OxContentCustomHostStylesheet[],
  stylesheet: OxContentCustomHostStylesheet,
): void {
  if (!seenCss.has(stylesheet.href)) {
    seenCss.add(stylesheet.href);
    stylesheets.push(stylesheet);
    return;
  }
  if (stylesheet.content == null) {
    return;
  }
  const index = stylesheets.findIndex((existing) => existing.href === stylesheet.href);
  if (index >= 0 && stylesheets[index].content == null) {
    stylesheets[index] = stylesheet;
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
  if (!isDevCssModule(pathname, suffix)) {
    return undefined;
  }
  if (pathname.startsWith("/@fs/")) {
    return joinBase(base, `${pathname}${suffix}`);
  }
  if (root && isWithinRoot(pathname, root)) {
    const relative = path.posix.relative(normalizeFilePath(root), normalizeFilePath(pathname));
    return joinBase(base, `/${relative}${suffix}`);
  }
  return joinBase(
    base,
    pathname.startsWith("/") ? `${pathname}${suffix}` : `/${pathname}${suffix}`,
  );
}

function devFrameworkCssHref(
  node: CustomHostDevModuleNode,
  base: string | undefined,
  root: string | undefined,
): string | undefined {
  const raw = node.file ?? node.id ?? node.url;
  if (!raw) {
    return undefined;
  }
  const [pathname] = splitModuleSuffix(raw);
  if (!pathname.endsWith(".svelte")) {
    return undefined;
  }
  if (pathname.startsWith("/@fs/")) {
    return joinBase(base, `${pathname}?svelte&type=style&lang.css`);
  }
  if (root && isWithinRoot(pathname, root)) {
    const relative = path.posix.relative(normalizeFilePath(root), normalizeFilePath(pathname));
    return joinBase(base, `/${relative}?svelte&type=style&lang.css`);
  }
  const sourcePath = pathname.startsWith("/") ? pathname : `/${pathname}`;
  return joinBase(base, `${sourcePath}?svelte&type=style&lang.css`);
}

function devFrameworkCssContent(node: CustomHostDevModuleNode): string | undefined {
  for (const meta of [node.meta, node.transformResult?.meta, node.ssrTransformResult?.meta]) {
    const css = svelteCssCode(meta);
    if (css?.trim()) {
      return css;
    }
  }
  return undefined;
}

function svelteCssCode(meta: Record<string, unknown> | null | undefined): string | undefined {
  const svelte = meta?.svelte;
  if (!svelte || typeof svelte !== "object") {
    return undefined;
  }
  const css = (svelte as { css?: { code?: unknown } }).css;
  return typeof css?.code === "string" ? css.code : undefined;
}

function isDevFrameworkStyleModule(node: CustomHostDevModuleNode): boolean {
  const raw = node.url ?? node.id;
  if (!raw) {
    return false;
  }
  const [pathname, suffix = ""] = splitModuleSuffix(raw);
  return pathname.endsWith(".svelte") && suffix.includes("type=style");
}

function isDevCssModule(pathname: string, suffix: string): boolean {
  if (pathname.endsWith(".css")) {
    return true;
  }
  if (!suffix) {
    return false;
  }
  const params = new URLSearchParams(suffix.slice(1));
  return (
    params.get("type") === "style" ||
    (/(?:[?&])type=style(?:&|$)/u.test(suffix) && /\.css(?:&|$)/u.test(suffix))
  );
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
  return (
    !!root && value.startsWith("/") && !isWithinRoot(value, root) && !value.startsWith("/@fs/")
  );
}

function splitModuleSuffix(moduleId: string): [string, string?] {
  const match = /[?#]/u.exec(moduleId);
  return match ? [moduleId.slice(0, match.index), moduleId.slice(match.index)] : [moduleId];
}

function joinBase(base: string | undefined, href: string): string {
  return withBase(base ?? "/", href);
}

function first<T>(set: Set<T> | undefined): T | undefined {
  return set?.values().next().value;
}
