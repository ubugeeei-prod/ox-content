import { escapeAttribute, escapeHtml, escapeText } from "./html";
import type { TweetBodyData, TweetIndexedEntity } from "./types";
import { sanitizeScreenName, visibleTextRange } from "./validate";

type EntityKind = "url" | "media" | "hashtag" | "mention" | "symbol";

const NAMED_ENTITIES: Record<string, string> = {
  amp: "&",
  apos: "'",
  gt: ">",
  lt: "<",
  nbsp: "\u00a0",
  quot: '"',
};

interface CollectedEntity extends TweetIndexedEntity {
  kind: EntityKind;
  href?: string;
  label?: string;
  source?: string;
}

export function renderTweetText(
  data: TweetBodyData,
  options?: { omitTrailingQuoteUrl?: boolean },
): string {
  const decoded = decodeTweetText(data.text);
  const text = decoded.text;
  const collected = collectEntities(data);
  const ranges = detectEntityRangeSpace(data.text, text, collected);
  const textData =
    ranges === "raw"
      ? remapTweetBodyData(data, text, decoded.rawToDecoded)
      : text === data.text
        ? data
        : { ...data, text };
  const [start, end] = visibleTextRange(textData, options?.omitTrailingQuoteUrl === true);
  const entities = collected
    .map((entity) =>
      ranges === "raw" ? remapCollectedEntity(entity, decoded.rawToDecoded) : entity,
    )
    .filter((entity) => validRange(entity.indices, start, end))
    .sort((left, right) => left.indices![0] - right.indices![0]);

  let cursor = start;
  let output = "";
  for (const entity of entities) {
    const [entityStart, entityEnd] = entity.indices!;
    if (entityStart < cursor) continue;
    output += escapeText(text.slice(cursor, entityStart));
    if (entity.href) {
      const label = entity.label ?? text.slice(entityStart, entityEnd);
      output += `<a href="${escapeAttribute(entity.href)}" target="_blank" rel="noopener noreferrer">${escapeHtml(label)}</a>`;
    }
    cursor = entityEnd;
  }
  output += escapeText(text.slice(cursor, end));
  return output.trim();
}

function collectEntities(data: TweetBodyData): CollectedEntity[] {
  const collected: CollectedEntity[] = [];
  for (const entity of data.entities?.urls ?? []) {
    collected.push({
      kind: "url",
      indices: entity.indices,
      href: entity.expanded_url ?? entity.url,
      label: entity.display_url ?? entity.expanded_url ?? entity.url,
      source: entity.url,
    });
  }
  for (const entity of data.entities?.media ?? []) {
    collected.push({ kind: "media", indices: entity.indices, source: entity.url });
  }
  for (const entity of data.entities?.hashtags ?? []) {
    if (!entity.text) continue;
    collected.push({
      kind: "hashtag",
      indices: entity.indices,
      href: `https://x.com/hashtag/${encodeURIComponent(entity.text)}`,
      source: `#${entity.text}`,
    });
  }
  for (const entity of data.entities?.user_mentions ?? []) {
    const screen = sanitizeScreenName(entity.screen_name);
    if (!screen) continue;
    collected.push({
      kind: "mention",
      indices: entity.indices,
      href: `https://x.com/${encodeURIComponent(screen)}`,
      source: `@${screen}`,
    });
  }
  for (const entity of data.entities?.symbols ?? []) {
    if (!entity.text) continue;
    collected.push({
      kind: "symbol",
      indices: entity.indices,
      href: `https://x.com/search?q=%24${encodeURIComponent(entity.text)}`,
      source: `$${entity.text}`,
    });
  }
  return collected;
}

function detectEntityRangeSpace(
  rawText: string,
  decodedText: string,
  entities: CollectedEntity[],
): "raw" | "decoded" {
  if (rawText === decodedText) {
    return "decoded";
  }
  for (const entity of entities) {
    if (!entity.indices || !entity.source) continue;
    const [start, end] = entity.indices;
    const rawMatches = rawText.slice(start, end) === entity.source;
    const decodedMatches = decodedText.slice(start, end) === entity.source;
    if (rawMatches && !decodedMatches) {
      return "raw";
    }
    if (decodedMatches && !rawMatches) {
      return "decoded";
    }
  }
  return "decoded";
}

