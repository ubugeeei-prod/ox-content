/**
 * Section-index listing HTML and href safety.
 *
 * Titles are escaped. `javascript:` / `data:` / `vbscript:` / `file:` hrefs
 * are dropped. The NAPI helper is preferred when present.
 */

import { importNapiModuleSync } from "./napi";
import type { SectionIndexStyle } from "./types";

const HOSTILE_SCHEME = /^(?:javascript|data|vbscript|file):/i;

/** One child link on a generated section index. */
export interface SectionIndexItem {
  title: string;
  href: string;
  description?: string;
}

/** `https:`-free, same-origin or relative href. `javascript:` is rejected. */
export function isSafeSectionHref(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed || /[\n\r\0\t]/.test(trimmed) || trimmed.startsWith("//")) {
    return false;
  }
  if (trimmed.startsWith("/")) {
    return true;
  }
  const scheme = trimmed.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
  if (scheme) {
    return false;
  }
  return !HOSTILE_SCHEME.test(trimmed);
}

/** Escapes text and attribute values in generated listing markup. */
export function escapeSectionIndexHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

/** Renders the listing body. Titles are escaped; hostile hrefs are dropped. */
export function renderSectionIndexHtml(
  title: string,
  items: readonly SectionIndexItem[],
  style: SectionIndexStyle,
): string {
  try {
    const napi = importNapiModuleSync() as typeof import("@ox-content/napi") & {
      renderSsgSectionIndex?: (
        title: string,
        items: Array<{ title: string; href: string; description?: string }>,
        style: string,
      ) => string;
    };
    if (typeof napi.renderSsgSectionIndex === "function") {
      return napi.renderSsgSectionIndex(
        title,
        items.map((item) => ({
          title: item.title,
          href: item.href,
          description: item.description,
        })),
        style,
      );
    }
  } catch {
    // Fall through to the local renderer when the native helper is absent.
  }
  return renderSectionIndexHtmlLocal(title, items, style);
}

function renderSectionIndexHtmlLocal(
  title: string,
  items: readonly SectionIndexItem[],
  style: SectionIndexStyle,
): string {
  const safe = items.filter((item) => isSafeSectionHref(item.href));
  const modifier = style === "list" ? "list" : "cards";
  const listClass = style === "list" ? "ox-section-index__list" : "ox-section-index__cards";
  const body = safe.map((item) => renderItem(item, style)).join("");
  return (
    `<nav class="ox-section-index ox-section-index--${modifier}" aria-label="Section pages">` +
    `<h1>${escapeSectionIndexHtml(title)}</h1>` +
    `<ul class="${listClass}">${body}</ul>` +
    `</nav>`
  );
}

function renderItem(item: SectionIndexItem, style: SectionIndexStyle): string {
  const href = escapeSectionIndexHtml(item.href.trim());
  const label = escapeSectionIndexHtml(item.title);
  if (style === "list") {
    return `<li><a href="${href}">${label}</a></li>`;
  }
  const description =
    typeof item.description === "string" && item.description.trim()
      ? `<span class="ox-section-index__desc">${escapeSectionIndexHtml(item.description)}</span>`
      : "";
  return (
    `<li class="ox-section-index__card">` +
    `<a href="${href}"><span class="ox-section-index__title">${label}</span>${description}</a>` +
    `</li>`
  );
}
