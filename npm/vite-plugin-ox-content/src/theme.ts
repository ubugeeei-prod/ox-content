/**
 * Theme API for ox-content SSG
 *
 * Provides VitePress-like theming with default theme + customization.
 */

import type { HeaderNavItem, ResolvedHeaderNavItem, ThemeAnnouncement } from "./header-chrome";
import { resolveHeaderNavItems } from "./header-chrome";
import { tokensToCss, type ThemeTokens } from "./theme-tokens";

export type { HeaderNavItem, ThemeAnnouncement } from "./header-chrome";

export type { ThemeTokens } from "./theme-tokens";

/**
 * Theme color configuration.
 */
export interface ThemeColors {
  /** Primary accent color */
  primary?: string;
  /** Primary color on hover */
  primaryHover?: string;
  /** Background color */
  background?: string;
  /** Alternative background color (sidebar, code blocks) */
  backgroundAlt?: string;
  /** Main text color */
  text?: string;
  /** Muted/secondary text color */
  textMuted?: string;
  /** Border color */
  border?: string;
  /** Code block background color */
  codeBackground?: string;
  /** Code block gradient color at the top; defaults to `codeBackground` when customized */
  codeBackgroundTop?: string;
  /** Code block text color */
  codeText?: string;
}

/**
 * Theme layout configuration.
 */
export interface ThemeLayout {
  /** Sidebar width (CSS value, e.g., "260px") */
  sidebarWidth?: string;
  /** Header height (CSS value, e.g., "60px") */
  headerHeight?: string;
  /** Maximum content width (CSS value, e.g., "960px") */
  maxContentWidth?: string;
}

/**
 * Theme font configuration.
 */
export interface ThemeFonts {
  /** Sans-serif font stack */
  sans?: string;
  /** Monospace font stack */
  mono?: string;
}

/**
 * Entry page theme configuration.
 */
export interface ThemeEntryPage {
  /** Landing page presentation mode */
  mode?: "default" | "subtle";
}

/**
 * Theme header configuration.
 */
export interface ThemeHeader {
  /** Logo image URL */
  logo?: string;
  /** Light mode logo image URL */
  logoLight?: string;
  /** Dark mode logo image URL */
  logoDark?: string;
  /** Whether to render the site name text next to the logo */
  showSiteNameText?: boolean;
  /** Logo width in pixels */
  logoWidth?: number;
  /** Logo height in pixels */
  logoHeight?: number;
}

/**
 * Theme footer configuration.
 */
export interface ThemeFooter {
  /** Footer message (supports HTML) */
  message?: string;
  /** Copyright text (supports HTML) */
  copyright?: string;
}

/** Custom social link icon. */
export type SocialLinkIcon = string | { svg: string };

/** Custom social link. */
export interface SocialLink {
  icon: SocialLinkIcon;
  link: string;
  ariaLabel?: string;
}

/** Legacy social links configuration. */
export interface LegacySocialLinks {
  /** GitHub URL */
  github?: string;
  /** Twitter/X URL */
  twitter?: string;
  /** Discord URL */
  discord?: string;
}

/** Social links configuration. */
export type SocialLinks = LegacySocialLinks | SocialLink[];

/**
 * Embedded HTML content for specific positions in the page layout.
 */
export interface ThemeEmbed {
  /** Content to embed into <head> */
  head?: string;
  /** Content before header */
  headerBefore?: string;
  /** Content after header */
  headerAfter?: string;
  /** Content before sidebar navigation */
  sidebarBefore?: string;
  /** Content after sidebar navigation */
  sidebarAfter?: string;
  /** Content before main content */
  contentBefore?: string;
  /** Content after main content */
  contentAfter?: string;
  /** Content before footer */
  footerBefore?: string;
  /** Custom footer content (replaces default footer) */
  footer?: string;
}

export interface SidebarItem {
  text?: string;
  link?: string;
  items?: SidebarItem[];
  collapsed?: boolean;
  stickyCollapsed?: boolean;
}

/**
 * Complete theme configuration.
 */
