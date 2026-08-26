import { decodeProviderArticleAttr, escapeProviderArticleAttr } from "./provider-article-attrs";

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
}

export interface ResolvedProviderArticleEmbedOptions {
  fetch: boolean;
  timeout: number;
  cache: boolean;
  cacheTTL: number;
}

interface ArticleReference {
  provider: "qiita" | "zenn";
  apiUrl: string;
}

interface ArticleMeta {
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

  const pending = inflight.get(key);
  if (pending) return pending;

  const request = requestArticleMeta(reference, options, fetchImpl).finally(() => {
    if (inflight.get(key) === request) inflight.delete(key);
  });
  inflight.set(key, request);

  const data = await request;
  if (options.cache) memoryCache.set(key, { data, timestamp: now });
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
    if (!response.ok) return null;
    const value = await response.json();
    return reference.provider === "qiita" ? qiitaMeta(value) : zennMeta(value);
  } catch {
    return null;
  } finally {
    clearTimeout(timeout);
  }
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

function appendAttr(attrs: string, name: string, value: string | undefined): string {
  if (!value || readAttr(attrs, name)) return attrs;
  return `${attrs} ${name}="${escapeProviderArticleAttr(value)}"`;
}

function readAttr(attrs: string, name: string): string | undefined {
  const match = attrs.match(new RegExp(`\\b${name}\\s*=\\s*("([^"]*)"|'([^']*)')`, "i"));
  const value = match?.[2] ?? match?.[3];
  return value ? decodeProviderArticleAttr(value) : undefined;
}

function namesList(value: unknown): string | undefined {
  if (!Array.isArray(value)) return undefined;
  const names = value.map((item) => trimmed(record(item)?.name)).filter(Boolean);
  return names.length ? names.join(", ") : undefined;
}

function count(value: unknown): string | undefined {
  return typeof value === "number" && Number.isFinite(value) && value >= 0
    ? String(Math.floor(value))
    : undefined;
}

function dateLabel(value: string | undefined): string | undefined {
  return value && /^\d{4}-\d{2}-\d{2}/.test(value) ? value.slice(0, 10) : undefined;
}

function excerpt(value: string | undefined): string | undefined {
  const normalized = value
    ?.replace(/```[\s\S]*?```/g, " ")
    .replace(/`([^`]*)`/g, "$1")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, " ")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/^#+\s*/gm, "")
    .replace(/\s+/g, " ")
    .trim();
  if (!normalized) return undefined;
  return normalized.length <= 280 ? normalized : `${normalized.slice(0, 279).trimEnd()}...`;
}

function compactMeta(meta: ArticleMeta): ArticleMeta {
  return Object.fromEntries(Object.entries(meta).filter(([, value]) => value)) as ArticleMeta;
}

function safeHttps(value: string | undefined): string | undefined {
  try {
    const url = value ? new URL(value) : null;
    return url?.protocol === "https:" && !url.username && !url.password ? value : undefined;
  } catch {
    return undefined;
  }
}

function safeSegments(url: URL): string[] | null {
  try {
    return url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  } catch {
    return null;
  }
}

function safeSlug(value: string | undefined): boolean {
  return Boolean(value && /^[A-Za-z0-9][A-Za-z0-9_-]{1,127}$/.test(value));
}

function safeId(value: string | undefined): boolean {
  return Boolean(value && /^[A-Za-z0-9][A-Za-z0-9_-]{5,127}$/.test(value));
}

function trimmed(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function record(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}
