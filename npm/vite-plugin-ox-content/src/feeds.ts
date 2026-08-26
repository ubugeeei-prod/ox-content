/**
 * Opt-in RSS / Atom / JSON Feed helpers.
 *
 * String bodies follow `ox_content_ssg::generate_feeds`. The Vite plugin
 * writes those files during SSG without adding a NAPI surface.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import { applyAtomMeta, applyJsonMeta, applyRssMeta, channelMeta } from "./feed-channel-meta";
import { generateAtom, generateJson, generateRss, parseDate } from "./feed-format";
import type { FeedDocument, FeedEntry } from "./feed-format";
import { classifyPublishState } from "./publish-state";
import type {
  FeedChannelOptions,
  FeedFormat,
  FeedsOptions,
  ResolvedFeedChannel,
  ResolvedFeedsOptions,
  ResolvedPublishStateOptions,
} from "./types";

const MISSING_SITE_URL =
  "[ox-content] feeds is enabled but ssg.siteUrl is not set; RSS, Atom, and JSON feeds were not written";

const DEFAULT_FORMATS: FeedFormat[] = ["rss", "atom", "json"];
const DEFAULT_LIMIT = 20;
const DEFAULT_PATH = "/";
const CHANNEL_KEYS = new Set([
  "formats",
  "collection",
  "limit",
  "path",
  "title",
  "description",
  "language",
  "image",
  "favicon",
  "copyright",
]);

/** One collection entry considered for a feed. */
export interface FeedItemInput {
  title?: string;
  description?: string;
  path?: string;
  loc?: string;
  date?: unknown;
  lastUpdated?: unknown;
  draft?: unknown;
  unlisted?: unknown;
  frontmatter?: Record<string, unknown>;
}

/** Inputs for rendering feed bodies. */
export interface FeedsRenderInput {
  options?: ResolvedFeedsOptions | null;
  siteUrl?: string;
  siteName?: string;
  siteDescription?: string;
  base?: string;
  collections?: Record<string, readonly FeedItemInput[]>;
  collectionNames?: readonly string[];
  items?: readonly FeedItemInput[];
  publishState?: ResolvedPublishStateOptions;
}

/** Rendered feed bodies, or a skip warning. */
export interface FeedsRenderResult {
  rssXml?: string;
  atomXml?: string;
  jsonFeed?: string;
  warning?: string;
}

/** Inputs for writing feeds next to generated HTML. */
export interface WriteFeedFilesInput extends FeedsRenderInput {
  outDir: string;
  base: string;
}

/**
 * Resolves `feeds` with defaults.
 *
 * `false` / omitted stays off. `true` enables all three formats with
 * collection `content` (or the first configured collection) and limit 20.
 * A single object is one default feed. A named record or array writes
 * multiple feeds.
 */
export function resolveFeedsOptions(
  value: boolean | FeedsOptions | undefined,
): ResolvedFeedsOptions {
  if (!value) {
    return { enabled: false, ...resolveChannel({}) };
  }
  if (value === true) {
    return { enabled: true, ...resolveChannel({}) };
  }
  if (isFeedList(value)) {
    const feeds = value.map((entry) => resolveChannel(entry, entry.collection));
    const first = feeds[0] ?? resolveChannel({});
    return { enabled: feeds.length > 0, ...first, ...(feeds.length ? { feeds } : {}) };
  }
  if (isNamedRecord(value)) {
    const feeds = Object.entries(value).map(([name, entry]) => resolveChannel(entry, name));
    return { enabled: true, ...feeds[0], feeds };
  }
  return { enabled: true, ...resolveChannel(value) };
}

/** Picks `content`, else the first configured collection name. */
export function resolveFeedCollectionName(
  requested: string | undefined,
  collectionNames: readonly string[],
): string | undefined {
  if (requested) {
    return requested;
  }
  if (collectionNames.includes("content")) {
    return "content";
  }
  return collectionNames[0];
}

/** Builds RSS / Atom / JSON Feed bodies without writing files. */
export function generateFeeds(input: FeedsRenderInput): FeedsRenderResult {
  if (!input.options?.enabled) {
    return {};
  }
  if (!hasSiteUrl(input.siteUrl)) {
    return { warning: MISSING_SITE_URL };
  }

  const published = publishedItems(input);
  const doc = feedDocument(input);
  const meta = channelMeta(doc, input.options);
  const result: FeedsRenderResult = {};
  if (input.options.formats.includes("rss")) {
    result.rssXml = applyRssMeta(generateRss(doc, published), meta);
  }
  if (input.options.formats.includes("atom")) {
    result.atomXml = applyAtomMeta(generateAtom(doc, published), meta);
  }
  if (input.options.formats.includes("json")) {
    result.jsonFeed = applyJsonMeta(generateJson(doc, published), meta);
  }
  return result;
}

/** Writes enabled feed files into `outDir`. */
export async function writeFeedFiles(
  input: WriteFeedFilesInput,
): Promise<{ files: string[]; warning?: string }> {
  const files: string[] = [];
  for (const channel of feedChannels(input.options)) {
    const generated = generateFeeds({ ...input, options: { enabled: true, ...channel } });
    if (generated.warning) {
      return { files: [], warning: generated.warning };
    }
    const outputs: Array<[string, string]> = [
      [generated.rssXml, "feed.xml"],
      [generated.atomXml, "atom.xml"],
      [generated.jsonFeed, "feed.json"],
    ].filter((entry): entry is [string, string] => entry[0] != null);
    if (outputs.length === 0) {
      continue;
    }
    const dest = outputDir(input.outDir, channel.path);
    await fs.mkdir(dest, { recursive: true });
    for (const [body, name] of outputs) {
      const outputPath = path.join(dest, name);
      await fs.writeFile(outputPath, body, "utf8");
      files.push(outputPath);
    }
  }
  return { files };
}

