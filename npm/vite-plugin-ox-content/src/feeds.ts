/**
 * Opt-in RSS / Atom / JSON Feed helpers.
 *
 * Bodies come from `ox_content_ssg::generate_feeds` through the NAPI binding.
 * What stays here is the part that binding does not model: which entries a
 * channel publishes, where each file lands, and writing them.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import { renderFeedBodies } from "./feeds-native";
import type { FeedDocument } from "./feed-format";
import {
  feedContentType,
  feedOutputFileName,
  feedOutputPath,
  homePageUrl,
  feedOutputWarning,
  siteUrlWarning,
  unsafeFeedPathWarning,
} from "./feeds-output";
import { publishedItems } from "./feeds-items";
import type {
  FeedChannelOptions,
  FeedFormat,
  FeedItemInput,
  FeedItemsResolveContext,
  FeedsOptions,
  ResolvedFeedChannel,
  ResolvedFeedsOptions,
  ResolvedPublishStateOptions,
} from "./types";

const DEFAULT_FORMATS: FeedFormat[] = ["rss", "atom", "json"];
const DEFAULT_LIMIT = 20;
const DEFAULT_PATH = "/";
const CHANNEL_KEYS = new Set([
  "formats",
  "collection",
  "items",
  "limit",
  "path",
  "title",
  "description",
  "language",
  "image",
  "favicon",
  "copyright",
]);

export type { FeedItemAttachment, FeedItemAuthor, FeedItemInput, FeedItemsSource } from "./types";

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

export interface RenderedFeedFile {
  path: string;
  contentType: string;
  content: string;
}

export interface RenderFeedFilesInput extends FeedsRenderInput {
  base: string;
  outDir?: string;
}

export interface RenderFeedFilesResult {
  files: RenderedFeedFile[];
  warning?: string;
}

export interface WriteFeedFilesInput extends RenderFeedFilesInput {
  outDir: string;
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
  const warning = siteUrlWarning(input.siteUrl);
  if (warning) {
    return { warning };
  }

  const bodies = renderFeedBodies(
    feedDocument(input),
    publishedItems(input),
    input.options.formats,
    input.options,
    input.siteUrl,
  );
  const result: FeedsRenderResult = {};
  if (bodies.rssXml != null) result.rssXml = bodies.rssXml;
  if (bodies.atomXml != null) result.atomXml = bodies.atomXml;
  if (bodies.jsonFeed != null) result.jsonFeed = bodies.jsonFeed;
  return result;
}

export async function renderFeedFiles(input: RenderFeedFilesInput): Promise<RenderFeedFilesResult> {
  if (input.options?.enabled) {
    const warning = siteUrlWarning(input.siteUrl);
    if (warning) {
      return { files: [], warning };
    }
  }

  const channels = feedChannels(input.options);
  const outputWarning = feedOutputWarning(input.outDir, channels);
  if (outputWarning) {
    return { files: [], warning: outputWarning };
  }

  const renderedFiles: RenderedFeedFile[] = [];
  for (const [index, channel] of channels.entries()) {
    const channelItems = await resolveChannelItems(channel, input);
    const generatedInput = channelItems === undefined ? input : { ...input, items: channelItems };
    const generated = generateFeeds({
      ...generatedInput,
      options: { enabled: true, ...channel },
    });
    if (generated.warning) {
      return { files: [], warning: generated.warning };
    }
    const outputs: Array<[FeedFormat, string]> = [
      ["rss", generated.rssXml],
      ["atom", generated.atomXml],
      ["json", generated.jsonFeed],
    ].filter((entry): entry is [FeedFormat, string] => entry[1] != null);
    if (outputs.length === 0) {
      continue;
    }
    for (const [format, content] of outputs) {
      const fileName = feedOutputFileName(format);
      const outputPath = feedOutputPath(channel.path, fileName);
      if (!outputPath) {
        return { files: [], warning: unsafeFeedPathWarning(channel, index) };
      }
      renderedFiles.push({
        path: outputPath,
        contentType: feedContentType(format),
        content,
      });
    }
  }
  return { files: renderedFiles };
}

export async function writeFeedFiles(
  input: WriteFeedFilesInput,
): Promise<{ files: string[]; warning?: string }> {
  const rendered = await renderFeedFiles(input);
  if (rendered.warning) {
    return { files: [], warning: rendered.warning };
  }

  const files: string[] = [];
  for (const file of rendered.files) {
    const outputPath = path.join(input.outDir, ...file.path.split("/"));
    await fs.mkdir(path.dirname(outputPath), { recursive: true });
    await fs.writeFile(outputPath, file.content, "utf8");
    files.push(outputPath);
  }
  return { files };
}

function resolveChannel(value: FeedChannelOptions, name?: string): ResolvedFeedChannel {
  if (value.collection && value.items != null) {
    throw new Error(
      `[ox-content] feeds channel ${feedChannelName(name)} cannot set both collection and items`,
    );
  }
  const channel: ResolvedFeedChannel = {
    formats: normalizeFormats(value.formats),
    limit: value.limit ?? DEFAULT_LIMIT,
    path: value.path ?? DEFAULT_PATH,
  };
  if (value.collection) {
    channel.collection = value.collection;
  }
  if (value.items != null) {
    channel.items = value.items;
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

async function resolveChannelItems(
  channel: ResolvedFeedChannel,
  input: RenderFeedFilesInput,
): Promise<readonly FeedItemInput[] | undefined> {
  const source = channel.items;
  if (source == null) {
    return undefined;
  }
  if (Array.isArray(source)) {
    return source;
  }
  const items = await source(feedItemsContext(channel, input));
  if (Array.isArray(items)) {
    return items;
  }
  throw new Error(
    `[ox-content] feeds channel ${feedChannelName(channel.name)} items must return an array`,
  );
}

function feedItemsContext(
  channel: ResolvedFeedChannel,
  input: RenderFeedFilesInput,
): FeedItemsResolveContext {
  return {
    name: channel.name,
    formats: channel.formats,
    path: channel.path,
    siteUrl: input.siteUrl,
    siteName: input.siteName,
    siteDescription: input.siteDescription,
    base: input.base,
    outDir: input.outDir,
  };
}

function feedChannelName(name: string | undefined): string {
  return name ? JSON.stringify(name) : "default";
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
