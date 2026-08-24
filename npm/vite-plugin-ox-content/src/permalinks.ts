/**
 * Opt-in permalink / slug routing and `_index` frontmatter cascade.
 *
 * Resolution follows `ox_content_ssg::resolve_page_routes`. The Vite plugin
 * applies those URLs during SSG and collection manifest builds.
 */

import type {
  CascadeOptions,
  PermalinksOptions,
  ResolvedCascadeOptions,
  ResolvedPermalinksOptions,
} from "./types";

const RESERVED_CASCADE_KEYS = new Set(["permalink", "slug"]);

/** One page considered for cascade and permalink resolution. */
export interface RoutePageInput {
  source: string;
  fileUrl: string;
  frontmatter: Record<string, unknown>;
}

/** A page after cascade and optional permalink / slug rewriting. */
export interface ResolvedRoutePage {
  source: string;
  urlPath: string;
  frontmatter: Record<string, unknown>;
}

/** Resolved pages plus collision / rejection errors. */
export interface RouteResolveOutput {
  pages: ResolvedRoutePage[];
  errors: string[];
}

/** Resolves `permalinks`. `false` / omitted stays off. `true` / `{}` enables. */
export function resolvePermalinksOptions(
  value: boolean | PermalinksOptions | undefined,
): ResolvedPermalinksOptions {
  return resolveFlag(value);
}

/** Resolves `cascade`. `false` / omitted stays off. `true` / `{}` enables. */
export function resolveCascadeOptions(
  value: boolean | CascadeOptions | undefined,
): ResolvedCascadeOptions {
  return resolveFlag(value);
}

/**
 * Applies cascade (when on) then permalink / slug rewriting (when on).
 *
 * Collisions skip the later page and keep the first. Rejected permalinks stay
 * on the file-tree URL. Hostile non-string values are ignored.
 */
export function resolvePageRoutes(input: {
  pages: readonly RoutePageInput[];
  permalinks?: ResolvedPermalinksOptions | null;
  cascade?: ResolvedCascadeOptions | null;
}): RouteResolveOutput {
  const cascaded = applyCascade(input.pages, input.cascade);
  if (!input.permalinks?.enabled) {
    return {
      pages: cascaded.map((page) => ({
        source: page.source,
        urlPath: normalizeUrlPath(page.fileUrl),
        frontmatter: page.frontmatter,
      })),
      errors: [],
    };
  }

  const pages: ResolvedRoutePage[] = [];
  const errors: string[] = [];
  const claimed = new Map<string, string>();
  for (const page of cascaded) {
    const { urlPath, error } = resolveOne(page);
    if (error) {
      errors.push(error);
    }
    const owner = claimed.get(urlPath);
    if (owner) {
      errors.push(
        `[ox-content] URL collision at "${urlPath}": ${owner} kept, ${page.source} skipped`,
      );
      continue;
    }
    claimed.set(urlPath, page.source);
    pages.push({ source: page.source, urlPath, frontmatter: page.frontmatter });
  }
  return { pages, errors };
}

/** Escapes a value for use in an HTML attribute. */
export function escapeAttribute(value: string): string {
  return value.replace(/[&<>"']/gu, (ch) => {
    switch (ch) {
      case "&":
        return "&amp;";
      case "<":
        return "&lt;";
      case ">":
        return "&gt;";
      case '"':
        return "&quot;";
      default:
        return "&#39;";
    }
  });
}

export function normalizeUrlPath(value: string): string {
  const segments = pathSegments(value);
  return segments.length === 0 ? "/" : segments.join("/");
}

function resolveFlag(value: boolean | { enabled?: boolean } | undefined): { enabled: boolean } {
  if (!value) {
    return { enabled: false };
  }
  if (value === true) {
    return { enabled: true };
  }
  return { enabled: value.enabled !== false };
}

