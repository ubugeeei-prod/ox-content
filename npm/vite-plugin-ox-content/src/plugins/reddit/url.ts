import type { RedditPostReference } from "./types";

const REDDIT_HOSTS = new Set(["reddit.com", "www.reddit.com", "old.reddit.com", "new.reddit.com"]);
const SHORT_HOSTS = new Set(["redd.it", "www.redd.it"]);
const SUBREDDIT = /^[A-Za-z0-9_]{1,21}$/;
const POST_ID = /^[A-Za-z0-9]{3,16}$/;
const SHARE_ID = /^[A-Za-z0-9_-]{4,32}$/;
const SLUG = /^[A-Za-z0-9][A-Za-z0-9_-]{0,240}$/;

export function parseRedditPostId(value: string): RedditPostReference | null {
  const id = normalizeId(value);
  if (!id) return null;
  return referenceFromParts({ id });
}

export function parseRedditPostReference(value: string): RedditPostReference | null {
  try {
    const url = new URL(value.trim());
    const host = url.hostname.toLowerCase();
    if (url.protocol !== "https:" || url.username || url.password) return null;

    const segments = pathSegments(url);
    if (!segments) return null;

    if (SHORT_HOSTS.has(host)) {
      return parseShortUrl(segments);
    }
    if (!REDDIT_HOSTS.has(host)) {
      return null;
    }

    return parseRedditPath(segments);
  } catch {
    return null;
  }
}

export function redditReferenceKey(reference: RedditPostReference): string {
  return reference.id ?? reference.url;
}

export function isSafeExternalUrl(value: string): boolean {
  try {
    const url = new URL(value);
    const host = url.hostname.toLowerCase();
    const ipv6 = host.replace(/^\[|\]$/g, "");
    if ((url.protocol !== "http:" && url.protocol !== "https:") || url.username || url.password) {
      return false;
    }
    if (host === "localhost" || host.endsWith(".localhost")) return false;
    if (
      ipv6.includes(":") &&
      (ipv6 === "::1" || ipv6.startsWith("fc") || ipv6.startsWith("fd") || ipv6.startsWith("fe80"))
    ) {
      return false;
    }
    return !isPrivateIPv4(host);
  } catch {
    return false;
  }
}

export function sameRedditPostUrl(left: string, right: string): boolean {
  const a = parseRedditPostReference(left);
  const b = parseRedditPostReference(right);
  return Boolean(a?.id && b?.id && a.id === b.id);
}

function parseShortUrl(segments: string[]): RedditPostReference | null {
  if (segments.length < 1) return null;
  const id = normalizeId(segments[0]);
  return id ? referenceFromParts({ id }) : null;
}

function parseRedditPath(segments: string[]): RedditPostReference | null {
  if (segments[0]?.toLowerCase() === "comments") {
    const id = normalizeId(segments[1]);
    if (!id) return null;
    return referenceFromParts({ id, slug: safeSlug(segments[2]) });
  }

  if (segments[0]?.toLowerCase() === "gallery") {
    const id = normalizeId(segments[1]);
    if (!id) return null;
    return referenceFromParts({ id });
  }

  if (segments[0]?.toLowerCase() !== "r") return null;
  const subreddit = safeSubreddit(segments[1]);
  if (!subreddit) return null;

  if (segments[2]?.toLowerCase() === "comments") {
    const id = normalizeId(segments[3]);
    if (!id) return null;
    return referenceFromParts({
      id,
      subreddit,
      slug: safeSlug(segments[4]),
    });
  }

  if (segments[2]?.toLowerCase() === "s") {
    const shareId = safeShareId(segments[3]);
    if (!shareId) return null;
    return {
      url: `https://www.reddit.com/r/${subreddit}/s/${shareId}/`,
      subreddit,
      shareId,
    };
  }

  return null;
}

function referenceFromParts(parts: {
  id: string;
  subreddit?: string;
  slug?: string;
}): RedditPostReference {
  const base = parts.subreddit
    ? `https://www.reddit.com/r/${parts.subreddit}/comments/${parts.id}/`
    : `https://www.reddit.com/comments/${parts.id}/`;
  const url = parts.slug ? `${base}${parts.slug}/` : base;
  return {
    ...parts,
    url,
    apiUrl: `https://www.reddit.com/comments/${parts.id}.json?raw_json=1`,
  };
}

function pathSegments(url: URL): string[] | null {
  try {
    return url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  } catch {
    return null;
  }
}

function normalizeId(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed && POST_ID.test(trimmed) ? trimmed.toLowerCase() : undefined;
}

function safeSubreddit(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed && SUBREDDIT.test(trimmed) ? trimmed : undefined;
}

function safeShareId(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed && SHARE_ID.test(trimmed) ? trimmed : undefined;
}

function safeSlug(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed && SLUG.test(trimmed) ? trimmed : undefined;
}

function isPrivateIPv4(hostname: string): boolean {
  const parts = hostname.split(".").map(Number);
  if (
    parts.length !== 4 ||
    parts.some((part) => !Number.isInteger(part) || part < 0 || part > 255)
  ) {
    return false;
  }
  const [a, b] = parts;
  return (
    a === 10 ||
    a === 127 ||
    a === 0 ||
    (a === 172 && b >= 16 && b <= 31) ||
    (a === 192 && b === 168) ||
    (a === 169 && b === 254)
  );
}
