/**
 * Opt-in sitemap.xml / robots.txt / llms.txt helpers.
 *
 * String bodies follow `ox_content_ssg::generate_site_maps`. The Vite plugin
 * writes those files during SSG without adding a NAPI surface.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { ResolvedSiteMapsOptions, SiteMapsOptions } from "./types";

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

  const published = input.pages
    .filter((page) => !page.draft && !page.unlisted && page.loc.length > 0)
    .slice()
    .sort((left, right) => (left.loc < right.loc ? -1 : left.loc > right.loc ? 1 : 0));

  const result: SiteMapsRenderResult = {
    sitemapXml: generateSitemapXml(published),
  };
  if (input.options.robots) {
    result.robotsTxt = generateRobotsTxt(input.sitemapLoc ?? "");
  }
  if (input.options.llms) {
    result.llmsTxt = generateLlmsTxt(input, published);
  }
  return result;
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

function generateSitemapXml(pages: readonly SiteMapPageInput[]): string {
  let xml =
    '<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n';
  for (const page of pages) {
    xml += "  <url>\n    <loc>";
    xml += escapeXml(page.loc);
    xml += "</loc>\n";
    const lastmod = formatLastmod(page.lastUpdated);
    if (lastmod) {
      xml += "    <lastmod>";
      xml += lastmod;
      xml += "</lastmod>\n";
    }
    xml += "  </url>\n";
  }
  xml += "</urlset>\n";
  return xml;
}

function generateRobotsTxt(sitemapLoc: string): string {
  let loc = "";
  for (const ch of sitemapLoc) {
    if (ch !== "\n" && ch !== "\r") {
      loc += ch;
    }
  }
  return `User-agent: *\nAllow: /\n\nSitemap: ${loc}\n`;
}

function generateLlmsTxt(input: SiteMapsRenderInput, pages: readonly SiteMapPageInput[]): string {
  let text = `# ${escapeLlmsText(input.siteName ?? "")}\n\n`;
  const siteDescription = input.siteDescription?.trim();
  if (siteDescription) {
    text += `> ${escapeLlmsText(siteDescription)}\n\n`;
  }
  text += "## Pages\n\n";
  for (const page of pages) {
    text += `- [${escapeLlmsText(page.title)}](${escapeLlmsUrl(page.loc)})`;
    const description = page.description?.trim();
    if (description) {
      text += `: ${escapeLlmsText(description)}`;
    }
    text += "\n";
  }
  return text;
}

function escapeXml(value: string): string {
  return value.replace(/[&<>"']/g, (ch) => {
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

function flattenText(value: string): string {
  return value.split(/\s+/u).filter(Boolean).join(" ");
}

function escapeLlmsText(value: string): string {
  return flattenText(value).replace(/[\\[\]()<>&"]/g, (ch) => {
    switch (ch) {
      case "\\":
        return "\\\\";
      case "[":
        return "\\[";
      case "]":
        return "\\]";
      case "(":
        return "\\(";
      case ")":
        return "\\)";
      case "<":
        return "&lt;";
      case ">":
        return "&gt;";
      case "&":
        return "&amp;";
      default:
        return "&quot;";
    }
  });
}

function escapeLlmsUrl(value: string): string {
  let escaped = "";
  for (const ch of value) {
    if (ch === " ") {
      escaped += "%20";
    } else if (ch === "(") {
      escaped += "%28";
    } else if (ch === ")") {
      escaped += "%29";
    } else if (ch !== "\n" && ch !== "\r" && ch !== "\t") {
      escaped += ch;
    }
  }
  return escaped;
}
