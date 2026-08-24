/**
 * Keeps navigation inside a frozen documentation-version tree.
 *
 * Locale resolution runs before these helpers. The lookup therefore uses
 * unversioned route keys while every destination points at the versioned
 * output tree.
 */

import type { HeaderNavItem } from "./header-chrome";
import { sitePathFromHref } from "./locale-nav";

/** @internal */
export interface VersionNavigationPage {
  /** Canonical route before the documentation-version prefix is added. */
  path: string;
  /** Canonical route after the documentation-version prefix is added. */
  versionedPath: string;
  /** Final versioned href. */
  href: string;
  /** File-tree route, used when a manual nav item predates a permalink. */
  sourcePath?: string;
  /** Frontmatter aliases that resolve to this page. */
  aliases?: readonly string[];
}

/** @internal */
export interface VersionNavigationContext {
  prefix: string;
  base: string;
  root: VersionNavigationTarget;
  pages: Array<{ path: string; href: string; aliases?: readonly string[] }>;
  lookup: ReadonlyMap<string, VersionNavigationTarget>;
}

interface VersionNavigationTarget {
  path: string;
  href: string;
}

interface VersionableNavItem {
  path: string;
  href: string;
  children?: VersionableNavItem[];
}

interface VersionableNavGroup {
  items: VersionableNavItem[];
}

/** @internal */
export function createVersionNavigationContext(input: {
  prefix: string;
  base: string;
  pages: readonly VersionNavigationPage[];
  redirects?: Readonly<Record<string, string>>;
}): VersionNavigationContext {
  const prefix = normalizeRouteKey(input.prefix, input.base);
  const root: VersionNavigationTarget = {
    path: prefix,
    href: siteHref(input.base, prefix),
  };
  const lookup = new Map<string, VersionNavigationTarget>();
  const targets = input.pages.map((page) => ({
    page,
    target: { path: normalizeRouteKey(page.versionedPath, input.base), href: page.href },
  }));

  // Canonical pages always win over a colliding source path or alias.
  for (const { page, target } of targets) {
    const key = routeLookupKey(page.path, input.base, prefix);
    if (key !== undefined) {
      lookup.set(key, target);
    }
  }
  for (const { page, target } of targets) {
    addLookup(lookup, page.sourcePath, target, input.base, prefix);
    for (const alias of page.aliases ?? []) {
      addLookup(lookup, alias, target, input.base, prefix);
    }
  }

  resolveRedirectAliases(lookup, input.redirects, input.base, prefix);
  return {
    prefix,
    base: input.base,
    root,
    pages: input.pages.map((page) => ({
      path: page.path,
      href: page.href,
      aliases: navigationAliases(page, lookup, input.redirects, input.base, prefix),
    })),
    lookup,
  };
}

function navigationAliases(
  page: VersionNavigationPage,
  lookup: ReadonlyMap<string, VersionNavigationTarget>,
  redirects: Readonly<Record<string, string>> | undefined,
  base: string,
  prefix: string,
): string[] | undefined {
  const target = lookup.get(routeLookupKey(page.path, base, prefix) ?? "");
  const aliases = [page.sourcePath, ...(page.aliases ?? [])].filter(
    (value): value is string => typeof value === "string",
  );
  if (target && redirects) {
    for (const from of Object.keys(redirects)) {
      const key = routeLookupKey(from, base, prefix);
      if (key !== undefined && lookup.get(key) === target) {
        aliases.push(key);
      }
    }
  }
  const unique = [...new Set(aliases.map((value) => normalizeRouteKey(value, base)))];
  return unique.length > 0 ? unique : undefined;
}

/**
 * Rewrites safe internal sidebar destinations and all nested children.
 * @internal
 */
export function rewriteVersionedNavGroups<T extends VersionableNavGroup>(
  groups: T[],
  context: VersionNavigationContext,
): T[] {
  return groups.map(
    (group) =>
      ({
        ...group,
        items: group.items.map((item) => rewriteNavItem(item, context)),
      }) as T,
  );
}

/**
 * Rewrites safe internal header destinations with the same sibling policy.
 * @internal
 */
export function rewriteVersionedHeaderNavItems(
  items: HeaderNavItem[] | undefined,
  context: VersionNavigationContext,
): HeaderNavItem[] | undefined {
  return items?.map((item) => ({
    ...item,
    link: item.link ? rewriteHref(item.link, context).href : item.link,
    items: rewriteVersionedHeaderNavItems(item.items, context),
  }));
}

