/**
 * Free-form `--octc-*` custom properties for themes that need more than the
 * typed `colors` / `fonts` / `layout` fields.
 *
 * Keys are written **without** the `--octc-` prefix, so `"surface-glass"`
 * becomes `--octc-surface-glass`. This is the seam that keeps the two theme
 * axes independent: a color package can restyle code-block line markers, brand
 * accents, and surface textures purely through tokens, while a skin package
 * lays out geometry against those same tokens without knowing any color.
 */
export type ThemeTokens = Record<string, string>;

const TOKEN_PREFIX = "--octc-";
const TOKEN_NAME_PATTERN = /^[a-z][a-z0-9-]*$/;

/**
 * The token-bearing shape of a theme.
 *
 * Declared structurally instead of importing `ThemeConfig` so this module keeps
 * an empty import graph: `@ox-content/vite-plugin/theme-tokens` has to be
 * loadable by a bare (`ssg.bare: true`) or custom host that never pulls in the
 * Vite plugin, the SSG, the native binding, or a filesystem API. Every
 * `ThemeConfig` — including the published `@ox-content/theme-color-*` and
 * `@ox-content/theme-*` packages — satisfies it.
 */
export interface ThemeTokenSource {
  tokens?: ThemeTokens;
  darkTokens?: ThemeTokens;
  extends?: ThemeTokenSource;
}

/**
 * Options for {@link renderThemeTokenCss}.
 */
export interface RenderThemeTokenCssOptions {
  /**
   * Keeps only the tokens whose name passes the predicate. Names arrive without
   * the `--octc-` prefix, so `(name) => name.startsWith("syntax-")` reuses a
   * color scheme's highlighter palette without adopting its page colors,
   * typography, or layout policy.
   *
   * Filtering runs per layer, before merging, so a token a later layer would
   * have overridden is dropped along with the override.
   */
  include?: (name: string) => boolean;
}

/**
 * Renders a theme's `--octc-*` tokens as a standalone stylesheet.
 *
 * The built-in SSG emits these declarations itself, but `ssg.bare: true` and
 * custom hosts render their own document — this is how they get the same
 * tokens. The built-in highlighter emits `var(--octc-syntax-*)` references, so
 * a bare host that wants only the highlighter palette can ask for it:
 *
 * ```ts
 * import { renderThemeTokenCss } from "@ox-content/vite-plugin/theme-tokens";
 * import { kanagawa } from "@ox-content/theme-color-kanagawa";
 *
 * const css = renderThemeTokenCss(kanagawa, {
 *   include: (name) => name.startsWith("syntax-"),
 * });
 * ```
 *
 * Layers compose left to right and each layer's `extends` chain is flattened
 * base-first, matching how `resolveTheme()` stacks a skin and a color scheme.
 */
export function renderThemeTokenCss(
  theme: ThemeTokenSource | ThemeTokenSource[],
  options: RenderThemeTokenCssOptions = {},
): string {
  const layers = (Array.isArray(theme) ? theme : [theme]).flatMap(expandExtendsChain);
  return tokensToCss(
    mergeTokens(layers, "tokens", options.include),
    mergeTokens(layers, "darkTokens", options.include),
  );
}

/**
 * Renders light and dark token records as the three selectors the SSG runtime
 * switches between: an explicit `[data-theme="dark"]` opt-in, the OS
 * `prefers-color-scheme` fallback, and the `:root` base.
 *
 * Emitted after the typed color variables and before the theme's own `css`, so
 * a token can override a typed color and raw `css` can override a token.
 */
export function tokensToCss(light: ThemeTokens, dark: ThemeTokens): string {
  const lightBody = declarations(light, "  ");
  const darkBody = declarations(dark, "  ");
  const blocks: string[] = [];

  if (lightBody) {
    blocks.push(`:root {\n${lightBody}\n}`);
  }
  if (darkBody) {
    blocks.push(`[data-theme="dark"] {\n${darkBody}\n}`);
    blocks.push(
      `@media (prefers-color-scheme: dark) {\n  :root:not([data-theme="light"]) {\n${declarations(dark, "    ")}\n  }\n}`,
    );
  }

  return blocks.join("\n");
}

function declarations(tokens: ThemeTokens, indent: string): string {
  return Object.entries(tokens)
    .filter(([, value]) => value !== undefined && value !== "")
    .map(([name, value]) => `${indent}${TOKEN_PREFIX}${assertTokenName(name)}: ${value};`)
    .join("\n");
}

function assertTokenName(name: string): string {
  // Token names land verbatim inside a declaration block, so a stray `:` or `}`
  // would silently break every rule after it. Fail the build with the offending
  // key instead of shipping a corrupt stylesheet.
  if (!TOKEN_NAME_PATTERN.test(name)) {
    throw new Error(
      `Invalid theme token name: ${JSON.stringify(name)}. ` +
        `Token names are lowercase kebab-case without the "${TOKEN_PREFIX}" prefix (e.g. "surface-glass").`,
    );
  }
  return name;
}

function mergeTokens(
  layers: ThemeTokenSource[],
  field: "tokens" | "darkTokens",
  include?: (name: string) => boolean,
): ThemeTokens {
  const merged: ThemeTokens = {};

  for (const layer of layers) {
    for (const [name, value] of Object.entries(layer[field] ?? {})) {
      if (!include || include(name)) {
        merged[name] = value;
      }
    }
  }

  return merged;
}

/**
 * Flattens one layer's `extends` chain into base-first order, mirroring the
 * SSG's own resolution. The `seen` guard keeps a theme that extends itself (or
 * forms a cycle across two packages) from hanging the caller.
 */
function expandExtendsChain(theme: ThemeTokenSource): ThemeTokenSource[] {
  const chain: ThemeTokenSource[] = [];
  const seen = new Set<ThemeTokenSource>();
  let current: ThemeTokenSource | undefined = theme;

  while (current && !seen.has(current)) {
    seen.add(current);
    chain.unshift(current);
    current = current.extends;
  }

  return chain;
}
