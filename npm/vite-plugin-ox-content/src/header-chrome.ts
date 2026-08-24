/**
 * Opt-in header nav, announcement, and per-page chrome helpers.
 */

/** Plain label or locale map (`{ en: "Guide", ja: "ガイド" }`). */
export type LocaleLabel = string | Record<string, string>;

/** Header nav link or dropdown. */
export interface HeaderNavItem {
  text: LocaleLabel;
  link?: string;
  items?: HeaderNavItem[];
}

/** Announcement bar. Text is escaped; no raw HTML slot. */
export interface ThemeAnnouncement {
  text: string;
  /** https or same-origin only. */
  link?: string;
  /** Best-effort localStorage key for dismiss. */
  dismissKey?: string;
}

/** Per-page frontmatter chrome flags. `false` hides that region. */
export interface PageChromeFlags {
  sidebar?: boolean;
  outline?: boolean;
  aside?: boolean;
  footer?: boolean;
  navbar?: boolean;
  lastUpdated?: boolean;
  editLink?: boolean;
}

/** `false` or omitted stays off. `true` or `{}` enables default flag reading. */
export function resolvePageChromeOption(
  value: boolean | Record<string, unknown> | undefined,
): boolean {
  return value === true || (typeof value === "object" && value !== null);
}

/** Reads hide flags from frontmatter. Non-boolean values are ignored. */
export function parsePageChromeFlags(frontmatter: Record<string, unknown>): PageChromeFlags {
  return {
    sidebar: readBool(frontmatter.sidebar),
    outline: readBool(frontmatter.outline),
    aside: readBool(frontmatter.aside),
    footer: readBool(frontmatter.footer),
    navbar: readBool(frontmatter.navbar),
    lastUpdated: readBool(frontmatter.lastUpdated),
    editLink: readBool(frontmatter.editLink),
  };
}

function readBool(value: unknown): boolean | undefined {
  return typeof value === "boolean" ? value : undefined;
}

/**
 * Picks the exact locale, its language, the default locale, then the first
 * non-empty own string in declaration order.
 */
export function resolveLocaleLabel(
  text: LocaleLabel,
  locale?: string,
  defaultLocale?: string,
): string {
  if (typeof text === "string") {
    return text;
  }
  const candidates = [locale, locale?.split("-")[0], defaultLocale, defaultLocale?.split("-")[0]];
  for (const candidate of candidates) {
    if (!candidate || !Object.hasOwn(text, candidate)) {
      continue;
    }
    const value = text[candidate];
    if (typeof value === "string" && value.length > 0) {
      return value;
    }
  }
  for (const value of Object.values(text)) {
    if (typeof value === "string" && value.length > 0) {
      return value;
    }
  }
  return "";
}

/** Nav item after locale maps are flattened to strings. */
export interface ResolvedHeaderNavItem {
  text: string;
  link?: string;
  items?: ResolvedHeaderNavItem[];
}

/** Resolves locale maps so NAPI always receives string labels. */
export function resolveHeaderNavItems(
  items: HeaderNavItem[] | undefined,
  locale?: string,
  defaultLocale?: string,
): ResolvedHeaderNavItem[] | undefined {
  if (!items?.length) {
    return undefined;
  }
  return items.map((item) => ({
    text: resolveLocaleLabel(item.text, locale, defaultLocale),
    link: item.link,
    items: resolveHeaderNavItems(item.items, locale, defaultLocale),
  }));
}
