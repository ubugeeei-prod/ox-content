const SPEAKERDECK_TAG =
  /<speakerdeck\b((?:[^>"']|"[^"]*"|'[^']*')*)>([\s\S]*?)<\/speakerdeck\s*>/gi;
const OEMBED_ENDPOINT = "https://speakerdeck.com/oembed.json?url=";
const PLAYER_SRC = /https:\/\/speakerdeck\.com\/player\/([A-Za-z0-9]{8,64})/;
const OEMBED_CONCURRENCY = 4;

export type SpeakerDeckFetch = (input: string, init?: RequestInit) => Promise<Response>;

type SpeakerDeckMeta = {
  player?: string;
  title?: string;
  author?: string;
  preview?: string;
};

export async function enrichSpeakerDeckEmbeds(
  html: string,
  fetchImpl: SpeakerDeckFetch = fetch,
): Promise<string> {
  if (!/<speakerdeck\b/i.test(html)) {
    return html;
  }

  const matches = Array.from(html.matchAll(SPEAKERDECK_TAG), (match) => ({
    attrs: match[1] ?? "",
    end: (match.index ?? 0) + match[0].length,
    full: match[0],
    index: match.index ?? 0,
  }));
  const enriched = await mapWithConcurrency(matches, OEMBED_CONCURRENCY, (match) =>
    enrichTag(match.full, match.attrs, fetchImpl),
  );

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

async function mapWithConcurrency<T, U>(
  values: T[],
  limit: number,
  worker: (value: T) => Promise<U>,
): Promise<U[]> {
  const results: U[] = [];
  let cursor = 0;

  async function runNext(): Promise<void> {
    const index = cursor;
    cursor += 1;
    if (index >= values.length) return;
    results[index] = await worker(values[index]!);
    await runNext();
  }

  await Promise.all(values.slice(0, limit).map(() => runNext()));
  return results;
}

async function enrichTag(
  full: string,
  attrs: string,
  fetchImpl: SpeakerDeckFetch,
): Promise<string> {
  if (readAttr(attrs, "player") || readAttr(attrs, "id")) {
    return full;
  }
  const url = readAttr(attrs, "url") ?? readAttr(attrs, "href") ?? readAttr(attrs, "src");
  if (!url || !isSpeakerDeckShareUrl(url)) {
    return full;
  }

  const meta = await fetchOembed(url, fetchImpl);
  if (!meta) {
    return full;
  }

  let next = attrs;
  if (meta.player) next += ` player="${escapeAttr(meta.player)}"`;
  if (meta.title && !readAttr(attrs, "title")) next += ` title="${escapeAttr(meta.title)}"`;
  if (meta.author && !readAttr(attrs, "author")) next += ` author="${escapeAttr(meta.author)}"`;
  if (meta.preview && !readAttr(attrs, "preview")) {
    next += ` preview="${escapeAttr(meta.preview)}"`;
  }
  const openEnd = full.indexOf(">");
  return `<SpeakerDeck${next}>${full.slice(openEnd + 1)}`;
}

async function fetchOembed(
  url: string,
  fetchImpl: SpeakerDeckFetch,
): Promise<SpeakerDeckMeta | null> {
  try {
    const response = await fetchImpl(`${OEMBED_ENDPOINT}${encodeURIComponent(url)}`, {
      signal: AbortSignal.timeout(10_000),
      headers: { accept: "application/json" },
    });
    if (!response.ok) {
      return null;
    }
    const data = (await response.json()) as Record<string, unknown>;
    const html = typeof data.html === "string" ? data.html : "";
    const player = html.match(PLAYER_SRC)?.[1];
    const title = asTrimmedString(data.title);
    const author = asTrimmedString(data.author_name);
    const preview = asTrimmedString(data.thumbnail_url);
    if (!player && !title && !author && !preview) {
      return null;
    }
    return {
      player,
      title,
      author,
      preview: preview && isSafeHttpsUrl(preview) ? preview : undefined,
    };
  } catch {
    return null;
  }
}

function isSpeakerDeckShareUrl(input: string): boolean {
  try {
    const url = new URL(input);
    if (url.protocol !== "https:" || url.hostname.toLowerCase() !== "speakerdeck.com") {
      return false;
    }
    const segments = url.pathname.split("/").filter(Boolean);
    return (
      segments.length === 2 &&
      segments[0] !== "player" &&
      isPathSegment(segments[0]) &&
      isPathSegment(segments[1])
    );
  } catch {
    return false;
  }
}

function isPathSegment(value: string): boolean {
  return /^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/.test(value);
}

function isSafeHttpsUrl(value: string): boolean {
  try {
    const url = new URL(value);
    return url.protocol === "https:" && !url.username && !url.password;
  } catch {
    return false;
  }
}

function readAttr(attrs: string, name: string): string | undefined {
  const match = attrs.match(new RegExp(`\\b${name}\\s*=\\s*("([^"]*)"|'([^']*)')`, "i"));
  const value = match?.[2] ?? match?.[3];
  return value ? value : undefined;
}

function asTrimmedString(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function escapeAttr(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}
