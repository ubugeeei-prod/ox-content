/**
 * Full-text search functionality for Ox Content.
 *
 * Generates search index at build time and provides client-side search.
 */

import * as fs from "fs/promises";
import * as path from "path";
import { importNapiModule, importNapiModuleSync } from "./napi";
import { DEFAULT_MARKDOWN_EXTENSIONS } from "./markdown";
import type {
  SearchOptions,
  ResolvedSearchOptions,
  SearchDocument,
  ScopedSearchQuery,
} from "./types";

// Import Rust bindings
let oxContent: typeof import("@ox-content/napi") | null = null;

async function getOxContent() {
  if (!oxContent) {
    try {
      oxContent = await importNapiModule();
    } catch {
      console.warn("[ox-content] Native bindings not available, search disabled");
      return null;
    }
  }
  return oxContent;
}

/**
 * Splits a raw query into free-text terms and `@scope` prefixes.
 */
export function parseScopedSearchQuery(query: string): ScopedSearchQuery {
  return importNapiModuleSync().parseScopedSearchQuery(query);
}

/**
 * Derives hierarchical search scopes from a document id or URL.
 *
 * For example, `api/math/index` yields `["api", "api/math"]`.
 */
export function getSearchDocumentScopes(doc: Pick<SearchDocument, "id" | "url">): string[] {
  return importNapiModuleSync().getSearchDocumentScopes(doc.id ?? "", doc.url ?? "");
}

/**
 * Returns true when a search document belongs to at least one requested scope.
 */
export function matchesSearchScopes(
  doc: Pick<SearchDocument, "id" | "url">,
  scopes: string[],
): boolean {
  return importNapiModuleSync().matchesSearchScopes(doc.id ?? "", doc.url ?? "", scopes);
}

/**
 * Resolves search options with defaults.
 */
export function resolveSearchOptions(
  options: SearchOptions | boolean | undefined,
): ResolvedSearchOptions {
  if (options === false) {
    return {
      enabled: false,
      limit: 10,
      prefix: true,
      placeholder: "Search documentation...",
      hotkey: "/",
    };
  }

  const opts = typeof options === "object" ? options : {};

  return {
    enabled: opts.enabled ?? true,
    limit: opts.limit ?? 10,
    prefix: opts.prefix ?? true,
    placeholder: opts.placeholder ?? "Search documentation...",
    hotkey: opts.hotkey ?? "/",
  };
}

/**
 * Builds the search index from Markdown files.
 */
export async function buildSearchIndex(
  srcDir: string,
  base: string,
  extensions: readonly string[] = DEFAULT_MARKDOWN_EXTENSIONS,
): Promise<string> {
  const napi = await getOxContent();

  if (!napi) {
    return JSON.stringify({
      documents: [],
      index: {},
      df: {},
      avg_dl: 0,
      doc_count: 0,
    });
  }

  return napi.buildSearchIndexFromDirectory(srcDir, base, [...extensions]);
}

/**
 * Writes the search index to a file.
 */
export async function writeSearchIndex(indexJson: string, outDir: string): Promise<void> {
  const indexPath = path.join(outDir, "search-index.json");

  // Ensure output directory exists
  await fs.mkdir(outDir, { recursive: true });

  // Write the index
  await fs.writeFile(indexPath, indexJson, "utf-8");
}

/**
 * Client-side search module code.
 * This is injected into the bundle as a virtual module.
 */
export function generateSearchModule(options: ResolvedSearchOptions, indexPath: string): string {
  return importNapiModuleSync().generateSearchModule(JSON.stringify(options), indexPath);
}
