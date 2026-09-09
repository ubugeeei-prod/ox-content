import * as fsSync from "node:fs";
import * as path from "node:path";
import type { OxContentCustomHostStylesheetDiagnostic } from "./custom-host-types";

const STYLE_EXTENSIONS = new Set([".css"]);
const SOURCE_EXTENSIONS = new Set([
  ".cjs",
  ".cts",
  ".js",
  ".jsx",
  ".mjs",
  ".mts",
  ".svelte",
  ".ts",
  ".tsx",
]);
const FRAMEWORK_STYLE_ROOT_EXTENSIONS = new Set([".svelte"]);

export function parseImports(source: string): { specifier: string; dynamic: boolean }[] {
  const imports: { index: number; specifier: string; dynamic: boolean }[] = [];
  const clean = stripComments(source);
  for (const match of clean.matchAll(/\bimport\s+["']([^"']+)["']/gu)) {
    imports.push({ index: match.index, specifier: match[1], dynamic: false });
  }
  for (const match of clean.matchAll(/\bimport\s+(type\s+)?[^;]*?\s+from\s*["']([^"']+)["']/gu)) {
    if (!match[1]) {
      imports.push({ index: match.index, specifier: match[2], dynamic: false });
    }
  }
  for (const match of clean.matchAll(
    /\bexport\s+(type\s+)?(?:\*|\{[^;]*?\})\s+from\s*["']([^"']+)["']/gu,
  )) {
    if (!match[1]) {
      imports.push({ index: match.index, specifier: match[2], dynamic: false });
    }
  }
  for (const match of clean.matchAll(/\bimport\s*\(\s*["']([^"']+)["']\s*\)/gu)) {
    imports.push({ index: match.index, specifier: match[1], dynamic: true });
  }
  return imports
    .sort((left, right) => left.index - right.index)
    .map(({ specifier, dynamic }) => ({ specifier, dynamic }));
}

export function moduleIdCandidates(moduleId: string, root: string | undefined): string[] {
  const file = root ? fileFromModuleId(moduleId, root) : undefined;
  return unique(
    [
      moduleId,
      cleanModulePath(moduleId),
      file,
      file && root ? publicModuleId(file, root) : undefined,
    ].filter((value): value is string => !!value),
  );
}

export function fileFromModuleId(moduleId: string, root: string): string | undefined {
  const clean = cleanModulePath(moduleId);
  if (clean.startsWith("/@fs/")) {
    return normalizeFilePath(clean.slice("/@fs".length));
  }
  if (path.isAbsolute(clean) && isWithinRoot(clean, root)) {
    return normalizeFilePath(clean);
  }
  if (clean.startsWith("/")) {
    return normalizeFilePath(path.join(root, clean.slice(1)));
  }
  if (!clean.startsWith(".") && clean.includes("/")) {
    return normalizeFilePath(path.join(root, clean));
  }
  return undefined;
}

export function publicModuleId(file: string, root: string): string {
  return `/${path.posix.relative(normalizeFilePath(root), normalizeFilePath(file))}`;
}

export function isLocalSpecifier(specifier: string): boolean {
  return specifier.startsWith(".") || specifier.startsWith("/") || specifier.startsWith("@/");
}

export function isStyleFile(file: string): boolean {
  return STYLE_EXTENSIONS.has(path.extname(cleanModulePath(file)));
}

export function isSourceFile(file: string): boolean {
  return SOURCE_EXTENSIONS.has(path.extname(cleanModulePath(file)));
}

export function isFrameworkStyleRoot(file: string): boolean {
  return FRAMEWORK_STYLE_ROOT_EXTENSIONS.has(path.extname(cleanModulePath(file)));
}

export function cleanModulePath(moduleId: string): string {
  const index = moduleId.search(/[?#]/u);
  return (index === -1 ? moduleId : moduleId.slice(0, index)).replace(/\\/g, "/");
}

export function isWithinRoot(file: string, root: string): boolean {
  const normalizedFile = normalizeFilePath(file);
  const normalizedRoot = normalizeFilePath(root);
  return normalizedFile === normalizedRoot || normalizedFile.startsWith(`${normalizedRoot}/`);
}

export function normalizeFilePath(file: string): string {
  const resolved = path.resolve(file);
  try {
    return fsSync.realpathSync.native(resolved).replace(/\\/g, "/");
  } catch {
    return resolved.replace(/\\/g, "/");
  }
}

export function hasGlobSyntax(value: string): boolean {
  return /[*?[\]{}]/u.test(value);
}

export function unique<T>(values: readonly T[]): T[] {
  return values.filter((value, index) => values.indexOf(value) === index);
}

export function unresolvedDiagnostic(
  moduleId: string,
  specifier: string,
  importer: string,
): OxContentCustomHostStylesheetDiagnostic {
  return {
    code: "missing-module",
    moduleId,
    specifier,
    importer,
    message: `SSR stylesheet import "${specifier}" from "${importer}" could not be resolved.`,
  };
}

export function outsideRootDiagnostic(
  moduleId: string,
  specifier: string,
  importer: string,
): OxContentCustomHostStylesheetDiagnostic {
  return {
    code: "outside-root",
    moduleId,
    specifier,
    importer,
    message: `SSR stylesheet import "${specifier}" from "${importer}" resolved outside the Vite root.`,
  };
}

export function dynamicDiagnostic(
  moduleId: string,
  specifier: string,
  importer: string,
): OxContentCustomHostStylesheetDiagnostic {
  return {
    code: "unsupported-import",
    moduleId,
    specifier,
    importer,
    message: `Dynamic SSR import "${specifier}" from "${importer}" cannot be used for stylesheet discovery.`,
  };
}

function stripComments(source: string): string {
  return source.replace(/\/\*[\s\S]*?\*\/|\/\/[^\n\r]*/gu, (comment) => " ".repeat(comment.length));
}
