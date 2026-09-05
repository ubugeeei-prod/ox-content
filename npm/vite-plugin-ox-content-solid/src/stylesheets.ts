export interface SolidStylesheetManifestChunk {
  file?: string;
  src?: string;
  css?: string[];
  imports?: string[];
}

export type SolidStylesheetManifest = Record<string, SolidStylesheetManifestChunk>;

export interface SolidDevModuleNode {
  id?: string | null;
  url?: string | null;
  file?: string | null;
  importedModules?: Iterable<SolidDevModuleNode>;
  ssrImportedModules?: Iterable<SolidDevModuleNode>;
}

export interface SolidDevModuleGraph {
  getModuleById(id: string): SolidDevModuleNode | undefined;
  getModulesByFile?(file: string): Set<SolidDevModuleNode> | undefined;
}

export interface ResolveSolidIslandStylesheetsInput {
  modules: readonly string[];
  base?: string;
  manifest?: SolidStylesheetManifest;
  moduleGraph?: SolidDevModuleGraph;
}

export interface SolidIslandStylesheet {
  href: string;
  moduleId: string;
}

export interface SolidIslandStylesheetDiagnostic {
  code: "missing-module" | "missing-resolver";
  message: string;
  moduleId: string;
}

export interface ResolveSolidIslandStylesheetsResult {
  stylesheets: SolidIslandStylesheet[];
  diagnostics: SolidIslandStylesheetDiagnostic[];
}

export function resolveSolidIslandStylesheets(
  input: ResolveSolidIslandStylesheetsInput,
): ResolveSolidIslandStylesheetsResult {
  if (input.manifest) {
    return resolveBuildStylesheets(input.modules, input.manifest, input.base);
  }
  if (input.moduleGraph) {
    return resolveDevStylesheets(input.modules, input.moduleGraph, input.base);
  }
  return {
    stylesheets: [],
    diagnostics: input.modules.map((moduleId) => ({
      code: "missing-resolver",
      moduleId,
      message: `No Vite manifest or module graph was supplied for "${moduleId}".`,
    })),
  };
}

function resolveBuildStylesheets(
  moduleIds: readonly string[],
  manifest: SolidStylesheetManifest,
  base: string | undefined,
): ResolveSolidIslandStylesheetsResult {
  const stylesheets: SolidIslandStylesheet[] = [];
  const diagnostics: SolidIslandStylesheetDiagnostic[] = [];
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
        message: `Vite manifest entry "${key}" was not found for Solid island "${requestedBy}".`,
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
        stylesheets.push({ href, moduleId: requestedBy });
      }
    }
    visiting.delete(key);
    visited.add(key);
  };

  for (const moduleId of moduleIds) {
    const key = manifestKey(manifest, moduleId);
    if (!key) {
      diagnostics.push({
        code: "missing-module",
        moduleId,
        message: `Vite manifest entry was not found for Solid island module "${moduleId}".`,
      });
      continue;
    }
    visit(key, moduleId);
  }

  return { stylesheets, diagnostics };
}

function resolveDevStylesheets(
  moduleIds: readonly string[],
  moduleGraph: SolidDevModuleGraph,
  base: string | undefined,
): ResolveSolidIslandStylesheetsResult {
  const stylesheets: SolidIslandStylesheet[] = [];
  const diagnostics: SolidIslandStylesheetDiagnostic[] = [];
  const seenCss = new Set<string>();

  for (const moduleId of moduleIds) {
    const entry =
      moduleGraph.getModuleById(moduleId) ?? first(moduleGraph.getModulesByFile?.(moduleId));
    if (!entry) {
      diagnostics.push({
        code: "missing-module",
        moduleId,
        message: `Vite module graph entry was not found for Solid island module "${moduleId}".`,
      });
      continue;
    }
    visitDevModule(entry, moduleId, base, new Set(), seenCss, stylesheets);
  }

  return { stylesheets, diagnostics };
}

function visitDevModule(
  node: SolidDevModuleNode,
  requestedBy: string,
  base: string | undefined,
  seenNodes: Set<SolidDevModuleNode>,
  seenCss: Set<string>,
  stylesheets: SolidIslandStylesheet[],
): void {
  if (seenNodes.has(node)) {
    return;
  }
  seenNodes.add(node);
  for (const imported of node.importedModules ?? []) {
    visitDevModule(imported, requestedBy, base, seenNodes, seenCss, stylesheets);
  }
  for (const imported of node.ssrImportedModules ?? []) {
    visitDevModule(imported, requestedBy, base, seenNodes, seenCss, stylesheets);
  }
  const href = devCssHref(node, base);
  if (href && !seenCss.has(href)) {
    seenCss.add(href);
    stylesheets.push({ href, moduleId: requestedBy });
  }
}

function manifestKey(manifest: SolidStylesheetManifest, moduleId: string): string | undefined {
  for (const candidate of manifestKeyCandidates(moduleId)) {
    if (manifest[candidate]) {
      return candidate;
    }
  }
  const candidates = new Set(manifestKeyCandidates(moduleId));
  return Object.entries(manifest).find(
    ([key, chunk]) => candidates.has(key) || (chunk.src ? candidates.has(chunk.src) : false),
  )?.[0];
}

function manifestKeyCandidates(moduleId: string): string[] {
  const [pathname, suffix = ""] = splitModuleSuffix(moduleId);
  const normalized = pathname.replace(/\\/g, "/");
  const withoutLeading = normalized.replace(/^\/+/, "");
  return [
    moduleId,
    `${normalized}${suffix}`,
    `${withoutLeading}${suffix}`,
    normalized,
    withoutLeading,
  ].filter((candidate, index, values) => candidate && values.indexOf(candidate) === index);
}

function splitModuleSuffix(moduleId: string): [string, string?] {
  const match = /[?#]/u.exec(moduleId);
  return match ? [moduleId.slice(0, match.index), moduleId.slice(match.index)] : [moduleId];
}

function devCssHref(node: SolidDevModuleNode, base: string | undefined): string | undefined {
  const value = node.url ?? node.id ?? node.file;
  if (!value) {
    return undefined;
  }
  const [pathname] = value.split("?");
  if (!pathname?.endsWith(".css")) {
    return undefined;
  }
  return joinBase(base, value.startsWith("/") ? value : `/${value}`);
}

function joinBase(base: string | undefined, href: string): string {
  if (/^[a-z][a-z0-9+.-]*:/i.test(href)) {
    return href;
  }
  const normalizedBase = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  const normalizedHref = href.replace(/^\/+/, "");
  return `${normalizedBase}${normalizedHref}`;
}

function first<T>(set: Set<T> | undefined): T | undefined {
  return set?.values().next().value;
}
