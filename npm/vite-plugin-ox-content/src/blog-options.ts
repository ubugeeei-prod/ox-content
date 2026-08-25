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
    };
  }
  if (value === true) {
    return {
      enabled: true,
      authors: {},
      pageSize: DEFAULT_PAGE_SIZE,
    };
  }
  return {
    enabled: true,
    collection: value.collection,
    authors: normalizeAuthors(value.authors),
    pageSize: normalizePageSize(value.pageSize),
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
