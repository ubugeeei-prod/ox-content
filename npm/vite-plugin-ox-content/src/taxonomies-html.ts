/**
 * Escaped taxonomy HTML and confined output paths.
 */

import * as path from "node:path";

/** One built page considered for terms and related lists. */
export interface TaxonomySourcePage {
  title: string;
  frontmatter: Record<string, unknown>;
  transformedHtml: string;
  inputPath?: string;
  routePaths: { href: string };
}

export interface TermBucket {
  label: string;
  slug: string;
  pages: TaxonomySourcePage[];
}

export function relatedMarkup(pages: readonly TaxonomySourcePage[]): string {
  const items = pages.map((page) => listItem(page.routePaths.href, page.title)).join("");
  return `<nav class="ox-related" aria-label="Related pages"><h2>Related pages</h2><ul>${items}</ul></nav>`;
}

export function listPageContent(
  terms: readonly TermBucket[],
  base: string,
  urlName: string,
): string {
  const items = terms
    .map((term) => listItem(siteHref(base, urlName, term.slug), term.label))
    .join("");
  return `<h1>${escapeHtml(displayTaxonomyName(urlName))}</h1><ul class="ox-taxonomy">${items}</ul>`;
}

export function termPageContent(term: TermBucket): string {
  const pages = [...term.pages].sort((left, right) => {
    const titleCmp = left.title.localeCompare(right.title);
    return titleCmp !== 0 ? titleCmp : left.routePaths.href.localeCompare(right.routePaths.href);
  });
  const items = pages.map((page) => listItem(page.routePaths.href, page.title)).join("");
  return `<h1>${escapeHtml(term.label)}</h1><ul class="ox-taxonomy-term">${items}</ul>`;
}

export function displayTaxonomyName(name: string): string {
  return name.charAt(0).toUpperCase() + name.slice(1);
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

function listItem(href: string, label: string): string {
  return `<li><a href="${escapeHtml(href)}">${escapeHtml(label)}</a></li>`;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}
