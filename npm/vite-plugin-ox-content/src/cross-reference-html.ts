/**
 * The escaping and prose-scanning helpers the citation pass still needs.
 *
 * The cross-reference pass that used to share this file now runs in
 * `ox_content_transform::cross_references`; what is left here is what
 * `citations.ts` uses, and it goes the same way when that pass moves.
 */

const PROTECTED_TEXT_RE = /<!--[\s\S]*?-->|<(pre|code|script|style|textarea|a)\b[\s\S]*?<\/\1>/gi;
const TAG_RE = /(<[^>]+>)/g;

/** Runs `replacer` over every stretch of prose, skipping verbatim elements. */
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
