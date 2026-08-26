import type { RedditPostData, RedditPostReference } from "./types";
import { isSafeExternalUrl } from "./url";

export function renderRedditPostCard(data: RedditPostData): string {
  const image = data.image
    ? `<img class="ox-reddit-card__image" src="${escapeAttribute(data.image.url)}" alt=""${sizeAttrs(data.image)} loading="lazy" decoding="async">`
    : "";
  const body = data.body ? `<p class="ox-reddit-card__body">${escapeText(data.body)}</p>` : "";
  const original = data.originalUrl ? renderOriginalLink(data.originalUrl) : "";

  return [
    `<article class="ox-reddit-card${data.image ? " ox-reddit-card--image" : ""}">`,
    `<a class="ox-reddit-card__main" href="${escapeAttribute(data.permalink)}" target="_blank" rel="noopener noreferrer">`,
    renderHeader(data),
    `<h3 class="ox-reddit-card__title">${escapeHtml(data.title)}</h3>`,
    body,
    image,
    renderMetrics(data),
    "</a>",
    original,
    "</article>",
  ].join("");
}

export function renderRedditFallbackCard(
  reference: RedditPostReference,
  message = "Metadata unavailable",
): string {
  const subreddit = reference.subreddit
    ? `<span class="ox-reddit-card__subreddit">r/${escapeHtml(reference.subreddit)}</span>`
    : "";
  return [
    '<article class="ox-reddit-card ox-reddit-card--fallback">',
    `<a class="ox-reddit-card__main" href="${escapeAttribute(reference.url)}" target="_blank" rel="noopener noreferrer">`,
    `<header class="ox-reddit-card__header"><span class="ox-reddit-card__network">Reddit</span>${subreddit}</header>`,
    '<h3 class="ox-reddit-card__title">Reddit post</h3>',
    `<p class="ox-reddit-card__body">${escapeHtml(message)}. Open the post on Reddit.</p>`,
    '<footer class="ox-reddit-card__meta"><span class="ox-reddit-card__source">Open post</span></footer>',
    "</a>",
    "</article>",
  ].join("");
}

export function renderRedditErrorCard(): string {
  return [
    '<article class="ox-reddit-card error">',
    '<a class="ox-reddit-card__main" href="#" target="_blank" rel="noopener noreferrer">',
    '<header class="ox-reddit-card__header"><span class="ox-reddit-card__network">Reddit</span></header>',
    '<h3 class="ox-reddit-card__title">Unsupported Reddit URL</h3>',
    '<p class="ox-reddit-card__body">Use a public reddit.com post URL or redd.it share URL.</p>',
    "</a>",
    "</article>",
  ].join("");
}

function renderHeader(data: RedditPostData): string {
  const author = data.author
    ? `<span class="ox-reddit-card__author">u/${escapeHtml(data.author)}</span>`
    : "";
  const time = data.createdAt
    ? `<time class="ox-reddit-card__time" datetime="${escapeAttribute(data.createdAt)}">${escapeHtml(dateLabel(data.createdAt))}</time>`
    : "";
  return [
    '<header class="ox-reddit-card__header">',
    '<span class="ox-reddit-card__network">Reddit</span>',
    `<span class="ox-reddit-card__subreddit">r/${escapeHtml(data.subreddit)}</span>`,
    author,
    time,
    "</header>",
  ].join("");
}

function renderMetrics(data: RedditPostData): string {
  const parts = [
    metric(data.score, "point", "points"),
    metric(data.commentCount, "comment", "comments"),
    '<span class="ox-reddit-card__source">Open post</span>',
  ].filter(Boolean);
  return `<footer class="ox-reddit-card__meta">${parts.join("")}</footer>`;
}

function renderOriginalLink(url: string): string {
  if (!isSafeExternalUrl(url)) return "";
  return `<a class="ox-reddit-card__original" href="${escapeAttribute(url)}" target="_blank" rel="noopener noreferrer">Original link: ${escapeHtml(domainLabel(url))}</a>`;
}

function metric(value: number | undefined, singular: string, plural: string): string {
  if (value === undefined) return "";
  return `<span class="ox-reddit-card__metric"><strong>${formatCount(value)}</strong> ${value === 1 ? singular : plural}</span>`;
}

function sizeAttrs(image: NonNullable<RedditPostData["image"]>): string {
  const width = image.width ? ` width="${image.width}"` : "";
  const height = image.height ? ` height="${image.height}"` : "";
  return `${width}${height}`;
}

function dateLabel(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) return value;
  return date.toISOString().slice(0, 10);
}

function domainLabel(value: string): string {
  try {
    return new URL(value).hostname.replace(/^www\./, "");
  } catch {
    return value;
  }
}

function formatCount(value: number): string {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1000) return `${(value / 1000).toFixed(1)}K`;
  return String(value);
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
