/**
 * Opt-in permalink / slug routing and `_index` frontmatter cascade.
 *
 * Resolution follows `ox_content_ssg::resolve_page_routes`. The Vite plugin
 * applies those URLs during SSG and collection manifest builds.
 */

import { importNapiModuleSync } from "./napi";
import type {
  CascadeOptions,
  PermalinksOptions,
  ResolvedCascadeOptions,
  ResolvedPermalinksOptions,
} from "./types";

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
  return importNapiModuleSync().resolveSsgPageRoutes(
    input.pages.map((page) => ({
      source: page.source,
      fileUrl: page.fileUrl,
      frontmatter: page.frontmatter,
    })),
    input.permalinks?.enabled === true,
    input.cascade?.enabled === true,
  );
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

function pathSegments(value: string): string[] {
  return value
    .trim()
    .replace(/^\/+|\/+$/gu, "")
    .split("/")
    .filter(Boolean);
}
