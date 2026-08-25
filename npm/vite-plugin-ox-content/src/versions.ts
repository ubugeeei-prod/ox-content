/**
 * Opt-in documentation versioning: prefixes, snapshots, and header chrome.
 *
 * Historical snapshot directories are read-only during the build. Recreate
 * them with an explicit snapshot command.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import { buildSearchIndex, writeSearchIndex } from "./search";
import type {
  ResolvedPublishStateOptions,
  ResolvedVersionEntry,
  ResolvedVersionsOptions,
  VersionBannerKind,
  VersionEntry,
  VersionsOptions,
} from "./types";
import { injectSearchVersionFilters } from "./search-filters";
import {
  injectVersionChrome,
  searchIndexUrl,
  versionBannerMarkup,
  versionSwitcherMarkup,
  type VersionLink,
} from "./versions-html";

export {
  injectVersionChrome,
  searchIndexUrl,
  versionBannerMarkup,
  versionSwitcherMarkup,
} from "./versions-html";

const DEFAULT_CURRENT_ID = "current";
const PREFIX_RE = /^[a-z0-9](?:[a-z0-9.-]{0,62})$/;

/**
 * Resolves `versions`. Omitted / `false` stay off. `true` enables a single
 * current entry. An object enables the feature and overrides set fields.
 */
export function resolveVersionsOptions(
  value: boolean | VersionsOptions | undefined,
): ResolvedVersionsOptions {
  if (!value) {
    return {
      enabled: false,
      current: DEFAULT_CURRENT_ID,
      switcher: true,
      badge: true,
      entries: [],
    };
  }
  if (value === true) {
    return {
      enabled: true,
      current: DEFAULT_CURRENT_ID,
      switcher: true,
      badge: true,
      entries: [defaultCurrentEntry()],
    };
  }
  const entries = normalizeEntries(value.entries);
  const current =
    typeof value.current === "string" && value.current.trim()
      ? value.current.trim()
      : (entries[0]?.id ?? DEFAULT_CURRENT_ID);
  return {
    enabled: true,
    current,
    switcher: value.switcher !== false,
    badge: value.badge !== false,
    entries: entries.length > 0 ? entries : [defaultCurrentEntry()],
  };
}

/** Prefix used for the active version (empty string = site root). */
export function currentVersionPrefix(options?: ResolvedVersionsOptions): string {
  if (!options?.enabled) {
    return "";
  }
  return options.entries.find((entry) => entry.id === options.current)?.prefix ?? "";
}

export function snapshotEntries(options?: ResolvedVersionsOptions): ResolvedVersionEntry[] {
  if (!options?.enabled) {
    return [];
  }
  return options.entries.filter((entry) => entry.dir && entry.prefix);
}

/** Confines a snapshot dir to `root`. */
export function resolveSnapshotDir(root: string, dir: string): string | undefined {
  const trimmed = dir.trim();
  if (!trimmed || trimmed.includes("\0") || trimmed.includes("..")) {
    return undefined;
  }
  const resolved = path.resolve(root, trimmed);
  const prefix = root.endsWith(path.sep) ? root : `${root}${path.sep}`;
  if (resolved === root || !resolved.startsWith(prefix)) {
    return undefined;
  }
  return resolved;
}

export function prefixRoutePaths(
  routes: { outputPath: string; urlPath: string; href: string },
  prefix: string,
  outDir: string,
  base: string,
): { outputPath: string; urlPath: string; href: string } {
  const safe = sanitizePrefix(prefix);
  if (!safe) {
    return routes;
  }
  const rel = path.relative(path.resolve(outDir), path.resolve(routes.outputPath));
  if (rel.startsWith("..") || path.isAbsolute(rel)) {
    return routes;
  }
  return {
    outputPath: path.join(outDir, safe, rel),
    urlPath: routes.urlPath ? `${safe}/${routes.urlPath}` : safe,
    href: siteHref(base, safe, routes.urlPath),
  };
}

export function versionLinks(
  options: ResolvedVersionsOptions,
  activeId: string,
  siblingPath: string,
  base: string,
  existingHrefs?: ReadonlySet<string>,
): VersionLink[] {
  return options.entries.map((entry) => {
    const siblingHref = siteHref(base, entry.prefix, siblingPath);
    const rootHref = siteHref(base, entry.prefix, "");
    const href =
      !existingHrefs || siblingPath === "" || existingHrefs.has(siblingHref)
        ? siblingHref
        : rootHref;
    return {
      id: entry.id,
      label: entry.label,
      href,
      current: entry.id === activeId,
      banner: entry.banner,
    };
  });
}

/** Version id and same-path remainder for a generated HTML file. */
export function versionLocation(
  outputPath: string,
  outDir: string,
  options: ResolvedVersionsOptions,
): { id: string; sibling: string } {
  const normalized = relativeUrl(outputPath, outDir);
  for (const entry of options.entries) {
    if (!entry.prefix) {
      continue;
    }
    if (normalized === entry.prefix) {
      return { id: entry.id, sibling: "" };
    }
    if (normalized.startsWith(`${entry.prefix}/`)) {
      return { id: entry.id, sibling: normalized.slice(entry.prefix.length + 1) };
    }
  }
  return { id: options.current, sibling: normalized };
}

