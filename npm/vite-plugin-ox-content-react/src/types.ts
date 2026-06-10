import type { OxContentOptions } from "@ox-content/vite-plugin";

/**
 * Code annotation options for the React integration.
 *
 * The React integration supports the same opt-in fence metadata as the core
 * plugin, but only exposes the attribute key because rendering is handled by
 * the React markdown transform.
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
 * React integration plugin options.
 *
 * This extends the core ox-content options with React component registration and
 * JSX runtime controls. Markdown files are transformed into React-aware modules
 * while the core plugin still provides shared parsing, embeds, and environment
 * setup.
 */
export interface ReactIntegrationOptions extends OxContentOptions {
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
   * components: './src/components/*.tsx'
   *
   * // Explicit map
   * components: { Counter: './src/components/Counter.tsx' }
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
   * JSX runtime used by generated React modules.
   *
   * `automatic` matches modern React setups. Use `classic` only for projects
   * that still require explicit `React.createElement` output.
   *
   * @default 'automatic'
   */
  jsxRuntime?: "automatic" | "classic";

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
 * Fetch options for React GitHub embeds.
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
 * Fetch options for React Open Graph embeds.
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
 * Built-in embed options for the React integration.
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

export interface ResolvedReactOptions {
  srcDir: string;
  outDir: string;
  base: string;
  extensions: string[];
  gfm: boolean;
  frontmatter: boolean;
  toc: boolean;
  tocMaxDepth: number;
  codeAnnotations: ResolvedCodeAnnotationsOptions;
  components: ComponentsMap;
  jsxRuntime: "automatic" | "classic";
  embeds: ResolvedBuiltinEmbedOptions;
  root?: string;
}

export interface ReactTransformResult {
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
