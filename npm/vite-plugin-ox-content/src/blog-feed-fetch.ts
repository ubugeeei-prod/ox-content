/**
 * Build-time fetch of configured blog feeds. Never used for document URLs.
 */

import { lookup as dnsLookup } from "node:dns/promises";
import { isBlockedFeedAddress, isSafeFeedUrl } from "./blog-feed-url";

export const BLOG_FEED_TIMEOUT_MS = 10_000;
export const BLOG_FEED_MAX_BYTES = 1_048_576;
export const BLOG_FEED_MAX_REDIRECTS = 5;

export type BlogFeedLookup = (hostname: string) => Promise<string[]>;
export type BlogFeedFetchFn = (input: string, init?: RequestInit) => Promise<Response>;

export interface BlogFeedFetchLimits {
  timeoutMs?: number;
  maxBytes?: number;
  maxRedirects?: number;
}

export interface BlogFeedNetwork {
  fetch?: BlogFeedFetchFn;
  lookup?: BlogFeedLookup;
  limits?: BlogFeedFetchLimits;
}

let installedNetwork: BlogFeedNetwork = {};

/** Test hook. Production builds leave this empty. */
export function installBlogFeedNetwork(network: BlogFeedNetwork): void {
  installedNetwork = network;
}

export function resetBlogFeedNetwork(): void {
  installedNetwork = {};
}

export async function fetchBlogFeedBody(
  url: string,
  network: BlogFeedNetwork = {},
): Promise<string> {
  const timeoutMs =
    network.limits?.timeoutMs ?? installedNetwork.limits?.timeoutMs ?? BLOG_FEED_TIMEOUT_MS;
  const maxBytes =
    network.limits?.maxBytes ?? installedNetwork.limits?.maxBytes ?? BLOG_FEED_MAX_BYTES;
  const maxRedirects =
    network.limits?.maxRedirects ??
    installedNetwork.limits?.maxRedirects ??
    BLOG_FEED_MAX_REDIRECTS;
  const fetchFn = network.fetch ?? installedNetwork.fetch ?? defaultFetch;
  const lookup = network.lookup ?? installedNetwork.lookup ?? defaultLookup;
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    return await followFeed(url, fetchFn, lookup, controller.signal, maxBytes, maxRedirects);
  } catch (error) {
    if (isAbortError(error)) {
      throw new Error("timeout");
    }
    throw error instanceof Error ? error : new Error(String(error));
  } finally {
    clearTimeout(timer);
  }
}

async function followFeed(
  startUrl: string,
  fetchFn: BlogFeedFetchFn,
  lookup: BlogFeedLookup,
  signal: AbortSignal,
  maxBytes: number,
  maxRedirects: number,
): Promise<string> {
  const seen = new Set<string>();
  let current = startUrl;
  for (let hops = 0; hops <= maxRedirects; hops += 1) {
    await assertSafeFeedTarget(current, lookup);
    if (seen.has(current)) {
      throw new Error("too many redirects");
    }
    seen.add(current);
    const response = await fetchFn(current, {
      method: "GET",
      redirect: "manual",
      signal,
      headers: {
        Accept: "application/rss+xml, application/atom+xml, application/xml, text/xml;q=0.9",
        "User-Agent": "ox-content-blog-feeds/1.0",
      },
    });
    if (isRedirect(response.status)) {
      const next = resolveRedirect(current, response.headers.get("location"));
      current = next;
      continue;
    }
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    assertFeedContentType(response.headers.get("content-type"));
    return readBoundedBody(response, maxBytes);
  }
  throw new Error("too many redirects");
}

export async function assertSafeFeedTarget(url: string, lookup: BlogFeedLookup): Promise<void> {
  if (!isSafeFeedUrl(url)) {
    throw new Error("unsafe URL");
  }
  const hostname = new URL(url).hostname;
  const addresses = await lookup(hostname);
  if (addresses.length === 0 || addresses.some((address) => isBlockedFeedAddress(address))) {
    throw new Error("private network");
  }
}

async function defaultLookup(hostname: string): Promise<string[]> {
  const records = await dnsLookup(hostname, { all: true, verbatim: true });
  return records.map((record) => record.address);
}

function defaultFetch(input: string, init?: RequestInit): Promise<Response> {
  return fetch(input, init);
}

function isRedirect(status: number): boolean {
  return status === 301 || status === 302 || status === 303 || status === 307 || status === 308;
}

function resolveRedirect(current: string, location: string | null): string {
  if (!location?.trim()) {
    throw new Error("too many redirects");
  }
  try {
    return new URL(location, current).href;
  } catch {
    throw new Error("unsafe URL");
  }
}

function assertFeedContentType(value: string | null): void {
  if (!value) {
    return;
  }
  const type = value.split(";")[0]?.trim().toLowerCase() ?? "";
  if (type === "text/html" || type === "application/xhtml+xml" || type === "application/json") {
    throw new Error("not a feed");
  }
}

async function readBoundedBody(response: Response, maxBytes: number): Promise<string> {
  const length = Number(response.headers.get("content-length"));
  if (Number.isFinite(length) && length > maxBytes) {
    throw new Error("oversized");
  }
  const reader = response.body?.getReader();
  if (!reader) {
    const text = await response.text();
    if (new TextEncoder().encode(text).byteLength > maxBytes) {
      throw new Error("oversized");
    }
    return text;
  }
  const chunks: Uint8Array[] = [];
  let total = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    if (!value) {
      continue;
    }
    total += value.byteLength;
    if (total > maxBytes) {
      await reader.cancel();
      throw new Error("oversized");
    }
    chunks.push(value);
  }
  return new TextDecoder("utf-8").decode(concatBytes(chunks, total));
}

function concatBytes(chunks: readonly Uint8Array[], total: number): Uint8Array {
  const out = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return out;
}

function isAbortError(error: unknown): boolean {
  return error instanceof Error && (error.name === "AbortError" || error.message === "timeout");
}
