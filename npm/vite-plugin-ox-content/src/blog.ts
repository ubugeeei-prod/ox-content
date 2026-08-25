/**
 * Opt-in blog index, authors, tags, reading time, and archive.
 *
 * Resolution and HTML live here. The Vite plugin injects post meta, then
 * writes paginated index, tag, and archive pages during SSG. Tags and
 * archive are part of this feature; they do not wait on taxonomies.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import { parseDate } from "./feed-format";
import {
  archiveIndexContent,
  archiveMonthContent,
  archiveYearContent,
  containedPath,
  indexPageContent,
  postMetaMarkup,
  siteHref,
  tagPageContent,
  type BlogSourcePage,
} from "./blog-html";
import type {
  BlogAuthor,
  BlogOptions,
  ResolvedBlogOptions,
  ResolvedCollectionsOptions,
} from "./types";

export type { BlogSourcePage } from "./blog-html";

const DEFAULT_PAGE_SIZE = 10;
const LATIN_WORDS_PER_MINUTE = 200;
const CJK_CHARS_PER_MINUTE = 500;
const HOSTILE_TERM = /^(?:javascript|data):/i;
const AMBIGUOUS_COLLECTION =
  "[ox-content] blog is enabled but multiple collections are configured; set blog.collection";

/** Synthetic page passed back to `generateHtmlPage`. */
export interface BlogGeneratedPage {
  title: string;
  content: string;
  outputPath: string;
  urlPath: string;
  href: string;
}

/**
 * Resolves `blog` / `ssg.blog` with defaults.
 *
 * `false` / omitted stays off. `true` enables an empty author map and
 * pageSize 10. An object enables the feature and overrides only set fields.
 */
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

/**
 * Deterministic reading time in whole minutes.
 *
 * After dropping YAML frontmatter, fenced code (unclosed fences run to EOF),
 * and inline code spans:
 *
 * `minutes = ceil(latin_words / 200 + cjk_chars / 500)`
 *
 * Latin words are `[A-Za-z0-9]+` sequences (an apostrophe may join two
 * parts). CJK characters are Hiragana, Katakana, CJK Unified Ideographs
 * (including Extension A and Compatibility Ideographs), and Hangul.
 * Empty input is 0. Any remaining text is at least 1 minute.
 * The same markdown always yields the same integer.
 */
export function readingTimeMinutes(markdown: string): number {
  const body = stripInlineCode(stripFences(stripFrontmatter(markdown)));
  let latin = 0;
  let cjk = 0;
  let latinRun = false;
  for (const char of body) {
    const code = char.codePointAt(0) ?? 0;
    if (isCjkCodePoint(code)) {
      cjk += 1;
      latinRun = false;
      continue;
    }
    if (isLatinWordChar(code)) {
      if (!latinRun) {
        latin += 1;
        latinRun = true;
      }
      continue;
    }
    if (char === "'" || char === "\u2019") {
      continue;
    }
    latinRun = false;
  }
  if (latin === 0 && cjk === 0) {
    return 0;
  }
  return Math.max(1, Math.ceil(latin / LATIN_WORDS_PER_MINUTE + cjk / CJK_CHARS_PER_MINUTE));
}

/** Prepends escaped author / reading-time / tag chrome on listed blog posts. */
export async function injectBlogPostMeta(input: {
  pages: BlogSourcePage[];
  listed: readonly BlogSourcePage[];
  options?: ResolvedBlogOptions;
  srcDir: string;
  collections?: ResolvedCollectionsOptions;
  base: string;
}): Promise<void> {
  if (!input.options?.enabled) {
    return;
  }
  const posts = selectBlogPosts(input.listed, input.options, input.srcDir, input.collections);
  if (posts === undefined) {
    return;
  }
  const listedPaths = new Set(posts.map((page) => page.inputPath));
  for (const page of input.pages) {
    if (!listedPaths.has(page.inputPath)) {
      continue;
    }
    const markdown = await readMarkdown(page.inputPath);
    page.transformedHtml =
      postMetaMarkup({
        authors: resolvePostAuthors(page.frontmatter, input.options.authors),
        minutes: readingTimeMinutes(markdown),
        tags: postTagLinks(page.frontmatter, input.base),
      }) + page.transformedHtml;
  }
}

