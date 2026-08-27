import type { EmbedFailure } from "./provider-failure";
import { classifyError, classifyStatus, warnEmbedFailure } from "./provider-failure";
import {
  readProviderCache,
  resolveProviderCacheDir,
  writeProviderCache,
} from "./provider-disk-cache";
import { decodeProviderArticleAttr, escapeProviderArticleAttr } from "./provider-article-attrs";
import {
  appendAttr,
  compactMeta,
  count,
  dateLabel,
  excerpt,
  namesList,
  readAttr,
  record,
  safeHttps,
  safeId,
  safeSegments,
  safeSlug,
  trimmed,
} from "./provider-article-values";

const ARTICLE_TAG = /<(qiita|zenn)\b((?:[^>"']|"[^"]*"|'[^']*')*)>([\s\S]*?)<\/\1\s*>/gi;
const DEFAULT_TIMEOUT = 10_000;
const DEFAULT_CACHE_TTL = 3_600_000;

export interface ProviderArticleEmbedOptions {
  /** Fetch article metadata at build time. @default true */
  fetch?: boolean;
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

export interface ResolvedProviderArticleEmbedOptions {
  fetch: boolean;
  timeout: number;
  cache: boolean;
  cacheTTL: number;
  persistCache: boolean;
  cacheDir?: string;
}

interface ArticleReference {
  provider: "qiita" | "zenn";
  apiUrl: string;
}

export interface ArticleMeta {
  title?: string;
  author?: string;
  avatar?: string;
  dateTime?: string;
  dateLabel?: string;
  tags?: string;
  likes?: string;
  comments?: string;
  image?: string;
  body?: string;
}

interface CacheRecord {
  data: ArticleMeta | null;
  timestamp: number;
}
type ResolvedArticleOptions = {
  qiita: ResolvedProviderArticleEmbedOptions | false;
  zenn: ResolvedProviderArticleEmbedOptions | false;
};

export type ProviderArticleFetch = (input: string, init?: RequestInit) => Promise<Response>;

const memoryCache = new Map<string, CacheRecord>();
const inflight = new Map<string, Promise<ArticleMeta | null>>();

export function clearProviderArticleCache(): void {
  memoryCache.clear();
  inflight.clear();
}

export function resolveProviderArticleEmbedOptions(
  options: ProviderArticleEmbedOptions = {},
): ResolvedProviderArticleEmbedOptions {
  return {
    fetch: options.fetch ?? true,
    timeout: options.timeout ?? DEFAULT_TIMEOUT,
    cache: options.cache ?? true,
    cacheTTL: options.cacheTTL ?? DEFAULT_CACHE_TTL,
    persistCache: options.persistCache ?? false,
    cacheDir: options.cacheDir,
  };
}

export async function enrichProviderArticleEmbeds(
  html: string,
  options: {
    qiita?: ProviderArticleEmbedOptions | false;
    zenn?: ProviderArticleEmbedOptions | false;
  },
  fetchImpl: ProviderArticleFetch = fetch,
): Promise<string> {
  if (!/<(?:qiita|zenn)\b/i.test(html)) return html;

  const resolved: ResolvedArticleOptions = {
    qiita: options.qiita ? resolveProviderArticleEmbedOptions(options.qiita) : false,
    zenn: options.zenn ? resolveProviderArticleEmbedOptions(options.zenn) : false,
  };
  if (!resolved.qiita && !resolved.zenn) return html;

  const matches = Array.from(html.matchAll(ARTICLE_TAG), (match) => ({
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

export function normalizeProviderArticleOptions(
  options: boolean | ProviderArticleEmbedOptions | undefined,
): ProviderArticleEmbedOptions | false {
  if (!options) return false;
  return options === true ? {} : options;
}

async function enrichTag(
  match: { tag: string; attrs: string; body: string; full: string },
  options: ResolvedArticleOptions,
  fetchImpl: ProviderArticleFetch,
): Promise<string> {
  const provider = match.tag.toLowerCase() as "qiita" | "zenn";
  const option = options[provider];
  if (!option || !option.fetch) return match.full;

  const reference = articleReference(provider, match.attrs);
  if (!reference) return match.full;

  const meta = await fetchArticleMeta(reference, option, fetchImpl);
  if (!meta) return match.full;

  let attrs = match.attrs;
  attrs = appendAttr(attrs, "title", meta.title);
  attrs = appendAttr(attrs, "author", meta.author);
  attrs = appendAttr(attrs, "avatar", meta.avatar);
  attrs = appendAttr(attrs, "dateTime", meta.dateTime);
  attrs = appendAttr(attrs, "dateLabel", meta.dateLabel);
  attrs = appendAttr(attrs, "tags", meta.tags);
  attrs = appendAttr(attrs, "likes", meta.likes);
  attrs = appendAttr(attrs, "comments", meta.comments);
  attrs = appendAttr(attrs, "image", meta.image);

  const body = match.body.trim() ? match.body : (meta.body ?? match.body);
  return `<${match.tag}${attrs}>${body}</${match.tag}>`;
}

async function fetchArticleMeta(
  reference: ArticleReference,
  options: ResolvedProviderArticleEmbedOptions,
  fetchImpl: ProviderArticleFetch,
): Promise<ArticleMeta | null> {
  const key = `${reference.provider}:${reference.apiUrl}`;
  const now = Date.now();
  if (options.cache) {
    const cached = memoryCache.get(key);
    if (cached && now - cached.timestamp < options.cacheTTL) return cached.data;
  }

  // A warm disk cache is what makes a clean build cheap; the memory map only
  // helps within one run.
  const cacheDir = resolveProviderCacheDir(options);
  const stored = options.cache
    ? await readProviderCache<ArticleMeta>(cacheDir, key, options.cacheTTL, now)
    : undefined;
  if (stored !== undefined) {
    memoryCache.set(key, { data: stored, timestamp: now });
    return stored;
  }

  const pending = inflight.get(key);
  if (pending) return pending;

  const request = requestArticleMeta(reference, options, fetchImpl).finally(() => {
    if (inflight.get(key) === request) inflight.delete(key);
  });
  inflight.set(key, request);

  const data = await request;
  if (options.cache) memoryCache.set(key, { data, timestamp: now });
  await writeProviderCache(cacheDir, key, data, now);
  return data;
}

async function requestArticleMeta(
  reference: ArticleReference,
  options: ResolvedProviderArticleEmbedOptions,
  fetchImpl: ProviderArticleFetch,
): Promise<ArticleMeta | null> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), options.timeout);
  try {
    const response = await fetchImpl(reference.apiUrl, {
      headers: { accept: "application/json" },
      signal: controller.signal,
    });
    if (!response.ok) {
      warnArticleFallback(reference, classifyStatus(response.status));
      return null;
    }
    const value = await response.json();
    return reference.provider === "qiita" ? qiitaMeta(value) : zennMeta(value);
  } catch (error) {
    warnArticleFallback(reference, classifyError(error));
    return null;
  } finally {
    clearTimeout(timeout);
  }
}

function warnArticleFallback(reference: ArticleReference, failure: EmbedFailure): void {
  warnEmbedFailure(reference.provider, reference.apiUrl, failure, "a link-only article card");
}

function articleReference(provider: "qiita" | "zenn", attrs: string): ArticleReference | null {
  const href = readAttr(attrs, "url") ?? readAttr(attrs, "href") ?? readAttr(attrs, "src");
  if (!href) return null;
  return provider === "qiita" ? qiitaReference(href) : zennReference(href);
}

function qiitaReference(input: string): ArticleReference | null {
  try {
    const url = new URL(input);
    if (url.protocol !== "https:" || url.hostname.toLowerCase() !== "qiita.com") return null;
    if (url.username || url.password) return null;
    const segments = safeSegments(url);
    if (segments?.[1] !== "items" || !safeSlug(segments[0]) || !safeId(segments[2])) return null;
    return { provider: "qiita", apiUrl: `https://qiita.com/api/v2/items/${segments[2]}` };
  } catch {
    return null;
  }
}

function zennReference(input: string): ArticleReference | null {
  try {
    const url = new URL(input);
    if (url.protocol !== "https:" || url.hostname.toLowerCase() !== "zenn.dev") return null;
    if (url.username || url.password) return null;
    const segments = safeSegments(url);
    if (!segments || !safeSlug(segments[0]) || !safeSlug(segments[2])) return null;
    const kind = segments[1];
    if (kind !== "articles" && kind !== "books" && kind !== "scraps") return null;
    return { provider: "zenn", apiUrl: `https://zenn.dev/api/${kind}/${segments[2]}` };
  } catch {
    return null;
  }
}

function qiitaMeta(value: unknown): ArticleMeta | null {
  const item = record(value);
  const title = trimmed(item?.title);
  if (!item || !title) return null;

  const user = record(item.user);
  return compactMeta({
    title,
    author: trimmed(user?.name) ?? trimmed(user?.id),
    avatar: safeHttps(trimmed(user?.profile_image_url)),
    dateTime: trimmed(item.created_at),
    dateLabel: dateLabel(trimmed(item.created_at)),
    tags: namesList(item.tags),
    likes: count(item.likes_count),
    comments: count(item.comments_count),
    body: excerpt(trimmed(item.body)),
  });
}

function zennMeta(value: unknown): ArticleMeta | null {
  const root = record(value);
  const item = record(root?.article) ?? record(root?.book) ?? record(root?.scrap);
  const title = trimmed(item?.title);
  if (!item || !title) return null;

  const user = record(item.user);
  return compactMeta({
    title,
    author: trimmed(user?.name) ?? trimmed(user?.username),
    avatar: safeHttps(trimmed(user?.avatar_small_url) ?? trimmed(user?.avatar_url)),
    dateTime: trimmed(item.published_at) ?? trimmed(item.created_at),
    dateLabel: dateLabel(trimmed(item.published_at) ?? trimmed(item.created_at)),
    tags: namesList(item.topics),
    likes: count(item.liked_count),
    comments: count(item.comments_count),
    image: safeHttps(trimmed(item.og_image_url) ?? trimmed(item.cover_image_small_url)),
    body: excerpt(trimmed(item.summary)),
  });
}
