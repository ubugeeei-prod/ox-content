/**
 * Type definitions for @ox-content/unplugin
 */

import type MarkdownIt from "markdown-it";

/**
 * Markdown-it plugin function type.
 */
type MarkdownItPluginFn = (md: MarkdownIt, ...options: unknown[]) => void;

/**
 * Markdown-it plugin type.
 * Can be a single plugin or a tuple of [plugin, ...options].
 */
export type MarkdownItPlugin = MarkdownItPluginFn | [MarkdownItPluginFn, ...unknown[]];

/**
 * Remark plugin type.
 * Can be a single plugin or a tuple of [plugin, options].
 */
// eslint-disable-next-line @typescript-eslint/no-redundant-type-constituents
export type RemarkPlugin = [unknown, unknown] | unknown;

/**
 * Rehype plugin type.
 * Can be a single plugin or a tuple of [plugin, options].
 */
// eslint-disable-next-line @typescript-eslint/no-redundant-type-constituents
export type RehypePlugin = [unknown, unknown] | unknown;

/**
 * Ox-content native plugin type.
 * Transforms HTML after rendering.
 */
export type OxContentPlugin = (html: string) => string | Promise<string>;

export interface CodeAnnotationsOptions {
  metaKey?: string;
}

export interface ResolvedCodeAnnotationsOptions {
  enabled: boolean;
  metaKey: string;
}

/**
 * API documentation generation configuration.
 * Similar to cargo docs for Rust.
 */
export interface DocsConfig {
  /**
   * Enable API documentation generation.
   * @default false
   */
  enabled?: boolean;

  /**
   * Source directories to scan for documentation.
   * @default ['./src']
   */
  src?: string[];

  /**
   * Output directory for generated documentation.
   * @default 'docs/api'
   */
  out?: string;

  /**
   * File patterns to include.
   * @default ['**\/*.ts', '**\/*.tsx', '**\/*.js', '**\/*.jsx']
   */
  include?: string[];

  /**
   * File patterns to exclude.
   * @default ['**\/*.test.*', '**\/*.spec.*', '**\/node_modules/**']
   */
  exclude?: string[];

  /**
   * Include private items (starting with _).
   * @default false
   */
  includePrivate?: boolean;

  /**
   * Generate table of contents.
   * @default true
   */
  toc?: boolean;

  /**
   * Group documentation by file or by kind.
   * @default 'file'
   */
  groupBy?: "file" | "kind";
}

/**
 * Plugin configuration for various markdown ecosystems.
 */
export interface PluginConfig {
  /**
   * Ox-content native plugins.
   * Transform HTML after rendering.
   */
  oxContent?: OxContentPlugin[];

  /**
   * Markdown-it plugins.
   * @see https://www.npmjs.com/search?q=markdown-it-plugin
   */
  markdownIt?: MarkdownItPlugin[];

  /**
   * Remark plugins (unified ecosystem).
   * @see https://github.com/remarkjs/remark/blob/main/doc/plugins.md
   */
  remark?: RemarkPlugin[];

  /**
   * Rehype plugins (unified ecosystem).
   * @see https://github.com/rehypejs/rehype/blob/main/doc/plugins.md
   */
  rehype?: RehypePlugin[];
}

/**
 * Plugin options.
 */
export interface OxContentOptions {
  /**
   * Source directory for Markdown files.
   * @default 'docs'
   */
  srcDir?: string;

  /**
   * Enable GitHub Flavored Markdown extensions.
   * @default true
   */
  gfm?: boolean;

  /**
   * Enable footnotes.
   * @default true
   */
  footnotes?: boolean;

  /**
   * Enable tables.
   * @default true
   */
  tables?: boolean;

  /**
   * Enable task lists.
   * @default true
   */
  taskLists?: boolean;

  /**
   * Enable strikethrough.
   * @default true
   */
  strikethrough?: boolean;

  /**
   * Enable syntax highlighting for code blocks.
   * @default false
   */
  highlight?: boolean;

  /**
   * Syntax highlighting theme.
   * @default 'github-dark'
   */
  highlightTheme?: string;

  /**
   * Opt-in line annotations for fenced code blocks.
   * Example: `annotate="highlight:1,3-4;warning:6"`
   * @default false
   */
  codeAnnotations?: boolean | CodeAnnotationsOptions;

  /**
   * Enable mermaid diagram rendering.
   * @default false
   */
  mermaid?: boolean;

  /**
   * Parse YAML frontmatter.
   * @default true
   */
  frontmatter?: boolean;

  /**
   * Generate table of contents.
   * @default true
   */
  toc?: boolean;

  /**
   * Maximum heading depth for TOC.
   * @default 3
   */
  tocMaxDepth?: number;

  /**
   * File extensions to process.
   * @default ['.md', '.markdown']
   */
  extensions?: string[];

  /**
   * Files/patterns to include.
   */
  include?: string | RegExp | RegExp[];

  /**
   * Files/patterns to exclude.
   */
  exclude?: string | RegExp | RegExp[];

  /**
   * Plugin configuration for markdown processing.
   */
  plugin?: PluginConfig;

  /**
   * API documentation generation configuration.
   * Set to false to disable, true to enable with defaults,
   * or provide a DocsConfig object for customization.
   * @default false
   */
  docs?: boolean | DocsConfig;
}

/**
 * Resolved docs configuration.
 */
export interface ResolvedDocsConfig {
  enabled: boolean;
  src: string[];
  out: string;
  include: string[];
  exclude: string[];
  includePrivate: boolean;
  toc: boolean;
  groupBy: "file" | "kind";
}

/**
 * Resolved options with all defaults applied.
 */
export interface ResolvedOptions {
  srcDir: string;
  gfm: boolean;
  footnotes: boolean;
  tables: boolean;
  taskLists: boolean;
  strikethrough: boolean;
  highlight: boolean;
  highlightTheme: string;
  codeAnnotations: ResolvedCodeAnnotationsOptions;
  mermaid: boolean;
  frontmatter: boolean;
  toc: boolean;
  tocMaxDepth: number;
  extensions: string[];
  include: (string | RegExp)[];
  exclude: (string | RegExp)[];
  plugin: Required<PluginConfig>;
  docs: ResolvedDocsConfig;
}

/**
 * Transform result.
 */
export interface TransformResult {
  code: string;
  html: string;
  frontmatter: Record<string, unknown>;
  toc: TocEntry[];
}

/**
 * Table of contents entry.
 */
export interface TocEntry {
  depth: number;
  text: string;
  slug: string;
  children: TocEntry[];
}
