import type { EmbedFailure } from "./provider-failure";
import { classifyError, classifyStatus, warnEmbedFailure } from "./provider-failure";
import {
  readProviderCache,
  resolveProviderCacheDir,
  writeProviderCache,
} from "./provider-disk-cache";
import { decodeProviderArticleAttr, escapeProviderArticleAttr } from "./provider-article-attrs";

const PLAYGROUND_TAG =
  /<(codepen|jsfiddle|observable)\b((?:[^>"']|"[^"]*"|'[^']*')*)>([\s\S]*?)<\/\1\s*>/gi;
const DEFAULT_TIMEOUT = 10_000;
const DEFAULT_CACHE_TTL = 3_600_000;

export interface ProviderPlaygroundEmbedOptions {
  /** Fetch playground metadata at build time where a public endpoint exists. @default true */
  fetch?: boolean;
  /** Add provider iframe URLs for supported providers. @default false */
  iframe?: boolean;
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

export interface ResolvedProviderPlaygroundEmbedOptions {
  fetch: boolean;
  iframe: boolean;
  timeout: number;
  cache: boolean;
  cacheTTL: number;
  persistCache: boolean;
  cacheDir?: string;
}

export interface PlaygroundReference {
  provider: "codepen" | "jsfiddle" | "observable";
  canonicalUrl: string;
  title: string;
  author?: string;
  apiUrl?: string;
  embedUrl?: string;
}

interface PlaygroundMeta {
  title?: string;
  author?: string;
  image?: string;
}

interface CacheRecord {
  data: PlaygroundMeta | null;
  timestamp: number;
}

export type ProviderPlaygroundFetch = (input: string, init?: RequestInit) => Promise<Response>;

const memoryCache = new Map<string, CacheRecord>();
const inflight = new Map<string, Promise<PlaygroundMeta | null>>();

export function clearProviderPlaygroundCache(): void {
  memoryCache.clear();
  inflight.clear();
}

export function resolveProviderPlaygroundEmbedOptions(
  options: ProviderPlaygroundEmbedOptions = {},
): ResolvedProviderPlaygroundEmbedOptions {
  return {
    fetch: options.fetch ?? true,
    iframe: options.iframe ?? false,
    timeout: options.timeout ?? DEFAULT_TIMEOUT,
    cache: options.cache ?? true,
    cacheTTL: options.cacheTTL ?? DEFAULT_CACHE_TTL,
    persistCache: options.persistCache ?? false,
    cacheDir: options.cacheDir,
  };
}

export function normalizeProviderPlaygroundOptions(
  options: boolean | ProviderPlaygroundEmbedOptions | undefined,
): ProviderPlaygroundEmbedOptions | false {
  if (!options) return false;
  return options === true ? {} : options;
}

export async function enrichProviderPlaygroundEmbeds(
  html: string,
  options: ProviderPlaygroundEmbedOptions | false,
  fetchImpl: ProviderPlaygroundFetch = fetch,
): Promise<string> {
  if (!options || !/<(?:codepen|jsfiddle|observable)\b/i.test(html)) return html;
  const resolved = resolveProviderPlaygroundEmbedOptions(options);
  const matches = Array.from(html.matchAll(PLAYGROUND_TAG), (match) => ({
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

export function parsePlaygroundReference(tag: string, input: string): PlaygroundReference | null {
  try {
    const url = new URL(input);
    if (url.protocol !== "https:" || url.username || url.password) return null;
    const segments = safeSegments(url);
    if (!segments) return null;
    switch (tag.toLowerCase()) {
      case "codepen":
        return codepenReference(url, segments);
      case "jsfiddle":
        return jsfiddleReference(url, segments);
      case "observable":
        return observableReference(url, segments);
      default:
        return null;
    }
  } catch {
    return null;
  }
}

async function enrichTag(
  match: { tag: string; attrs: string; body: string; full: string },
  options: ResolvedProviderPlaygroundEmbedOptions,
  fetchImpl: ProviderPlaygroundFetch,
): Promise<string> {
  const href =
    readAttr(match.attrs, "url") ?? readAttr(match.attrs, "href") ?? readAttr(match.attrs, "src");
  if (!href) return match.full;
  const reference = parsePlaygroundReference(match.tag, href);
  if (!reference) return match.full;
  const meta =
    options.fetch && reference.apiUrl
      ? await fetchPlaygroundMeta(reference, options, fetchImpl)
      : null;

  let attrs = match.attrs;
  attrs = appendAttr(attrs, "title", meta?.title ?? reference.title);
  attrs = appendAttr(attrs, "author", meta?.author ?? reference.author);
  attrs = appendAttr(attrs, "image", meta?.image);
  if (options.iframe) {
    attrs = appendAttr(attrs, "embed", reference.embedUrl);
  }
  return `<${match.tag}${attrs}>${match.body}</${match.tag}>`;
}

async function fetchPlaygroundMeta(
  reference: PlaygroundReference,
  options: ResolvedProviderPlaygroundEmbedOptions,
  fetchImpl: ProviderPlaygroundFetch,
): Promise<PlaygroundMeta | null> {
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
    ? await readProviderCache<PlaygroundMeta>(cacheDir, reference.apiUrl, options.cacheTTL, now)
    : undefined;
  if (stored !== undefined) {
    memoryCache.set(reference.apiUrl, { data: stored, timestamp: now });
    return stored;
  }
  const pending = inflight.get(reference.apiUrl);
  if (pending) return pending;

  const request = requestPlaygroundMeta(reference, options, fetchImpl).finally(() => {
    if (inflight.get(reference.apiUrl!) === request) inflight.delete(reference.apiUrl!);
  });
  inflight.set(reference.apiUrl, request);
  const data = await request;
  if (options.cache) memoryCache.set(reference.apiUrl, { data, timestamp: now });
  await writeProviderCache(cacheDir, reference.apiUrl, data, now);
  return data;
}

async function requestPlaygroundMeta(
  reference: PlaygroundReference,
  options: ResolvedProviderPlaygroundEmbedOptions,
  fetchImpl: ProviderPlaygroundFetch,
): Promise<PlaygroundMeta | null> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), options.timeout);
  try {
    const response = await fetchImpl(reference.apiUrl!, {
      headers: { accept: "application/json" },
      signal: controller.signal,
    });
    if (!response.ok) {
      warnPlaygroundFallback(reference, classifyStatus(response.status));
      return null;
    }
    return playgroundMetaFromJson(await response.json());
  } catch (error) {
    warnPlaygroundFallback(reference, classifyError(error));
    return null;
  } finally {
    clearTimeout(timeout);
  }
}

function codepenReference(url: URL, segments: string[]): PlaygroundReference | null {
  if (url.hostname !== "codepen.io" || segments[1] !== "pen") return null;
  const author = safeSegment(segments[0]);
  const slug = safeSegment(segments[2]);
  if (!author || !slug) return null;
  const canonicalUrl = `https://codepen.io/${author}/pen/${slug}`;
  return {
    provider: "codepen",
    canonicalUrl,
    title: titleize(slug),
    author,
    apiUrl: `https://codepen.io/api/oembed?format=json&url=${encodeURIComponent(canonicalUrl)}`,
    embedUrl: `https://codepen.io/${author}/embed/${slug}?default-tab=result`,
  };
}

function jsfiddleReference(url: URL, segments: string[]): PlaygroundReference | null {
  if (!["jsfiddle.net", "www.jsfiddle.net"].includes(url.hostname) || segments.length < 1) {
    return null;
  }
  const safe = segments.slice(0, 3).map(safeSegment);
  if (safe.some((segment) => !segment)) return null;
  const path = safe.join("/");
  const slug = safe[1] ?? safe[0];
  return {
    provider: "jsfiddle",
    canonicalUrl: `https://jsfiddle.net/${path}/`,
    title: titleize(slug!),
    ...(safe.length > 1 ? { author: safe[0] } : {}),
    embedUrl: `https://jsfiddle.net/${path}/embedded/result/`,
  };
}

function observableReference(url: URL, segments: string[]): PlaygroundReference | null {
  if (url.hostname !== "observablehq.com" || segments.length < 2) return null;
  const first = safeObservableSegment(segments[0]);
  const second = safeObservableSegment(segments[1]);
  if (!first || !second) return null;
  if (first.startsWith("@")) {
    return {
      provider: "observable",
      canonicalUrl: `https://observablehq.com/${first}/${second}`,
      title: titleize(second),
      author: first,
      embedUrl: `https://observablehq.com/embed/${first}/${second}`,
    };
  }
  if (first === "d") {
    return {
      provider: "observable",
      canonicalUrl: `https://observablehq.com/d/${second}`,
      title: `Notebook ${second}`,
      embedUrl: `https://observablehq.com/embed/d/${second}`,
    };
  }
  return null;
}

function playgroundMetaFromJson(value: unknown): PlaygroundMeta | null {
  const item = record(value);
  if (!item) return null;
  const title = trimmed(item.title);
  const author = trimmed(item.author_name);
  const image = safeHttps(trimmed(item.thumbnail_url));
  return title || author || image ? compactMeta({ title, author, image }) : null;
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

function safeSegments(url: URL): string[] | null {
  try {
    return url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  } catch {
    return null;
  }
}

function safeSegment(value: string | undefined): string | undefined {
  return value && /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/.test(value) ? value : undefined;
}

function safeObservableSegment(value: string | undefined): string | undefined {
  return value && /^@?[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/.test(value) ? value : undefined;
}

function safeHttps(value: string | undefined): string | undefined {
  try {
    const url = value ? new URL(value) : null;
    return url?.protocol === "https:" && !url.username && !url.password ? value : undefined;
  } catch {
    return undefined;
  }
}

function titleize(value: string): string {
  return value.replaceAll("-", " ").replaceAll("_", " ");
}

function warnPlaygroundFallback(reference: PlaygroundReference, failure: EmbedFailure): void {
  warnEmbedFailure(reference.provider, reference.apiUrl, failure, "a link-only playground card");
}

function compactMeta(meta: PlaygroundMeta): PlaygroundMeta {
  return Object.fromEntries(Object.entries(meta).filter(([, value]) => value)) as PlaygroundMeta;
}

function trimmed(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function record(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}
