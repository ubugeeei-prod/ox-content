/**
 * Rewrites default-theme nav hrefs to the current locale sibling when it exists.
 */

import type { HeaderNavItem } from "./header-chrome";
import { normalizeLocalePath, pathForLocale, remainderPath } from "./locale-switcher";
import type { LocaleConfig } from "./types";

export interface LocalePageRef {
  path: string;
  href: string;
}

export interface LocalizeNavOptions {
  locale: string;
  locales: readonly Pick<LocaleConfig, "code">[];
  defaultLocale: string;
  hideDefaultLocale: boolean;
  pages: readonly LocalePageRef[];
  base: string;
}

export interface LocalizableNavItem {
  title: string;
  path: string;
  href: string;
  children?: LocalizableNavItem[];
  collapsed?: boolean;
  stickyCollapsed?: boolean;
}

export interface LocalizableNavGroup {
  title: string;
  items: LocalizableNavItem[];
  collapsed?: boolean;
  stickyCollapsed?: boolean;
}

/**
 * Prefixes sidebar hrefs/paths with the current locale when that page exists.
 * Missing siblings and the hidden default locale stay as authored.
 */
export function localizeNavGroups<T extends LocalizableNavGroup>(
  groups: T[],
  options: LocalizeNavOptions,
): T[] {
  const lookup = pageLookup(options);
  if (!lookup) {
    return groups;
  }
  return groups.map((group) => ({
    ...group,
    items: group.items.map((item) => localizeNavItem(item, options, lookup)),
  }));
}

/**
 * Rewrites header nav `link` values the same way as the sidebar.
 */
export function localizeHeaderNavItems(
  items: HeaderNavItem[] | undefined,
  options: LocalizeNavOptions,
): HeaderNavItem[] | undefined {
  if (!items?.length) {
    return items;
  }
  const lookup = pageLookup(options);
  if (!lookup) {
    return items;
  }
  return items.map((item) => ({
    ...item,
    link: item.link ? localizeHref(item.link, options, lookup) : item.link,
    items: localizeHeaderNavItems(item.items, options),
  }));
}

export function localizeHref(
  href: string,
  options: LocalizeNavOptions,
  lookup = pageLookup(options),
): string {
  if (!lookup) {
    return href;
  }
  const hash = href.includes("#") ? href.slice(href.indexOf("#")) : "";
  const sitePath = sitePathFromHref(href, options.base);
  if (sitePath === undefined) {
    return href;
  }
  const remainder = stripLocalePrefix(sitePath, options.locales);
  const siblingPath = pathForLocale(
    remainder,
    options.locale,
    options.defaultLocale,
    options.hideDefaultLocale,
  );
  const sibling = lookup.get(normalizeLocalePath(siblingPath));
  return sibling ? `${sibling.href}${hash}` : href;
}

export function sitePathFromHref(href: string, base: string): string | undefined {
  const trimmed = href.trim();
  if (!trimmed || trimmed.startsWith("#") || trimmed.startsWith("//")) {
    return undefined;
  }
  const noHash = trimmed.split("#")[0]?.split("?")[0] ?? "";
  const compact = noHash.replace(/\s+/g, "").toLowerCase();
  if (
    compact.startsWith("javascript:") ||
    compact.startsWith("data:") ||
    compact.startsWith("vbscript:")
  ) {
    return undefined;
  }
  if (/^[a-z][a-z0-9+.-]*:/i.test(noHash)) {
    return undefined;
  }
  const normalizedBase = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  let path = noHash;
  if (normalizedBase !== "/" && path.startsWith(normalizedBase)) {
    path = path.slice(normalizedBase.length);
  } else if (path.startsWith("/")) {
    path = path.slice(1);
  } else {
    return undefined;
  }
  path = path
    .replace(/\/index\.html$/i, "")
    .replace(/\.html$/i, "")
    .replace(/\.(mdx|markdown|md)$/i, "")
    .replace(/\/+$/g, "");
  if (path === "index") {
    return "";
  }
  return path;
}

function localizeNavItem<T extends LocalizableNavItem>(
  item: T,
  options: LocalizeNavOptions,
  lookup: Map<string, LocalePageRef>,
): T {
  const hash = item.href.includes("#") ? item.href.slice(item.href.indexOf("#")) : "";
  const sitePath = sitePathFromHref(item.href, options.base) ?? normalizeLocalePath(item.path);
  const remainder = stripLocalePrefix(sitePath, options.locales);
  const siblingPath = pathForLocale(
    remainder,
    options.locale,
    options.defaultLocale,
    options.hideDefaultLocale,
  );
  const sibling = lookup.get(normalizeLocalePath(siblingPath));
  return {
    ...item,
    href: sibling ? `${sibling.href}${hash}` : item.href,
    path: sibling ? sibling.path : item.path,
    children: (item.children ?? []).map((child) => localizeNavItem(child, options, lookup)),
  };
}

function pageLookup(options: LocalizeNavOptions): Map<string, LocalePageRef> | undefined {
  if (!options.locale || options.pages.length === 0) {
    return undefined;
  }
  if (options.hideDefaultLocale && options.locale === options.defaultLocale) {
    return undefined;
  }
  return new Map(options.pages.map((page) => [normalizeLocalePath(page.path), page]));
}

function stripLocalePrefix(
  sitePath: string,
  locales: readonly Pick<LocaleConfig, "code">[],
): string {
  const normalized = normalizeLocalePath(sitePath);
  const codes = locales.map((locale) => locale.code).sort((a, b) => b.length - a.length);
  for (const code of codes) {
    if (normalized === code || normalized.startsWith(`${code}/`)) {
      return remainderPath(normalized, code);
    }
  }
  return normalized;
}
