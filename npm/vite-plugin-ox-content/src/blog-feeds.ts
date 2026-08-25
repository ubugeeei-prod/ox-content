/**
 * Load, normalize, and merge configured external blog feeds.
 */

import type { BlogSourcePage } from "./blog-html";
import { feedDateIso } from "./blog-feed-date";
import { fetchBlogFeedBody, type BlogFeedNetwork } from "./blog-feed-fetch";
import { parseBlogFeed, type ParsedFeedItem } from "./blog-feed-parse";
import { canonicalizeFeedItemUrl } from "./blog-feed-url";
import { sortPosts } from "./blog-posts";
import type { ResolvedBlogFeedSource } from "./types";

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
  const results = await Promise.all(
    sources.map(async (source) => {
      try {
        const body = await cachedBody(source.url, bodies, network);
        const items = parseBlogFeed(body, source.language).map((item) =>
          toExternalPage(item, source),
        );
        return { source, pages: items.filter((page): page is BlogSourcePage => page != null) };
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        const message = `[ox-content] blog feed ${source.url}: ${detail}`;
        return { source, message };
      }
    }),
  );

  for (const result of results) {
    if ("pages" in result) {
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
