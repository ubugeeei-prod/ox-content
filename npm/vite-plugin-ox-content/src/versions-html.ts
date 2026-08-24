/**
 * Escaped version switcher, banner, and badge markup.
 */

import type { VersionBannerKind } from "./types";

export interface VersionLink {
  id: string;
  label: string;
  href: string;
  current: boolean;
  banner?: VersionBannerKind | false;
}

const STYLE = `<style>
.ox-version-switcher{position:relative;display:inline-flex;align-items:center;margin-inline-end:.5rem}
.ox-version-switcher > button{font:inherit;background:transparent;border:1px solid currentColor;border-radius:.4rem;padding:.2rem .55rem;cursor:pointer}
.ox-version-switcher[open] > button{border-bottom-left-radius:0;border-bottom-right-radius:0}
.ox-version-switcher ul{position:absolute;inset-inline-start:0;top:100%;z-index:20;margin:0;padding:.25rem 0;list-style:none;min-width:100%;background:var(--ox-bg,Canvas);border:1px solid currentColor;border-radius:0 .4rem .4rem}
.ox-version-switcher a,.ox-version-switcher span{display:block;padding:.25rem .7rem;white-space:nowrap;color:inherit;text-decoration:none}
.ox-version-badge{display:inline-block;margin-inline-start:.35rem;font-size:.75em;opacity:.75}
.ox-version-banner{padding:.6rem 1rem;border-bottom:1px solid currentColor}
.ox-version-banner--unreleased{background:#fff7ed}
.ox-version-banner--unmaintained{background:#f4f4f5}
</style>`;

export function versionSwitcherMarkup(links: readonly VersionLink[], badge: boolean): string {
  if (links.length === 0) {
    return "";
  }
  const current = links.find((link) => link.current) ?? links[0];
  const items = links
    .map((link) => {
      const label = `${escapeHtml(link.label)}${badgeMarkup(link, badge)}`;
      if (link.current || !isSafeHref(link.href)) {
        return `<li><span aria-current="page">${label}</span></li>`;
      }
      return `<li><a href="${escapeHtml(link.href)}">${label}</a></li>`;
    })
    .join("");
  return `${STYLE}<nav class="ox-version-switcher" aria-label="Version"><button type="button" aria-expanded="false" aria-haspopup="listbox">${escapeHtml(current.label)}${badgeMarkup(current, badge)}</button><ul hidden>${items}</ul></nav><script>(function(){var n=document.currentScript&&document.currentScript.previousElementSibling;if(!n||!n.classList.contains("ox-version-switcher"))return;var b=n.querySelector("button"),u=n.querySelector("ul");if(!b||!u)return;function c(){var o=b.getAttribute("aria-expanded")==="true";b.setAttribute("aria-expanded",o?"false":"true");u.hidden=o;}b.addEventListener("click",c);document.addEventListener("keydown",function(e){if(e.key==="Escape"){b.setAttribute("aria-expanded","false");u.hidden=true;b.focus();}});})()</script>`;
}

export function versionBannerMarkup(kind: VersionBannerKind | false | undefined): string {
  if (kind === "unreleased") {
    return `<aside class="ox-version-banner ox-version-banner--unreleased" role="status">This documentation describes an unreleased version.</aside>`;
  }
  if (kind === "unmaintained") {
    return `<aside class="ox-version-banner ox-version-banner--unmaintained" role="status">This documentation is unmaintained.</aside>`;
  }
  return "";
}

export function injectVersionChrome(
  html: string,
  switcher: string,
  banner: string,
  searchFrom?: string,
  searchTo?: string,
): string {
  let next = html;
  if (banner) {
    next = next.replace(/<body([^>]*)>/, `<body$1>${banner}`);
  }
  if (switcher) {
    if (next.includes('<div class="header-actions">')) {
      next = next.replace(
        '<div class="header-actions">',
        `<div class="header-actions">${switcher}`,
      );
    } else if (next.includes("</header>")) {
      next = next.replace("</header>", `${switcher}</header>`);
    }
  }
  if (searchTo && isSafeHref(searchTo)) {
    next = next.replace(/<html([^>]*)>/i, (match, attrs: string) => {
      if (/\sdata-ox-search-index=/.test(attrs)) {
        return match;
      }
      return `<html${attrs} data-ox-search-index="${escapeHtml(searchTo)}">`;
    });
  }
  if (searchFrom && searchTo && searchFrom !== searchTo && isSafeHref(searchTo)) {
    next = next.split(searchFrom).join(searchTo);
    const script = `<script>(function(){var f=${JSON.stringify(searchFrom)},t=${JSON.stringify(searchTo)};var o=window.fetch;window.fetch=function(i,n){if(typeof i==="string"&&i.indexOf(f)!==-1)i=i.split(f).join(t);return o.call(this,i,n);};})()</script>`;
    next = next.includes("</body>")
      ? next.replace("</body>", `${script}</body>`)
      : `${next}${script}`;
  }
  return next;
}

export function searchIndexUrl(base: string, prefix: string): string {
  const root = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  return prefix ? `${root}${prefix}/search-index.json` : `${root}search-index.json`;
}

export function isSafeHref(href: string): boolean {
  const trimmed = href.trim();
  if (!trimmed || trimmed.startsWith("//")) {
    return false;
  }
  const lower = trimmed.replace(/\s+/g, "").toLowerCase();
  if (
    lower.startsWith("javascript:") ||
    lower.startsWith("data:") ||
    lower.startsWith("vbscript:")
  ) {
    return false;
  }
  return trimmed.startsWith("/") || trimmed.startsWith("./") || !trimmed.includes(":");
}

export function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function badgeMarkup(link: VersionLink, badge: boolean): string {
  if (!badge || !link.banner) {
    return "";
  }
  const text = link.banner === "unreleased" ? "unreleased" : "unmaintained";
  return `<span class="ox-version-badge">${text}</span>`;
}
