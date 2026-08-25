// Static HTML for appearance: "full". Visual contract follows MIT-licensed
// react-tweet (Copyright (c) 2023 Luis Alvarez) and sveltweet (Copyright (c)
// 2024 ryoppippi). Notices live in social-tweet-full.css and docs/content/credits.md.
import { escapeAttribute, escapeHtml } from "./html";
import { renderMedia } from "./markup";
import { renderTweetText } from "./text";
import type { TweetAssets, TweetBodyData, TweetData, TweetUser } from "./types";
import { quotedPermalink, replyPermalink, sanitizeScreenName, sanitizeStatusId } from "./validate";

const HELP_HREF = "https://help.x.com/en/x-for-websites-ads-info-and-privacy";

export function renderFullTweet(permalink: string, data: TweetData, assets: TweetAssets): string {
  const quote = data.quoted_tweet ? renderFullQuote(data.quoted_tweet, assets.quoted) : "";
  return [
    '<figure class="ox-tweet ox-tweet--fetched ox-tweet--full">',
    renderFullHeader(data.user, assets.avatar, permalink),
    renderReply(data),
    `<div class="ox-tweet__body">${renderTweetText(data, { omitTrailingQuoteUrl: Boolean(quote) })}</div>`,
    renderMedia(assets, permalink),
    quote,
    renderInfo(permalink, data.created_at),
    renderActions(permalink, data),
    renderReplies(permalink, data.conversation_count),
    "</figure>",
  ].join("");
}

function renderFullQuote(data: TweetBodyData, assets: TweetAssets | undefined): string {
  const permalink = quotedPermalink(data) ?? "";
  return [
    '<blockquote class="ox-tweet__quote">',
    renderQuoteHeader(data.user, assets?.avatar, permalink),
    `<div class="ox-tweet__quote-body">${renderTweetText(data)}</div>`,
    renderMedia(assets ?? { media: [] }, permalink),
    "</blockquote>",
  ].join("");
}

function renderFullHeader(
  user: TweetUser,
  avatarSrc: string | undefined,
  permalink: string,
): string {
  const profile = profileHref(user);
  const follow = followHref(user);
  return [
    '<header class="ox-tweet__header">',
    `<a class="ox-tweet__avatar-link" href="${escapeAttribute(profile)}" target="_blank" rel="noopener noreferrer">`,
    avatar(avatarSrc, 48),
    "</a>",
    '<div class="ox-tweet__author">',
    `<a class="ox-tweet__author-name" href="${escapeAttribute(profile)}" target="_blank" rel="noopener noreferrer">${escapeHtml(user.name)}${verifiedBadge(user)}</a>`,
    '<div class="ox-tweet__author-meta">',
    `<a class="ox-tweet__author-handle" href="${escapeAttribute(profile)}" target="_blank" rel="noopener noreferrer">@${escapeHtml(user.screen_name)}</a>`,
    follow
      ? `<span class="ox-tweet__sep" aria-hidden="true">·</span><a class="ox-tweet__follow" href="${escapeAttribute(follow)}" target="_blank" rel="noopener noreferrer">Follow</a>`
      : "",
    "</div></div>",
    `<a class="ox-tweet__brand" href="${escapeAttribute(permalink)}" target="_blank" rel="noopener noreferrer" aria-label="View on X"><span class="ox-tweet__icon ox-tweet__icon--x"></span></a>`,
    "</header>",
  ].join("");
}