function resolveChannel(value: FeedChannelOptions, name?: string): ResolvedFeedChannel {
  const channel: ResolvedFeedChannel = {
    formats: normalizeFormats(value.formats),
    limit: value.limit ?? DEFAULT_LIMIT,
    path: value.path ?? DEFAULT_PATH,
  };
  if (value.collection) {
    channel.collection = value.collection;
  }
  if (name) {
    channel.name = name;
  }
  for (const key of [
    "title",
    "description",
    "language",
    "image",
    "favicon",
    "copyright",
  ] as const) {
    if (value[key]) {
      channel[key] = value[key];
    }
  }
  return channel;
}

function isFeedList(value: FeedsOptions): value is readonly FeedChannelOptions[] {
  return Array.isArray(value);
}

function isNamedRecord(
  value: FeedChannelOptions | { [name: string]: FeedChannelOptions },
): value is { [name: string]: FeedChannelOptions } {
  const keys = Object.keys(value);
  return keys.length > 0 && keys.every((key) => !CHANNEL_KEYS.has(key));
}

function feedChannels(options?: ResolvedFeedsOptions | null): ResolvedFeedChannel[] {
  if (!options?.enabled) {
    return [];
  }
  return options.feeds?.length ? options.feeds : [options];
}

function normalizeFormats(formats: readonly FeedFormat[] | undefined): FeedFormat[] {
  if (!formats) {
    return [...DEFAULT_FORMATS];
  }
  const seen = new Set<FeedFormat>();
  const resolved: FeedFormat[] = [];
  for (const format of formats) {
    if ((format === "rss" || format === "atom" || format === "json") && !seen.has(format)) {
      seen.add(format);
      resolved.push(format);
    }
  }
  return resolved;
}

function hasSiteUrl(siteUrl: string | undefined): boolean {
  return Boolean(siteUrl && siteUrl.trim());
}

function homePageUrl(siteUrl: string | undefined, base = "/"): string {
  const origin = (siteUrl ?? "").trim().replace(/\/+$/, "");
  const prefix = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  return `${origin}${prefix}`;
}

function feedDocument(input: FeedsRenderInput): FeedDocument {
  const home = homePageUrl(input.siteUrl, input.base);
  const dir = (input.options?.path ?? DEFAULT_PATH).replace(/^\/+|\/+$/g, "");
  const prefix = dir ? `${home}${dir}/` : home;
  return {
    siteName: input.options?.title ?? input.siteName ?? "",
    siteDescription: input.options?.description ?? input.siteDescription,
    home,
    atomUrl: `${prefix}atom.xml`,
    jsonUrl: `${prefix}feed.json`,
  };
}

function outputDir(outDir: string, feedPath: string): string {
  const relative = feedPath.replace(/^\/+|\/+$/g, "");
  return relative ? path.join(outDir, relative) : outDir;
}

function rawItems(input: FeedsRenderInput): readonly FeedItemInput[] {
  if (input.items) {
    return input.items;
  }
  const names = input.collectionNames ?? Object.keys(input.collections ?? {});
  const name = resolveFeedCollectionName(input.options?.collection, names);
  return name ? (input.collections?.[name] ?? []) : [];
}

function publishedItems(input: FeedsRenderInput): FeedEntry[] {
  const published = rawItems(input)
    .filter((item) => !isExcludedFromFeed(item, input.publishState))
    .map((item) => normalizeItem(item, input))
    .filter((item) => item.loc.length > 0);
  published.sort((left, right) => {
    const dateCmp =
      (right.date?.unix ?? Number.NEGATIVE_INFINITY) -
      (left.date?.unix ?? Number.NEGATIVE_INFINITY);
    return dateCmp !== 0 ? dateCmp : left.loc < right.loc ? -1 : left.loc > right.loc ? 1 : 0;
  });
  return published.slice(0, input.options?.limit ?? DEFAULT_LIMIT);
}

function isExcludedFromFeed(
  item: FeedItemInput,
  publishState: ResolvedPublishStateOptions | undefined,
): boolean {
  const frontmatter = item.frontmatter ?? {};
  if (item.draft === true || frontmatter.draft === true) {
    return true;
  }
  if (item.unlisted === true || frontmatter.unlisted === true) {
    return true;
  }
  if (frontmatter.external === true) {
    return true;
  }
  if (!publishState?.enabled) {
    return false;
  }
  return !classifyPublishState(
    {
      ...frontmatter,
      ...(item.draft === true ? { draft: true } : {}),
      ...(item.unlisted === true ? { unlisted: true } : {}),
    },
    publishState,
  ).listed;
}

function normalizeItem(item: FeedItemInput, input: FeedsRenderInput): FeedEntry {
  return {
    title: item.title ?? "",
    description: typeof item.description === "string" ? item.description : undefined,
    loc: item.loc || itemLoc(input, item),
    date:
      parseDate(dateField(item.date ?? item.frontmatter?.date)) ??
      parseDate(dateField(item.lastUpdated ?? item.frontmatter?.lastUpdated)),
  };
}

function itemLoc(input: FeedsRenderInput, item: FeedItemInput): string {
  const home = homePageUrl(input.siteUrl, input.base);
  const urlPath = (item.path ?? "").replace(/^\/+|\/+$/g, "");
  return urlPath ? `${home}${urlPath}/` : home;
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
