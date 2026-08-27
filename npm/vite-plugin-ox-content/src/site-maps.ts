/**
 * Opt-in sitemap.xml / robots.txt / llms.txt helpers.
 *
 * Bodies come from `ox_content_ssg::generate_site_maps` through the NAPI
 * binding. What stays here is what that binding does not model: validating
 * `siteUrl`, deriving the sitemap location, and writing the files.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import { importNapiModuleSync } from "./napi";
import type { ResolvedSiteMapsOptions, SiteMapsOptions } from "./types";

interface NativeSiteMapsModule {
  generateSiteMapBodies(
    options: Record<string, unknown>,
    pages: readonly SiteMapPageInput[],
  ): SiteMapsRenderResult;
}

const MISSING_SITE_URL =
  "[ox-content] siteMaps is enabled but ssg.siteUrl is not set; sitemap.xml, robots.txt, and llms.txt were not written";
const UNSAFE_SITE_URL =
  "[ox-content] siteMaps requires ssg.siteUrl to be a safe absolute http(s) URL; sitemap.xml, robots.txt, and llms.txt were not written";

/** One page considered for crawl manifests. */
export interface SiteMapPageInput {
  loc: string;
  title: string;
  description?: string;
  /** Source-file git commit time in milliseconds. Omitted when Git has no history. */
  lastUpdated?: number;
  draft?: boolean;
  unlisted?: boolean;
}

/** Inputs for rendering crawl-manifest bodies. */
export interface SiteMapsRenderInput {
  options?: ResolvedSiteMapsOptions | null;
  siteUrl?: string;
  sitemapLoc?: string;
  siteName?: string;
  siteDescription?: string;
  pages: readonly SiteMapPageInput[];
}

/** Rendered crawl-manifest bodies, or a skip warning. */
export interface SiteMapsRenderResult {
  sitemapXml?: string;
  robotsTxt?: string;
  llmsTxt?: string;
  warning?: string;
}

/** Inputs for writing crawl manifests next to generated HTML. */
export interface WriteSiteMapFilesInput {
  outDir: string;
  siteUrl?: string;
  base: string;
  siteName?: string;
  siteDescription?: string;
  options?: ResolvedSiteMapsOptions;
  pages: readonly SiteMapPageInput[];
}

/**
 * Resolves `siteMaps` with defaults.
 *
 * `false` / omitted stays off. `true` enables all three files. An object
 * enables the feature and overrides only the fields the site set.
 */
export function resolveSiteMapsOptions(
  value: boolean | SiteMapsOptions | undefined,
): ResolvedSiteMapsOptions {
  if (!value) {
    return { enabled: false, robots: true, llms: true };
  }
  if (value === true) {
    return { enabled: true, robots: true, llms: true };
  }
  return {
    enabled: true,
    robots: value.robots ?? true,
    llms: value.llms ?? true,
  };
}

/** Builds sitemap / robots / llms bodies without writing files. */
export function generateSiteMaps(input: SiteMapsRenderInput): SiteMapsRenderResult {
  if (!input.options?.enabled) {
    return {};
  }
  const warning = siteUrlWarning(input.siteUrl);
  if (warning) {
    return { warning };
  }

  // Selection, ordering, and escaping all live on the native side.
  const napi = importNapiModuleSync() as unknown as NativeSiteMapsModule;
  return napi.generateSiteMapBodies(
    {
      enabled: true,
      siteUrl: input.siteUrl,
      sitemapLoc: input.sitemapLoc ?? "",
      siteName: input.siteName ?? "",
      siteDescription: input.siteDescription,
      robots: input.options.robots,
      llms: input.options.llms,
    },
    input.pages,
  );
}

/** Writes enabled crawl manifests into `outDir`. */
export async function writeSiteMapFiles(
  input: WriteSiteMapFilesInput,
): Promise<{ files: string[]; warning?: string }> {
  const generated = generateSiteMaps({
    options: input.options,
    siteUrl: input.siteUrl,
    sitemapLoc: absoluteSitemapUrl(input.siteUrl, input.base),
    siteName: input.siteName,
    siteDescription: input.siteDescription,
    pages: input.pages,
  });
  if (generated.warning) {
    return { files: [], warning: generated.warning };
  }

  const outputs: Array<[string, string]> = [
    [generated.sitemapXml, "sitemap.xml"],
    [generated.robotsTxt, "robots.txt"],
    [generated.llmsTxt, "llms.txt"],
  ].filter((entry): entry is [string, string] => entry[0] != null);
  if (outputs.length === 0) {
    return { files: [] };
  }

  await fs.mkdir(input.outDir, { recursive: true });
  const files: string[] = [];
  for (const [body, name] of outputs) {
    const outputPath = path.join(input.outDir, name);
    await fs.writeFile(outputPath, body, "utf8");
    files.push(outputPath);
  }
  return { files };
}

/** UTC `YYYY-MM-DD` for W3C lastmod. Invalid or negative timestamps are dropped. */
export function formatLastmod(timestampMs: number | undefined): string | undefined {
  if (timestampMs == null || !Number.isFinite(timestampMs) || timestampMs < 0) {
    return undefined;
  }
  const date = new Date(timestampMs);
  if (Number.isNaN(date.getTime())) {
    return undefined;
  }
  return date.toISOString().slice(0, 10);
}

function absoluteSitemapUrl(siteUrl: string | undefined, base: string): string {
  if (siteUrlWarning(siteUrl)) {
    return "";
  }
  const origin = (siteUrl ?? "").trim().replace(/\/+$/, "");
  const prefix = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  return `${origin}${prefix}sitemap.xml`;
}

function siteUrlWarning(siteUrl: string | undefined): string | undefined {
  const trimmed = siteUrl?.trim();
  if (!trimmed) {
    return MISSING_SITE_URL;
  }
  return isSafeHttpUrl(trimmed) ? undefined : UNSAFE_SITE_URL;
}

function isSafeHttpUrl(value: string): boolean {
  if (/\s/u.test(value)) {
    return false;
  }
  const lower = value.toLowerCase();
  if (!lower.startsWith("https://") && !lower.startsWith("http://")) {
    return false;
  }
  try {
    const url = new URL(value);
    return (url.protocol === "https:" || url.protocol === "http:") && url.hostname.length > 0;
  } catch {
    return false;
  }
}
