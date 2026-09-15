export interface ThemeBootstrapDocumentColor {
  /** Canvas background applied before external CSS has loaded. */
  background?: string;
  /**
   * Browser UI colour written to the shared `theme-color` meta tag.
   *
   * @default background
   */
  themeColor?: string;
  /**
   * Root `color-scheme` for form controls and browser chrome.
   *
   * @default "light" or "dark"
   */
  colorScheme?: string;
}

type ThemeBootstrapPreference = "light" | "dark" | "system";
type ThemeBootstrapResolvedTheme = "light" | "dark";

export interface ThemeBootstrapDocumentColors {
  light: ThemeBootstrapDocumentColor;
  dark: ThemeBootstrapDocumentColor;
  /**
   * Single browser-facing colour meta updated by the bootstrap. Set to `false`
   * to leave metadata to the host.
   *
   * @default `meta[name="theme-color"]`
   */
  themeColorMetaSelector?: string | false;
}

export interface ResolvedThemeBootstrapDocumentColor {
  background: string | false;
  themeColor: string | false;
  colorScheme: string;
}

export interface ResolvedThemeBootstrapDocumentColors {
  light: ResolvedThemeBootstrapDocumentColor;
  dark: ResolvedThemeBootstrapDocumentColor;
  themeColorMetaSelector: string | false;
}

export interface ResolvedThemeBootstrapDocumentColorOptions {
  defaultPreference: ThemeBootstrapPreference;
  documentColors: ResolvedThemeBootstrapDocumentColors | false;
}

interface RenderDocumentColorOptions {
  id?: string;
  nonce?: string;
}

const DEFAULT_THEME_COLOR_META_SELECTOR = 'meta[name="theme-color"]';

export function resolveThemeBootstrapDocumentColors(
  colors: ThemeBootstrapDocumentColors | undefined,
): ResolvedThemeBootstrapDocumentColors | false {
  if (!colors) {
    return false;
  }
  return {
    light: resolveThemeBootstrapDocumentColor(colors.light, "light"),
    dark: resolveThemeBootstrapDocumentColor(colors.dark, "dark"),
    themeColorMetaSelector:
      colors.themeColorMetaSelector === false
        ? false
        : (cleanString(colors.themeColorMetaSelector) ?? DEFAULT_THEME_COLOR_META_SELECTOR),
  };
}

export function applyThemeBootstrapDocumentColors(
  theme: ThemeBootstrapResolvedTheme,
  colors: ResolvedThemeBootstrapDocumentColors,
): void {
  const color = colors[theme];
  const root = globalThis.document?.documentElement as HTMLElement | undefined;
  if (root?.style) {
    if (color.background) {
      root.style.backgroundColor = color.background;
    }
    root.style.colorScheme = color.colorScheme;
  }
  if (!colors.themeColorMetaSelector || !color.themeColor) {
    return;
  }
  resolveThemeColorMeta(colors.themeColorMetaSelector)?.setAttribute("content", color.themeColor);
}

export function createThemeBootstrapDocumentStyleFromResolved(
  resolved: ResolvedThemeBootstrapDocumentColorOptions,
): string {
  const colors = resolved.documentColors;
  if (!colors) {
    return "";
  }
  const initialTheme = resolved.defaultPreference === "dark" ? "dark" : "light";
  const rules = [
    styleRule("html", colors[initialTheme]),
    styleRule('html[data-theme="light"]', colors.light),
    styleRule('html[data-theme="dark"]', colors.dark),
  ];
  if (resolved.defaultPreference === "system") {
    rules.push(
      `@media (prefers-color-scheme: dark){${styleRule('html:not([data-theme="light"])', colors.dark)}}`,
    );
  }
  return rules.filter(Boolean).join("");
}

export function renderThemeBootstrapDocumentColorsFromResolved(
  resolved: ResolvedThemeBootstrapDocumentColorOptions,
  renderOptions: RenderDocumentColorOptions = {},
): string {
  const colors = resolved.documentColors;
  if (!colors) {
    return "";
  }
  const initialTheme = resolved.defaultPreference === "dark" ? "dark" : "light";
  const initialColor = colors[initialTheme];
  const tags: string[] = [];
  if (colors.themeColorMetaSelector && initialColor.themeColor) {
    tags.push(`<meta name="theme-color" content="${escapeAttr(initialColor.themeColor)}">`);
  }
  const style = createThemeBootstrapDocumentStyleFromResolved(resolved);
  if (style) {
    tags.push(`<style${renderAttrs(renderOptions)}>${style}</style>`);
  }
  return tags.join("");
}

function resolveThemeBootstrapDocumentColor(
  color: ThemeBootstrapDocumentColor | undefined,
  theme: ThemeBootstrapResolvedTheme,
): ResolvedThemeBootstrapDocumentColor {
  const background = cleanString(color?.background) ?? false;
  return {
    background,
    themeColor: cleanString(color?.themeColor) ?? background,
    colorScheme: cleanString(color?.colorScheme) ?? theme,
  };
}

function resolveThemeColorMeta(selector: string): Element | null {
  const doc = globalThis.document;
  if (!doc) {
    return null;
  }
  try {
    const existing = doc.querySelector(selector);
    if (existing) {
      return existing;
    }
  } catch {
    return null;
  }
  if (selector !== DEFAULT_THEME_COLOR_META_SELECTOR || !doc.createElement) {
    return null;
  }
  const meta = doc.createElement("meta");
  meta.setAttribute("name", "theme-color");
  doc.head?.appendChild(meta);
  return meta;
}

function styleRule(selector: string, color: ResolvedThemeBootstrapDocumentColor): string {
  const declarations = [
    cssDeclaration("color-scheme", color.colorScheme),
    color.background ? cssDeclaration("background-color", color.background) : "",
  ].join("");
  return declarations ? `${selector}{${declarations}}` : "";
}

function cssDeclaration(property: string, value: string): string {
  return /[;{}<>]/.test(value) ? "" : `${property}:${value};`;
}

function cleanString(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function renderAttrs(options: RenderDocumentColorOptions): string {
  const attrs: string[] = [];
  if (options.id) {
    attrs.push(`id="${escapeAttr(options.id)}"`);
  }
  if (options.nonce) {
    attrs.push(`nonce="${escapeAttr(options.nonce)}"`);
  }
  return attrs.length > 0 ? ` ${attrs.join(" ")}` : "";
}

function escapeAttr(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}
