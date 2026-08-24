/**
 * Opt-in RSS 2.0 / Atom / JSON Feed 1.1 helpers.
 *
 * String bodies follow `ox_content_ssg::generate_feeds`. The Vite plugin
 * writes those files during SSG without adding a NAPI surface.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import {
  FEED_EPOCH,
  compareFeedDates,
  formatFeedRfc3339,
  formatFeedRfc822,
  newestFeedUpdated,
  parseFeedDate,
} from "./feeds-dates";
import type { FeedsOptions, ResolvedFeedsOptions } from "./types";

const MISSING_SITE_URL =
  "[ox-content] feeds is enabled but ssg.siteUrl is not set; feed.xml, atom.xml, and feed.json were not written";

/** One feed entry built from a collection page. */
export interface FeedItemInput {
  title: string;
  loc: string;
  description?: string;
  date?: string;
}

/** Inputs for rendering feed bodies. */
export interface FeedsRenderInput {
  options?: ResolvedFeedsOptions | null;
  siteUrl?: string;
  siteName?: string;
  siteDescription?: string;
  homePageUrl?: string;
  feedRssLoc?: string;
  feedAtomLoc?: string;
  feedJsonLoc?: string;
  items: readonly FeedItemInput[];
}

/** Rendered feed bodies, or a skip warning. */
export interface FeedsRenderResult {
  rssXml?: string;
  atomXml?: string;
  jsonFeed?: string;
  warning?: string;
}

/** Inputs for writing feeds next to generated HTML. */
export interface WriteFeedFilesInput {
  outDir: string;
  siteUrl?: string;
  base: string;
  siteName?: string;
  siteDescription?: string;
  options?: ResolvedFeedsOptions;
  items: readonly FeedItemInput[];
}

/**
 * Resolves `feeds` with defaults.
 *
 * `false` / omitted stays off. `true` enables RSS, Atom, and JSON Feed. An
 * object enables the feature and overrides only the fields the site set.
 */
export function resolveFeedsOptions(
  value: boolean | FeedsOptions | undefined,
): ResolvedFeedsOptions {
  const defaults: ResolvedFeedsOptions = {
    enabled: false,
    rss: true,
    atom: true,
    json: true,
    limit: 20,
    collection: "content",
    dateField: "date",
  };
  if (!value) return defaults;
  if (value === true) return { ...defaults, enabled: true };
  return {
    enabled: true,
    rss: value.rss ?? true,
    atom: value.atom ?? true,
    json: value.json ?? true,
    limit: value.limit ?? 20,
    collection: value.collection ?? "content",
    dateField: value.dateField ?? "date",
  };
}

/** Reads a frontmatter date field used to sort feed items. */
export function feedDateFromFrontmatter(
  frontmatter: Record<string, unknown> | undefined,
  dateField = "date",
): string | undefined {
  const value = frontmatter?.[dateField];
  if (typeof value === "string") {
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : undefined;
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return new Date(value).toISOString();
  }
  return undefined;
}

/** Builds RSS / Atom / JSON Feed bodies without writing files. */
export function generateFeeds(input: FeedsRenderInput): FeedsRenderResult {
  if (!input.options?.enabled) {
    return {};
  }
  if (!hasSiteUrl(input.siteUrl)) {
    return { warning: MISSING_SITE_URL };
  }
  const selected = selectItems(input.items, input.options.limit);
  const home = resolveUrl(input.homePageUrl, input.siteUrl, "");
  const result: FeedsRenderResult = {};
  if (input.options.rss) {
    result.rssXml = generateRss(input, home, selected);
  }
  if (input.options.atom) {
    result.atomXml = generateAtom(
      input,
      home,
      resolveUrl(input.feedAtomLoc, input.siteUrl, "atom.xml"),
      selected,
    );
  }
  if (input.options.json) {
    result.jsonFeed = generateJson(
      input,
      home,
      resolveUrl(input.feedJsonLoc, input.siteUrl, "feed.json"),
      selected,
    );
  }
  return result;
}

