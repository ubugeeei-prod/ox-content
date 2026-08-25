import type { TweetBodyData, TweetData, TweetEntity, TweetUser } from "./types";

const SCREEN_NAME = /^[A-Za-z0-9_]{1,15}$/;
const STATUS_ID = /^\d+$/;
const STATUS_PATH = /(?:x\.com|twitter\.com)\/(?:[^/]+|i\/web)\/status\/(\d+)/i;
const TCO = /^https?:\/\/t\.co\/[A-Za-z0-9]+$/i;

export function isTweetData(data: unknown): data is TweetData {
  return isTweetBodyData(data);
}

export function parseTweetData(data: unknown): TweetData | null {
  return isTweetData(data) ? normalizeTweetData(data) : null;
}

export function isTweetBodyData(data: unknown): data is TweetBodyData {
  if (!data || typeof data !== "object") return false;
  const value = data as Partial<TweetBodyData>;
  return typeof value.text === "string" && isTweetUser(value.user);
}

function isTweetUser(value: unknown): value is TweetUser {
  if (!value || typeof value !== "object") return false;
  const user = value as Partial<TweetUser>;
  return typeof user.name === "string" && typeof user.screen_name === "string";
}

export function normalizeTweetData(data: TweetData): TweetData {
  const handle = sanitizeScreenName(data.in_reply_to_screen_name);
  return {
    ...data,
    quoted_tweet: isTweetBodyData(data.quoted_tweet)
      ? stripNestedQuote(data.quoted_tweet)
      : undefined,
    in_reply_to_screen_name: handle,
    in_reply_to_status_id_str: handle
      ? sanitizeStatusId(data.in_reply_to_status_id_str)
      : undefined,
  };
}

function stripNestedQuote(data: TweetBodyData): TweetBodyData {
  const { quoted_tweet: _nested, ...quoted } = data as TweetBodyData & { quoted_tweet?: unknown };
  return quoted;
}

export function sanitizeScreenName(value: string | undefined): string | undefined {
  return value && SCREEN_NAME.test(value) ? value : undefined;
}

export function sanitizeStatusId(value: string | undefined): string | undefined {
  return value && STATUS_ID.test(value) ? value : undefined;
}

export function quotedPermalink(quoted: TweetBodyData): string | undefined {
  const id = sanitizeStatusId(quoted.id_str);
  if (!id) return undefined;
  const screen = sanitizeScreenName(quoted.user.screen_name);
  return `https://x.com/${screen ?? "i/web"}/status/${id}`;
}

export function replyPermalink(data: TweetData): string | undefined {
  const handle = sanitizeScreenName(data.in_reply_to_screen_name);
  if (!handle) return undefined;
  const id = sanitizeStatusId(data.in_reply_to_status_id_str);
  return id ? `https://x.com/${handle}/status/${id}` : `https://x.com/${handle}`;
}

export function visibleTextRange(
  data: TweetBodyData,
  omitTrailingQuoteUrl = false,
): [number, number] {
  const start = Math.max(0, data.display_text_range?.[0] ?? 0);
  let end = Math.min(data.text.length, data.display_text_range?.[1] ?? data.text.length);
  if (!omitTrailingQuoteUrl || start >= end) return [start, end];

  const quoted = "quoted_tweet" in data ? (data as TweetData).quoted_tweet : undefined;
  for (const entity of data.entities?.urls ?? []) {
    const indices = entity.indices;
    if (!indices || !isQuoteUrlEntity(entity, quoted)) continue;
    const [entityStart, entityEnd] = indices;
    if (entityStart >= start && isTrailingEntity(entityEnd, end, data.text)) {
      end = Math.min(end, entityStart);
    }
  }

  while (end > start && isUtf16Space(data.text, end - 1)) end -= 1;
  return [start, end];
}

function isQuoteUrlEntity(entity: TweetEntity, quoted?: TweetBodyData): boolean {
  for (const href of [entity.expanded_url, entity.url, entity.display_url]) {
    if (!href) continue;
    const match = href.match(STATUS_PATH);
    if (match) return !quoted?.id_str || match[1] === quoted.id_str;
    if (TCO.test(href)) return true;
  }
  return false;
}

function isTrailingEntity(entityEnd: number, rangeEnd: number, text: string): boolean {
  if (entityEnd >= rangeEnd || entityEnd === text.length) return true;
  return entityEnd > 0 && /^[\t\n\r ]*$/.test(text.slice(entityEnd, rangeEnd));
}

function isUtf16Space(text: string, index: number): boolean {
  const char = text[index];
  return char === " " || char === "\n" || char === "\t" || char === "\r";
}