export interface ThemeConfig {
  /** Theme name for identification */
  name?: string;
  /** Base theme to extend */
  extends?: ThemeConfig;
  /**
   * Preserve the current surface during same-origin MPA navigation with the
   * browser's cross-document View Transition API.
   *
   * Unsupported browsers use normal navigation. Reduced-motion preferences
   * never enable the transition. Set `false` to opt out.
   *
   * @default true
   */
  viewTransitions?: boolean;
  /**
   * Show the right-hand "On this page" outline.
   *
   * Default `false`. When `true`, the outline is rendered only on pages
   * that have TOC entries, using the existing `<aside class="toc">` markup.
   */
  aside?: boolean;
  /**
   * Show a breadcrumb trail from the site root through sidebar ancestors.
   *
   * Default `false`. `true` or an object enables the trail. Frontmatter
   * `breadcrumbs: false` still hides it on that page.
   */
  breadcrumbs?: boolean | Record<string, unknown>;
  /** Light mode colors (maps to CSS variables) */
  colors?: ThemeColors;
  /** Dark mode colors (maps to CSS variables) */
  darkColors?: ThemeColors;
  /** Font configuration (maps to CSS variables) */
  fonts?: ThemeFonts;
  /** Entry page configuration */
  entryPage?: ThemeEntryPage;
  /** Layout configuration (maps to CSS variables) */
  layout?: ThemeLayout;
  /** Header configuration */
  header?: ThemeHeader;
  /**
   * Opt-in header nav. Each item is `{ text, link }` or a dropdown
   * `{ text, items }`. Labels are escaped. `javascript:`, `data:`,
   * `vbscript:`, and protocol-relative `//` links are omitted.
   */
  nav?: HeaderNavItem[];
  /**
   * Opt-in announcement bar above the header. Text is escaped.
   * Optional `link` must be https or same-origin.
   */
  announcement?: ThemeAnnouncement;
  /** Footer configuration */
  footer?: ThemeFooter;
  /** Social links configuration */
  socialLinks?: SocialLinks;
  sidebar?: SidebarItem[];
  /** Embedded HTML content at specific positions */
  embed?: ThemeEmbed;
  /**
   * Extra `--octc-*` custom properties for light mode, keyed without the
   * prefix. Merged key-by-key across composed layers, so a later layer can
   * restyle one token without redeclaring the rest.
   */
  tokens?: ThemeTokens;
  /** Extra `--octc-*` custom properties for dark mode. */
  darkTokens?: ThemeTokens;
  /**
   * Additional custom CSS. Composed layers **concatenate** this rather than
   * overwrite, so stacking a skin and a color scheme keeps both stylesheets.
   */
  css?: string;
  /** Additional custom JavaScript. Concatenated across composed layers. */
  js?: string;
}

/**
 * Resolved theme configuration (after merging with defaults).
 */
export interface ResolvedThemeConfig {
  name: string;
  viewTransitions: boolean;
  aside: boolean;
  breadcrumbs: boolean;
  colors: ThemeColors;
  darkColors: ThemeColors;
  fonts: ThemeFonts;
  entryPage: ThemeEntryPage;
  layout: ThemeLayout;
  header: ThemeHeader;
  nav?: HeaderNavItem[];
  announcement?: ThemeAnnouncement;
  footer: ThemeFooter;
  socialLinks: SocialLinks;
  sidebar: SidebarItem[];
  embed: ThemeEmbed;
  tokens: ThemeTokens;
  darkTokens: ThemeTokens;
  css: string;
  js: string;
}

/**
 * Default theme configuration.
 * Based on the current ox-content SSG styles.
 */
export const defaultTheme: ThemeConfig = {
  name: "default",
  viewTransitions: true,
  aside: false,
  breadcrumbs: false,
  colors: {
    primary: "#4f6fae",
    primaryHover: "#425f96",
    background: "#ffffff",
    backgroundAlt: "#f5f7fb",
    text: "#131a30",
    textMuted: "#4f607b",
    border: "#d2dbea",
    codeBackground: "#101a31",
    codeBackgroundTop: "#18264a",
    codeText: "#edf3ff",
  },
  darkColors: {
    primary: "#86a4da",
    primaryHover: "#a3bbe8",
    background: "#060816",
    backgroundAlt: "#0d1528",
    text: "#ebf2ff",
    textMuted: "#8ea0bf",
    border: "#223252",
    codeBackground: "#0a1020",
    codeBackgroundTop: "#0a1020",
    codeText: "#e7f0ff",
  },
  fonts: {
    sans: '"IBM Plex Sans", "Avenir Next", "Segoe UI Variable", "Segoe UI", sans-serif',
    mono: '"IBM Plex Mono", "SFMono-Regular", Consolas, monospace',
  },
  entryPage: {
    mode: "default",
  },
  layout: {
    sidebarWidth: "260px",
    headerHeight: "60px",
    maxContentWidth: "960px",
  },
  header: {
    logo: undefined,
    logoLight: undefined,
    logoDark: undefined,
    showSiteNameText: true,
    logoWidth: 28,
    logoHeight: 28,
  },
  footer: {
    message: undefined,
    copyright: undefined,
  },
  socialLinks: {},
  embed: {},
  tokens: {},
  darkTokens: {},
  css: "",
  js: "",
};