/** Writes enabled feed files into `outDir`. */
export async function writeFeedFiles(
  input: WriteFeedFilesInput,
): Promise<{ files: string[]; warning?: string }> {
  const generated = generateFeeds({
    options: input.options,
    siteUrl: input.siteUrl,
    siteName: input.siteName,
    siteDescription: input.siteDescription,
    homePageUrl: absoluteFeedUrl(input.siteUrl, input.base, ""),
    feedRssLoc: absoluteFeedUrl(input.siteUrl, input.base, "feed.xml"),
    feedAtomLoc: absoluteFeedUrl(input.siteUrl, input.base, "atom.xml"),
    feedJsonLoc: absoluteFeedUrl(input.siteUrl, input.base, "feed.json"),
    items: input.items,
  });
  if (generated.warning) {
    return { files: [], warning: generated.warning };
  }
  const outputs = (
    [
      [generated.rssXml, "feed.xml"],
      [generated.atomXml, "atom.xml"],
      [generated.jsonFeed, "feed.json"],
    ] as const
  ).filter((entry): entry is [string, string] => entry[0] != null);
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

function hasSiteUrl(siteUrl: string | undefined): boolean {
  return Boolean(siteUrl && siteUrl.trim());
}

function absoluteFeedUrl(siteUrl: string | undefined, base: string, name: string): string {
  if (!hasSiteUrl(siteUrl)) {
    return "";
  }
  const origin = (siteUrl ?? "").trim().replace(/\/+$/, "");
  const prefix = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  return `${origin}${prefix}${name}`;
}

function resolveUrl(
  explicit: string | undefined,
  siteUrl: string | undefined,
  suffix: string,
): string {
  if (explicit?.trim()) {
    return explicit.trim();
  }
  const origin = (siteUrl ?? "").trim().replace(/\/+$/, "");
  return suffix ? `${origin}/${suffix}` : `${origin}/`;
}

function selectItems(items: readonly FeedItemInput[], limit: number): FeedItemInput[] {
  return items
    .filter((item) => item.loc.length > 0)
    .slice()
    .sort(compareItems)
    .slice(0, limit);
}

function compareItems(left: FeedItemInput, right: FeedItemInput): number {
  const leftDate = parseFeedDate(left.date);
  const rightDate = parseFeedDate(right.date);
  if (leftDate && rightDate) {
    const byDate = compareFeedDates(rightDate, leftDate);
    return byDate !== 0 ? byDate : left.loc.localeCompare(right.loc);
  }
  if (leftDate) return -1;
  if (rightDate) return 1;
  return left.loc.localeCompare(right.loc);
}

function generateRss(
  input: FeedsRenderInput,
  home: string,
  items: readonly FeedItemInput[],
): string {
  const rows = items.map((item) => {
    const date = parseFeedDate(item.date);
    return `    <item>\n      ${xml("title", item.title)}\n      ${xml("link", item.loc)}\n      <guid isPermaLink="true">${escapeXml(item.loc)}</guid>\n${item.description == null ? "" : `      ${xml("description", item.description)}\n`}${date ? `      ${xml("pubDate", formatFeedRfc822(date))}\n` : ""}    </item>\n`;
  });
  return `<?xml version="1.0" encoding="UTF-8"?>\n<rss version="2.0">\n  <channel>\n    ${xml("title", input.siteName ?? "")}\n    ${xml("link", home)}\n    ${xml("description", input.siteDescription?.trim() || input.siteName || "")}\n${rows.join("")}  </channel>\n</rss>\n`;
}

function generateAtom(
  input: FeedsRenderInput,
  home: string,
  atomLoc: string,
  items: readonly FeedItemInput[],
): string {
  const subtitle = input.siteDescription?.trim()
    ? `  ${xml("subtitle", input.siteDescription.trim())}\n`
    : "";
  const rows = items.map((item) => {
    const date = parseFeedDate(item.date);
    return `  <entry>\n    ${xml("title", item.title)}\n    <link href="${escapeXml(item.loc)}"/>\n    ${xml("id", item.loc)}\n    ${xml("updated", date ? formatFeedRfc3339(date) : FEED_EPOCH)}\n${item.description == null ? "" : `    ${xml("summary", item.description)}\n`}  </entry>\n`;
  });
  return `<?xml version="1.0" encoding="UTF-8"?>\n<feed xmlns="http://www.w3.org/2005/Atom">\n  ${xml("title", input.siteName ?? "")}\n  <link href="${escapeXml(home)}" rel="alternate"/>\n  <link href="${escapeXml(atomLoc)}" rel="self"/>\n  ${xml("id", home)}\n  ${xml("updated", newestFeedUpdated(items.map((item) => item.date)))}\n${subtitle}${rows.join("")}</feed>\n`;
}

function generateJson(
  input: FeedsRenderInput,
  home: string,
  jsonLoc: string,
  items: readonly FeedItemInput[],
): string {
  const description = input.siteDescription?.trim()
    ? `,\n  "description": "${escapeJson(input.siteDescription.trim())}"`
    : "";
  const rows = items.map((item, index) => {
    const date = parseFeedDate(item.date);
    const extra = [
      item.description == null ? "" : `,\n      "content_text": "${escapeJson(item.description)}"`,
      date ? `,\n      "date_published": "${formatFeedRfc3339(date)}"` : "",
    ].join("");
    return `${index > 0 ? "," : ""}\n    {\n      "id": "${escapeJson(item.loc)}",\n      "url": "${escapeJson(item.loc)}",\n      "title": "${escapeJson(item.title)}"${extra}\n    }`;
  });
  return `{\n  "version": "https://jsonfeed.org/version/1.1",\n  "title": "${escapeJson(input.siteName ?? "")}",\n  "home_page_url": "${escapeJson(home)}",\n  "feed_url": "${escapeJson(jsonLoc)}"${description},\n  "items": [${rows.join("")}\n  ]\n}\n`;
}

function xml(tag: string, value: string): string {
  return `<${tag}>${escapeXml(value)}</${tag}>`;
}

function escapeXml(value: string): string {
  return value.replace(/[&<>"']/g, (ch) =>
    ch === "&"
      ? "&amp;"
      : ch === "<"
        ? "&lt;"
        : ch === ">"
          ? "&gt;"
          : ch === '"'
            ? "&quot;"
            : "&#39;",
  );
}

function escapeJson(value: string): string {
  let escaped = "";
  for (const ch of value) {
    const code = ch.charCodeAt(0);
    if (ch === "\\") escaped += "\\\\";
    else if (ch === '"') escaped += '\\"';
    else if (ch === "\n") escaped += "\\n";
    else if (ch === "\r") escaped += "\\r";
    else if (ch === "\t") escaped += "\\t";
    else if (ch === "<") escaped += "\\u003c";
    else if (ch === ">") escaped += "\\u003e";
    else if (ch === "&") escaped += "\\u0026";
    else if (code < 0x20) escaped += `\\u${code.toString(16).padStart(4, "0")}`;
    else escaped += ch;
  }
  return escaped;
}