export function outputToHref(outputPath: string, outDir: string, base: string): string {
  return siteHref(base, "", relativeUrl(outputPath, outDir));
}

/** Applies switcher / banner / search rewrite after every version tree is generated. */
export function decorateVersionedPages(
  pages: Array<{ outputPath: string; html: string }>,
  options: ResolvedVersionsOptions,
  outDir: string,
  base: string,
): void {
  if (!options.enabled) {
    return;
  }
  const existingHrefs = new Set(pages.map((page) => outputToHref(page.outputPath, outDir, base)));
  for (const page of pages) {
    const { id, sibling } = versionLocation(page.outputPath, outDir, options);
    page.html = applyVersionChrome(page.html, options, id, sibling, base, existingHrefs);
  }
}

export async function writeSnapshotSearchIndex(input: {
  srcDir: string;
  outDir: string;
  prefix: string;
  base: string;
  extensions: readonly string[];
  publishState?: ResolvedPublishStateOptions;
  mdx?: boolean;
}): Promise<string | undefined> {
  const prefix = sanitizePrefix(input.prefix);
  if (!prefix) {
    return undefined;
  }
  const destDir = path.join(input.outDir, prefix);
  const prefixBase = searchIndexUrl(input.base, prefix).replace(/search-index\.json$/, "");
  const json = await buildSearchIndex(
    input.srcDir,
    prefixBase,
    input.extensions,
    input.publishState,
    [],
    input.mdx,
  );
  await fs.mkdir(destDir, { recursive: true });
  await writeSearchIndex(json, destDir);
  const dest = path.join(destDir, "search-index.json");
  try {
    await fs.access(dest);
  } catch {
    await fs.writeFile(dest, json, "utf8");
  }
  return dest;
}

export function applyVersionChrome(
  html: string,
  options: ResolvedVersionsOptions,
  activeId: string,
  siblingPath: string,
  base: string,
  existingHrefs?: ReadonlySet<string>,
): string {
  if (!options.enabled) {
    return html;
  }
  const active = options.entries.find((entry) => entry.id === activeId);
  const switcher = options.switcher
    ? versionSwitcherMarkup(
        versionLinks(options, activeId, siblingPath, base, existingHrefs),
        options.badge,
      )
    : "";
  const banner = versionBannerMarkup(active?.banner);
  const from = searchIndexUrl(base, currentVersionPrefix(options));
  const to = searchIndexUrl(base, active?.prefix ?? "");
  return injectSearchVersionFilters(
    injectVersionChrome(html, switcher, banner, from, to),
    options.entries.map((entry) => ({
      id: entry.id,
      label: entry.label,
      prefix: entry.prefix,
      indexUrl: searchIndexUrl(base, entry.prefix),
      current: entry.id === activeId,
    })),
  );
}

export function sanitizePrefix(prefix: string): string {
  const trimmed = prefix.trim().replace(/^\/+|\/+$/g, "");
  if (!trimmed) {
    return "";
  }
  return PREFIX_RE.test(trimmed) && !trimmed.includes("..") ? trimmed : "";
}

function defaultCurrentEntry(): ResolvedVersionEntry {
  return {
    id: DEFAULT_CURRENT_ID,
    label: "Latest",
    prefix: "",
    banner: false,
  };
}

function normalizeEntries(entries: VersionEntry[] | undefined): ResolvedVersionEntry[] {
  if (!entries) {
    return [];
  }
  const seen = new Set<string>();
  const resolved: ResolvedVersionEntry[] = [];
  for (const entry of entries) {
    if (!entry || typeof entry.id !== "string" || typeof entry.label !== "string") {
      continue;
    }
    const id = entry.id.trim();
    const label = entry.label.trim();
    if (!id || !label || seen.has(id)) {
      continue;
    }
    const prefix = sanitizePrefix(typeof entry.prefix === "string" ? entry.prefix : "");
    if (entry.prefix && !prefix) {
      continue;
    }
    const dir = typeof entry.dir === "string" && entry.dir.trim() ? entry.dir.trim() : undefined;
    if (dir && (dir.includes("\0") || dir.includes(".."))) {
      continue;
    }
    seen.add(id);
    resolved.push({
      id,
      label,
      prefix,
      dir,
      banner: normalizeBanner(entry.banner),
    });
  }
  return resolved;
}

function normalizeBanner(value: VersionEntry["banner"]): VersionBannerKind | false {
  return value === "unreleased" || value === "unmaintained" ? value : false;
}

function siteHref(base: string, prefix: string, rest: string): string {
  const root = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  const parts = [prefix, rest].filter((part) => part && part !== "/");
  return parts.length === 0 ? root : `${root}${parts.join("/")}/`;
}

function relativeUrl(outputPath: string, outDir: string): string {
  const rel = path.posix.normalize(
    path.relative(path.resolve(outDir), path.resolve(outputPath)).replaceAll(path.sep, "/"),
  );
  if (rel.startsWith("..")) {
    return "";
  }
  const dir = rel.endsWith("/index.html")
    ? rel.slice(0, -"/index.html".length)
    : rel.replace(/\.html$/, "");
  return dir === "." ? "" : dir;
}