/**
 * Deep merge two objects.
 */
function deepMerge<T extends Record<string, unknown>>(target: T, source: Partial<T>): T {
  const result = { ...target };

  for (const key of Object.keys(source) as (keyof T)[]) {
    const sourceValue = source[key];
    const targetValue = target[key];

    if (
      sourceValue !== undefined &&
      typeof sourceValue === "object" &&
      sourceValue !== null &&
      !Array.isArray(sourceValue) &&
      typeof targetValue === "object" &&
      targetValue !== null &&
      !Array.isArray(targetValue)
    ) {
      result[key] = deepMerge(
        targetValue as Record<string, unknown>,
        sourceValue as Record<string, unknown>,
      ) as T[keyof T];
    } else if (sourceValue !== undefined) {
      result[key] = sourceValue as T[keyof T];
    }
  }

  return result;
}

/**
 * Defines a theme configuration with type checking.
 *
 * @example
 * ```ts
 * const myTheme = defineTheme({
 *   extends: defaultTheme,
 *   colors: {
 *     primary: '#3498db',
 *   },
 *   footer: {
 *     copyright: '2025 My Company',
 *   },
 * });
 * ```
 */
export function defineTheme(config: ThemeConfig): ThemeConfig {
  return config;
}

/**
 * Merges multiple theme configurations.
 * Later themes override earlier ones.
 *
 * Object fields (`colors`, `tokens`, `layout`, …) merge key-by-key, but `css`
 * and `js` **concatenate** in layer order — overwriting them would throw away
 * one half of a `[skin, colorScheme]` stack. Identical fragments are joined
 * once, so a layer reached through both an array and an `extends` chain does
 * not emit its stylesheet twice.
 *
 * @example
 * ```ts
 * const merged = mergeThemes(defaultTheme, pixelSkin, tokyoNight, overrides);
 * ```
 */
export function mergeThemes(...themes: (ThemeConfig | ThemeConfig[])[]): ThemeConfig {
  const layers = themes.flat();
  if (layers.length === 0) {
    return { ...defaultTheme };
  }

  let result: ThemeConfig = {};

  for (const theme of layers) {
    const { css, js, ...rest } = theme;
    result = deepMerge(
      result as Record<string, unknown>,
      rest as Record<string, unknown>,
    ) as ThemeConfig;

    const mergedCss = appendSource(result.css, css);
    if (mergedCss) {
      result.css = mergedCss;
    }
    const mergedJs = appendSource(result.js, js);
    if (mergedJs) {
      result.js = mergedJs;
    }
  }

  return result;
}

function appendSource(existing: string | undefined, addition: string | undefined): string {
  const next = addition?.trim() ?? "";
  const current = existing ?? "";
  if (!next || current.includes(next)) {
    return current;
  }
  return current ? `${current}\n${next}` : next;
}

/**
 * Resolves a theme configuration by merging with its extends chain and defaults.
 *
 * An array composes independent layers left to right, which is how a skin
 * package and a color package are stacked:
 *
 * ```ts
 * resolveTheme([pixelSkin, tokyoNight, { footer: { copyright: "2026" } }]);
 * ```
 */
