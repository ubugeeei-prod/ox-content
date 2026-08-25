/**
 * Blog option resolution.
 */

import type { BlogAuthor, BlogOptions, ResolvedBlogOptions } from "./types";

const DEFAULT_PAGE_SIZE = 10;

export function resolveBlogOptions(value: boolean | BlogOptions | undefined): ResolvedBlogOptions {
  if (!value) {
    return {
      enabled: false,
      authors: {},
      pageSize: DEFAULT_PAGE_SIZE,
      feeds: [],
    };
  }
  if (value === true) {
    return {
      enabled: true,
      authors: {},
      pageSize: DEFAULT_PAGE_SIZE,
      feeds: [],
    };
  }
  return {
    enabled: true,
    collection: value.collection,
    authors: normalizeAuthors(value.authors),
    pageSize: normalizePageSize(value.pageSize),
    feeds: normalizeFeeds(value.feeds),
  };
}

/**
 * Picks a collection named `blog`, else the only configured collection.
 *
 * An explicit name always wins. Several collections and no `blog` name
 * require `blog.collection`.
 */
export function resolveBlogCollectionName(
  requested: string | undefined,
  collectionNames: readonly string[],
): string | undefined {
  if (requested) {
    return requested;
  }
  if (collectionNames.includes("blog")) {
    return "blog";
  }
  if (collectionNames.length === 1) {
    return collectionNames[0];
  }
  return undefined;
}

function normalizeAuthors(authors: BlogOptions["authors"]): Record<string, BlogAuthor> {
  if (!authors || typeof authors !== "object") {
    return {};
  }
  const resolved: Record<string, BlogAuthor> = {};
  for (const [key, value] of Object.entries(authors)) {
    if (!value || typeof value.name !== "string") {
      continue;
    }
    resolved[key] = {
      name: value.name,
      bio: typeof value.bio === "string" ? value.bio : undefined,
      url: typeof value.url === "string" ? value.url : undefined,
    };
  }
  return resolved;
}

function normalizePageSize(value: number | undefined): number {
  if (typeof value === "number" && Number.isFinite(value) && value >= 1) {
    return Math.floor(value);
  }
  return DEFAULT_PAGE_SIZE;
}

function normalizeFeeds(feeds: BlogOptions["feeds"]): ResolvedBlogOptions["feeds"] {
  if (!Array.isArray(feeds)) {
    return [];
  }
  const resolved: ResolvedBlogOptions["feeds"] = [];
  for (const entry of feeds) {
    if (typeof entry === "string") {
      const url = entry.trim();
      if (url) {
        resolved.push({ url, onError: "warn" });
      }
      continue;
    }
    if (!entry || typeof entry !== "object" || typeof entry.url !== "string") {
      continue;
    }
    const url = entry.url.trim();
    if (!url) {
      continue;
    }
    const language = trimOptional(entry.language);
    const author = trimOptional(entry.author);
    resolved.push({
      url,
      ...(language ? { language } : {}),
      ...(author ? { author } : {}),
      onError: entry.onError === "error" ? "error" : "warn",
    });
  }
  return resolved;
}

function trimOptional(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}
