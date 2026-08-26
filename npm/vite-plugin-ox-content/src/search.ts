/**
 * Full-text search functionality for Ox Content.
 *
 * Generates search index at build time and provides client-side search.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import { glob } from "glob";
import { importNapiModule, importNapiModuleSync } from "./napi";
import {
  DEFAULT_MARKDOWN_EXTENSIONS,
  markdownGlobPattern,
  stripMarkdownExtension,
} from "./markdown";
import { toJsConditionalBlockOptions } from "./conditional-block-options";
import { collectCitationSearchText } from "./citations";
import type {
  SearchOptions,
  ResolvedSearchOptions,
  ResolvedPublishStateOptions,
  ResolvedConditionalBlockOptions,
  ResolvedCitationsOptions,
  SearchDocument,
  ScopedSearchQuery,
} from "./types";
import { toNapiPublishState } from "./publish-state";
import { generateHostedSearchModule, resolveHostedSearchConfig } from "./search-provider";
import { generateLocalSearchModule } from "./search/local-runtime";

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
      fuzzy: false,
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
    fuzzy: opts.fuzzy ?? false,
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
  conditionalBlocks?: ResolvedConditionalBlockOptions,
  citations?: ResolvedCitationsOptions,
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

  let indexJson = napi.buildSearchIndexFromDirectory(srcDir, base, [...extensions], {
    publishState: toNapiPublishState(publishState),
    mdx,
    conditionalBlocks: toJsConditionalBlockOptions(conditionalBlocks),
  });
  if (excludeDocumentIds.length > 0) {
    indexJson = excludeSearchDocuments(napi, indexJson, excludeDocumentIds);
  }
  if (citations?.enabled) {
    indexJson = await augmentCitationSearchText(napi, indexJson, srcDir, extensions, citations);
  }
  return indexJson;
}

async function augmentCitationSearchText(
  napi: NonNullable<Awaited<ReturnType<typeof getOxContent>>>,
  indexJson: string,
  srcDir: string,
  extensions: readonly string[],
  citations: ResolvedCitationsOptions,
): Promise<string> {
  const documents = parseSearchDocuments(indexJson);
  if (!documents.length) return indexJson;

  const filesById = await collectMarkdownFilesBySearchId(srcDir, extensions);
  let changed = false;
  const nextDocuments: SearchDocument[] = [];
  for (const document of documents) {
    const file = filesById.get(document.id);
    if (!file) {
      nextDocuments.push(document);
      continue;
    }
    const citationText = await collectCitationSearchText(
      await fs.readFile(file, "utf8"),
      citations,
    );
    if (!citationText) {
      nextDocuments.push(document);
      continue;
    }
    changed = true;
    nextDocuments.push({ ...document, body: `${document.body}\n${citationText}` });
  }

  return changed ? napi.buildSearchIndex(nextDocuments) : indexJson;
}

function parseSearchDocuments(indexJson: string): SearchDocument[] {
  try {
    const parsed = JSON.parse(indexJson) as { documents?: unknown };
    return Array.isArray(parsed.documents) ? (parsed.documents as SearchDocument[]) : [];
  } catch {
    return [];
  }
}

async function collectMarkdownFilesBySearchId(
  srcDir: string,
  extensions: readonly string[],
): Promise<Map<string, string>> {
  const root = path.resolve(srcDir);
  const files = await glob(markdownGlobPattern(root, extensions), { absolute: true, nodir: true });
  const byId = new Map<string, string>();
  for (const file of files) {
    const relative = path.relative(root, file).split(path.sep).join("/");
    const id = stripMarkdownExtension(relative, extensions);
    if (!byId.has(id)) byId.set(id, file);
  }
  return byId;
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
  return generateLocalSearchModule(options, indexPath);
}