export function resolveTheme(config?: ThemeConfig | ThemeConfig[]): ResolvedThemeConfig {
  const layers = config === undefined ? [defaultTheme] : Array.isArray(config) ? config : [config];
  const chain = layers.flatMap(expandExtendsChain);

  // Always start with default theme
  if (chain.length === 0) {
    chain.push(defaultTheme);
  }
  if (chain[0] !== defaultTheme && chain[0]?.name !== "default") {
    chain.unshift(defaultTheme);
  }

  // Merge all themes in the chain
  const merged = mergeThemes(...chain.map(withDerivedCodeBackgroundTop));

  // Return resolved config with all required fields
  return {
    name: merged.name ?? "custom",
    viewTransitions: merged.viewTransitions ?? defaultTheme.viewTransitions ?? true,
    aside: merged.aside ?? defaultTheme.aside ?? false,
    breadcrumbs: resolveThemeFlag(merged.breadcrumbs),
    colors: merged.colors ?? defaultTheme.colors!,
    darkColors: merged.darkColors ?? defaultTheme.darkColors!,
    fonts: merged.fonts ?? defaultTheme.fonts!,
    entryPage: merged.entryPage ?? defaultTheme.entryPage!,
    layout: merged.layout ?? defaultTheme.layout!,
    header: merged.header ?? defaultTheme.header!,
    nav: merged.nav,
    announcement: merged.announcement,
    footer: merged.footer ?? defaultTheme.footer!,
    socialLinks: merged.socialLinks ?? defaultTheme.socialLinks!,
    sidebar: merged.sidebar ?? [],
    embed: merged.embed ?? {},
    tokens: merged.tokens ?? {},
    darkTokens: merged.darkTokens ?? {},
    css: merged.css ?? "",
    js: merged.js ?? "",
  };
}

/**
 * Flattens one layer's `extends` chain into base-first order.
 *
 * The `seen` guard keeps a theme that accidentally extends itself (or forms a
 * cycle through two packages) from hanging the build.
 */
function expandExtendsChain(config: ThemeConfig): ThemeConfig[] {
  const chain: ThemeConfig[] = [];
  const seen = new Set<ThemeConfig>();
  let current: ThemeConfig | undefined = config;

  while (current && !seen.has(current)) {
    seen.add(current);
    chain.unshift(current);
    current = current.extends;
  }

  return chain;
}

function withDerivedCodeBackgroundTop(theme: ThemeConfig): ThemeConfig {
  const derive = (colors: ThemeColors | undefined): ThemeColors | undefined => {
    if (colors?.codeBackground !== undefined && colors.codeBackgroundTop === undefined) {
      return { ...colors, codeBackgroundTop: colors.codeBackground };
    }
    return colors;
  };

  return {
    ...theme,
    colors: derive(theme.colors),
    darkColors: derive(theme.darkColors),
  };
}

/**
 * Converts resolved theme to the format expected by Rust NAPI.
 */
export function themeToNapi(theme: ResolvedThemeConfig, locale?: string): NapiThemeConfig {
  const socialLinks = socialLinksToNapi(theme.socialLinks);

  return {
    viewTransitions: theme.viewTransitions,
    aside: theme.aside,
    breadcrumbs: theme.breadcrumbs,
    colors: theme.colors.primary
      ? {
          primary: theme.colors.primary,
          primaryHover: theme.colors.primaryHover,
          background: theme.colors.background,
          backgroundAlt: theme.colors.backgroundAlt,
          text: theme.colors.text,
          textMuted: theme.colors.textMuted,
          border: theme.colors.border,
          codeBackground: theme.colors.codeBackground,
          codeBackgroundTop: theme.colors.codeBackgroundTop,
          codeText: theme.colors.codeText,
        }
      : undefined,
    darkColors: theme.darkColors.primary
      ? {
          primary: theme.darkColors.primary,
          primaryHover: theme.darkColors.primaryHover,
          background: theme.darkColors.background,
          backgroundAlt: theme.darkColors.backgroundAlt,
          text: theme.darkColors.text,
          textMuted: theme.darkColors.textMuted,
          border: theme.darkColors.border,
          codeBackground: theme.darkColors.codeBackground,
          codeBackgroundTop: theme.darkColors.codeBackgroundTop,
          codeText: theme.darkColors.codeText,
        }
      : undefined,
    fonts: theme.fonts.sans
      ? {
          sans: theme.fonts.sans,
          mono: theme.fonts.mono,
        }
      : undefined,
    entryPage: theme.entryPage.mode
      ? {
          mode: theme.entryPage.mode,
        }
      : undefined,
    layout: theme.layout.sidebarWidth
      ? {
          sidebarWidth: theme.layout.sidebarWidth,
          headerHeight: theme.layout.headerHeight,
          maxContentWidth: theme.layout.maxContentWidth,
        }
      : undefined,
    header:
      theme.header.logo || theme.header.logoLight || theme.header.logoDark
        ? {
            logo: theme.header.logo,
            logoLight: theme.header.logoLight,
            logoDark: theme.header.logoDark,
            showSiteNameText: theme.header.showSiteNameText,
            logoWidth: theme.header.logoWidth,
            logoHeight: theme.header.logoHeight,
          }
        : undefined,
    nav: resolveHeaderNavItems(theme.nav, locale),
    announcement: theme.announcement?.text ? theme.announcement : undefined,
    footer:
      theme.footer.message || theme.footer.copyright
        ? {
            message: theme.footer.message,
            copyright: theme.footer.copyright,
          }
        : undefined,
    socialLinks,
    embed: Object.keys(theme.embed).length > 0 ? theme.embed : undefined,
    css: themeCss(theme) || undefined,
    js: theme.js || undefined,
  };
}

