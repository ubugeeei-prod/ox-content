import { escapeAttribute, escapeHtml, escapeText } from "./html";
import type { TweetBodyData, TweetEntity } from "./types";
import { visibleTextRange } from "./validate";

export function renderTweetText(
  data: TweetBodyData,
  options?: { omitTrailingQuoteUrl?: boolean },
): string {
  const [start, end] = visibleTextRange(data, options?.omitTrailingQuoteUrl === true);
  const entities = collectEntities(data)
    .filter((entity) => validRange(entity.indices, start, end))
    .sort((left, right) => left.indices![0] - right.indices![0]);

  let cursor = start;
  let output = "";
  for (const entity of entities) {
    const [entityStart, entityEnd] = entity.indices!;
    if (entityStart < cursor) continue;
    output += escapeText(data.text.slice(cursor, entityStart));
    if (entity.kind === "url") {
      const href = entity.expanded_url ?? entity.url;
      const label = entity.display_url ?? href;
      output += `<a href="${escapeAttribute(href)}" target="_blank" rel="noopener noreferrer">${escapeHtml(label)}</a>`;
    }
    cursor = entityEnd;
  }
  output += escapeText(data.text.slice(cursor, end));
  return output.trim();
}

function collectEntities(data: TweetBodyData): Array<TweetEntity & { kind: "url" | "media" }> {
  return [
    ...(data.entities?.urls ?? []).map((entity) => ({ ...entity, kind: "url" as const })),
    ...(data.entities?.media ?? []).map((entity) => ({ ...entity, kind: "media" as const })),
  ];
}

function validRange(
  indices: [number, number] | undefined,
  start: number,
  end: number,
): indices is [number, number] {
  return Boolean(indices && indices[0] >= start && indices[1] <= end && indices[0] < indices[1]);
}
