import * as fsSync from "node:fs";
import * as path from "node:path";
import {
  dynamicDiagnostic,
  fileFromModuleId,
  isFrameworkStyleRoot,
  isLocalSpecifier,
  isStyleFile,
  isSourceFile,
  isWithinRoot,
  moduleIdCandidates,
  normalizeFilePath,
  outsideRootDiagnostic,
  parseImports,
  publicModuleId,
  unresolvedDiagnostic,
} from "./custom-host-ssr-imports";
import type {
  OxContentCustomHostSsrStylesheetDescriptor,
  OxContentCustomHostSsrStylesheetsResult,
  OxContentCustomHostStylesheet,
  OxContentCustomHostStylesheetDiagnostic,
} from "./custom-host-types";
import { withBase } from "./custom-host-utils";

const OUTSIDE_ROOT = Symbol("outside-root");

export type CustomHostDevImportResolver = (
  specifier: string,
  importer: string,
) => string | undefined;

export function resolveStaticDevSsrStylesheets(input: {
  modules: readonly string[];
  base?: string;
  root: string;
  resolveImport?: CustomHostDevImportResolver;
}): OxContentCustomHostSsrStylesheetsResult | undefined {
  const stylesheets: OxContentCustomHostStylesheet[] = [];
  const diagnostics: OxContentCustomHostStylesheetDiagnostic[] = [];
  const dependencies = new Set<string>();
  const descriptors: OxContentCustomHostSsrStylesheetDescriptor[] = [];
  const seenAggregateCss = new Set<string>();

  for (const moduleId of input.modules) {
    const rootFile = resolveRootModule(moduleId, input.root);
    if (!rootFile) {
      diagnostics.push({
        code: "missing-module",
        moduleId,
        message: `SSR stylesheet module "${moduleId}" was not found in the development graph or filesystem.`,
      });
      continue;
    }
    const record = collectRoot(
      moduleId,
      rootFile,
      input.root,
      input.base,
      dependencies,
      input.resolveImport,
    );
    for (const stylesheet of record.stylesheets) {
      if (!seenAggregateCss.has(stylesheet.href)) {
        seenAggregateCss.add(stylesheet.href);
        stylesheets.push(stylesheet);
      }
    }
    diagnostics.push(...record.diagnostics);
    descriptors.push({
      moduleId,
      stylesheets: record.stylesheets,
      dependencies: record.dependencies,
    });
  }

  if (stylesheets.length === 0 && diagnostics.length === input.modules.length) {
    return undefined;
  }
  return { stylesheets, diagnostics, dependencies: [...dependencies], descriptors };
}

function collectRoot(
  moduleId: string,
  rootFile: string,
  root: string,
  base: string | undefined,
  dependencies: Set<string>,
  resolveImport: CustomHostDevImportResolver | undefined,
) {
  const stylesheets: OxContentCustomHostStylesheet[] = [];
  const diagnostics: OxContentCustomHostStylesheetDiagnostic[] = [];
  const rootDependencies = new Set<string>();
  const seenRootCss = new Set<string>();

  const visit = (file: string, seen: Set<string>) => {
    const clean = normalizeFilePath(file);
    if (seen.has(clean)) {
      return;
    }
    seen.add(clean);
    dependencies.add(clean);
    rootDependencies.add(clean);
    const source = fsSync.readFileSync(clean, "utf8");
    if (isFrameworkStyleRoot(clean) && hasFrameworkStyleBlock(source)) {
      const href = frameworkStyleHref(clean, root, base);
      if (!seenRootCss.has(href)) {
        seenRootCss.add(href);
        stylesheets.push({ kind: "style", href, moduleId });
      }
    }
    for (const item of parseImports(source)) {
      if (!isLocalSpecifier(item.specifier)) {
        continue;
      }
      if (item.dynamic) {
        diagnostics.push(dynamicDiagnostic(moduleId, item.specifier, clean));
        continue;
      }
      const imported = resolveImportSpecifier(item.specifier, clean, root, resolveImport);
      if (imported === OUTSIDE_ROOT) {
        diagnostics.push(outsideRootDiagnostic(moduleId, item.specifier, clean));
        continue;
      }
      if (!imported) {
        diagnostics.push(unresolvedDiagnostic(moduleId, item.specifier, clean));
        continue;
      }
      dependencies.add(imported);
      rootDependencies.add(imported);
      if (isStyleFile(imported)) {
        const href = withBase(base ?? "/", publicModuleId(imported, root));
        if (!seenRootCss.has(href)) {
          seenRootCss.add(href);
          stylesheets.push({ kind: "style", href, moduleId });
        }
      } else {
        visit(imported, seen);
      }
    }
  };

  visit(rootFile, new Set());
  return { stylesheets, diagnostics, dependencies: [...rootDependencies] };
}

function resolveRootModule(moduleId: string, root: string): string | undefined {
  for (const candidate of moduleIdCandidates(moduleId, root)) {
    if (isSourceFile(candidate) && fsSync.existsSync(candidate)) {
      return normalizeFilePath(candidate);
    }
  }
  return undefined;
}

function resolveLocalImport(
  specifier: string,
  importer: string,
  root: string,
): string | typeof OUTSIDE_ROOT | undefined {
  const base = specifier.startsWith(".")
    ? path.resolve(path.dirname(importer), specifier)
    : (fileFromModuleId(specifier, root) ?? path.resolve(root, specifier));
  return resolveImportFile(base, root);
}

function resolveImportSpecifier(
  specifier: string,
  importer: string,
  root: string,
  resolveImport: CustomHostDevImportResolver | undefined,
): string | typeof OUTSIDE_ROOT | undefined {
  const resolved = resolveImport?.(specifier, importer);
  if (resolved) {
    return resolveImportFile(resolved, root);
  }
  return resolveLocalImport(specifier, importer, root);
}

function resolveImportFile(base: string, root: string): string | typeof OUTSIDE_ROOT | undefined {
  const resolved = firstExisting(candidateFiles(base));
  if (!resolved) {
    return undefined;
  }
  if (!isWithinRoot(resolved, root) && isStyleFile(resolved)) {
    return OUTSIDE_ROOT;
  }
  return isStyleFile(resolved) || isSourceFile(resolved) ? normalizeFilePath(resolved) : undefined;
}

function candidateFiles(file: string): string[] {
  const clean = normalizeFilePath(file);
  if (path.extname(clean)) {
    return [clean];
  }
  return [".ts", ".tsx", ".js", ".jsx", ".mjs", ".mts", ".svelte", ".css"].map(
    (extension) => `${clean}${extension}`,
  );
}

function hasFrameworkStyleBlock(source: string): boolean {
  return /<style(?:\s[^>]*)?>[\s\S]*?<\/style>/iu.test(source);
}

function frameworkStyleHref(file: string, root: string, base: string | undefined): string {
  return withBase(base ?? "/", `${publicModuleId(file, root)}?svelte&type=style&lang.css`);
}

function firstExisting(files: readonly string[]): string | undefined {
  return files.find((file) => fsSync.existsSync(file));
}
