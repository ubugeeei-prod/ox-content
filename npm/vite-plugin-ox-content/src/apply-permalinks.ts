/**
 * Applies resolved permalinks / cascade to SSG pages and collection entries.
 */

import * as path from "node:path";
import { importNapiModuleSync } from "./napi";
import { normalizeUrlPath, resolvePageRoutes } from "./permalinks";
import type {
  CollectionManifest,
  ResolvedCascadeOptions,
  ResolvedPermalinksOptions,
} from "./types";

/** SSG page shape that can have its `routePaths` rewritten. */
export interface SsgRoutablePage {
  inputPath: string;
  routePaths: {
    outputPath: string;
    urlPath: string;
    href: string;
    ogImagePath: string;
    ogImageUrl: string;
  };
  frontmatter: Record<string, unknown>;
}

interface NavItem {
  title: string;
  path: string;
  href: string;
  children?: NavItem[];
  collapsed?: boolean;
  stickyCollapsed?: boolean;
}

interface NavGroup {
  title: string;
  items: NavItem[];
  collapsed?: boolean;
  stickyCollapsed?: boolean;
}

/** Rewrites SSG `routePaths` from resolved permalinks / slugs. */
export function applySsgPageRoutes(input: {
  pages: readonly SsgRoutablePage[];
  permalinks?: ResolvedPermalinksOptions | null;
  cascade?: ResolvedCascadeOptions | null;
  srcDir: string;
  outDir: string;
  base: string;
  extension: string;
  siteUrl?: string;
}): { pages: SsgRoutablePage[]; errors: string[] } {
  const resolved = resolvePageRoutes({
    pages: input.pages.map((page) => ({
      source: page.inputPath,
      fileUrl: page.routePaths.urlPath,
      frontmatter: page.frontmatter,
    })),
    permalinks: input.permalinks,
    cascade: input.cascade,
  });
  const bySource = new Map(resolved.pages.map((page) => [page.source, page]));
  const pages: SsgRoutablePage[] = [];
  for (const page of input.pages) {
    const hit = bySource.get(page.inputPath);
    if (!hit) {
      continue;
    }
    pages.push({
      ...page,
      frontmatter: hit.frontmatter,
      routePaths: routePathsFromUrl(
        hit.urlPath,
        input.srcDir,
        input.outDir,
        input.base,
        input.extension,
        input.siteUrl,
      ),
    });
  }
  return { pages, errors: resolved.errors };
}

/** Rewrites collection `path` / `stem` / inherited frontmatter. */
export function applyCollectionRoutes(
  manifest: CollectionManifest,
  permalinks?: ResolvedPermalinksOptions | null,
  cascade?: ResolvedCascadeOptions | null,
): { manifest: CollectionManifest; errors: string[] } {
  if (!permalinks?.enabled && !cascade?.enabled) {
    return { manifest, errors: [] };
  }
  const errors: string[] = [];
  const collections: CollectionManifest["collections"] = {};
  for (const [name, entries] of Object.entries(manifest.collections)) {
    const resolved = resolvePageRoutes({
      pages: entries.map((entry) => ({
        source: entry.source,
        fileUrl: entry.path,
        frontmatter: { ...entry.frontmatter },
      })),
      permalinks,
      cascade,
    });
    errors.push(...resolved.errors);
    const bySource = new Map(resolved.pages.map((page) => [page.source, page]));
    collections[name] = entries.flatMap((entry) => {
      const hit = bySource.get(entry.source);
      if (!hit) {
        return [];
      }
      const urlPath = hit.urlPath;
      const pathValue = urlPath === "/" ? "/" : `/${urlPath.replace(/^\/+/u, "")}`;
      return [
        {
          ...entry,
          ...pickInherited(hit.frontmatter),
          path: pathValue,
          stem: pathValue === "/" ? "" : pathValue.slice(1),
          frontmatter: hit.frontmatter,
        },
      ];
    });
  }
  return { manifest: { collections }, errors };
}

/** Updates auto-nav hrefs after permalinks change a page URL. */
export function remapNavGroups<T extends NavGroup>(
  nav: T[],
  kept: readonly { fileUrl: string; urlPath: string; href: string }[],
  skippedFileUrls: readonly string[],
): T[] {
  const skipped = new Set(skippedFileUrls.map(normalizeUrlPath));
  const byFile = new Map(kept.map((page) => [normalizeUrlPath(page.fileUrl), page]));
  return nav
    .map((group) => ({ ...group, items: remapNavItems(group.items, byFile, skipped) }))
    .filter((group) => group.items.length > 0);
}

function routePathsFromUrl(
  urlPath: string,
  srcDir: string,
  outDir: string,
  base: string,
  extension: string,
  siteUrl?: string,
) {
  const relative =
    urlPath === "/" || !urlPath ? "index.md" : `${urlPath.replace(/^\/+|\/+$/gu, "")}.md`;
  return importNapiModuleSync().resolveSsgRoutePaths(
    path.join(srcDir, relative),
    srcDir,
    outDir,
    base,
    extension,
    siteUrl,
  );
}

function remapNavItems<T extends NavItem>(
  items: T[],
  byFile: Map<string, { urlPath: string; href: string }>,
  skipped: Set<string>,
): T[] {
  return items.flatMap((item) => {
    const key = normalizeUrlPath(item.path);
    if (skipped.has(key)) {
      return [];
    }
    const hit = byFile.get(key);
    const children = item.children ? remapNavItems(item.children, byFile, skipped) : undefined;
    return [{ ...item, path: hit?.urlPath ?? item.path, href: hit?.href ?? item.href, children }];
  });
}

function pickInherited(frontmatter: Record<string, unknown>): Record<string, unknown> {
  const skip = new Set(["id", "collection", "path", "stem", "source", "extension", "frontmatter"]);
  const picked: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(frontmatter)) {
    if (!skip.has(key)) {
      picked[key] = value;
    }
  }
  return picked;
}
