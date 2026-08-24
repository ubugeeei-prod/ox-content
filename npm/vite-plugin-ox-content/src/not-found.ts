/**
 * Opt-in custom 404 page helpers.
 *
 * Resolution and path rules live here. The Vite plugin writes the themed HTML
 * during SSG and omits the page from the search index and sitemap.
 */

import * as path from "node:path";
import type { NotFoundOptions, ResolvedNotFoundOptions } from "./types";
import { stripMarkdownExtension } from "./markdown";

export const DEFAULT_NOT_FOUND_SOURCE = "404.md";
export const DEFAULT_NOT_FOUND_OUTPUT = "404.html";
export const FALLBACK_NOT_FOUND_TITLE = "Page not found";

/** Built-in Markdown used when the configured source file is missing. */
export const FALLBACK_NOT_FOUND_MARKDOWN = `---
title: ${FALLBACK_NOT_FOUND_TITLE}
---

# ${FALLBACK_NOT_FOUND_TITLE}

The page you requested does not exist. Use search or the navigation to find what you need.
`;

/**
 * Resolves `ssg.notFound` with defaults.
 *
 * `false` / omitted stays off. `true` enables `404.md` → `404.html`. An object
 * enables the feature and overrides only the fields the site set.
 */
export function resolveNotFoundOptions(
  value: boolean | NotFoundOptions | undefined,
): ResolvedNotFoundOptions {
  if (!value) {
    return {
      enabled: false,
      source: DEFAULT_NOT_FOUND_SOURCE,
      output: DEFAULT_NOT_FOUND_OUTPUT,
    };
  }
  if (value === true) {
    return {
      enabled: true,
      source: DEFAULT_NOT_FOUND_SOURCE,
      output: DEFAULT_NOT_FOUND_OUTPUT,
    };
  }
  return {
    enabled: true,
    source: value.source ?? DEFAULT_NOT_FOUND_SOURCE,
    output: value.output ?? DEFAULT_NOT_FOUND_OUTPUT,
  };
}

/** Absolute source path, confined to `srcDir`. */
export function resolveNotFoundSourcePath(srcDir: string, source: string): string {
  return resolveContainedPath(srcDir, source, DEFAULT_NOT_FOUND_SOURCE);
}

/** Absolute output path, confined to `outDir`. */
export function resolveNotFoundOutputPath(outDir: string, output: string): string {
  return resolveContainedPath(outDir, output, DEFAULT_NOT_FOUND_OUTPUT);
}

/** True when `filePath` is the enabled not-found source. */
export function isNotFoundSourceFile(
  filePath: string,
  srcDir: string,
  options?: ResolvedNotFoundOptions,
): boolean {
  if (!options?.enabled) {
    return false;
  }
  return path.resolve(filePath) === resolveNotFoundSourcePath(srcDir, options.source);
}

/** Search document id for a not-found source path. */
export function notFoundSearchDocumentId(source: string): string {
  const normalized = source.replaceAll("\\", "/").replace(/^\.?\//, "");
  return stripMarkdownExtension(normalized);
}

/** Search document ids that must not be indexed when the feature is on. */
export function notFoundSearchExcludeIds(options?: ResolvedNotFoundOptions): string[] {
  if (!options?.enabled) {
    return [];
  }
  return [notFoundSearchDocumentId(options.source)];
}

function resolveContainedPath(rootDir: string, relativePath: string, fallback: string): string {
  const root = path.resolve(rootDir);
  const resolved = path.resolve(root, relativePath);
  const prefix = root.endsWith(path.sep) ? root : `${root}${path.sep}`;
  if (resolved === root || resolved.startsWith(prefix)) {
    return resolved;
  }
  return path.join(root, fallback);
}
