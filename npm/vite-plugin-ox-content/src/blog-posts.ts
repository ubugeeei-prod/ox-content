/**
 * Blog post selection, tags, authors, and dates.
 */

import * as path from "node:path";
import { parseDate } from "./feed-format";
import { resolveBlogCollectionName } from "./blog-options";
import { siteHref, type BlogSourcePage } from "./blog-html";
import type { BlogAuthor, ResolvedBlogOptions, ResolvedCollectionsOptions } from "./types";

const HOSTILE_TERM = /^(?:javascript|data):/i;

export function selectBlogPosts(
  listed: readonly BlogSourcePage[],
  options: ResolvedBlogOptions,
  srcDir: string,
  collections: ResolvedCollectionsOptions | undefined,
): BlogSourcePage[] | undefined {
  if (isAmbiguousCollection(options, collections)) {
    return undefined;
  }
  const names = collectionNames(collections);
  const name = resolveBlogCollectionName(options.collection, names);
  const sources = name && collections?.enabled ? collections.collections[name]?.source : undefined;
  return listed.filter((page) => {
    if (isExcludedPost(page.frontmatter)) {
      return false;
    }
    if (!sources) {
      return true;
    }
    return pageMatchesSources(page.inputPath, srcDir, sources);
  });
}

export function isAmbiguousCollection(
  options: ResolvedBlogOptions,
  collections: ResolvedCollectionsOptions | undefined,
): boolean {
  if (options.collection) {
    return false;
  }
  const names = collectionNames(collections);
  return names.length > 1 && !names.includes("blog");
}

function collectionNames(collections: ResolvedCollectionsOptions | undefined): string[] {
  if (!collections?.enabled) {
    return [];
  }
  return Object.keys(collections.collections);
}

function pageMatchesSources(
  inputPath: string,
  srcDir: string,
  sources: readonly string[],
): boolean {
  const relative = path.relative(srcDir, inputPath).split(path.sep).join("/");
  return sources.some((source) => matchGlob(relative, source));
}

function matchGlob(relative: string, pattern: string): boolean {
  const normalized = pattern.replace(/^\/+/, "");
  let out = "^";
  for (let i = 0; i < normalized.length; i += 1) {
    if (normalized.startsWith("**/", i)) {
      out += "(?:.*/)?";
      i += 2;
      continue;
    }
    const ch = normalized[i] ?? "";
    if (ch === "*") {
      out += "[^/]*";
      continue;
    }
    if (ch === "?") {
      out += "[^/]";
      continue;
    }
    if (/[.+^${}()|[\]\\]/.test(ch)) {
      out += `\\${ch}`;
      continue;
    }
    out += ch;
  }
  out += "$";
  return new RegExp(out).test(relative);
}

export function sortPosts(posts: readonly BlogSourcePage[]): BlogSourcePage[] {
  return [...posts].sort((left, right) => {
    const dateCmp =
      (pageUnix(right.frontmatter) ?? Number.NEGATIVE_INFINITY) -
      (pageUnix(left.frontmatter) ?? Number.NEGATIVE_INFINITY);
    if (dateCmp !== 0) {
      return dateCmp;
    }
    return left.routePaths.href < right.routePaths.href
      ? -1
      : left.routePaths.href > right.routePaths.href
        ? 1
        : 0;
  });
}

export function collectTags(
  posts: readonly BlogSourcePage[],
): Array<{ label: string; slug: string; pages: BlogSourcePage[] }> {
  const buckets = new Map<string, { label: string; slug: string; pages: BlogSourcePage[] }>();
  for (const page of posts) {
    for (const label of termsFromValue(page.frontmatter.tags)) {
      const slug = tagSlug(label);
      if (!slug) {
        continue;
      }
      const existing = buckets.get(slug);
      if (existing) {
        existing.pages.push(page);
      } else {
        buckets.set(slug, { label, slug, pages: [page] });
      }
    }
  }
  return [...buckets.values()].sort((left, right) => left.label.localeCompare(right.label));
}

