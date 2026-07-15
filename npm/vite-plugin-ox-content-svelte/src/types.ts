import type { OxContentOptions } from "@ox-content/vite-plugin";

/**
 * Code annotation options for the Svelte integration.
 *
 * The Svelte integration supports opt-in fence metadata and exposes the
 * attribute key used by the Svelte markdown transform.
 */
export interface CodeAnnotationsOptions {
  /**
   * Attribute name read from the code fence meta string.
   *
   * @default 'annotate'
   */
  metaKey?: string;
}

export interface ResolvedCodeAnnotationsOptions {
  enabled: boolean;
  metaKey: string;
}

/**
 * Map from Markdown component names to import paths.
 */
export type ComponentsMap = Record<string, string>;

/**
 * Component registration options.
 *
 * Can be a map, a glob pattern, or an array of glob patterns.
 * Globbed components are resolved relative to the Vite project root during
 * `configResolved`.
 */
export type ComponentsOption = ComponentsMap | string | string[];

/**
 * Svelte integration plugin options.
 *
 * This extends the core ox-content options with Svelte component registration
 * and runtime mode controls. Markdown files are transformed into Svelte-aware
 * modules while the core plugin provides shared parsing and embeds.
 */
export interface SvelteIntegrationOptions extends OxContentOptions {
  /**
   * Markdown-like file extensions to process.
   *
   * Values are normalized with a leading dot before matching file paths.
   *
   * @default ['.md', '.markdown', '.mdx']
   */
  extensions?: string[];

  /**
   * Components to register for use in Markdown.
   *
   * Can be a map of names to paths, a glob pattern, or an array of globs.
   * When using glob patterns, component names are derived from file names.
   *
   * @default {}
   *
   * @example
   * ```ts
   * // Glob pattern (recommended)
   * components: './src/components/*.svelte'
   *
   * // Explicit map
   * components: { Counter: './src/components/Counter.svelte' }
   * ```
   */
  components?: ComponentsOption;

  /**
   * Enable opt-in line annotations for fenced code blocks.
   *
   * Pass `true` to use the default `annotate` meta key, or an object to change it.
   *
   * @default false
   */
  codeAnnotations?: boolean | CodeAnnotationsOptions;

  /**
   * Generate Svelte 5 runes-compatible runtime integration.
   *
   * Keep this enabled for Svelte 5 projects. Disable it only for compatibility
   * with older runtime assumptions.
   *
   * @default true
   */
  runes?: boolean;

  /**
   * Built-in static embeds rendered during Markdown transformation.
   *
   * Set to `false` to disable all built-in embeds.
   * Configure individual fetch-related options under `github` and `openGraph`.
   *
   * @default { github: true, openGraph: true }
   */
  embeds?: BuiltinEmbedOptions | false;
}

/**
 * Fetch options for Svelte GitHub embeds.
 */
export interface GitHubEmbedOptions {
  /**
   * GitHub API token used for higher rate limits and private repository access.
   * @default ''
   */
  token?: string;

  /**
   * Cache fetched repository and source data in memory for the current process.
   * @default true
   */
  cache?: boolean;

  /**
   * Cache TTL in milliseconds.
   * @default 3600000
   */
  cacheTTL?: number;

  /**
   * Maximum source file size to inline in bytes.
   * @default 200000
   */
  maxSourceBytes?: number;

  /**
   * Maximum source lines to inline when no line range is specified.
   * @default 120
   */
  maxSourceLines?: number;
}

/**
 * Fetch options for Svelte Open Graph embeds.
 */
export interface OpenGraphEmbedOptions {
  /**
   * Request timeout in milliseconds.
   * @default 10000
   */
  timeout?: number;

  /**
   * Cache fetched Open Graph metadata in memory for the current process.
   * @default true
   */
  cache?: boolean;

  /**
   * Cache TTL in milliseconds.
   * @default 3600000
   */
  cacheTTL?: number;

  /**
   * User agent sent with metadata fetch requests.
   * @default 'ox-content-ogp-bot/1.0 (compatible; +https://github.com/ubugeeei-prod/ox-content)'
   */
  userAgent?: string;
}

/**
 * Built-in embed options for the Svelte integration.
 */
export interface BuiltinEmbedOptions {
  /**
   * Render `<GitHub repo="owner/name" />` repository cards.
   * @default true
   */
  github?: boolean | GitHubEmbedOptions;

  /**
   * Render `<OgCard url="https://example.com" />` Open Graph link cards.
   * @default true
   */
  openGraph?: boolean | OpenGraphEmbedOptions;
}

export interface ResolvedBuiltinEmbedOptions {
  github: GitHubEmbedOptions | false;
  openGraph: OpenGraphEmbedOptions | false;
}

export interface ResolvedSvelteOptions {
  srcDir: string;
  outDir: string;
  base: string;
  extensions: string[];
  gfm: boolean;
  autolinks: boolean;
  frontmatter: boolean;
  toc: boolean;
  tocMaxDepth: number;
  codeAnnotations: ResolvedCodeAnnotationsOptions;
  components: ComponentsMap;
  runes: boolean;
  embeds: ResolvedBuiltinEmbedOptions;
  root?: string;
}

export interface SvelteTransformResult {
  code: string;
  map: null;
  usedComponents: string[];
  frontmatter: Record<string, unknown>;
}

export interface ComponentIsland {
  name: string;
  props: Record<string, unknown>;
  position: number;
  id: string;
  content?: string;
}