function renderQuoteHeader(
  user: TweetUser,
  avatarSrc: string | undefined,
  permalink: string,
): string {
  const href = permalink || profileHref(user);
  return [
    '<header class="ox-tweet__quote-header">',
    `<a class="ox-tweet__profile" href="${escapeAttribute(href)}" target="_blank" rel="noopener noreferrer">`,
    avatar(avatarSrc, 20),
    `<span class="ox-tweet__author-name">${escapeHtml(user.name)}${verifiedBadge(user)}</span>`,
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

function renderInfo(permalink: string, createdAt: string | undefined): string {
  const formatted = formatFullDate(createdAt);
  const time = formatted
    ? `<a class="ox-tweet__permalink" href="${escapeAttribute(permalink)}" target="_blank" rel="noopener noreferrer"><time datetime="${formatted.iso}">${escapeHtml(formatted.label)}</time></a>`
    : `<a class="ox-tweet__permalink" href="${escapeAttribute(permalink)}" target="_blank" rel="noopener noreferrer">View on X</a>`;
  return `<div class="ox-tweet__info">${time}<a class="ox-tweet__info-help" href="${HELP_HREF}" target="_blank" rel="noopener noreferrer" aria-label="X for Websites, Ads Information and Privacy"><span class="ox-tweet__icon ox-tweet__icon--info"></span></a></div>`;
}

function renderActions(permalink: string, data: TweetData): string {
  const id = statusId(data, permalink);
  if (!id) return "";
  const likes = formatCount(data.favorite_count);
  return [
    '<div class="ox-tweet__actions">',
    `<a class="ox-tweet__action ox-tweet__action--like" href="https://x.com/intent/like?tweet_id=${id}" target="_blank" rel="noopener noreferrer"><span class="ox-tweet__icon ox-tweet__icon--like"></span><span>${likes}</span></a>`,
    `<a class="ox-tweet__action ox-tweet__action--reply" href="https://x.com/intent/tweet?in_reply_to=${id}" target="_blank" rel="noopener noreferrer"><span class="ox-tweet__icon ox-tweet__icon--reply"></span>Reply</a>`,
    "</div>",
  ].join("");
}

function renderReplies(permalink: string, conversationCount: unknown): string {
  return `<p class="ox-tweet__replies"><a class="ox-tweet__replies-link" href="${escapeAttribute(permalink)}" target="_blank" rel="noopener noreferrer">${escapeHtml(repliesLabel(conversationCount))}</a></p>`;
}

function avatar(src: string | undefined, size: number): string {
  return src
    ? `<img class="ox-tweet__avatar" src="${escapeAttribute(src)}" alt="" width="${size}" height="${size}" loading="lazy" decoding="async">`
    : "";
}

function verifiedBadge(user: TweetUser): string {
  const kind = verifiedKind(user);
  return kind
    ? `<span class="ox-tweet__badge ox-tweet__badge--${kind}" title="Verified"></span>`
    : "";
}

function verifiedKind(user: TweetUser): "blue" | "gold" | "gray" | undefined {
  if (user.verified_type === "Government") return "gray";
  if (user.verified_type === "Business") return "gold";
  if (user.is_blue_verified) return "blue";
  if (user.verified) return "gray";
  return undefined;
}

function profileHref(user: TweetUser): string {
  const screen = sanitizeScreenName(user.screen_name) ?? user.screen_name;
  return `https://x.com/${encodeURIComponent(screen)}`;
}

function followHref(user: TweetUser): string | undefined {
  const screen = sanitizeScreenName(user.screen_name);
  return screen
    ? `https://x.com/intent/follow?screen_name=${encodeURIComponent(screen)}`
    : undefined;
}

function statusId(data: TweetData, permalink: string): string | undefined {
  return sanitizeStatusId(data.id_str) ?? permalink.match(/\/status\/(\d+)/)?.[1];
}

function formatCount(value: unknown): string {
  const n =
    typeof value === "number" && Number.isFinite(value) && value >= 0 ? Math.floor(value) : 0;
  if (n > 999_999) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n > 999) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

function repliesLabel(value: unknown): string {
  const n =
    typeof value === "number" && Number.isFinite(value) && value > 0 ? Math.floor(value) : 0;
  if (n === 0) return "Read more on X";
  if (n === 1) return "Read 1 reply";
  return `Read ${formatCount(n)} replies`;
}

function formatFullDate(createdAt: string | undefined): { iso: string; label: string } | undefined {
  if (!createdAt) return undefined;
  const date = new Date(createdAt);
  if (Number.isNaN(date.valueOf())) return undefined;
  const parts = new Intl.DateTimeFormat("en-US", {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
    month: "short",
    day: "numeric",
    year: "numeric",
    timeZone: "UTC",
  }).formatToParts(date);
  const get = (type: Intl.DateTimeFormatPartTypes) =>
    parts.find((part) => part.type === type)?.value ?? "";
  return {
    iso: date.toISOString(),
    label: `${get("hour")}:${get("minute")} ${get("dayPeriod")} · ${get("month")} ${get("day")}, ${get("year")}`,
  };
}
