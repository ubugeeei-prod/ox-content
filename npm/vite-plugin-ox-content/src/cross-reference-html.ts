import type { CrossReferenceKind } from "./cross-reference-types";

const PROTECTED_TEXT_RE = /<!--[\s\S]*?-->|<(pre|code|script|style|textarea|a)\b[\s\S]*?<\/\1>/gi;
const TAG_RE = /(<[^>]+>)/g;

export function transformText(html: string, replacer: (text: string) => string): string {
  let output = "";
  let cursor = 0;
  for (const match of html.matchAll(PROTECTED_TEXT_RE)) {
    const start = match.index ?? 0;
    output += transformTextOutsideTags(html.slice(cursor, start), replacer);
    output += match[0];
    cursor = start + match[0].length;
  }
  return output + transformTextOutsideTags(html.slice(cursor), replacer);
}

export function findFirstImageWithId(body: string): {
  id: string;
  attrs: string;
  start: number;
  end: number;
  alt?: string;
} | null {
  const match = /<img\b([^>]*)>/i.exec(body);
  if (!match) return null;
  const attrs = match[1] ?? "";
  const id = readAttr(attrs, "id");
  if (!id) return null;
  return {
    id,
    attrs,
    start: match.index,
    end: match.index + match[0].length,
    alt: readAttr(attrs, "alt"),
  };
}

export function figureCaption(body: string): string | undefined {
  const match = /<figcaption\b[^>]*>([\s\S]*?)<\/figcaption>/i.exec(body);
  return match ? textContent(match[1] ?? "") : undefined;
}

export function expectedKind(id: string): CrossReferenceKind | null {
  const prefix = id.toLowerCase().split("-")[0];
  if (prefix === "fig" || prefix === "figure") return "figure";
  if (prefix === "tbl" || prefix === "table") return "table";
  if (prefix === "sec" || prefix === "section") return "section";
  return null;
}

export function shouldTrackTarget(id: string): boolean {
  return expectedKind(id) !== null;
}

export function appendDataAttrs(
  attrs: string,
  kind: CrossReferenceKind,
  number: string,
  text: string,
): string {
  return appendAttr(
    appendAttr(appendAttr(attrs, "data-ox-xref-kind", kind), "data-ox-xref-number", number),
    "data-ox-xref-label",
    text,
  );
}

export function appendAttr(attrs: string, name: string, value: string): string {
  if (readAttr(attrs, name) !== undefined) return attrs;
  return `${attrs} ${name}="${escapeAttr(value)}"`;
}

export function readAttr(attrs: string, name: string): string | undefined {
  const escapedName = escapeRegExp(name);
  const quoted = new RegExp(`(?:^|\\s)${escapedName}\\s*=\\s*("([^"]*)"|'([^']*)')`, "i").exec(
    attrs,
  );
  if (quoted) return decodeHtml(quoted[2] ?? quoted[3] ?? "");
  const bare = new RegExp(`(?:^|\\s)${escapedName}(?:\\s|$)`, "i").exec(attrs);
  return bare ? "" : undefined;
}

export function textContent(html: string): string {
  return decodeHtml(
    html
      .replace(/<a\b[^>]*class=(["'])header-anchor\1[\s\S]*?<\/a>/gi, "")
      .replace(/<[^>]+>/g, "")
      .replace(/\s+/g, " ")
      .trim(),
  );
}

export function escapeHtml(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

export function escapeAttr(value: string): string {
  return escapeHtml(value).replace(/"/g, "&quot;");
}

export function escapeUrlFragment(value: string): string {
  return encodeURIComponent(value).replace(/%2D/gi, "-").replace(/%5F/gi, "_");
}

function transformTextOutsideTags(segment: string, replacer: (text: string) => string): string {
  return segment
    .split(TAG_RE)
    .map((part) => (part.startsWith("<") ? part : replacer(part)))
    .join("");
}

function decodeHtml(value: string): string {
  return value
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
