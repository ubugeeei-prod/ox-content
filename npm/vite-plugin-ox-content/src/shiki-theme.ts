import { createCssVariablesTheme } from "shiki";
import type { ThemeRegistration } from "shiki";

/**
 * Name callers pass as `highlightTheme` to render syntax colors as CSS custom
 * properties instead of baked-in hex values.
 */
export const CSS_VARIABLES_THEME = "css-variables";

/** Prefix for the emitted properties, matching the rest of the design tokens. */
const VARIABLE_PREFIX = "--octc-shiki-";

/**
 * Fallbacks baked into each `var()` so a site with no color scheme installed
 * still renders GitHub Dark colors — the previous default — rather than
 * unstyled text. A `@ox-content/theme-color-*` package overrides them by
 * defining the same properties per mode.
 */
const VARIABLE_DEFAULTS: Record<string, string> = {
  foreground: "#e6edf3",
  background: "#0d1117",
  "token-constant": "#79c0ff",
  "token-string": "#a5d6ff",
  "token-comment": "#8b949e",
  "token-keyword": "#ff7b72",
  "token-parameter": "#ffa657",
  "token-function": "#d2a8ff",
  "token-string-expression": "#a5d6ff",
  "token-punctuation": "#c9d1d9",
  "token-link": "#a5d6ff",
};

let cached: ThemeRegistration | undefined;

/**
 * Shiki theme whose every color is a `--octc-shiki-*` custom property.
 *
 * This is what lets syntax highlighting track the active color scheme in both
 * light and dark from a single build: the HTML is generated once, and the
 * properties resolve per mode. A fixed theme like `github-dark` cannot do that,
 * and lands dark token colors on a light code block.
 */
export function cssVariablesTheme(): ThemeRegistration {
  cached ??= createCssVariablesTheme({
    name: CSS_VARIABLES_THEME,
    variablePrefix: VARIABLE_PREFIX,
    variableDefaults: VARIABLE_DEFAULTS,
    fontStyle: true,
  }) as ThemeRegistration;
  return cached;
}

/** Resolves the `css-variables` alias; any other value passes through. */
export function resolveHighlightTheme(
  theme: string | ThemeRegistration,
): string | ThemeRegistration {
  return theme === CSS_VARIABLES_THEME ? cssVariablesTheme() : theme;
}
