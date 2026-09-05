/**
 * Load, normalize, and merge configured external blog feeds.
 */

import type { BlogSourcePage } from "./blog-html";
import { feedDateIso } from "./blog-feed-date";
import { fetchBlogFeedBody, type BlogFeedNetwork } from "./blog-feed-fetch";
import { parseBlogFeed, type ParsedFeedItem } from "./blog-feed-parse";
import { canonicalizeFeedItemUrl } from "./blog-feed-url";
import { resolveBlogOptions } from "./blog-options";
import { sortPosts } from "./blog-posts";
import type { BlogFeedSource, ResolvedBlogFeedSource } from "./types";

export type {
  BlogFeedFetchFn,
  BlogFeedFetchLimits,
  BlogFeedLookup,
  BlogFeedNetwork,
} from "./blog-feed-fetch";

export class BlogFeedError extends Error {
  readonly issues: string[];

  constructor(issues: string[]) {
    super(issues.join("\n"));
    this.name = "BlogFeedError";
    this.issues = issues;
  }
}

export interface LoadedBlogFeeds {
  pages: BlogSourcePage[];
  warnings: string[];
  fatals: string[];
}

export interface BlogFeedEntry {
  title: string;
  url: string;
  id: string;
  canonical?: string;
  date?: string;
  language?: string;
  author?: string;
  summary?: string;
  external?: boolean;
  sourceUrl?: string;
}

export interface LoadBlogFeedEntriesInput {
  sources?: readonly (string | BlogFeedSource)[];
  network?: BlogFeedNetwork;
}

export interface LoadBlogFeedEntriesResult {
  entries: BlogFeedEntry[];
  warnings: string[];
  fatals: string[];
}

export async function loadBlogFeedEntries(
  input: LoadBlogFeedEntriesInput = {},
): Promise<LoadBlogFeedEntriesResult> {
  const sources = resolveBlogOptions({ feeds: [...(input.sources ?? [])] }).feeds;
  const loaded = await loadExternalBlogPosts(sources, input.network);
  return {
    entries: loaded.pages.map(toFeedEntry),
    warnings: loaded.warnings,
    fatals: loaded.fatals,
  };
}

export function mergeBlogFeedEntries(
  local: readonly BlogFeedEntry[],
  external: readonly BlogFeedEntry[],
): BlogFeedEntry[] {
  return mergeBlogPosts(local.map(toBlogPage), external.map(toBlogPage)).map(toFeedEntry);
}

export async function loadExternalBlogPosts(
  sources: readonly ResolvedBlogFeedSource[],
  network: BlogFeedNetwork = {},
): Promise<LoadedBlogFeeds> {
  const pages: BlogSourcePage[] = [];
  const warnings: string[] = [];
  const fatals: string[] = [];
  if (sources.length === 0) {
    return { pages, warnings, fatals };
  }

  const bodies = new Map<string, Promise<string>>();
  const results: Array<
    | { ok: true; pages: BlogSourcePage[] }
    | { ok: false; source: ResolvedBlogFeedSource; message: string }
  > = await Promise.all(
    sources.map(async (source) => {
      try {
        const body = await cachedBody(source.url, bodies, network);
        const items = parseBlogFeed(body, source.language).map((item) =>
          toExternalPage(item, source),
        );
        return { ok: true, pages: items.filter((page): page is BlogSourcePage => page != null) };
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        const message = `[ox-content] blog feed ${source.url}: ${detail}`;
        return { ok: false, source, message };
      }
    }),
  );

  for (const result of results) {
    if (result.ok) {
      pages.push(...result.pages);
      continue;
    }
    if (result.source.onError === "error") {
      fatals.push(result.message);
    } else {
      warnings.push(result.message);
    }
  }
  return { pages, warnings, fatals };
}

export function mergeBlogPosts(
  local: readonly BlogSourcePage[],
  external: readonly BlogSourcePage[],
): BlogSourcePage[] {
  const seenUrls = new Set<string>();
  const seenIds = new Set<string>();
  const merged: BlogSourcePage[] = [];
  for (const page of [...local, ...external]) {
    const keys = identityKeys(page);
    if (seenUrls.has(keys.url) || seenIds.has(keys.id)) {
      continue;
    }
    seenUrls.add(keys.url);
    seenIds.add(keys.id);
    merged.push(page);
  }
  return sortPosts(merged);
}

function cachedBody(
  url: string,
  cache: Map<string, Promise<string>>,
  network: BlogFeedNetwork,
): Promise<string> {
  const existing = cache.get(url);
  if (existing) {
    return existing;
  }
  const pending = fetchBlogFeedBody(url, network);
  cache.set(url, pending);
  return pending;
}

function toExternalPage(
  item: ParsedFeedItem,
  source: ResolvedBlogFeedSource,
): BlogSourcePage | undefined {
  const link = canonicalizeFeedItemUrl(item.link);
  if (!link) {
    return undefined;
  }
  const id = item.id.trim() || link;
  const language = item.language ?? source.language;
  const author = source.author;
  return {
    title: item.title,
    inputPath: `external:${id}`,
    transformedHtml: "",
    external: true,
    routePaths: { href: link },
    frontmatter: {
      external: true,
      id,
      date: item.date ? feedDateIso(item.date) : undefined,
      language,
      author,
      summary: item.summary,
      sourceUrl: source.url,
    },
  };
}

function toFeedEntry(page: BlogSourcePage): BlogFeedEntry {
  const id = stringField(page.frontmatter.id) ?? page.routePaths.href;
  return {
    title: page.title,
    url: page.routePaths.href,
    id,
    canonical: stringField(page.frontmatter.canonical),
    date: stringField(page.frontmatter.date),
    language: stringField(page.frontmatter.language),
    author: stringField(page.frontmatter.author),
    summary: stringField(page.frontmatter.summary),
    external: page.external || page.frontmatter.external === true,
    sourceUrl: stringField(page.frontmatter.sourceUrl),
  };
}

function toBlogPage(entry: BlogFeedEntry): BlogSourcePage {
  return {
    title: entry.title,
    inputPath: `${entry.external ? "external" : "host"}:${entry.id || entry.url}`,
    transformedHtml: "",
    external: entry.external,
    routePaths: { href: entry.url },
    frontmatter: {
      ...(entry.id ? { id: entry.id } : {}),
      ...(entry.canonical ? { canonical: entry.canonical } : {}),
      ...(entry.date ? { date: entry.date } : {}),
      ...(entry.language ? { language: entry.language } : {}),
      ...(entry.author ? { author: entry.author } : {}),
      ...(entry.summary ? { summary: entry.summary } : {}),
      ...(entry.sourceUrl ? { sourceUrl: entry.sourceUrl } : {}),
      ...(entry.external ? { external: true } : {}),
    },
  };
}

function identityKeys(page: BlogSourcePage): { url: string; id: string } {
  const explicitId = stringField(page.frontmatter.id);
  const canonical = stringField(page.frontmatter.canonical);
  const href = page.routePaths.href;
  const url = canonicalizeFeedItemUrl(canonical ?? "") ?? canonicalizeFeedItemUrl(href) ?? href;
  return { url, id: explicitId || url };
}

function stringField(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}
