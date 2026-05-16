import type { OxContentOptions } from "@ox-content/vite-plugin";

export interface CodeAnnotationsOptions {
  metaKey?: string;
}

export interface ResolvedCodeAnnotationsOptions {
  enabled: boolean;
  metaKey: string;
}

export type ComponentsMap = Record<string, string>;

/**
 * Component registration options.
 * Can be a map, a glob pattern, or an array of glob patterns.
 */
export type ComponentsOption = ComponentsMap | string | string[];

export interface SvelteIntegrationOptions extends OxContentOptions {
  /**
   * Components to register for use in Markdown.
   * Can be a map of names to paths, a glob pattern, or an array of globs.
   * When using glob patterns, component names are derived from file names.
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
  codeAnnotations?: boolean | CodeAnnotationsOptions;
  runes?: boolean;
}

export interface ResolvedSvelteOptions {
  srcDir: string;
  outDir: string;
  base: string;
  gfm: boolean;
  frontmatter: boolean;
  toc: boolean;
  tocMaxDepth: number;
  codeAnnotations: ResolvedCodeAnnotationsOptions;
  components: ComponentsMap;
  runes: boolean;
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
