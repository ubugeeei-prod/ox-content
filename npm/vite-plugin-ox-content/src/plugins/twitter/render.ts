import { renderFullTweet } from "./full";
import { escapeAttribute, escapeHtml } from "./html";
import { renderMedia } from "./markup";
import { renderTweetText } from "./text";
import type { ResolvedTwitterEmbedOptions, TweetAssets, TweetBodyData, TweetData } from "./types";
import { quotedPermalink, replyPermalink, sanitizeScreenName } from "./validate";

export { renderTweetText } from "./text";

export function renderFetchedTweet(
  permalink: string,
  data: TweetData,
  assets: TweetAssets,
  options: ResolvedTwitterEmbedOptions,
): string {
  if (options.appearance === "full") {
    return renderFullTweet(permalink, data, assets);
  }
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