/**
 * Rewrites one safe internal destination, including pager overrides.
 * @internal
 */
export function rewriteVersionedHref(href: string, context: VersionNavigationContext): string {
  return rewriteHref(href, context).href;
}

/**
 * Removes only the active version prefix, leaving locale/path resolution intact.
 * @internal
 */
export function unversionedPath(path: string, context: VersionNavigationContext): string {
  const normalized = normalizeRouteKey(path, context.base);
  if (normalized === context.prefix) {
    return "";
  }
  return normalized.startsWith(`${context.prefix}/`)
    ? normalized.slice(context.prefix.length + 1)
    : normalized;
}

/** @internal Keeps missing locale siblings inside the active version tree. */
export function versionedLocaleRoots(
  context: VersionNavigationContext,
  locales: readonly { code: string }[],
  defaultLocale: string,
  hideDefaultLocale: boolean,
): Record<string, string> {
  return Object.fromEntries(
    locales.map((locale) => {
      const route = hideDefaultLocale && locale.code === defaultLocale ? "" : locale.code;
      return [locale.code, context.lookup.get(route)?.href ?? context.root.href];
    }),
  );
}

function rewriteNavItem<T extends VersionableNavItem>(
  item: T,
  context: VersionNavigationContext,
): T {
  const rewritten = rewriteHref(item.href, context, item.path);
  return {
    ...item,
    href: rewritten.href,
    path: rewritten.path,
    children: (item.children ?? []).map((child) => rewriteNavItem(child, context)),
  };
}

function rewriteHref(
  href: string,
  context: VersionNavigationContext,
  path?: string,
): VersionNavigationTarget {
  const hrefKey = sitePathFromHref(href, context.base);
  if (hrefKey === undefined) {
    return { path: path ?? "", href };
  }
  const suffixIndex = href.search(/[?#]/u);
  const suffix = suffixIndex === -1 ? "" : href.slice(suffixIndex);
  const target = [path, hrefKey]
    .map((candidate) => routeLookupKey(candidate, context.base, context.prefix))
    .find((candidate) => candidate !== undefined && context.lookup.has(candidate));
  const resolved = target === undefined ? context.root : context.lookup.get(target)!;
  return { path: resolved.path, href: `${resolved.href}${suffix}` };
}

function addLookup(
  lookup: Map<string, VersionNavigationTarget>,
  value: string | undefined,
  target: VersionNavigationTarget,
  base: string,
  prefix: string,
): void {
  const key = routeLookupKey(value, base, prefix);
  if (key !== undefined && !lookup.has(key)) {
    lookup.set(key, target);
  }
}

function resolveRedirectAliases(
  lookup: Map<string, VersionNavigationTarget>,
  redirects: Readonly<Record<string, string>> | undefined,
  base: string,
  prefix: string,
): void {
  if (!redirects) {
    return;
  }
  const pending = Object.entries(redirects);
  for (let pass = 0; pass <= pending.length; pass++) {
    let changed = false;
    for (const [from, to] of pending) {
      const fromKey = routeLookupKey(from, base, prefix);
      const toKey = routeLookupKey(to, base, prefix);
      const target = toKey === undefined ? undefined : lookup.get(toKey);
      if (fromKey !== undefined && target && !lookup.has(fromKey)) {
        lookup.set(fromKey, target);
        changed = true;
      }
    }
    if (!changed) {
      break;
    }
  }
}

function routeLookupKey(
  value: string | undefined,
  base: string,
  prefix: string,
): string | undefined {
  if (value === undefined) {
    return undefined;
  }
  const fromHref = sitePathFromHref(value, base);
  const key = normalizeRouteKey(fromHref ?? value, base);
  if (key === prefix) {
    return "";
  }
  return key.startsWith(`${prefix}/`) ? key.slice(prefix.length + 1) : key;
}

function normalizeRouteKey(value: string, base: string): string {
  const fromHref = sitePathFromHref(value, base);
  return (fromHref ?? value)
    .trim()
    .split(/[?#]/u, 1)[0]!
    .replace(/^\/+|\/+$/gu, "")
    .replace(/\/index\.html$/iu, "")
    .replace(/\.(?:mdx|markdown|md|html)$/iu, "");
}

function siteHref(base: string, path: string): string {
  const root = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  return path ? `${root}${path}/` : root;
}
