/**
 * Escaped blog index, tag, archive, and post-meta HTML.
 */

import * as path from "node:path";
import type { BlogAuthor } from "./types";

/** One listed post considered for blog surfaces. */
export interface BlogSourcePage {
  title: string;
  frontmatter: Record<string, unknown>;
  transformedHtml: string;
  inputPath: string;
  routePaths: { href: string };
  /** When true, href is a remote canonical URL and must not become a local route. */
  external?: boolean;
}

export interface BlogPostMeta {
  authors: readonly BlogAuthor[];
  minutes: number;
  tags: readonly { label: string; href: string }[];
}

export interface BlogListItem {
  title: string;
  href: string;
  dateLabel?: string;
  external?: boolean;
}

/** `https:` or a same-origin path starting with `/` but not `//`. */
export function isSafeBlogUrl(value: string): boolean {
  const trimmed = value.trim();
  if (
    trimmed.length === 0 ||
    trimmed.split("").some((ch) => ch === "\n" || ch === "\r" || ch === "\0" || ch === "\t")
  ) {
    return false;
  }
  if (trimmed.startsWith("//")) {
    return false;
  }
  if (trimmed.startsWith("/")) {
    return true;
  }
  return trimmed.toLowerCase().startsWith("https:");
}

export function postMetaMarkup(meta: BlogPostMeta): string {
  const parts = [
    `<p class="ox-blog-meta__reading-time">${escapeHtml(String(meta.minutes))} min read</p>`,
  ];
  if (meta.authors.length > 0) {
    const items = meta.authors.map((author) => authorMarkup(author)).join("");
    parts.push(`<ul class="ox-blog-meta__authors">${items}</ul>`);
  }
  if (meta.tags.length > 0) {
    const items = meta.tags
      .map((tag) => `<li><a href="${escapeHtml(tag.href)}">${escapeHtml(tag.label)}</a></li>`)
      .join("");
    parts.push(`<ul class="ox-blog-meta__tags">${items}</ul>`);
  }
  return `<aside class="ox-blog-meta">${parts.join("")}</aside>\n`;
}

export function indexPageContent(
  items: readonly BlogListItem[],
  pager: { newerHref?: string; olderHref?: string },
): string {
  const list = items.map((item) => listItem(item)).join("");
  const links: string[] = [];
  if (pager.newerHref) {
    links.push(`<a href="${escapeHtml(pager.newerHref)}" rel="prev">Newer</a>`);
  }
  if (pager.olderHref) {
    links.push(`<a href="${escapeHtml(pager.olderHref)}" rel="next">Older</a>`);
  }
  const nav = links.length > 0 ? `<nav class="ox-blog-pager">${links.join("")}</nav>` : "";
  return `<h1>Blog</h1><ul class="ox-blog">${list}</ul>${nav}`;
}

export function tagPageContent(label: string, items: readonly BlogListItem[]): string {
  const list = items.map((item) => listItem(item)).join("");
  return `<h1>${escapeHtml(label)}</h1><ul class="ox-blog-tag">${list}</ul>`;
}

export function archiveIndexContent(years: readonly { year: string; href: string }[]): string {
  const items = years
    .map((entry) => `<li><a href="${escapeHtml(entry.href)}">${escapeHtml(entry.year)}</a></li>`)
    .join("");
  return `<h1>Archive</h1><ul class="ox-blog-archive">${items}</ul>`;
}

export function archiveYearContent(
  year: string,
  months: readonly { month: string; href: string }[],
  items: readonly BlogListItem[],
): string {
  const monthList = months
    .map((entry) => `<li><a href="${escapeHtml(entry.href)}">${escapeHtml(entry.month)}</a></li>`)
    .join("");
  const posts = items.map((item) => listItem(item)).join("");
  return `<h1>${escapeHtml(year)}</h1><ul class="ox-blog-archive-months">${monthList}</ul><ul class="ox-blog">${posts}</ul>`;
}

export function archiveMonthContent(label: string, items: readonly BlogListItem[]): string {
  const list = items.map((item) => listItem(item)).join("");
  return `<h1>${escapeHtml(label)}</h1><ul class="ox-blog">${list}</ul>`;
}

export function siteHref(base: string, ...segments: string[]): string {
  const prefix = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  const rest = segments.filter(Boolean).join("/");
  return rest ? `${prefix}${rest}/` : prefix;
}

export function containedPath(outDir: string, ...segments: string[]): string | undefined {
  const root = path.resolve(outDir);
  const resolved = path.resolve(root, ...segments);
  const prefix = root.endsWith(path.sep) ? root : `${root}${path.sep}`;
  if (resolved === root || !resolved.startsWith(prefix)) {
    return undefined;
  }
  return resolved;
}

export function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function authorMarkup(author: BlogAuthor): string {
  const name = escapeHtml(author.name);
  const url = author.url?.trim();
  const heading =
    url && isSafeBlogUrl(url)
      ? `<a class="ox-blog-meta__name" href="${escapeHtml(url)}">${name}</a>`
      : `<span class="ox-blog-meta__name">${name}</span>`;
  const bio =
    author.bio && author.bio.length > 0
      ? `<p class="ox-blog-meta__bio">${escapeHtml(author.bio)}</p>`
      : "";
  return `<li>${heading}${bio}</li>`;
}

function listItem(item: BlogListItem): string {
  const time = item.dateLabel
    ? ` <time datetime="${escapeHtml(item.dateLabel)}">${escapeHtml(item.dateLabel)}</time>`
    : "";
  if (item.external) {
    return `<li class="ox-blog-external" data-ox-blog-external="true"><a href="${escapeHtml(item.href)}" rel="external noopener noreferrer">${escapeHtml(item.title)}</a>${time}</li>`;
  }
  return `<li><a href="${escapeHtml(item.href)}">${escapeHtml(item.title)}</a>${time}</li>`;
}
