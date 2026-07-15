import type { TweetReference } from "./types";

const STATUS_PATH = /^\/(?:[^/]+|i\/web)\/status\/(\d+)(?:\/.*)?$/;

export function createSyndicationToken(id: string): string {
  return ((Number(id) / 1e15) * Math.PI).toString(36).replaceAll(/(0+|\.)/g, "");
}

export function parseTweetReference(value: string): TweetReference | null {
  const trimmed = value.trim();
  if (/^\d+$/.test(trimmed)) {
    return { id: trimmed, url: `https://x.com/i/web/status/${trimmed}` };
  }

  try {
    const url = new URL(trimmed);
    const hostname = url.hostname.toLowerCase().replace(/^(?:www\.|mobile\.)/, "");
    if (url.protocol !== "https:" || (hostname !== "x.com" && hostname !== "twitter.com")) {
      return null;
    }

    const match = url.pathname.match(STATUS_PATH);
    if (!match) return null;
    const screenName = url.pathname.startsWith("/i/web/status/")
      ? "i/web"
      : url.pathname.split("/")[1];
    return {
      id: match[1],
      url: `https://x.com/${screenName}/status/${match[1]}`,
    };
  } catch {
    return null;
  }
}

export function referenceFromAttributes(attributes: string): TweetReference | null {
  const values = new Map<string, string>();
  const pattern = /\b(url|href|id)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/gi;
  for (const match of attributes.matchAll(pattern)) {
    values.set(match[1].toLowerCase(), match[2] ?? match[3] ?? match[4] ?? "");
  }
  return parseTweetReference(values.get("url") ?? values.get("href") ?? values.get("id") ?? "");
}
