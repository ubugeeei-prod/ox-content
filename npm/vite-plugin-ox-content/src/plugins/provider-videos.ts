import {
  readProviderCache,
  resolveProviderCacheDir,
  writeProviderCache,
} from "./provider-disk-cache";
import { decodeProviderArticleAttr, escapeProviderArticleAttr } from "./provider-article-attrs";
import { vimeoMetaFromJson, type VideoMeta } from "./provider-video-metadata";
import {
  normalizeTwitchParents,
  parseVideoProviderReference,
  videoEmbedUrl,
  type VideoProviderReference,
} from "./provider-video-references";

const VIDEO_TAG = /<(vimeo|twitch)\b((?:[^>"']|"[^"]*"|'[^']*')*)>([\s\S]*?)<\/\1\s*>/gi;
const DEFAULT_TIMEOUT = 10_000;
const DEFAULT_CACHE_TTL = 3_600_000;

export interface ProviderVideoEmbedOptions {
  /** Fetch public video metadata at build time where supported. @default true */
  fetch?: boolean;
  /** Add lazy provider iframe URLs when supported. @default false */
  iframe?: boolean;
  /** Twitch embed parent domain or domains. Required for Twitch iframes. */
  parent?: string | string[];
  /** Metadata request timeout in milliseconds. @default 10000 */
  timeout?: number;
  /** Cache fetched metadata in memory for the current process. @default true */
  cache?: boolean;
  /** Cache TTL in milliseconds. @default 3600000 */
  cacheTTL?: number;
  /** Persist metadata across builds. Off by default. */
  persistCache?: boolean;
  /** Directory for the persistent cache. */
  cacheDir?: string;
}

export interface ResolvedProviderVideoEmbedOptions {
  fetch: boolean;
  iframe: boolean;
  parent: string[];
  timeout: number;
  cache: boolean;
  cacheTTL: number;
  persistCache: boolean;
  cacheDir?: string;
}

interface CacheRecord {
  data: VideoMeta | null;
  timestamp: number;
}

type ProviderVideoOptions = {
  vimeo?: ProviderVideoEmbedOptions | false;
  twitch?: ProviderVideoEmbedOptions | false;
};

type ResolvedVideoOptions = {
  vimeo: ResolvedProviderVideoEmbedOptions | false;
  twitch: ResolvedProviderVideoEmbedOptions | false;
};

export type ProviderVideoFetch = (input: string, init?: RequestInit) => Promise<Response>;

const memoryCache = new Map<string, CacheRecord>();
const inflight = new Map<string, Promise<VideoMeta | null>>();

export { parseVideoProviderReference, type VideoProviderReference };

export function clearProviderVideoCache(): void {
  memoryCache.clear();
  inflight.clear();
}

export function resolveProviderVideoEmbedOptions(
  options: ProviderVideoEmbedOptions = {},
): ResolvedProviderVideoEmbedOptions {
  return {
    fetch: options.fetch ?? true,
    iframe: options.iframe ?? false,
    parent: normalizeTwitchParents(options.parent),
    timeout: options.timeout ?? DEFAULT_TIMEOUT,
    cache: options.cache ?? true,
    cacheTTL: options.cacheTTL ?? DEFAULT_CACHE_TTL,
    persistCache: options.persistCache ?? false,
    cacheDir: options.cacheDir,
  };
}

export function normalizeProviderVideoOptions(
  options: boolean | ProviderVideoEmbedOptions | undefined,
): ProviderVideoEmbedOptions | false {
  if (!options) return false;
  return options === true ? {} : options;
}

export async function enrichProviderVideoEmbeds(
  html: string,
  options: ProviderVideoOptions,
  fetchImpl: ProviderVideoFetch = fetch,
): Promise<string> {
  if (!/<(?:vimeo|twitch)\b/i.test(html)) return html;
  const resolved: ResolvedVideoOptions = {
    vimeo: options.vimeo ? resolveProviderVideoEmbedOptions(options.vimeo) : false,
    twitch: options.twitch ? resolveProviderVideoEmbedOptions(options.twitch) : false,
  };
  if (!resolved.vimeo && !resolved.twitch) return html;

  const matches = Array.from(html.matchAll(VIDEO_TAG), (match) => ({
    tag: match[1] ?? "",
    attrs: match[2] ?? "",
    body: match[3] ?? "",
    full: match[0],
    index: match.index ?? 0,
    end: (match.index ?? 0) + match[0].length,
  }));
  const enriched = await Promise.all(matches.map((match) => enrichTag(match, resolved, fetchImpl)));

  let output = "";
  let cursor = 0;
  for (let index = 0; index < matches.length; index += 1) {
    const match = matches[index]!;
    output += html.slice(cursor, match.index);
    output += enriched[index];
    cursor = match.end;
  }
  return output + html.slice(cursor);
}

async function enrichTag(
  match: { tag: string; attrs: string; body: string; full: string },
  options: ResolvedVideoOptions,
  fetchImpl: ProviderVideoFetch,
): Promise<string> {
  const provider = match.tag.toLowerCase() as "vimeo" | "twitch";
  const option = options[provider];
  if (!option) return match.full;

  const href =
    readAttr(match.attrs, "url") ?? readAttr(match.attrs, "href") ?? readAttr(match.attrs, "src");
  if (!href) return match.full;
  const reference = parseVideoProviderReference(match.tag, href);
  if (!reference) return match.full;
  const meta =
    option.fetch && reference.apiUrl ? await fetchVideoMeta(reference, option, fetchImpl) : null;

  let attrs = match.attrs;
  attrs = appendAttr(attrs, "title", meta?.title ?? reference.title);
  attrs = appendAttr(attrs, "author", meta?.author ?? reference.author);
  attrs = appendAttr(attrs, "image", meta?.image);
  attrs = appendAttr(attrs, "duration", meta?.duration);
  attrs = appendAttr(attrs, "views", meta?.views);
  attrs = appendAttr(attrs, "status", meta?.status);
  if (option.iframe) {
    attrs = appendAttr(attrs, "embed", videoEmbedUrl(reference, option.parent, match.attrs));
  }
  return `<${match.tag}${attrs}>${match.body}</${match.tag}>`;
}

async function fetchVideoMeta(
  reference: VideoProviderReference,
  options: ResolvedProviderVideoEmbedOptions,
  fetchImpl: ProviderVideoFetch,
): Promise<VideoMeta | null> {
  if (!reference.apiUrl) return null;
  const now = Date.now();
  if (options.cache) {
    const cached = memoryCache.get(reference.apiUrl);
    if (cached && now - cached.timestamp < options.cacheTTL) return cached.data;
  }

  // A warm disk cache is what makes a clean build cheap; the memory map only
  // helps within one run.
  const cacheDir = resolveProviderCacheDir(options);
  const stored = options.cache
    ? await readProviderCache<VideoMeta>(cacheDir, reference.apiUrl, options.cacheTTL, now)
    : undefined;
  if (stored !== undefined) {
    memoryCache.set(reference.apiUrl, { data: stored, timestamp: now });
    return stored;
  }
  const pending = inflight.get(reference.apiUrl);
  if (pending) return pending;

  const request = requestVideoMeta(reference, options, fetchImpl).finally(() => {
    if (inflight.get(reference.apiUrl!) === request) inflight.delete(reference.apiUrl!);
  });
  inflight.set(reference.apiUrl, request);
  const data = await request;
  if (options.cache) memoryCache.set(reference.apiUrl, { data, timestamp: now });
  await writeProviderCache(cacheDir, reference.apiUrl, data, now);
  return data;
}

async function requestVideoMeta(
  reference: VideoProviderReference,
  options: ResolvedProviderVideoEmbedOptions,
  fetchImpl: ProviderVideoFetch,
): Promise<VideoMeta | null> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), options.timeout);
  try {
    const response = await fetchImpl(reference.apiUrl!, {
      headers: { accept: "application/json" },
      signal: controller.signal,
    });
    if (!response.ok) {
      warnVideoFallback(reference, String(response.status));
      return null;
    }
    return vimeoMetaFromJson(await response.json());
  } catch (error) {
    warnVideoFallback(reference, error instanceof Error ? error.message : "unknown error");
    return null;
  } finally {
    clearTimeout(timeout);
  }
}

function appendAttr(attrs: string, name: string, value: string | undefined): string {
  if (!value || readAttr(attrs, name)) return attrs;
  return `${attrs} ${name}="${escapeProviderArticleAttr(value)}"`;
}

function readAttr(attrs: string, name: string): string | undefined {
  const match = attrs.match(new RegExp(`\\b${name}\\s*=\\s*("([^"]*)"|'([^']*)')`, "i"));
  const value = match?.[2] ?? match?.[3];
  return value ? decodeProviderArticleAttr(value) : undefined;
}

function warnVideoFallback(reference: VideoProviderReference, reason: string): void {
  console.warn(
    `[ox-content] Failed to fetch ${reference.provider} metadata for ${reference.canonicalUrl}: ${reason}; rendering a link-only video card.`,
  );
}
