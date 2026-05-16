/**
 * Type definitions for Vue integration plugin.
 */

import type { OxContentOptions } from "@ox-content/vite-plugin";

export interface CodeAnnotationsOptions {
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
 * Can be a map, a glob pattern, or an array of glob patterns.
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
 */
export interface VueIntegrationOptions extends OxContentOptions {
  /**
   * Components to register for use in Markdown.
   * Can be a map of names to paths, a glob pattern, or an array of globs.
   * When using glob patterns, component names are derived from file names.
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

  /**
   * Enable Vue Reactivity Transform.
   * @default false
   */

  /**
   * Enable custom blocks in Markdown (e.g., `:::tip`).
   * @default true
   */
  components?: ComponentsOption;
  codeAnnotations?: boolean | CodeAnnotationsOptions;
  reactivityTransform?: boolean;
  customBlocks?: boolean;
}

/**
 * Resolved Vue integration options with all defaults applied.
 */
export interface ResolvedVueOptions {
  srcDir: string;
  outDir: string;
  base: string;
  gfm: boolean;
  frontmatter: boolean;
  toc: boolean;
  tocMaxDepth: number;
  codeAnnotations: ResolvedCodeAnnotationsOptions;
  components: ComponentsOption;
  reactivityTransform: boolean;
  customBlocks: boolean;
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
