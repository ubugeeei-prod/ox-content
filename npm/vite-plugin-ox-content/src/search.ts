/**
 * Full-text search functionality for Ox Content.
 *
 * Generates search index at build time and provides client-side search.
 */

import { importNapiModule, importNapiModuleSync } from "./napi";
import { DEFAULT_MARKDOWN_EXTENSIONS } from "./markdown";
import type {
  SearchOptions,
  ResolvedSearchOptions,
  ResolvedPublishStateOptions,
  SearchDocument,
  ScopedSearchQuery,
} from "./types";
import { toNapiPublishState } from "./publish-state";
import {
  generateHostedSearchModule,
  resolveHostedSearchConfig,
  toLocalSearchRuntimeOptions,
} from "./search-provider";

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
      provider: "local",
    };
  }

  const opts = typeof options === "object" ? options : {};
  const enabled = opts.enabled ?? true;
  const provider = opts.provider === "hosted" ? "hosted" : "local";
  const resolved: ResolvedSearchOptions = {
    enabled,
    limit: opts.limit ?? 10,
    prefix: opts.prefix ?? true,
    placeholder: opts.placeholder ?? "Search documentation...",
    hotkey: opts.hotkey ?? "/",
    provider,
  };

  if (!enabled || provider !== "hosted") {
    return resolved;
  }

  const hosted = resolveHostedSearchConfig(opts, process.env);
  if (!hosted) {
    console.warn("[ox-content] Hosted search is not configured");
    return resolved;
  }

  return {
    ...resolved,
    appId: hosted.appId,
    indexName: hosted.indexName,
    searchKey: hosted.searchKey,
    endpoint: hosted.endpoint,
  };
}

/**
 * Builds the search index from Markdown files.
 *
 * `publishState` is forwarded to the native indexer. `excludeDocumentIds`
 * then drops matching documents and rebuilds the BM25 index so omitted
 * pages (such as the opt-in 404 source) are not searchable.
 */
export async function buildSearchIndex(
  srcDir: string,
  base: string,
  extensions: readonly string[] = DEFAULT_MARKDOWN_EXTENSIONS,
  publishState?: ResolvedPublishStateOptions,
  excludeDocumentIds: readonly string[] = [],
  mdx?: boolean,
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

  const indexJson = napi.buildSearchIndexFromDirectory(srcDir, base, [...extensions], {
    publishState: toNapiPublishState(publishState),
    mdx,
  });
  if (excludeDocumentIds.length === 0) {
    return indexJson;
  }
  return excludeSearchDocuments(napi, indexJson, excludeDocumentIds);
}

function excludeSearchDocuments(
  napi: NonNullable<Awaited<ReturnType<typeof getOxContent>>>,
  indexJson: string,
  excludeDocumentIds: readonly string[],
): string {
  const excluded = new Set(excludeDocumentIds);
  let documents: Array<{
    id: string;
    title: string;
    url: string;
    body: string;
    headings: string[];
    code: string[];
  }>;
  try {
    const parsed = JSON.parse(indexJson) as { documents?: typeof documents };
    documents = parsed.documents ?? [];
  } catch {
    return indexJson;
  }

  const kept = documents.filter((doc) => !excluded.has(doc.id));
  if (kept.length === documents.length) {
    return indexJson;
  }
  return napi.buildSearchIndex(kept);
}

/**
 * Writes the search index to a file.
 */
export async function writeSearchIndex(indexJson: string, outDir: string): Promise<void> {
  const napi = await getOxContent();

  if (!napi) {
    return;
  }

  napi.writeSearchIndex(indexJson, outDir);
}

/**
 * Client-side search module code.
 * This is injected into the bundle as a virtual module.
 */
export function generateSearchModule(options: ResolvedSearchOptions, indexPath: string): string {
  if (options.provider === "hosted") {
    return generateHostedSearchModule(options);
  }
  return importNapiModuleSync().generateSearchModuleFromOptions(
    toLocalSearchRuntimeOptions(options),
    indexPath,
  );
}
