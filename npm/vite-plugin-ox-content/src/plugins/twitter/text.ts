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
}

export function renderTweetText(
  data: TweetBodyData,
  options?: { omitTrailingQuoteUrl?: boolean },
): string {
  const text = decodeTweetText(data.text);
  const textData = text === data.text ? data : { ...data, text };
  const [start, end] = visibleTextRange(textData, options?.omitTrailingQuoteUrl === true);
  const entities = collectEntities(data)
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
    });
  }
  for (const entity of data.entities?.media ?? []) {
    collected.push({ kind: "media", indices: entity.indices });
  }
  for (const entity of data.entities?.hashtags ?? []) {
    if (!entity.text) continue;
    collected.push({
      kind: "hashtag",
      indices: entity.indices,
      href: `https://x.com/hashtag/${encodeURIComponent(entity.text)}`,
    });
  }
  for (const entity of data.entities?.user_mentions ?? []) {
    const screen = sanitizeScreenName(entity.screen_name);
    if (!screen) continue;
    collected.push({
      kind: "mention",
      indices: entity.indices,
      href: `https://x.com/${encodeURIComponent(screen)}`,
    });
  }
  for (const entity of data.entities?.symbols ?? []) {
    if (!entity.text) continue;
    collected.push({
      kind: "symbol",
      indices: entity.indices,
      href: `https://x.com/search?q=%24${encodeURIComponent(entity.text)}`,
    });
  }
  return collected;
}

function validRange(
  indices: [number, number] | undefined,
  start: number,
  end: number,
): indices is [number, number] {
  return Boolean(indices && indices[0] >= start && indices[1] <= end && indices[0] < indices[1]);
}

function decodeTweetText(value: string): string {
  if (!value.includes("&")) {
    return value;
  }
  return value.replace(/&(#[0-9]+|#[xX][0-9a-fA-F]+|[a-zA-Z][a-zA-Z0-9]*);/g, (match, body) => {
    if (body.startsWith("#")) {
      const code =
        body[1] === "x" || body[1] === "X"
          ? Number.parseInt(body.slice(2), 16)
          : Number.parseInt(body.slice(1), 10);
      if (!Number.isInteger(code) || code <= 0 || code > 0x10ffff) {
        return match;
      }
      try {
        return String.fromCodePoint(code);
      } catch {
        return match;
      }
    }
    return NAMED_ENTITIES[body.toLowerCase()] ?? match;
  });
}
