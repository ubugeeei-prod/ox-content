/**
 * Collect local `src`, `poster`, and relevant `href` values from HTML tags.
 */

const RESOURCE_TAG = /<(?:img|video|audio|source|track|a)\b[^>]*>/gi;
const RESOURCE_ATTR = /\b(src|poster|href)\s*=\s*(?:"([^"]*)"|'([^']*)')/gi;

export type ResourceAttrName = "src" | "poster" | "href";

export interface ResourceHtmlRef {
  attr: ResourceAttrName;
  raw: string;
  value: string;
}

export function collectResourceTags(html: string): { tag: string; refs: ResourceHtmlRef[] }[] {
  const tags = html.match(RESOURCE_TAG) ?? [];
  return tags
    .map((tag) => ({ tag, refs: collectResourceRefs(tag) }))
    .filter((entry) => entry.refs.length > 0);
}

function collectResourceRefs(tag: string): ResourceHtmlRef[] {
  const name = /^<([a-z]+)/i.exec(tag)?.[1]?.toLowerCase();
  if (!name) {
    return [];
  }
  const refs: ResourceHtmlRef[] = [];
  RESOURCE_ATTR.lastIndex = 0;
  let match = RESOURCE_ATTR.exec(tag);
  while (match) {
    const attr = match[1]!.toLowerCase() as ResourceAttrName;
    if (isRelevantAttr(name, attr)) {
      const raw = match[2] ?? match[3] ?? "";
      refs.push({ attr, raw, value: unescapeHtml(raw) });
    }
    match = RESOURCE_ATTR.exec(tag);
  }
  return refs;
}

function isRelevantAttr(tagName: string, attr: ResourceAttrName): boolean {
  if (tagName === "a") {
    return attr === "href";
  }
  if (attr === "href") {
    return false;
  }
  if (attr === "poster") {
    return tagName === "video";
  }
  return attr === "src";
}

export function unescapeHtml(value: string): string {
  return value
    .replaceAll("&amp;", "&")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">");
}

export function escapeAttribute(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

export function replaceAttributeRaw(tag: string, raw: string, nextRaw: string): string {
  const index = tag.indexOf(raw);
  if (index === -1) {
    return tag;
  }
  return tag.slice(0, index) + nextRaw + tag.slice(index + raw.length);
}
