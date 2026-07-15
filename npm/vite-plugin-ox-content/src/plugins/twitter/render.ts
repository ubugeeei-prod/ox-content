import type { ResolvedTwitterEmbedOptions, TweetAssets, TweetData, TweetEntity } from "./types";

export function renderFetchedTweet(
  permalink: string,
  data: TweetData,
  assets: TweetAssets,
  options: ResolvedTwitterEmbedOptions,
): string {
  const profile = `https://x.com/${encodeURIComponent(data.user.screen_name)}`;
  const author = escapeHtml(data.user.name);
  const handle = escapeHtml(data.user.screen_name);
  const avatar = assets.avatar
    ? `<img class="ox-tweet__avatar" src="${escapeAttribute(assets.avatar)}" alt="" width="48" height="48" loading="lazy" decoding="async">`
    : "";
  const media = renderMedia(assets);
  const footer = renderFooter(permalink, data.created_at, options.lang);

  return [
    '<figure class="ox-tweet ox-tweet--fetched">',
    '<header class="ox-tweet__header">',
    `<a class="ox-tweet__profile" href="${escapeAttribute(profile)}" target="_blank" rel="noopener noreferrer">`,
    avatar,
    `<span class="ox-tweet__author-name">${author}</span>`,
    `<span class="ox-tweet__author-handle">@${handle}</span>`,
    "</a></header>",
    `<div class="ox-tweet__body">${renderTweetText(data)}</div>`,
    media,
    footer,
    "</figure>",
  ].join("");
}

export function renderTweetText(data: TweetData): string {
  const [start, end] = data.display_text_range ?? [0, data.text.length];
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

function collectEntities(data: TweetData): Array<TweetEntity & { kind: "url" | "media" }> {
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

function renderMedia(assets: TweetAssets): string {
  if (assets.media.length === 0) return "";
  const images = assets.media
    .map((item) => {
      const size = [
        item.width ? ` width="${item.width}"` : "",
        item.height ? ` height="${item.height}"` : "",
      ].join("");
      return `<img class="ox-tweet__media-item" src="${escapeAttribute(item.src)}" alt="${escapeAttribute(item.alt ?? "")}"${size} loading="lazy" decoding="async">`;
    })
    .join("");
  return `<div class="ox-tweet__media" data-count="${assets.media.length}">${images}</div>`;
}

function renderFooter(permalink: string, createdAt: string | undefined, lang: string): string {
  if (!createdAt) {
    return `<footer class="ox-tweet__footer"><a class="ox-tweet__permalink" href="${escapeAttribute(permalink)}" target="_blank" rel="noopener noreferrer">View on X</a></footer>`;
  }
  const date = new Date(createdAt);
  if (Number.isNaN(date.valueOf())) return renderFooter(permalink, undefined, lang);
  const iso = date.toISOString();
  let label: string;
  try {
    label = new Intl.DateTimeFormat(lang, { dateStyle: "medium", timeZone: "UTC" }).format(date);
  } catch {
    label = new Intl.DateTimeFormat("en", { dateStyle: "medium", timeZone: "UTC" }).format(date);
  }
  return `<footer class="ox-tweet__footer"><a class="ox-tweet__permalink" href="${escapeAttribute(permalink)}" target="_blank" rel="noopener noreferrer"><time datetime="${iso}">${escapeHtml(label)}</time></a></footer>`;
}

function escapeText(value: string): string {
  return escapeHtml(value).replaceAll("\n", "<br>");
}

function escapeAttribute(value: string): string {
  return escapeHtml(value).replaceAll("`", "&#96;");
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}
