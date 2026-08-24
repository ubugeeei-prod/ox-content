/**
 * Opt-in draft / unlisted / scheduled page classification.
 */

import { importNapiModuleSync } from "./napi";
import type { PublishStateOptions, ResolvedPublishStateOptions } from "./types";

interface NavItemLike {
  title: string;
  path: string;
  href: string;
  children?: NavItemLike[];
}

interface NavGroupLike {
  title: string;
  items: NavItemLike[];
}

/** One page considered for publish-state filtering. */
export interface PublishStatePage {
  inputPath: string;
  title: string;
  frontmatter: Record<string, unknown>;
  routePaths: {
    href: string;
    urlPath: string;
  };
}

/** Split pages into production output vs listing surfaces. */
export interface PartitionedPages<T> {
  output: T[];
  listed: T[];
}

/**
 * Resolves `publishState` with defaults.
 *
 * `false` / omitted stays off. `true` enables production filtering. An object
 * enables the feature and overrides only the fields the site set.
 */
export function resolvePublishStateOptions(
  value: boolean | PublishStateOptions | undefined,
): ResolvedPublishStateOptions {
  if (!value) {
    return { enabled: false, includeDrafts: false };
  }
  if (value === true) {
    return { enabled: true, includeDrafts: false };
  }
  return {
    enabled: value.enabled ?? true,
    now: value.now,
    includeDrafts: value.includeDrafts ?? false,
  };
}

/** Classifies one frontmatter object. Never throws. */
export function classifyPublishState(
  frontmatter: Record<string, unknown>,
  options: ResolvedPublishStateOptions | undefined,
): { output: boolean; listed: boolean } {
  try {
    return importNapiModuleSync().classifyPublishState(
      JSON.stringify(frontmatter ?? {}),
      toNapiPublishState(options),
    );
  } catch {
    return { output: true, listed: true };
  }
}

/** Splits pages into those that write HTML and those that appear in listings. */
export function partitionPublishedPages<T extends { frontmatter: Record<string, unknown> }>(
  pages: readonly T[],
  options: ResolvedPublishStateOptions | undefined,
): PartitionedPages<T> {
  if (!options?.enabled) {
    return { output: [...pages], listed: [...pages] };
  }
  const output: T[] = [];
  const listed: T[] = [];
  for (const page of pages) {
    const decision = classifyPublishState(page.frontmatter, options);
    if (decision.output) {
      output.push(page);
    }
    if (decision.listed) {
      listed.push(page);
    }
  }
  return { output, listed };
}

/** Drops nav items that resolve to hidden (unpublished or unlisted) pages. */
export function filterNavGroups<T extends NavGroupLike>(
  groups: T[],
  hidden: ReadonlySet<string>,
): T[] {
  return groups
    .map((group) => ({
      ...group,
      items: filterNavItems(group.items, hidden),
    }))
    .filter((group) => group.items.length > 0);
}

function filterNavItems<T extends NavItemLike>(items: T[], hidden: ReadonlySet<string>): T[] {
  const kept: SsgNavItem[] = [];
  for (const item of items) {
    if (isHiddenNavTarget(item, hidden)) {
      continue;
    }
    const children = item.children?.length ? filterNavItems(item.children, hidden) : item.children;
    kept.push(children === item.children ? item : { ...item, children });
  }
  return kept;
}

function isHiddenNavTarget(item: NavItemLike, hidden: ReadonlySet<string>): boolean {
  return hidden.has(item.path) || hidden.has(item.href);
}

/** Keys used to match a page against generated nav items. */
export function hiddenNavKeys(
  pages: readonly PublishStatePage[],
  listed: readonly PublishStatePage[],
): Set<string> {
  const listedPaths = new Set(listed.map((page) => page.inputPath));
  const hidden = new Set<string>();
  for (const page of pages) {
    if (listedPaths.has(page.inputPath)) {
      continue;
    }
    hidden.add(page.routePaths.urlPath);
    hidden.add(page.routePaths.href);
  }
  return hidden;
}

export function toNapiPublishState(
  options: ResolvedPublishStateOptions | undefined,
): { enabled?: boolean; now?: string; includeDrafts?: boolean } | undefined {
  if (!options) {
    return undefined;
  }
  return {
    enabled: options.enabled,
    now: options.now,
    includeDrafts: options.includeDrafts,
  };
}
