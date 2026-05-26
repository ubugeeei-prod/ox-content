import { importNapiModuleSync } from "./napi";
import { defineTheme, mergeThemes, type ThemeConfig } from "./theme";
import type { OxContentOptions, SsgNavigationGroup, SsgNavigationItem } from "./types";

export interface VitePressLogo {
  light?: string;
  dark?: string;
  src?: string;
  alt?: string;
}

export interface VitePressSocialLink {
  icon: string;
  link: string;
  ariaLabel?: string;
}

export interface VitePressFooter {
  message?: string;
  copyright?: string;
}

export interface VitePressSidebarItem {
  text?: string;
  link?: string;
  items?: VitePressSidebarItem[];
  collapsed?: boolean;
}

export type VitePressSidebar = VitePressSidebarItem[] | Record<string, VitePressSidebarItem[]>;

export interface VitePressNavItem {
  text?: string;
  link?: string;
  items?: VitePressNavItem[];
  activeMatch?: string;
}

export interface VitePressThemeConfig {
  siteTitle?: string | false;
  logo?: string | VitePressLogo;
  nav?: VitePressNavItem[];
  sidebar?: VitePressSidebar;
  socialLinks?: VitePressSocialLink[];
  footer?: VitePressFooter;
  search?: {
    placeholder?: string;
  };
}

export interface VitePressConfig {
  title?: string;
  description?: string;
  base?: string;
  themeConfig?: VitePressThemeConfig;
}

