import { escapeAttribute } from "./html";
import type { TweetAssets, TweetMediaAsset } from "./types";

export function renderMedia(assets: TweetAssets, permalink: string): string {
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
