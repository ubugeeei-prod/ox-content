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