export interface GenerateVitePressMigrationConfigOptions {
  importSource?: string;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isExternalLink(value: string): boolean {
  return /^[a-z][a-z0-9+.-]*:/i.test(value) || value.startsWith("//");
}

function splitLink(value: string): { pathname: string; suffix: string } {
  const match = /^([^?#]*)([?#].*)?$/.exec(value);
  return {
    pathname: match?.[1] ?? value,
    suffix: match?.[2] ?? "",
  };
}

function normalizeInternalPath(value: string): string {
  const { pathname } = splitLink(value.trim());
  let normalized = pathname || "/";

  if (!normalized.startsWith("/")) {
    normalized = `/${normalized}`;
  }

  normalized = normalized
    .replace(/\/index(?:\.(?:html?|md|markdown))?$/i, "/")
    .replace(/\.(?:html?|md|markdown)$/i, "");

  if (normalized !== "/") {
    normalized = normalized.replace(/\/+$/, "");
  }

  return normalized || "/";
}

function formatTitle(value: string): string {
  return value
    .replace(/[-_]([a-z])/g, (_, char: string) => ` ${char.toUpperCase()}`)
    .replace(/^[a-z]/, (char) => char.toUpperCase());
}

function titleFromPath(value: string): string {
  const normalized = normalizeInternalPath(value);
  if (normalized === "/") {
    return "Home";
  }

  const segment = normalized.split("/").filter(Boolean).pop() ?? "Page";
  return formatTitle(segment);
}

function titleFromSidebarKey(value: string): string {
  const segment = value
    .replace(/^\/+|\/+$/g, "")
    .split("/")
    .filter(Boolean)
    .pop();
  return formatTitle(segment ?? "guide");
}

function toNavigationItem(text: string | undefined, link: string): SsgNavigationItem {
  const title = text?.trim() || titleFromPath(link);

  if (isExternalLink(link) || link.startsWith("#")) {
    return { title, href: link };
  }

  const { suffix } = splitLink(link);
  const path = normalizeInternalPath(link);

  return suffix ? { title, path, href: `${path}${suffix}` } : { title, path };
}

function dedupeNavigationItems(items: SsgNavigationItem[]): SsgNavigationItem[] {
  const seen = new Set<string>();
  const next: SsgNavigationItem[] = [];

  for (const item of items) {
    const key = `${item.title}::${item.path ?? ""}::${item.href ?? ""}`;
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    next.push(item);
  }

  return next;
}

function dedupeNavigationGroups(groups: SsgNavigationGroup[]): SsgNavigationGroup[] {
  const merged = new Map<string, SsgNavigationItem[]>();
  const orderedTitles: string[] = [];

  for (const group of groups) {
    if (group.items.length === 0) {
      continue;
    }

    if (!merged.has(group.title)) {
      merged.set(group.title, []);
      orderedTitles.push(group.title);
    }

    merged.get(group.title)!.push(...group.items);
  }

  return orderedTitles.map((title) => ({
    title,
    items: dedupeNavigationItems(merged.get(title) ?? []),
  }));
}

function collectSidebarLinks(items: VitePressSidebarItem[]): SsgNavigationItem[] {
  const links: SsgNavigationItem[] = [];

  for (const item of items) {
    if (item.link) {
      links.push(toNavigationItem(item.text, item.link));
    }

    if (item.items?.length) {
      links.push(...collectSidebarLinks(item.items));
    }
  }

  return dedupeNavigationItems(links);
}

function sidebarArrayToGroups(
  items: VitePressSidebarItem[],
  fallbackTitle: string,
): SsgNavigationGroup[] {
  const groups: SsgNavigationGroup[] = [];
  const rootItems: SsgNavigationItem[] = [];

  for (const item of items) {
    if (item.link) {
      rootItems.push(toNavigationItem(item.text, item.link));
    }

    if (item.items?.length) {
      const children = collectSidebarLinks(item.items);
      if (children.length > 0) {
        groups.push({
          title: item.text?.trim() || fallbackTitle,
          items: children,
        });
      }
    }
  }

  if (rootItems.length > 0) {
    groups.unshift({
      title: fallbackTitle,
      items: dedupeNavigationItems(rootItems),
    });
  }

  return groups;
}

function collectNavLinks(items: VitePressNavItem[]): SsgNavigationItem[] {
  const links: SsgNavigationItem[] = [];

  for (const item of items) {
    if (item.link) {
      links.push(toNavigationItem(item.text, item.link));
    }

    if (item.items?.length) {
      links.push(...collectNavLinks(item.items));
    }
  }

  return dedupeNavigationItems(links);
}

function resolveLogoSrc(logo: string | VitePressLogo | undefined): string | undefined {
  if (!logo) {
    return undefined;
  }

  if (typeof logo === "string") {
    return logo;
  }

  return logo.light ?? logo.dark ?? logo.src;
}

function normalizeSocialIcon(icon: string): "github" | "twitter" | "discord" | undefined {
  const normalized = icon.trim().toLowerCase();

  if (normalized === "github") return "github";
  if (normalized === "discord") return "discord";
  if (normalized === "twitter" || normalized === "x" || normalized === "x-twitter") {
    return "twitter";
  }

  return undefined;
}

function toThemeConfig(themeConfig: VitePressThemeConfig | undefined): ThemeConfig | undefined {
  if (!themeConfig) {
    return undefined;
  }

  const logo = resolveLogoSrc(themeConfig.logo);
  const socialLinks = Object.fromEntries(
    (themeConfig.socialLinks ?? [])
      .map((link) => {
        const key = normalizeSocialIcon(link.icon);
        return key ? [key, link.link] : null;
      })
      .filter((entry): entry is [string, string] => entry !== null),
  );

  const theme: ThemeConfig = {
    ...(logo
      ? {
          header: {
            logo,
          },
        }
      : {}),
    ...(themeConfig.footer?.message || themeConfig.footer?.copyright
      ? {
          footer: {
            message: themeConfig.footer.message,
            copyright: themeConfig.footer.copyright,
          },
        }
      : {}),
    ...(Object.keys(socialLinks).length > 0
      ? {
          socialLinks,
        }
      : {}),
  };

  return logo || Object.keys(socialLinks).length > 0 || themeConfig.footer
    ? defineTheme(theme)
    : undefined;
}

function resolveSiteName(config: VitePressConfig): string | undefined {
  const siteTitle = config.themeConfig?.siteTitle;
  if (typeof siteTitle === "string" && siteTitle.trim()) {
    return siteTitle;
  }

  return config.title;
}

function mergeOxContentOptions(
  baseOptions: OxContentOptions,
  overrides: OxContentOptions,
): OxContentOptions {
  const mergedSsg =
    overrides.ssg === false
      ? false
      : {
          ...(typeof baseOptions.ssg === "object" ? baseOptions.ssg : {}),
          ...(typeof overrides.ssg === "object" ? overrides.ssg : {}),
          theme:
            typeof baseOptions.ssg === "object" &&
            typeof overrides.ssg === "object" &&
            baseOptions.ssg.theme &&
            overrides.ssg.theme
              ? defineTheme(mergeThemes(baseOptions.ssg.theme, overrides.ssg.theme))
              : typeof overrides.ssg === "object" && overrides.ssg.theme
                ? overrides.ssg.theme
                : typeof baseOptions.ssg === "object"
                  ? baseOptions.ssg.theme
                  : undefined,
        };

  const mergedSearch =
    overrides.search === false
      ? false
      : typeof overrides.search === "object"
        ? {
            ...(typeof baseOptions.search === "object" ? baseOptions.search : {}),
            ...overrides.search,
          }
        : baseOptions.search;

  return {
    ...baseOptions,
    ...overrides,
    ssg: mergedSsg,
    search: mergedSearch,
  };
}

/**
 * Converts a VitePress sidebar config into ox-content navigation groups.
 * Nested VitePress items are flattened into the nearest ox-content group.
 */
export function convertVitePressSidebar(sidebar: VitePressSidebar): SsgNavigationGroup[] {
  if (Array.isArray(sidebar)) {
    return dedupeNavigationGroups(sidebarArrayToGroups(sidebar, "Guide"));
  }

  const groups = Object.entries(sidebar).flatMap(([key, items]) =>
    sidebarArrayToGroups(items, titleFromSidebarKey(key)),
  );

  return dedupeNavigationGroups(groups);
}

/**
 * Converts VitePress top navigation into ox-content sidebar groups.
 * This is used as a fallback when no explicit sidebar is defined.
 */
export function convertVitePressNav(nav: VitePressNavItem[]): SsgNavigationGroup[] {
  const groups: SsgNavigationGroup[] = [];
  const rootItems: SsgNavigationItem[] = [];

  for (const item of nav) {
    if (item.link) {
      rootItems.push(toNavigationItem(item.text, item.link));
    }

    if (item.items?.length) {
      const children = collectNavLinks(item.items);
      if (children.length > 0) {
        groups.push({
          title: item.text?.trim() || "Navigation",
          items: children,
        });
      }
    }
  }

  if (rootItems.length > 0) {
    groups.unshift({
      title: "Navigation",
      items: dedupeNavigationItems(rootItems),
    });
  }

  return dedupeNavigationGroups(groups);
}

/**
 * Creates ox-content plugin options from an existing VitePress config.
 */
export function fromVitePressConfig(
  config: VitePressConfig,
  overrides: OxContentOptions = {},
): OxContentOptions {
  const theme = toThemeConfig(config.themeConfig);
  const navigation = config.themeConfig?.sidebar
    ? convertVitePressSidebar(config.themeConfig.sidebar)
    : config.themeConfig?.nav
      ? convertVitePressNav(config.themeConfig.nav)
      : undefined;

  const migrated: OxContentOptions = {
    ...(config.base ? { base: config.base } : {}),
    ...(config.themeConfig?.search?.placeholder
      ? {
          search: {
            placeholder: config.themeConfig.search.placeholder,
          },
        }
      : {}),
    ssg: {
      ...(resolveSiteName(config) ? { siteName: resolveSiteName(config) } : {}),
      ...(theme ? { theme } : {}),
      ...(navigation ? { navigation } : {}),
    },
  };

  return mergeOxContentOptions(migrated, overrides);
}

/**
 * Generates a TypeScript module exporting migrated ox-content options.
 *
 * This is used by the migration CLI so users can inspect and edit the resulting
 * object instead of keeping a runtime dependency on their VitePress config.
 */
export function generateVitePressMigrationConfig(
  config: VitePressConfig,
  overrides: OxContentOptions = {},
  options: GenerateVitePressMigrationConfigOptions = {},
): string {
  const importSource = options.importSource ?? "@ox-content/vite-plugin";
  const migrated = fromVitePressConfig(config, overrides);

  return `import type { OxContentOptions } from ${JSON.stringify(importSource)};

const config = ${formatTsValue(migrated)} satisfies OxContentOptions;

export default config;
`;
}

function formatTsValue(value: unknown, depth = 0): string {
  if (value === undefined) {
    return "undefined";
  }

  if (value === null || typeof value === "boolean" || typeof value === "number") {
    return JSON.stringify(value);
  }

  if (typeof value === "string") {
    return JSON.stringify(value);
  }

  if (Array.isArray(value)) {
    if (value.length === 0) {
      return "[]";
    }

    const indent = "  ".repeat(depth + 1);
    const closingIndent = "  ".repeat(depth);
    return `[\n${value.map((item) => `${indent}${formatTsValue(item, depth + 1)},`).join("\n")}\n${closingIndent}]`;
  }

  if (isRecord(value)) {
    const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined);
    if (entries.length === 0) {
      return "{}";
    }

    const indent = "  ".repeat(depth + 1);
    const closingIndent = "  ".repeat(depth);
    return `{\n${entries
      .map(
        ([key, entryValue]) =>
          `${indent}${formatObjectKey(key)}: ${formatTsValue(entryValue, depth + 1)},`,
      )
      .join("\n")}\n${closingIndent}}`;
  }

  return "undefined";
}

function formatObjectKey(key: string): string {
  return /^[A-Za-z_$][\w$]*$/.test(key) ? key : JSON.stringify(key);
}

/**
 * Normalizes VitePress-specific frontmatter into ox-content's entry-page shape.
 */
export function normalizeVitePressFrontmatter(
  frontmatter: Record<string, unknown>,
): Record<string, unknown> {
  return importNapiModuleSync().normalizeVitePressFrontmatter(frontmatter);
}
