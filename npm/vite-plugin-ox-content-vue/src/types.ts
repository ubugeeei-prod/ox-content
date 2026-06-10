/**
 * Type definitions for Vue integration plugin.
 */

import type { OxContentOptions } from "@ox-content/vite-plugin";

/**
 * Code annotation options for the Vue integration.
 *
 * The Vue integration supports opt-in fence metadata and exposes the attribute
 * key used by the Vue markdown transform.
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
 * Component registration map.
 * Key is the component name to use in Markdown, value is the import path.
 */
export type ComponentsMap = Record<string, string>;

/**
 * Component registration options.
 *
 * Can be a map, a glob pattern, or an array of glob patterns.
 * Globbed components are resolved relative to the Vite project root during
 * `configResolved`.
 *
 * @example
 * ```ts
 * // Using a glob pattern
 * components: './src/components/*.vue'
 *
 * // Using multiple glob patterns
 * components: ['./src/components/*.vue', './src/ui/*.vue']
 *
 * // Using a map for explicit names
 * components: {
 *   Counter: './src/components/Counter.vue',
 * }
 * ```
 */
export type ComponentsOption = ComponentsMap | string | string[];

/**
 * Vue integration plugin options.
 *
 * This extends the core ox-content options with Vue component registration and
 * Vue-specific markdown behavior. Markdown files are transformed into Vue-aware
 * modules while the core plugin provides shared parsing and embeds.
 */
export interface VueIntegrationOptions extends OxContentOptions {
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
   * components: './src/components/*.vue'
   *
   * // Explicit map
   * components: {
   *   Counter: './src/components/Counter.vue',
   * }
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
   * Enable Vue Reactivity Transform.
   *
   * Keep this disabled unless the project still relies on Vue's experimental
   * reactivity transform syntax.
   *
   * @default false
   */
  reactivityTransform?: boolean;

  /**
   * Enable custom blocks in Markdown (e.g., `:::tip`).
   *
   * Disable this only when another Markdown plugin owns the same container
   * syntax.
   *
   * @default true
   */
  customBlocks?: boolean;

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
 * Fetch options for Vue GitHub embeds.
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
 * Fetch options for Vue Open Graph embeds.
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
 * Built-in embed options for the Vue integration.
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

/**
 * Resolved Vue integration options with all defaults applied.
 */
export interface ResolvedVueOptions {
  srcDir: string;
  outDir: string;
  base: string;
  extensions: string[];
  gfm: boolean;
  frontmatter: boolean;
  toc: boolean;
  tocMaxDepth: number;
  codeAnnotations: ResolvedCodeAnnotationsOptions;
  components: ComponentsOption;
  reactivityTransform: boolean;
  customBlocks: boolean;
  embeds: ResolvedBuiltinEmbedOptions;
}

/**
 * Transform result with Vue component information.
 */
export interface VueTransformResult {
  code: string;
  map: null;

  /**
   * List of components used in the Markdown.
   */
  usedComponents: string[];

  /**
   * Extracted frontmatter.
   */
  frontmatter: Record<string, unknown>;
}

/**
 * Island information for component rendering.
 */
export interface ComponentIsland {
  /**
   * Component name.
   */
  name: string;

  /**
   * Props to pass to the component.
   */
  props: Record<string, unknown>;

  /**
   * Position in the HTML output.
   */
  position: number;

  /**
   * Island placeholder ID.
   */
  id: string;

  /**
   * Raw island content extracted from Markdown.
   */
  content?: string;
}

/**
 * Parsed Markdown content with Vue component islands.
 */
export interface ParsedMarkdownContent {
  /**
   * HTML content with island placeholders.
   */
  html: string;

  /**
   * Component islands to render.
   */
  islands: ComponentIsland[];

  /**
   * Frontmatter data.
   */
  frontmatter: Record<string, unknown>;

  /**
   * Table of contents.
   */
  toc: TocEntry[];
}

/**
 * Table of contents entry.
 */
export interface TocEntry {
  level: number;
  text: string;
  slug: string;
  children?: TocEntry[];
}
