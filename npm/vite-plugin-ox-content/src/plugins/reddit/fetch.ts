import { classifyError, classifyStatus, warnEmbedFailure } from "../provider-failure";
import type { RedditPostData, RedditPostReference, ResolvedRedditEmbedOptions } from "./types";
import {
  isSafeExternalUrl,
  parseRedditPostReference,
  redditReferenceKey,
  sameRedditPostUrl,
} from "./url";

interface MemoryRecord {
  data: RedditPostData | null;
  timestamp: number;
}

const memoryCache = new Map<string, MemoryRecord>();
const inflight = new Map<string, Promise<RedditPostData | null>>();

export function clearRedditCache(): void {
  memoryCache.clear();
  inflight.clear();
}

export async function fetchRedditPostData(
  reference: RedditPostReference,
  options: ResolvedRedditEmbedOptions,
): Promise<RedditPostData | null> {
  if (!reference.apiUrl) return null;

  const key = redditReferenceKey(reference);
  const now = Date.now();
  if (options.cache) {
    const cached = memoryCache.get(key);
    if (cached && now - cached.timestamp < options.cacheTTL) {
      return cached.data;
    }
  }

  const pending = inflight.get(key);
  if (pending) return pending;

  const request = requestRedditPostData(reference, options).finally(() => {
    if (inflight.get(key) === request) inflight.delete(key);
  });
  inflight.set(key, request);

  const data = await request;
  if (options.cache) {
    memoryCache.set(key, { data, timestamp: now });
  }
  return data;
}

export async function requestRedditPostData(
  reference: RedditPostReference,
  options: ResolvedRedditEmbedOptions,
): Promise<RedditPostData | null> {
  if (!reference.apiUrl) return null;

  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), options.timeout);
  try {
    const response = await fetch(reference.apiUrl, {
      headers: {
        Accept: "application/json",
        "User-Agent": options.userAgent,
      },
      signal: controller.signal,
    });
    if (!response.ok) {
      warnEmbedFailure(
        "reddit",
        reference.apiUrl,
        classifyStatus(response.status),
        "a link-only card",
      );
      return null;
    }
    return parseRedditListing(await response.json(), reference);
  } catch (error) {
    warnEmbedFailure("reddit", reference.apiUrl, classifyError(error), "a link-only card");
    return null;
  } finally {
    clearTimeout(timeoutId);
  }
}

export function parseRedditListing(
  value: unknown,
  reference: RedditPostReference,
): RedditPostData | null {
  for (const child of listingChildren(value)) {
    const thing = record(child);
    if (thing?.kind !== "t3") continue;
    const post = parseRedditThing(thing.data, reference);
    if (post) return post;
  }
  return null;
}

function parseRedditThing(value: unknown, reference: RedditPostReference): RedditPostData | null {
  const data = record(value);
  if (!data) return null;

  const title = trimmedString(data.title);
  const subreddit = safeName(trimmedString(data.subreddit) ?? reference.subreddit);
  if (!title || !subreddit) return null;

  const permalink = normalizePermalink(trimmedString(data.permalink), reference);
  const originalUrl = originalLink(data, permalink);
  return {
    permalink,
    subreddit,
    title: truncateText(title, 300),
    author: safeAuthor(trimmedString(data.author)),
    body: bodyExcerpt(trimmedString(data.selftext)),
    score: nonNegativeInteger(data.score),
    commentCount: nonNegativeInteger(data.num_comments),
    createdAt: timestampIso(data.created_utc),
    originalUrl,
    image: postImage(data),
  };
}

function listingChildren(value: unknown): unknown[] {
  const listings = Array.isArray(value) ? value : [value];
  for (const listing of listings) {
    const children = record(record(listing)?.data)?.children;
    if (Array.isArray(children)) return children;
  }
  return [];
}

function originalLink(data: Record<string, unknown>, permalink: string): string | undefined {
  const value = trimmedString(data.url_overridden_by_dest) ?? trimmedString(data.url);
  if (!value) return undefined;
  const url = decodeEntities(value);
  if (!isSafeExternalUrl(url) || sameRedditPostUrl(url, permalink)) return undefined;
  return url;
}

function postImage(data: Record<string, unknown>): RedditPostData["image"] {
  const preview = previewImage(data);
  if (preview) return preview;

  const hinted = trimmedString(data.url_overridden_by_dest) ?? trimmedString(data.url);
  if (data.post_hint === "image" && hinted) {
    const url = decodeEntities(hinted);
    if (isSafeExternalUrl(url) && isImageUrl(url)) return { url };
  }
  return undefined;
}

function previewImage(data: Record<string, unknown>): RedditPostData["image"] {
  const preview = record(data.preview);
  const images = preview?.images;
  const first = Array.isArray(images) ? record(images[0]) : undefined;
  const source = record(first?.source);
  const url = trimmedString(source?.url);
  if (!url) return undefined;

  const decoded = decodeEntities(url);
  if (!isSafeExternalUrl(decoded)) return undefined;
  return {
    url: decoded,
    width: nonNegativeInteger(source?.width),
    height: nonNegativeInteger(source?.height),
  };
}

function normalizePermalink(value: string | undefined, reference: RedditPostReference): string {
  if (!value) return reference.url;
  const absolute = value.startsWith("/") ? `https://www.reddit.com${value}` : value;
  return parseRedditPostReference(absolute)?.url ?? reference.url;
}

function bodyExcerpt(value: string | undefined): string | undefined {
  const body = value?.trim();
  if (!body || body === "[deleted]" || body === "[removed]") return undefined;
  return truncateText(body, 280);
}

function truncateText(value: string, maxLength: number): string {
  const normalized = value.replace(/\r\n?/g, "\n").trim();
  if (normalized.length <= maxLength) return normalized;
  return `${normalized.slice(0, maxLength - 1).trimEnd()}...`;
}

function timestampIso(value: unknown): string | undefined {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) return undefined;
  return new Date(value * 1000).toISOString();
}

function nonNegativeInteger(value: unknown): number | undefined {
  if (typeof value === "number" && Number.isFinite(value) && value >= 0) {
    return Math.floor(value);
  }
  return undefined;
}

function safeAuthor(value: string | undefined): string | undefined {
  if (!value || value === "[deleted]") return undefined;
  return safeName(value);
}

function safeName(value: string | undefined): string | undefined {
  return value && /^[A-Za-z0-9_-]{1,32}$/.test(value) ? value : undefined;
}

function trimmedString(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function isImageUrl(value: string): boolean {
  try {
    return /\.(?:avif|gif|jpe?g|png|webp)$/i.test(new URL(value).pathname);
  } catch {
    return false;
  }
}

function decodeEntities(value: string): string {
  return value
    .replaceAll("&amp;", "&")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">");
}

function record(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}
