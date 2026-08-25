/**
 * Language and version filters for the default search dialog.
 */

import { escapeHtml, isSafeHref } from "./versions-html";

const RESULTS_MARKUP = '<div class="search-results"></div>';

export interface SearchLocaleOption {
  code: string;
  name: string;
}

export interface SearchVersionOption {
  id: string;
  label: string;
  prefix: string;
  indexUrl: string;
  current: boolean;
}

export interface SearchLocaleFilterInput {
  locales: readonly SearchLocaleOption[];
  current?: string;
  defaultLocale: string;
}

/** Resolves a document path to a locale code after stripping version prefixes. */
export function searchDocumentLocale(
  path: string,
  localeCodes: readonly string[],
  defaultLocale: string,
  versionPrefixes: readonly string[] = [],
): string {
  const fallback = (defaultLocale || "en").toLowerCase();
  let source = path.replace(/^\/+/u, "").toLowerCase();
  if (versionPrefixes.length === 1) {
    source = stripVersionPrefix(source, versionPrefixes[0]!);
  } else if (versionPrefixes.length > 1) {
    const prefixes = versionPrefixes
      .map((prefix) => prefix.replace(/^\/+|\/+$/gu, "").toLowerCase())
      .filter(Boolean)
      .sort((left, right) => right.length - left.length);
    for (const prefix of prefixes) {
      const next = stripVersionPrefix(source, prefix);
      if (next !== source) {
        source = next;
        break;
      }
    }
  }
  const slash = source.indexOf("/");
  const first = slash === -1 ? source : source.slice(0, slash);
  if (!first) return fallback;
  for (const code of localeCodes) {
    if (code.toLowerCase() === first) return first;
  }
  return fallback;
}

function stripVersionPrefix(source: string, prefix: string): string {
  const normalized = prefix.replace(/^\/+|\/+$/gu, "").toLowerCase();
  if (!normalized) return source;
  if (source === normalized) return "";
  return source.startsWith(`${normalized}/`) ? source.slice(normalized.length + 1) : source;
}

export function injectSearchLocaleFilters(html: string, input: SearchLocaleFilterInput): string {
  const locales = input.locales.filter((locale) => locale.code.trim() && locale.name.trim());
  if (locales.length < 2) {
    return html;
  }
  const next = ensureSearchFilters(html);
  const select = selectMarkup(next, "locale");
  if (!select) {
    return next;
  }
  const defaultLocale = input.defaultLocale.trim() || locales[0]!.code;
  const selected = locales.some((locale) => locale.code === input.current)
    ? input.current!
    : defaultLocale;
  const options = [
    `<option value="">All languages</option>`,
    ...locales.map((locale) => {
      const code = escapeHtml(locale.code);
      const selectedAttr = locale.code === selected ? " selected" : "";
      return `<option value="${code}"${selectedAttr}>${escapeHtml(locale.name)}</option>`;
    }),
  ].join("");
  return revealFilter(
    next.replace(
      select.markup,
      `<select class="search-filter-select" data-search-filter="locale" data-default-locale="${escapeHtml(defaultLocale)}" aria-label="Language">${options}</select>`,
    ),
    "locale",
  );
}

export function injectSearchVersionFilters(
  html: string,
  versions: readonly SearchVersionOption[],
): string {
  const safe = versions.filter(
    (version) => version.id.trim() && version.label.trim() && isSafeHref(version.indexUrl),
  );
  if (safe.length < 2) {
    return html;
  }
  const next = ensureSearchFilters(html);
  const select = selectMarkup(next, "version");
  if (!select) {
    return next;
  }
  const current = safe.find((version) => version.current) ?? safe[0]!;
  const options = safe
    .map((version) => {
      const selected = version.id === current.id ? " selected" : "";
      return `<option value="${escapeHtml(version.id)}" data-prefix="${escapeHtml(version.prefix)}" data-index="${escapeHtml(version.indexUrl)}"${selected}>${escapeHtml(version.label)}</option>`;
    })
    .join("");
  return revealFilter(
    next.replace(
      select.markup,
      `<select class="search-filter-select" data-search-filter="version" aria-label="Version">${options}</select>`,
    ),
    "version",
  );
}

function ensureSearchFilters(html: string): string {
  if (html.includes('class="search-filters"') || !html.includes(RESULTS_MARKUP)) {
    return html;
  }
  return html.replace(RESULTS_MARKUP, `${searchFiltersMarkup()}${RESULTS_MARKUP}`);
}

function searchFiltersMarkup(): string {
  return `${searchFiltersStyle()}<div class="search-filters"><label class="search-filter" data-search-filter-label="locale" hidden><span class="search-filter-label">Language</span><select class="search-filter-select" data-search-filter="locale" data-default-locale="" aria-label="Language"></select></label><label class="search-filter" data-search-filter-label="version" hidden><span class="search-filter-label">Version</span><select class="search-filter-select" data-search-filter="version" aria-label="Version"></select></label></div>`;
}

function searchFiltersStyle(): string {
  return `<style class="ox-search-filters-style">.search-filters{display:flex;flex-wrap:wrap;gap:.75rem;align-items:center;padding:.65rem 1rem;border-bottom:1px solid var(--octc-color-border);background:var(--octc-color-bg-alt)}.search-filter{display:flex;align-items:center;gap:.4rem;min-width:0}.search-filter[hidden]{display:none}.search-filter-label{font-size:.75rem;color:var(--octc-color-text-muted);white-space:nowrap}.search-filter-select{min-width:8rem;max-width:12rem;padding:.25rem .4rem;border:1px solid var(--octc-color-border);border-radius:4px;background:var(--octc-color-bg);color:var(--octc-color-text);font:inherit;font-size:.8125rem}.search-filter-select:focus{outline:2px solid var(--octc-color-primary);outline-offset:1px}</style>`;
}

function selectMarkup(html: string, kind: "locale" | "version"): { markup: string } | undefined {
  const match = html.match(
    new RegExp(
      `<select class="search-filter-select" data-search-filter="${kind}"[^>]*>[\\s\\S]*?<\\/select>`,
    ),
  );
  return match?.[0] ? { markup: match[0] } : undefined;
}

function revealFilter(html: string, kind: "locale" | "version"): string {
  return html.replace(
    `data-search-filter-label="${kind}" hidden`,
    `data-search-filter-label="${kind}"`,
  );
}