function applyCascade(
  pages: readonly RoutePageInput[],
  options?: ResolvedCascadeOptions | null,
): RoutePageInput[] {
  if (!options?.enabled) {
    return pages.map((page) => ({ ...page, frontmatter: { ...page.frontmatter } }));
  }
  const indexes = new Map<string, Record<string, unknown>>();
  for (const page of pages) {
    const source = normalizeSeparators(page.source);
    if (isIndexFile(source)) {
      indexes.set(directoryOf(source), { ...page.frontmatter });
    }
  }
  return pages.map((page) => {
    const source = normalizeSeparators(page.source);
    const frontmatter = { ...page.frontmatter };
    for (const dir of ancestorDirs(source)) {
      const defaults = indexes.get(dir);
      if (!defaults || (isIndexFile(source) && directoryOf(source) === dir)) {
        continue;
      }
      for (const [key, value] of Object.entries(defaults)) {
        if (!RESERVED_CASCADE_KEYS.has(key) && !(key in frontmatter)) {
          frontmatter[key] = value;
        }
      }
    }
    return { ...page, frontmatter };
  });
}

function resolveOne(page: RoutePageInput): { urlPath: string; error?: string } {
  const fileUrl = normalizeUrlPath(page.fileUrl);
  const permalink = readString(page.frontmatter.permalink);
  if (permalink !== undefined) {
    const url = isSafePermalink(permalink) ? normalizeUrlPath(permalink) : undefined;
    return url
      ? { urlPath: url }
      : {
          urlPath: fileUrl,
          error: `[ox-content] rejected permalink ${JSON.stringify(permalink)} on ${page.source} (path escape); using the file-tree URL`,
        };
  }
  const slug = readString(page.frontmatter.slug);
  if (slug !== undefined) {
    const url = rewriteSlug(fileUrl, slug);
    return url
      ? { urlPath: url }
      : {
          urlPath: fileUrl,
          error: `[ox-content] rejected slug ${JSON.stringify(slug)} on ${page.source} (path escape); using the file-tree URL`,
        };
  }
  return { urlPath: fileUrl };
}

function rewriteSlug(fileUrl: string, slug: string): string | undefined {
  const trimmed = slug.trim();
  if (trimmed.includes("/") || !isSafePermalink(trimmed)) {
    return undefined;
  }
  const normalized = normalizeUrlPath(trimmed);
  if (normalized === "/") {
    return undefined;
  }
  if (fileUrl === "/") {
    return normalized;
  }
  const segments = fileUrl.split("/").filter(Boolean);
  segments.pop();
  segments.push(normalized);
  return segments.join("/");
}

function isSafePermalink(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed || /[\n\r\0]/u.test(trimmed) || trimmed.includes("\\") || trimmed.startsWith("//")) {
    return false;
  }
  if (/^[A-Za-z]:/u.test(trimmed)) {
    return false;
  }
  const lower = trimmed.toLowerCase();
  if (
    lower.includes("javascript:") ||
    lower.includes("data:") ||
    lower.includes("vbscript:") ||
    lower.includes("file:") ||
    lower.includes("://")
  ) {
    return false;
  }
  return pathSegments(trimmed).every((segment) => segment !== ".." && segment !== ".");
}

function pathSegments(value: string): string[] {
  return value
    .trim()
    .replace(/^\/+|\/+$/gu, "")
    .split("/")
    .filter(Boolean);
}

function readString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function normalizeSeparators(value: string): string {
  return value.replaceAll("\\", "/");
}

function isIndexFile(source: string): boolean {
  const name = source.split("/").pop() ?? source;
  const stem = name.includes(".") ? name.slice(0, name.lastIndexOf(".")) : name;
  return stem.toLowerCase() === "_index";
}

function directoryOf(source: string): string {
  const index = source.lastIndexOf("/");
  return index === -1 ? "" : source.slice(0, index);
}

function ancestorDirs(source: string): string[] {
  const dir = directoryOf(source);
  const dirs = [""];
  if (!dir) {
    return dirs;
  }
  let acc = "";
  for (const segment of dir.split("/")) {
    acc = acc ? `${acc}/${segment}` : segment;
    dirs.push(acc);
  }
  return dirs;
}
