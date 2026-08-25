import { escapeAttribute, escapeHtml } from "./html";
import { renderTweetText } from "./text";
import type {
  ResolvedTwitterEmbedOptions,
  TweetAssets,
  TweetBodyData,
  TweetData,
  TweetMediaAsset,
} from "./types";
import { quotedPermalink, replyPermalink, sanitizeScreenName } from "./validate";

export { renderTweetText } from "./text";

export function renderFetchedTweet(
  permalink: string,
  data: TweetData,
  assets: TweetAssets,
  options: ResolvedTwitterEmbedOptions,
): string {
  const quote = data.quoted_tweet ? renderQuotedTweet(data.quoted_tweet, assets.quoted) : "";
  return [
    '<figure class="ox-tweet ox-tweet--fetched">',
    renderHeader(data.user, assets.avatar),
    renderReply(data),
    `<div class="ox-tweet__body">${renderTweetText(data, { omitTrailingQuoteUrl: Boolean(quote) })}</div>`,
    renderMedia(assets, permalink),
    quote,
    renderFooter(permalink, data.created_at, options.lang),
    "</figure>",
  ].join("");
}

function renderQuotedTweet(data: TweetBodyData, assets: TweetAssets | undefined): string {
  const permalink = quotedPermalink(data) ?? "";
  return [
    '<blockquote class="ox-tweet__quote">',
    renderHeader(data.user, assets?.avatar, permalink || undefined, "ox-tweet__quote-header"),
    `<div class="ox-tweet__quote-body">${renderTweetText(data)}</div>`,
    renderMedia(assets ?? { media: [] }, permalink),
    "</blockquote>",
  ].join("");
}

function renderHeader(
  user: TweetBodyData["user"],
  avatarSrc: string | undefined,
  href?: string,
  headerClass = "ox-tweet__header",
): string {
  const screen = sanitizeScreenName(user.screen_name) ?? user.screen_name;
  const profile = href ?? `https://x.com/${encodeURIComponent(screen)}`;
  const avatar = avatarSrc
    ? `<img class="ox-tweet__avatar" src="${escapeAttribute(avatarSrc)}" alt="" width="48" height="48" loading="lazy" decoding="async">`
    : "";
  return [
    `<header class="${headerClass}">`,
    `<a class="ox-tweet__profile" href="${escapeAttribute(profile)}" target="_blank" rel="noopener noreferrer">`,
    avatar,
    `<span class="ox-tweet__author-name">${escapeHtml(user.name)}</span>`,
    `<span class="ox-tweet__author-handle">@${escapeHtml(user.screen_name)}</span>`,
    "</a></header>",
  ].join("");
}

function renderReply(data: TweetData): string {
  const href = replyPermalink(data);
  const handle = data.in_reply_to_screen_name;
  if (!href || !handle) return "";
  return `<p class="ox-tweet__reply"><a class="ox-tweet__reply-link" href="${escapeAttribute(href)}" target="_blank" rel="noopener noreferrer">Replying to @${escapeHtml(handle)}</a></p>`;
}

function renderMedia(assets: TweetAssets, permalink: string): string {
  if (assets.media.length === 0) return "";
  const items = assets.media.map((item) => renderMediaItem(item, permalink)).join("");
  return `<div class="ox-tweet__media" data-count="${assets.media.length}">${items}</div>`;
}

function renderMediaItem(item: TweetMediaAsset, permalink: string): string {
  if (item.kind === "video" || item.kind === "animated_gif") {
    return renderVideoItem(item, permalink);
  }
  const size = sizeAttributes(item);
  return `<img class="ox-tweet__media-item" src="${escapeAttribute(item.src ?? "")}" alt="${escapeAttribute(item.alt ?? "")}"${size} loading="lazy" decoding="async">`;
}

function renderVideoItem(item: TweetMediaAsset, permalink: string): string {
  const watch = watchOnX(permalink);
  const size = sizeAttributes(item);
  const src = selfHostedMediaSrc(item.src);
  if (src) {
    const poster = item.poster ? ` poster="${escapeAttribute(item.poster)}"` : "";
    const gif = item.kind === "animated_gif" ? " muted loop" : "";
    return `<video class="ox-tweet__media-item" src="${escapeAttribute(src)}"${poster}${size} controls playsinline preload="none"${gif}>${watch}</video>`;
  }
  const poster = item.poster
    ? `<img src="${escapeAttribute(item.poster)}" alt="${escapeAttribute(item.alt ?? "")}"${size} loading="lazy" decoding="async">`
    : "";
  return `<div class="ox-tweet__media-item ox-tweet__media-fallback">${poster}${watch}</div>`;
}

function selfHostedMediaSrc(src: string | undefined): string | undefined {
  return src && !src.includes("video.twimg.com") ? src : undefined;
}

function sizeAttributes(item: Pick<TweetMediaAsset, "width" | "height">): string {
  return [
    item.width ? ` width="${item.width}"` : "",
    item.height ? ` height="${item.height}"` : "",
  ].join("");
}

function watchOnX(permalink: string): string {
  if (!permalink) return "";
  return `<a class="ox-tweet__watch" href="${escapeAttribute(permalink)}" target="_blank" rel="noopener noreferrer">Watch on X</a>`;
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
