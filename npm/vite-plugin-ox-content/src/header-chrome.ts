/**
 * Opt-in header nav, announcement, and per-page chrome helpers.
 */

/** Header nav link or dropdown. */
export interface HeaderNavItem {
  text: string;
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
