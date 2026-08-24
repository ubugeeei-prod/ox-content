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

/** Picks `locale`, then the language prefix, then the first map value. */
export function resolveLocaleLabel(text: LocaleLabel, locale?: string): string {
  if (typeof text === "string") {
    return text;
  }
  if (locale && text[locale]) {
    return text[locale];
  }
  const lang = locale?.split("-")[0];
  if (lang && text[lang]) {
    return text[lang];
  }
  return Object.values(text)[0] ?? "";
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
): ResolvedHeaderNavItem[] | undefined {
  if (!items?.length) {
    return undefined;
  }
  return items.map((item) => ({
    text: resolveLocaleLabel(item.text, locale),
    link: item.link,
    items: resolveHeaderNavItems(item.items, locale),
  }));
}
