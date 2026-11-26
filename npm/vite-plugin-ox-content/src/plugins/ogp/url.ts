import { createHash } from "node:crypto";
import { isBlockedFeedHost } from "../../blog-feed-url";

export function isSafeOgpUrl(value: string): boolean {
  try {
    const url = new URL(value);
    if (url.protocol !== "http:" && url.protocol !== "https:") return false;
    if (url.username || url.password) return false;
    // `localhost.` names the same host as `localhost`.
    return !isBlockedFeedHost(url.hostname.replace(/\.$/, ""));
  } catch {
    return false;
  }
}

export function extractDomain(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return url;
  }
}

/**
 * Normalize a URL for cache keys: lowercase host, drop default ports and
 * fragments, and strip a trailing slash that is not the root path.
 */
export function normalizeOgpUrl(url: string): string {
  try {
    const parsed = new URL(url);
    parsed.hash = "";
    parsed.hostname = parsed.hostname.toLowerCase();
    if (
      (parsed.protocol === "https:" && parsed.port === "443") ||
      (parsed.protocol === "http:" && parsed.port === "80")
    ) {
      parsed.port = "";
    }
    if (parsed.pathname.length > 1 && parsed.pathname.endsWith("/")) {
      parsed.pathname = parsed.pathname.slice(0, -1);
    }
    return parsed.href;
  } catch {
    return url;
  }
}

export function ogpCacheKey(url: string): string {
  return createHash("sha256").update(normalizeOgpUrl(url)).digest("hex");
}