/**
 * Token blocks come first so a theme's own `css` stays the final word, and both
 * land after the typed color variables the Rust renderer emits.
 */
function themeCss(theme: ResolvedThemeConfig): string {
  const tokenCss = tokensToCss(theme.tokens, theme.darkTokens);
  if (!tokenCss) {
    return theme.css;
  }
  return theme.css ? `${tokenCss}\n${theme.css}` : tokenCss;
}

function socialLinksToNapi(links: SocialLinks): NapiSocialLinks | undefined {
  if (Array.isArray(links)) {
    const items = links.map((item) => {
      const icon = typeof item.icon === "string" ? item.icon : undefined;
      const iconSvg = typeof item.icon === "object" ? item.icon.svg : undefined;
      return { icon, iconSvg, link: item.link, ariaLabel: item.ariaLabel };
    });
    return items.length > 0 ? { links: items } : undefined;
  }

  return links.github || links.twitter || links.discord
    ? { github: links.github, twitter: links.twitter, discord: links.discord }
    : undefined;
}

/**
 * NAPI-compatible theme colors type.
 */
export interface NapiThemeColors {
  primary?: string;
  primaryHover?: string;
  background?: string;
  backgroundAlt?: string;
  text?: string;
  textMuted?: string;
  border?: string;
  codeBackground?: string;
  codeBackgroundTop?: string;
  codeText?: string;
}

/**
 * NAPI-compatible theme fonts type.
 */
export interface NapiThemeFonts {
  sans?: string;
  mono?: string;
}

/**
 * NAPI-compatible entry page theme type.
 */
export interface NapiThemeEntryPage {
  mode?: "default" | "subtle";
}

/**
 * NAPI-compatible theme layout type.
 */
export interface NapiThemeLayout {
  sidebarWidth?: string;
  headerHeight?: string;
  maxContentWidth?: string;
}

/**
 * NAPI-compatible theme header type.
 */
export interface NapiThemeHeader {
  logo?: string;
  logoLight?: string;
  logoDark?: string;
  showSiteNameText?: boolean;
  logoWidth?: number;
  logoHeight?: number;
}

/**
 * NAPI-compatible theme footer type.
 */
export interface NapiThemeFooter {
  message?: string;
  copyright?: string;
}

/**
 * NAPI-compatible social links type.
 */
export interface NapiSocialLinks {
  github?: string;
  twitter?: string;
  discord?: string;
  links?: NapiSocialLink[];
}

export interface NapiSocialLink {
  icon?: string;
  iconSvg?: string;
  link: string;
  ariaLabel?: string;
}

/**
 * NAPI-compatible theme embed type.
 */
export interface NapiThemeEmbed {
  head?: string;
  headerBefore?: string;
  headerAfter?: string;
  sidebarBefore?: string;
  sidebarAfter?: string;
  contentBefore?: string;
  contentAfter?: string;
  footerBefore?: string;
  footer?: string;
}

function resolveThemeFlag(value: boolean | Record<string, unknown> | undefined): boolean {
  return value === true || (typeof value === "object" && value !== null);
}

/**
 * NAPI-compatible theme configuration type.
 */
export interface NapiThemeConfig {
  /** Progressive cross-document transitions for same-origin MPA navigation. */
  viewTransitions?: boolean;
  /** Right-hand "On this page" outline. */
  aside?: boolean;
  /** Breadcrumb trail from the site root through sidebar ancestors. */
  breadcrumbs?: boolean;
  nav?: ResolvedHeaderNavItem[];
  announcement?: ThemeAnnouncement;
  colors?: NapiThemeColors;
  darkColors?: NapiThemeColors;
  fonts?: NapiThemeFonts;
  entryPage?: NapiThemeEntryPage;
  layout?: NapiThemeLayout;
  header?: NapiThemeHeader;
  footer?: NapiThemeFooter;
  socialLinks?: NapiSocialLinks;
  embed?: NapiThemeEmbed;
  css?: string;
  js?: string;
}