export function datedPosts(
  posts: readonly BlogSourcePage[],
): Array<{ page: BlogSourcePage; year: string; month: string; label: string }> {
  const dated: Array<{ page: BlogSourcePage; year: string; month: string; label: string }> = [];
  for (const page of posts) {
    const parsed = pageDate(page.frontmatter);
    if (!parsed) {
      continue;
    }
    dated.push({
      page,
      year: String(parsed.year).padStart(4, "0"),
      month: String(parsed.month).padStart(2, "0"),
      label: `${String(parsed.year).padStart(4, "0")}-${String(parsed.month).padStart(2, "0")}-${String(parsed.day).padStart(2, "0")}`,
    });
  }
  return dated;
}

export function uniqueYears(dated: readonly { year: string }[]): string[] {
  return [...new Set(dated.map((entry) => entry.year))].sort((left, right) =>
    right.localeCompare(left),
  );
}

export function uniqueMonths(dated: readonly { month: string }[]): string[] {
  return [...new Set(dated.map((entry) => entry.month))].sort((left, right) =>
    left.localeCompare(right),
  );
}

export function toListItem(page: BlogSourcePage): {
  title: string;
  href: string;
  dateLabel?: string;
} {
  const parsed = pageDate(page.frontmatter);
  return {
    title: page.title,
    href: page.routePaths.href,
    dateLabel: parsed
      ? `${String(parsed.year).padStart(4, "0")}-${String(parsed.month).padStart(2, "0")}-${String(parsed.day).padStart(2, "0")}`
      : undefined,
    ...(page.external || page.frontmatter.external === true ? { external: true } : {}),
  };
}

export function resolvePostAuthors(
  frontmatter: Record<string, unknown>,
  map: Record<string, BlogAuthor>,
): BlogAuthor[] {
  const seen = new Set<string>();
  const authors: BlogAuthor[] = [];
  for (const key of authorKeys(frontmatter)) {
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    authors.push(map[key] ?? { name: key });
  }
  return authors;
}

function authorKeys(frontmatter: Record<string, unknown>): string[] {
  return [...keysFromValue(frontmatter.author), ...keysFromValue(frontmatter.authors)];
}

function keysFromValue(value: unknown): string[] {
  if (typeof value === "string") {
    return value.trim() ? [value.trim()] : [];
  }
  if (!Array.isArray(value)) {
    return [];
  }
  return value.flatMap((item) => (typeof item === "string" && item.trim() ? [item.trim()] : []));
}

export function postTagLinks(
  frontmatter: Record<string, unknown>,
  base: string,
): Array<{ label: string; href: string }> {
  const links: Array<{ label: string; href: string }> = [];
  const seen = new Set<string>();
  for (const label of termsFromValue(frontmatter.tags)) {
    const slug = tagSlug(label);
    if (!slug || seen.has(slug)) {
      continue;
    }
    seen.add(slug);
    links.push({ label, href: siteHref(base, "blog", "tags", slug) });
  }
  return links;
}

function tagSlug(term: string): string | undefined {
  const trimmed = term.trim();
  if (!trimmed || HOSTILE_TERM.test(trimmed) || trimmed.includes("..") || trimmed.includes("//")) {
    return undefined;
  }
  const slug = trimmed
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return slug || undefined;
}

function termsFromValue(value: unknown): string[] {
  if (typeof value === "string") {
    return value.trim() ? [value.trim()] : [];
  }
  if (!Array.isArray(value)) {
    return [];
  }
  return value.flatMap((item) => (typeof item === "string" && item.trim() ? [item.trim()] : []));
}

function isExcludedPost(frontmatter: Record<string, unknown>): boolean {
  return frontmatter.draft === true || frontmatter.unlisted === true;
}

function pageDate(frontmatter: Record<string, unknown>): ReturnType<typeof parseDate> {
  return parseDate(dateField(frontmatter.date));
}

function pageUnix(frontmatter: Record<string, unknown>): number | undefined {
  return pageDate(frontmatter)?.unix;
}

function dateField(value: unknown): string | undefined {
  if (typeof value === "string" && value.trim()) {
    return value.trim();
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return String(value);
  }
  if (value instanceof Date && !Number.isNaN(value.getTime())) {
    return value.toISOString();
  }
  return undefined;
}