/** Maps a generated blog page onto the SSG render shape. */
export function toBlogProcessResult(page: BlogGeneratedPage): {
  inputPath: string;
  routePaths: {
    outputPath: string;
    urlPath: string;
    href: string;
    ogImagePath: string;
    ogImageUrl: string;
  };
  transformedHtml: string;
  title: string;
  frontmatter: Record<string, unknown>;
  toc: [];
} {
  return {
    inputPath: page.outputPath,
    routePaths: {
      outputPath: page.outputPath,
      urlPath: page.urlPath,
      href: page.href,
      ogImagePath: "",
      ogImageUrl: "",
    },
    transformedHtml: page.content,
    title: page.title,
    frontmatter: {},
    toc: [],
  };
}

/** Renders index, tag, and archive pages and appends them to the build. */
export async function appendBlogPages(input: {
  generatedPages: Array<{ inputPath: string; outputPath: string; html: string }>;
  listedPages: readonly BlogSourcePage[];
  options?: ResolvedBlogOptions;
  collections?: ResolvedCollectionsOptions;
  srcDir: string;
  outDir: string;
  base: string;
  render: (page: BlogGeneratedPage) => Promise<string>;
  errors: string[];
}): Promise<void> {
  if (!input.options?.enabled) {
    return;
  }
  if (isAmbiguousCollection(input.options, input.collections)) {
    input.errors.push(AMBIGUOUS_COLLECTION);
    return;
  }
  const posts = selectBlogPosts(
    input.listedPages,
    input.options,
    input.srcDir,
    input.collections,
  );
  if (posts === undefined) {
    return;
  }
  for (const spec of blogPageSpecs(posts, input.options, input.outDir, input.base)) {
    try {
      input.generatedPages.push({
        inputPath: spec.outputPath,
        outputPath: spec.outputPath,
        html: await input.render(spec),
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      input.errors.push(`Failed to generate blog page ${spec.href}: ${message}`);
    }
  }
}

function blogPageSpecs(
  posts: readonly BlogSourcePage[],
  options: ResolvedBlogOptions,
  outDir: string,
  base: string,
): BlogGeneratedPage[] {
  const sorted = sortPosts(posts);
  const pages: BlogGeneratedPage[] = [];
  const pageSize = options.pageSize;
  const pageCount = Math.max(1, Math.ceil(sorted.length / pageSize) || 1);
  const totalPages = sorted.length === 0 ? 1 : pageCount;

  for (let pageNumber = 1; pageNumber <= totalPages; pageNumber += 1) {
    const slice = sorted.slice((pageNumber - 1) * pageSize, pageNumber * pageSize);
    const isFirst = pageNumber === 1;
    const urlPath = isFirst ? "blog" : `blog/page/${pageNumber}`;
    const outputPath = isFirst
      ? containedPath(outDir, "blog", "index.html")
      : containedPath(outDir, "blog", "page", String(pageNumber), "index.html");
    if (!outputPath) {
      continue;
    }
    pages.push({
      title: isFirst ? "Blog" : `Blog · page ${pageNumber}`,
      content: indexPageContent(slice.map(toListItem), {
        newerHref: isFirst
          ? undefined
          : siteHref(base, ...(pageNumber === 2 ? ["blog"] : ["blog", "page", String(pageNumber - 1)])),
        olderHref: pageNumber < totalPages ? siteHref(base, "blog", "page", String(pageNumber + 1)) : undefined,
      }),
      outputPath,
      urlPath,
      href: siteHref(base, ...urlPath.split("/")),
    });
  }

  const tags = collectTags(sorted);
  for (const tag of tags) {
    const outputPath = containedPath(outDir, "blog", "tags", tag.slug, "index.html");
    if (!outputPath) {
      continue;
    }
    pages.push({
      title: tag.label,
      content: tagPageContent(tag.label, tag.pages.map(toListItem)),
      outputPath,
      urlPath: `blog/tags/${tag.slug}`,
      href: siteHref(base, "blog", "tags", tag.slug),
    });
  }

  const dated = datedPosts(sorted);
  if (dated.length > 0) {
    const years = uniqueYears(dated);
    const archiveIndex = containedPath(outDir, "blog", "archive", "index.html");
    if (archiveIndex) {
      pages.push({
        title: "Archive",
        content: archiveIndexContent(
          years.map((year) => ({ year, href: siteHref(base, "blog", "archive", year) })),
        ),
        outputPath: archiveIndex,
        urlPath: "blog/archive",
        href: siteHref(base, "blog", "archive"),
      });
    }
    for (const year of years) {
      const yearPosts = dated.filter((entry) => entry.year === year);
      const months = uniqueMonths(yearPosts);
      const yearPath = containedPath(outDir, "blog", "archive", year, "index.html");
      if (yearPath) {
        pages.push({
          title: year,
          content: archiveYearContent(
            year,
            months.map((month) => ({
              month: `${year}-${month}`,
              href: siteHref(base, "blog", "archive", year, month),
            })),
            yearPosts.map((entry) => toListItem(entry.page)),
          ),
          outputPath: yearPath,
          urlPath: `blog/archive/${year}`,
          href: siteHref(base, "blog", "archive", year),
        });
      }
      for (const month of months) {
        const monthPosts = yearPosts.filter((entry) => entry.month === month);
        const monthPath = containedPath(outDir, "blog", "archive", year, month, "index.html");
        if (!monthPath) {
          continue;
        }
        pages.push({
          title: `${year}-${month}`,
          content: archiveMonthContent(
            `${year}-${month}`,
            monthPosts.map((entry) => toListItem(entry.page)),
          ),
          outputPath: monthPath,
          urlPath: `blog/archive/${year}/${month}`,
          href: siteHref(base, "blog", "archive", year, month),
        });
      }
    }
  }

  return pages;
}

function selectBlogPosts(
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
  const sources =
    name && collections?.enabled ? collections.collections[name]?.source : undefined;
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

function isAmbiguousCollection(
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

function pageMatchesSources(inputPath: string, srcDir: string, sources: readonly string[]): boolean {
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
    if (ch === "*" ) {
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

function sortPosts(posts: readonly BlogSourcePage[]): BlogSourcePage[] {
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

function collectTags(
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

function datedPosts(
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

function uniqueYears(
  dated: readonly { year: string }[],
): string[] {
  return [...new Set(dated.map((entry) => entry.year))].sort((left, right) =>
    right.localeCompare(left),
  );
}

function uniqueMonths(dated: readonly { month: string }[]): string[] {
  return [...new Set(dated.map((entry) => entry.month))].sort((left, right) =>
    left.localeCompare(right),
  );
}

function toListItem(page: BlogSourcePage): { title: string; href: string; dateLabel?: string } {
  const parsed = pageDate(page.frontmatter);
  return {
    title: page.title,
    href: page.routePaths.href,
    dateLabel: parsed
      ? `${String(parsed.year).padStart(4, "0")}-${String(parsed.month).padStart(2, "0")}-${String(parsed.day).padStart(2, "0")}`
      : undefined,
  };
}

function resolvePostAuthors(
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

function postTagLinks(
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

function stripFrontmatter(markdown: string): string {
  if (!markdown.startsWith("---")) {
    return markdown;
  }
  const match = markdown.match(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/);
  return match ? markdown.slice(match[0].length) : markdown;
}

function stripFences(text: string): string {
  return text.replace(/```[\s\S]*?(?:```|$)/g, " ");
}

function stripInlineCode(text: string): string {
  return text.replace(/`[^`\n]*`/g, " ");
}

function isCjkCodePoint(code: number): boolean {
  return (
    (code >= 0x3040 && code <= 0x30ff) ||
    (code >= 0x31f0 && code <= 0x31ff) ||
    (code >= 0x3400 && code <= 0x4dbf) ||
    (code >= 0x4e00 && code <= 0x9fff) ||
    (code >= 0xf900 && code <= 0xfaff) ||
    (code >= 0xac00 && code <= 0xd7af) ||
    (code >= 0x1100 && code <= 0x11ff)
  );
}

function isLatinWordChar(code: number): boolean {
  return (
    (code >= 0x30 && code <= 0x39) ||
    (code >= 0x41 && code <= 0x5a) ||
    (code >= 0x61 && code <= 0x7a)
  );
}

async function readMarkdown(inputPath: string): Promise<string> {
  try {
    return await fs.readFile(inputPath, "utf8");
  } catch {
    return "";
  }
}
