/**
 * Type definitions for OG image generation.
 */

/**
 * Props passed to OG image template functions.
 */
export interface OgImageTemplateProps {
  /** Page title */
  title: string;
  /** Page description */
  description?: string;
  /** Site name */
  siteName?: string;
  /** Author name */
  author?: string;
  /** Tags/categories */
  tags?: string[];
  /** Custom data from frontmatter (arbitrary key-value pairs) */
  [key: string]: unknown;
}

/**
 * Template function that receives page metadata and returns an HTML string.
 */
export type OgImageTemplateFn = (props: OgImageTemplateProps) => string | Promise<string>;

/**
 * OG image rendering backend.
 */
export type OgImageRenderer = "chromium" | "satori";

/**
 * Font weight values supported by Satori.
 */
export type OgImageSatoriFontWeight = 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900;

/**
 * Font file loaded by the Satori renderer.
 */
export interface OgImageSatoriFont {
  /**
   * Absolute path, or a path relative to the project root.
   */
  path: string;

  /**
   * Font family name used by template CSS.
   */
  name?: string;

  /**
   * Font weight.
   * @default 400
   */
  weight?: OgImageSatoriFontWeight;

  /**
   * Font style.
   * @default "normal"
   */
  style?: "normal" | "italic";
}

/**
 * Satori renderer options.
 */
export interface OgImageSatoriOptions {
  /**
   * Font files passed to Satori.
   *
   * Satori cannot render text without at least one font. When omitted,
   * Ox Content tries a small set of system font paths unless
   * `systemFontFallback` is disabled.
   */
  fonts?: OgImageSatoriFont[];

  /**
   * Try known OS font paths when `fonts` is empty.
   * @default true
   */
  systemFontFallback?: boolean;
}

/**
 * OG image generation options (user-facing).
 */
export interface OgImageOptions {
  /**
   * Rendering backend.
   * - `"chromium"`: full browser rendering, best template compatibility
   * - `"satori"`: fast HTML-to-SVG-to-PNG rendering, limited CSS subset
   * @default "chromium"
   */
  renderer?: OgImageRenderer;

  /**
   * Path to a custom template file (.ts, .vue, .svelte, .tsx/.jsx).
   * - `.ts`: default-export a function `(props) => string`
   * - `.vue`: Vue SFC, rendered via SSR
   * - `.svelte`: Svelte SFC, rendered via SSR
   * - `.tsx`/`.jsx`: React Server Component, rendered via SSR
   * If not specified, the built-in default template is used.
   */
  template?: string;

  /**
   * Vue plugin to use for compiling `.vue` templates.
   * - `'vitejs'`: Use `@vue/compiler-sfc` (official, default)
   * - `'vizejs'`: Use `@vizejs/vite-plugin` (Rust-based)
   * @default 'vitejs'
   */
  vuePlugin?: "vitejs" | "vizejs";

  /**
   * Image width in pixels.
   * @default 1200
   */
  width?: number;

  /**
   * Image height in pixels.
   * @default 630
   */
  height?: number;

  /**
   * Enable content-hash based caching.
   * Skips rendering when content hasn't changed.
   * @default true
   */
  cache?: boolean;

  /**
   * Number of concurrent page instances for parallel rendering.
   * @default 1
   */
  concurrency?: number;

  /**
   * Options for the Satori renderer.
   */
  satori?: OgImageSatoriOptions;
}

/**
 * Resolved OG image options with all defaults applied.
 */
export interface ResolvedOgImageOptions {
  renderer: OgImageRenderer;
  template?: string;
  vuePlugin: "vitejs" | "vizejs";
  width: number;
  height: number;
  cache: boolean;
  concurrency: number;
  satori: {
    fonts: OgImageSatoriFont[];
    systemFontFallback: boolean;
  };
}
