import * as path from "node:path";
import type { DocumentAssetManifest } from "./document-assets";
import {
  cleanModulePath,
  isWithinRoot,
  normalizeFilePath,
  unique,
} from "./custom-host-ssr-imports";
import type {
  OxContentCustomHostStylesheet,
  OxContentCustomHostStylesheetDiagnostic,
  OxContentCustomHostStylesheetsInput,
  OxContentCustomHostStylesheetsResult,
} from "./custom-host-types";
import {
  resolveDevStylesheets,
  type CustomHostDevModuleGraph,
} from "./custom-host-dev-stylesheet-graph";
import { withBase } from "./custom-host-utils";

export type {
  CustomHostDevModuleGraph,
  CustomHostDevModuleNode,
} from "./custom-host-dev-stylesheet-graph";

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

  const addStylesheet = (css: string, requestedBy: string) => {
    const href = joinBase(base, css);
    if (!seenCss.has(href)) {
      seenCss.add(href);
      stylesheets.push({ kind: "style", href, moduleId: requestedBy, outputPath: css });
    }
  };

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
    if (chunk.file?.endsWith(".css")) {
      addStylesheet(chunk.file, requestedBy);
    }
    for (const css of chunk.css ?? []) {
      addStylesheet(css, requestedBy);
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

function joinBase(base: string | undefined, href: string): string {
  return withBase(base ?? "/", href);
}
