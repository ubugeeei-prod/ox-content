/**
 * Rewrites default-theme nav hrefs to the current locale sibling when it exists.
 */

import { resolveLocaleLabel, type HeaderNavItem, type LocaleLabel } from "./header-chrome";
import { normalizeLocalePath, pathForLocale, remainderPath } from "./locale-switcher";
import type { SidebarItem } from "./theme";
import type { LocaleConfig } from "./types";

/** @internal Label metadata kept off the serializable navigation shape. */
const localizedNavTitle: unique symbol = Symbol("ox-content.localized-nav-title");

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

type LocalizedNavItem = LocalizableNavItem & {
  [localizedNavTitle]?: LocaleLabel;
  children?: LocalizedNavItem[];
};

type LocalizedNavGroup = LocalizableNavGroup & {
  [localizedNavTitle]?: LocaleLabel;
  items: LocalizedNavItem[];
};

interface ResolvedSidebarItem {
  text?: string;
  link?: string;
  items?: ResolvedSidebarItem[];
  collapsed?: boolean;
  stickyCollapsed?: boolean;
}

/** @internal Flattens sidebar locale maps before crossing the string-only NAPI boundary. */
export function resolveSidebarItems(
  sidebar: readonly SidebarItem[],
  locale?: string,
  defaultLocale?: string,
): ResolvedSidebarItem[] {
  return sidebar.map((item) => ({
    text:
      item.text === undefined ? undefined : resolveLocaleLabel(item.text, locale, defaultLocale),
    link: item.link,
    items: item.items ? resolveSidebarItems(item.items, locale, defaultLocale) : undefined,
    collapsed: item.collapsed,
    stickyCollapsed: item.stickyCollapsed,
  }));
}

/** @internal Associates rendered nav nodes with authored locale maps by tree position. */
export function attachSidebarLabels<T extends LocalizableNavGroup>(
  groups: T[],
  sidebar: readonly SidebarItem[],
): T[] {
  const sources = sidebarGroupSources(sidebar);
  return groups.map((group, index) => {
    const source = sources[index];
    return {
      ...group,
      ...(source?.title === undefined ? {} : { [localizedNavTitle]: source.title }),
      items: attachItemLabels(group.items, source?.items ?? []),
    } as T;
  });
}

function sidebarGroupSources(sidebar: readonly SidebarItem[]): Array<{
  title?: LocaleLabel;
  items: readonly SidebarItem[];
}> {
  const groups: Array<{ title?: LocaleLabel; items: readonly SidebarItem[] }> = [];
  let loose: SidebarItem[] = [];
  const flushLoose = () => {
    if (loose.length > 0) {
      groups.push({ items: loose });
      loose = [];
    }
  };
  for (const item of sidebar) {
    if ((item.items?.length ?? 0) > 0 && item.link === undefined) {
      flushLoose();
      groups.push({ title: item.text, items: item.items ?? [] });
    } else {
      loose.push(item);
    }
  }
  flushLoose();
  return groups;
}

function attachItemLabels<T extends LocalizableNavItem>(
  items: T[],
  sources: readonly SidebarItem[],
): T[] {
  return items.map((item, index) => {
    const source = sources[index];
    return {
      ...item,
      ...(source?.text === undefined ? {} : { [localizedNavTitle]: source.text }),
      children: attachItemLabels(item.children ?? [], source?.items ?? []),
    };
  });
}

/**
 * Resolves authored sidebar label maps and prefixes hrefs/paths with the
 * current locale when that page exists. Missing siblings stay as authored.
 */
export function localizeNavGroups<T extends LocalizableNavGroup>(
  groups: T[],
  options: LocalizeNavOptions,
): T[] {
  const lookup = pageLookup(options);
  if (!lookup && !hasLocalizedTitles(groups)) {
    return groups;
  }
  return groups.map((group) => ({
    ...group,
    title: resolveNavTitle(group, options),
    items: group.items.map((item) => localizeNavItem(item, options, lookup)),
  }));
}

/**
 * Resolves header labels and rewrites `link` values the same way as the sidebar.
 */
export function localizeHeaderNavItems(
  items: HeaderNavItem[] | undefined,
  options: LocalizeNavOptions,
): HeaderNavItem[] | undefined {
  if (!items?.length) {
    return items;
  }
  const lookup = pageLookup(options);
  return items.map((item) => ({
    ...item,
    text: resolveLocaleLabel(item.text, options.locale, options.defaultLocale),
    link: item.link && lookup ? localizeHref(item.link, options, lookup) : item.link,
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
  lookup: Map<string, LocalePageRef> | undefined,
): T {
  if (!lookup) {
    return {
      ...item,
      title: resolveNavTitle(item, options),
      children: (item.children ?? []).map((child) => localizeNavItem(child, options, lookup)),
    };
  }
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
    title: resolveNavTitle(item, options),
    href: sibling ? `${sibling.href}${hash}` : item.href,
    path: sibling ? sibling.path : item.path,
    children: (item.children ?? []).map((child) => localizeNavItem(child, options, lookup)),
  };
}

function resolveNavTitle(
  item: LocalizableNavItem | LocalizableNavGroup,
  options: LocalizeNavOptions,
): string {
  const label = (item as LocalizedNavItem | LocalizedNavGroup)[localizedNavTitle];
  return label === undefined
    ? item.title
    : resolveLocaleLabel(label, options.locale, options.defaultLocale);
}

function hasLocalizedTitles(groups: readonly LocalizableNavGroup[]): boolean {
  return groups.some(
    (group) =>
      (group as LocalizedNavGroup)[localizedNavTitle] !== undefined ||
      hasLocalizedItemTitles(group.items),
  );
}

function hasLocalizedItemTitles(items: readonly LocalizableNavItem[]): boolean {
  return items.some(
    (item) =>
      (item as LocalizedNavItem)[localizedNavTitle] !== undefined ||
      hasLocalizedItemTitles(item.children ?? []),
  );
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