function remapTweetBodyData(
  data: TweetBodyData,
  text: string,
  rawToDecoded: (offset: number) => number,
): TweetBodyData {
  return {
    ...data,
    text,
    display_text_range: remapRange(data.display_text_range, rawToDecoded),
    entities: data.entities
      ? {
          urls: data.entities.urls?.map((entity) => ({
            ...entity,
            indices: remapRange(entity.indices, rawToDecoded),
          })),
          media: data.entities.media?.map((entity) => ({
            ...entity,
            indices: remapRange(entity.indices, rawToDecoded),
          })),
          hashtags: data.entities.hashtags?.map((entity) => ({
            ...entity,
            indices: remapRange(entity.indices, rawToDecoded),
          })),
          user_mentions: data.entities.user_mentions?.map((entity) => ({
            ...entity,
            indices: remapRange(entity.indices, rawToDecoded),
          })),
          symbols: data.entities.symbols?.map((entity) => ({
            ...entity,
            indices: remapRange(entity.indices, rawToDecoded),
          })),
        }
      : undefined,
  };
}

function remapCollectedEntity(
  entity: CollectedEntity,
  rawToDecoded: (offset: number) => number,
): CollectedEntity {
  return { ...entity, indices: remapRange(entity.indices, rawToDecoded) };
}

function remapRange(
  range: [number, number] | undefined,
  rawToDecoded: (offset: number) => number,
): [number, number] | undefined {
  if (!range) return undefined;
  return [rawToDecoded(range[0]), rawToDecoded(range[1])];
}

function validRange(
  indices: [number, number] | undefined,
  start: number,
  end: number,
): indices is [number, number] {
  return Boolean(indices && indices[0] >= start && indices[1] <= end && indices[0] < indices[1]);
}

function decodeTweetText(value: string): {
  text: string;
  rawToDecoded: (offset: number) => number;
} {
  if (!value.includes("&")) {
    return { text: value, rawToDecoded: clampOffset(value.length) };
  }
  const rawToDecoded = Array.from({ length: value.length + 1 }, () => 0);
  const pattern = /&(#[0-9]+|#[xX][0-9a-fA-F]+|[a-zA-Z][a-zA-Z0-9]*);/g;
  let rawCursor = 0;
  let decoded = "";
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(value))) {
    const entity = decodeTweetEntity(match[1]);
    if (!entity) continue;
    appendUnchangedOffsetMap(rawToDecoded, rawCursor, match.index, decoded.length);
    decoded += value.slice(rawCursor, match.index);

    const decodedStart = decoded.length;
    decoded += entity;
    const decodedEnd = decoded.length;
    for (let offset = match.index; offset <= pattern.lastIndex; offset += 1) {
      rawToDecoded[offset] = offset === match.index ? decodedStart : decodedEnd;
    }
    rawCursor = pattern.lastIndex;
  }

  appendUnchangedOffsetMap(rawToDecoded, rawCursor, value.length, decoded.length);
  decoded += value.slice(rawCursor);

  return {
    text: decoded,
    rawToDecoded: (offset) => {
      const clamped = Math.max(0, Math.min(value.length, offset));
      return rawToDecoded[clamped] ?? decoded.length;
    },
  };
}

function appendUnchangedOffsetMap(
  rawToDecoded: number[],
  rawStart: number,
  rawEnd: number,
  decodedStart: number,
): void {
  for (let offset = rawStart; offset <= rawEnd; offset += 1) {
    rawToDecoded[offset] = decodedStart + (offset - rawStart);
  }
}

function decodeTweetEntity(body: string): string | undefined {
  if (body.startsWith("#")) {
    const code =
      body[1] === "x" || body[1] === "X"
        ? Number.parseInt(body.slice(2), 16)
        : Number.parseInt(body.slice(1), 10);
    if (!Number.isInteger(code) || code <= 0 || code > 0x10ffff) {
      return undefined;
    }
    try {
      return String.fromCodePoint(code);
    } catch {
      return undefined;
    }
  }
  return NAMED_ENTITIES[body.toLowerCase()] ?? undefined;
}

function clampOffset(length: number): (offset: number) => number {
  return (offset) => Math.max(0, Math.min(length, offset));
}
