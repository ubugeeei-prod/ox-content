/**
 * RSS 2.0 / Atom 1.0 item extraction. HTML documents are rejected.
 */

import { canonicalizeFeedItemUrl } from "./blog-feed-url";
import { parseFeedDate } from "./blog-feed-date";
import type { ParsedDate } from "./feed-format";

const ITEM_BLOCK = /<(?:[\w.-]+:)?item\b[^>]*>([\s\S]*?)<\/(?:[\w.-]+:)?item>/gi;
const ENTRY_BLOCK = /<(?:[\w.-]+:)?entry\b[^>]*>([\s\S]*?)<\/(?:[\w.-]+:)?entry>/gi;

export interface ParsedFeedItem {
  title: string;
  link: string;
  id: string;
  date?: ParsedDate;
  language?: string;
  summary?: string;
}

export function parseBlogFeed(body: string, feedLanguage?: string): ParsedFeedItem[] {
  const xml = stripBom(body);
  if (looksLikeHtml(xml)) {
    throw new Error("not a feed");
  }
  if (!looksLikeXmlFeed(xml)) {
    throw new Error("malformed XML");
  }
  const channelLanguage =
    textChild(xml, ["language", "dc:language"]) ?? xmlLang(xml) ?? feedLanguage;
  const items = collectBlocks(xml, ITEM_BLOCK).map((block) =>
    normalizeRssItem(block, channelLanguage),
  );
  if (items.length > 0 || /<(?:[\w.-]+:)?rss\b/i.test(xml)) {
    return items.filter((item): item is ParsedFeedItem => item != null);
  }
  return collectBlocks(xml, ENTRY_BLOCK)
    .map((block) => normalizeAtomEntry(block, channelLanguage))
    .filter((item): item is ParsedFeedItem => item != null);
}

function normalizeRssItem(block: string, fallbackLanguage?: string): ParsedFeedItem | undefined {
  const title = textChild(block, ["title"]);
  const guid = guidChild(block);
  const link =
    canonicalizeFeedItemUrl(textChild(block, ["link"]) ?? "") ??
    (guid?.permalink ? canonicalizeFeedItemUrl(guid.value) : undefined);
  if (!title || !link) {
    return undefined;
  }
  const id = guid?.value || link;
  return {
    title,
    link,
    id,
    date: parseFeedDate(textChild(block, ["pubDate", "dc:date", "published", "updated"])),
    language: textChild(block, ["language", "dc:language"]) ?? xmlLang(block) ?? fallbackLanguage,
    summary: textChild(block, ["description", "summary", "content:encoded", "content"]),
  };
}

function normalizeAtomEntry(block: string, fallbackLanguage?: string): ParsedFeedItem | undefined {
  const title = textChild(block, ["title"]);
  const link = canonicalizeFeedItemUrl(atomLink(block) ?? "");
  if (!title || !link) {
    return undefined;
  }
  const id = textChild(block, ["id"]) || link;
  return {
    title,
    link,
    id,
    date: parseFeedDate(textChild(block, ["published", "updated", "dc:date"])),
    language: xmlLang(block) ?? textChild(block, ["language", "dc:language"]) ?? fallbackLanguage,
    summary: textChild(block, ["summary", "content", "description"]),
  };
}

function collectBlocks(xml: string, pattern: RegExp): string[] {
  return [...xml.matchAll(pattern)].flatMap((match) => (match[1] ? [match[1]] : []));
}

function textChild(block: string, names: readonly string[]): string | undefined {
  for (const name of names) {
    const pattern = new RegExp(
      `<(?:[\\w.-]+:)?${escapeRegExp(localName(name))}(?:\\s[^>]*)?>([\\s\\S]*?)</(?:[\\w.-]+:)?${escapeRegExp(localName(name))}>`,
      "i",
    );
    const match = block.match(pattern);
    if (match?.[1] != null) {
      const text = decodeXmlText(match[1]);
      if (text) {
        return text;
      }
    }
  }
  return undefined;
}

function guidChild(block: string): { value: string; permalink: boolean } | undefined {
  const match = block.match(/<(?:[\w.-]+:)?guid\b([^>]*)>([\s\S]*?)<\/(?:[\w.-]+:)?guid>/i);
  if (!match?.[2]) {
    return undefined;
  }
  const value = decodeXmlText(match[2]);
  if (!value) {
    return undefined;
  }
  const attrs = match[1] ?? "";
  const permalink = !/isPermaLink\s*=\s*(['"]?)false\1/i.test(attrs);
  return { value, permalink };
}

function atomLink(block: string): string | undefined {
  const links: Array<{ href: string; rel: string }> = [];
  const pattern = /<(?:[\w.-]+:)?link\b([^>]*)\/?>/gi;
  for (const match of block.matchAll(pattern)) {
    const href = attrValue(match[1] ?? "", "href");
    if (!href) {
      continue;
    }
    links.push({ href, rel: (attrValue(match[1] ?? "", "rel") ?? "alternate").toLowerCase() });
  }
  return links.find((link) => link.rel === "alternate")?.href ?? links[0]?.href;
}

function xmlLang(block: string): string | undefined {
  const match = block.match(/\bxml:lang\s*=\s*(['"])([^'"]+)\1/i);
  const value = match?.[2]?.trim();
  return value || undefined;
}

function attrValue(attrs: string, name: string): string | undefined {
  const match = attrs.match(new RegExp(`(?:^|\\s)${name}\\s*=\\s*(['"])([\\s\\S]*?)\\1`, "i"));
  return match?.[2]?.trim() || undefined;
}

function decodeXmlText(value: string): string {
  const withoutCdata = value.replace(/<!\[CDATA\[([\s\S]*?)]]>/g, "$1");
  const withoutTags = withoutCdata.replace(/<[^>]+>/g, " ");
  return decodeEntities(withoutTags).replace(/\s+/g, " ").trim();
}

function decodeEntities(value: string): string {
  return value.replace(/&(#x?[0-9a-f]+|[a-z]+);/gi, (entity, name: string) => {
    const lower = name.toLowerCase();
    if (lower === "amp") return "&";
    if (lower === "lt") return "<";
    if (lower === "gt") return ">";
    if (lower === "quot") return '"';
    if (lower === "apos") return "'";
    if (lower.startsWith("#x")) {
      const code = Number.parseInt(lower.slice(2), 16);
      return Number.isFinite(code) ? String.fromCodePoint(code) : entity;
    }
    if (lower.startsWith("#")) {
      const code = Number.parseInt(lower.slice(1), 10);
      return Number.isFinite(code) ? String.fromCodePoint(code) : entity;
    }
    return entity;
  });
}

function looksLikeHtml(body: string): boolean {
  const start = body.trim().slice(0, 256).toLowerCase();
  return start.startsWith("<!doctype html") || start.startsWith("<html");
}

function looksLikeXmlFeed(body: string): boolean {
  return /<(?:[\w.-]+:)?(?:rss|feed|rdf:RDF|item|entry)\b/i.test(body);
}

function stripBom(value: string): string {
  return value.charCodeAt(0) === 0xfeff ? value.slice(1) : value;
}

function localName(name: string): string {
  const index = name.indexOf(":");
  return index === -1 ? name : name.slice(index + 1);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
