/**
 * Markdown Transformation Engine
 *
 * This module handles the complete transformation pipeline for Markdown files,
 * converting raw Markdown content into JavaScript modules that can be imported
 * by web applications. The transformation process includes:
 *
 * 1. **Parsing**: Uses Rust-based parser via NAPI bindings for high performance
 * 2. **Rendering**: Converts parsed AST to semantic HTML
 * 3. **Enhancement**: Applies syntax highlighting, Mermaid diagram rendering, etc.
 * 4. **Code Generation**: Generates JavaScript/TypeScript module code
 *
 * The generated modules export:
 * - `html`: Rendered HTML content
 * - `frontmatter`: Parsed YAML metadata
 * - `toc`: Hierarchical table of contents
 * - `render`: Client-side render function for dynamic updates
 *
 * @example
 * ```typescript
 * import { transformMarkdown } from './transform';
 *
 * const content = await transformMarkdown(
 *   '# Hello\n\nWorld',
 *   'path/to/file.md',
 *   resolvedOptions
 * );
 *
 * console.log(content.html); // '<h1>Hello</h1><p>World</p>'
 * console.log(content.toc);  // [{ depth: 1, text: 'Hello', slug: 'hello', children: [] }]
 * ```
 */

import YAML from "yaml";
import type { ResolvedOptions, TransformResult, TocEntry } from "./types";
import { highlightCode } from "./highlight";
import { importNapiModule } from "./napi";
import { transformMermaidStatic } from "./plugins/mermaid";
import { protectMermaidSvgs, restoreMermaidSvgs } from "./plugins/mermaid-protect";

/**
 * NAPI bindings for Rust-based Markdown processing.
 *
 * Provides access to compiled Rust functions for high-performance
 * Markdown parsing and rendering operations.
 */
interface NapiBindings {
  /**
   * Simple Markdown parser and renderer in one step.
   * Faster for simple use cases but lacks advanced features.
   *
   * @param source - Raw Markdown content
   * @param options - Parser configuration (GFM flag)
   * @returns Rendered HTML and parsing errors
   */
  parseAndRender: (
    source: string,
    options?: { gfm?: boolean },
  ) => { html: string; errors: string[] };

  /**
   * Full-featured Markdown transformation pipeline.
   * Handles frontmatter extraction, TOC generation, and advanced parsing.
   *
   * @param source - Raw Markdown content (may include frontmatter)
   * @param options - Comprehensive transformation options
   * @returns Transformed result with HTML, metadata, and TOC
   */
  transform: (
    source: string,
    options?: JsTransformOptions,
  ) => {
    html: string;
    frontmatter: string;
    toc: { depth: number; text: string; slug: string }[];
    errors: string[];
  };

  /**
   * Generates an OG image as SVG.
   *
   * @param data - OG image data (title, description, etc.)
   * @param config - Optional OG image configuration
   * @returns SVG string
   */
  generateOgImageSvg: (data: OgImageData, config?: OgImageConfig) => string;

  /**
   * Restores code block metadata after JavaScript-side syntax highlighting.
   *
   * @param originalHtml - HTML before syntax highlighting
   * @param highlightedHtml - HTML after Shiki highlighting
   * @returns Highlighted HTML with original code block metadata reapplied
   */
  mergeHighlightedCodeBlocks: (originalHtml: string, highlightedHtml: string) => string;
}

/**
 * OG image data for generating social media preview images.
 */
export interface OgImageData {
  /** Page title */
  title: string;
  /** Page description */
  description?: string;
  /** Site name */
  siteName?: string;
  /** Author name */
  author?: string;
}

/**
 * OG image configuration.
 */
export interface OgImageConfig {
  /** Image width in pixels */
  width?: number;
  /** Image height in pixels */
  height?: number;
  /** Background color (hex) */
  backgroundColor?: string;
  /** Text color (hex) */
  textColor?: string;
  /** Title font size */
  titleFontSize?: number;
  /** Description font size */
  descriptionFontSize?: number;
}

/**
 * Options for Rust-based Markdown transformation.
 *
 * Controls which Markdown extensions and features are enabled
 * during parsing and rendering.
 */
interface JsTransformOptions {
  /**
   * Enable GitHub Flavored Markdown extensions.
   * Includes tables, task lists, strikethrough, and autolinks.
   * @default false
   */
  gfm?: boolean;

  /**
   * Enable footnotes syntax ([^1]: definition).
   * @default false
   */
  footnotes?: boolean;

  /**
   * Enable task list syntax (- [ ] unchecked, - [x] checked).
   * @default false
   */
  taskLists?: boolean;

  /**
   * Enable table rendering (GFM extension).
   * Requires GFM to be enabled for full functionality.
   * @default false
   */
  tables?: boolean;

  /**
   * Enable strikethrough syntax (~~text~~).
   * Requires GFM to be enabled.
   * @default false
   */
  strikethrough?: boolean;

  /**
   * Enable automatic link conversion (URLs become clickable).
   * @default false
   */
  autolinks?: boolean;

  /**
   * Maximum heading depth for table of contents.
   * Headings deeper than this level are excluded from TOC.
   * @default 3
   * @min 1
   * @max 6
   */
  tocMaxDepth?: number;

  /**
   * Convert `.md` links to `.html` links for SSG output.
   * @default false
   */
  convertMdLinks?: boolean;

  /**
   * Base URL for absolute link conversion (e.g., "/" or "/docs/").
   * @default "/"
   */
  baseUrl?: string;

  /**
   * Source file path for relative link resolution.
   * Used to determine if the current file is an index file.
   */
  sourcePath?: string;

  /**
   * Enable line annotations for code blocks using fence meta.
   * @default false
   */
  codeAnnotations?: boolean;

  /**
   * Fence meta key used to read code annotations.
   * @default "annotate"
   */
  codeAnnotationMetaKey?: string;

  /**
   * Code annotation syntax mode.
   * @default "attribute"
   */
  codeAnnotationSyntax?: "attribute" | "vitepress" | "both";

  /**
   * Enable line numbers for all code blocks by default.
   * @default false
   */
  codeAnnotationDefaultLineNumbers?: boolean;
}

/**
 * Cached NAPI bindings instance.
 * Loaded on first use and reused for subsequent transformations.
 * @internal
 */
let napiBindings: NapiBindings | null | undefined;

/**
 * Flag to prevent repeated NAPI loading attempts.
 * Set to true after first load attempt (success or failure).
 * @internal
 */
let napiLoadAttempted = false;

/**
 * Lazily loads and caches NAPI bindings.
 *
 * This function uses lazy loading to defer the import of NAPI bindings
 * until they're actually needed. The bindings are loaded only once and
 * cached for subsequent uses. If loading fails (e.g., bindings not built),
 * the failure is cached to avoid repeated load attempts.
 *
 * ## Performance Considerations
 *
 * The first call to this function may have a slight performance penalty
 * due to module loading. Subsequent calls use the cached result and are
 * essentially zero-cost.
 *
 * ## Error Handling
 *
 * If NAPI bindings are not available (not built, wrong architecture, etc.),
 * this function returns `null`. The caller should handle this gracefully
 * or provide fallback behavior.
 *
 * @returns Promise resolving to NAPI bindings or null if unavailable
 *
 * @example
 * ```typescript
 * // Simple check with fallback
 * const napi = await loadNapiBindings();
 * if (!napi) {
 *   console.warn('NAPI bindings not available, using fallback');
 *   return fallbackRender(content);
 * }
 *
 * // Use Rust implementation
 * const result = napi.transform(content, { gfm: true });
 * ```
 *
 * @internal
 */
async function loadNapiBindings(): Promise<NapiBindings | null> {
  // Return cached result (success or failure)
  if (napiLoadAttempted) {
    return napiBindings ?? null;
  }

  // Mark attempt as made to prevent retry loops
  napiLoadAttempted = true;

  try {
    // Dynamic import to handle cases where NAPI isn't built
    const mod = await importNapiModule();
    napiBindings = mod;
    return mod;
  } catch (error) {
    // NAPI not available (not built, missing dependencies, etc.)
    // Log for debugging but don't throw - allow graceful degradation
    if (process.env.DEBUG) {
      console.debug("[ox-content] NAPI bindings load failed:", error);
    }
    napiBindings = null;
    return null;
  }
}

/**
 * Transforms Markdown content into a JavaScript module.
 *
 * This is the primary entry point for transforming Markdown files. It handles
 * the complete transformation pipeline including parsing, rendering, syntax
 * highlighting, and code generation.
 *
 * ## Pipeline Steps
 *
 * 1. **Parse & Render**: Uses Rust-based parser via NAPI for high performance
 * 2. **Extract Metadata**: Parses YAML frontmatter and generates table of contents
 * 3. **Enhance HTML**: Applies syntax highlighting and Mermaid diagram rendering
 * 4. **Generate Code**: Creates importable JavaScript module
 *
 * ## Generated Module Exports
 *
 * - `html` (string): Rendered HTML content with all enhancements applied
 * - `frontmatter` (object): Parsed YAML frontmatter as JavaScript object
 * - `toc` (array): Hierarchical table of contents entries
 * - `render` (function): Client-side render function for dynamic updates
 *
 * ## Markdown Features Supported
 *
 * The supported features depend on parser options:
 * - **Commonmark**: Headings, paragraphs, lists, code blocks, links, images
 * - **GFM Extensions**: Tables, task lists, strikethrough, autolinks
 * - **Enhancements**: Syntax highlighting, Mermaid diagrams, TOC generation
 * - **Metadata**: YAML frontmatter parsing
 *
 * ## Performance
 *
 * Uses Rust-based parsing via NAPI bindings for optimal performance. Falls back
 * gracefully if Rust bindings are unavailable.
 *
 * @param source - Raw Markdown source code (may include YAML frontmatter)
 * @param filePath - File path for source attribution and relative link resolution
 * @param options - Resolved plugin options controlling transformation behavior
 *
 * @returns Promise resolving to transformation result with HTML and metadata
 *
 * @throws Error if NAPI bindings are unavailable (can be handled gracefully)
 *
 * @example
 * ```typescript
 * import { transformMarkdown } from './transform';
 * import { resolveOptions } from './index';
 *
 * // Transform a Markdown file with YAML frontmatter
 * const markdown = `---
 * title: Getting Started
 * author: john
 * ---
 *
 * # Getting Started
 *
 * Welcome! This guide explains [transformMarkdown] function.
 *
 * ## Installation
 *
 * \`\`\`bash
 * npm install @ox-content/vite-plugin
 * \`\`\`
 * `;
 *
 * const options = resolveOptions({
 *   highlight: true,
 *   highlightTheme: 'github-dark',
 *   toc: true,
 *   gfm: true,
 *   mermaid: true,
 * });
 *
 * const result = await transformMarkdown(markdown, 'docs/getting-started.md', options);
 *
 * // Generated module exports
 * console.log(result.html);        // Rendered HTML with syntax highlighting
 * console.log(result.frontmatter); // { title: 'Getting Started', author: 'john' }
 * console.log(result.toc);         // [{ depth: 1, text: 'Getting Started', ... }]
 * console.log(result.code);        // ES module export statement
 * ```
 */
/**
 * SSG-specific transform options.
 */
export interface SsgTransformOptions {
  /** Convert `.md` links to `.html` links */
  convertMdLinks?: boolean;
  /** Base URL for absolute link conversion */
  baseUrl?: string;
  /** Source file path for relative link resolution */
  sourcePath?: string;
}

export async function transformMarkdown(
  source: string,
  filePath: string,
  options: ResolvedOptions,
  ssgOptions?: SsgTransformOptions,
): Promise<TransformResult> {
  const napi = await loadNapiBindings();

  if (!napi) {
    throw new Error(
      "[ox-content] NAPI bindings not available. Please ensure @ox-content/napi is built.",
    );
  }

  // Parse frontmatter using YAML on TypeScript side for proper nested object support
  const { content: markdownContent, frontmatter } = parseFrontmatter(source);

  // Use Rust-based transformation (pass content without frontmatter)
  const result = napi.transform(markdownContent, {
    gfm: options.gfm,
    footnotes: options.footnotes,
    taskLists: options.taskLists,
    tables: options.tables,
    strikethrough: options.strikethrough,
    tocMaxDepth: options.tocMaxDepth,
    convertMdLinks: ssgOptions?.convertMdLinks,
    baseUrl: ssgOptions?.baseUrl,
    sourcePath: ssgOptions?.sourcePath ?? filePath,
    codeAnnotations: options.codeAnnotations.enabled,
    codeAnnotationMetaKey: options.codeAnnotations.metaKey,
    codeAnnotationSyntax: options.codeAnnotations.notation,
    codeAnnotationDefaultLineNumbers: options.codeAnnotations.defaultLineNumbers,
  });

  if (result.errors.length > 0) {
    console.warn("[ox-content] Transform warnings:", result.errors);
  }

  let html = result.html;

  // Convert flat TOC from Rust to nested TOC
  const flatToc: TocEntry[] = result.toc.map((item) => ({
    ...item,
    children: [],
  }));
  const toc = options.toc ? buildTocTree(flatToc) : [];

  // Transform mermaid diagrams before highlighting to avoid entity re-encoding
  if (options.mermaid) {
    html = await transformMermaidStatic(html);
  }

  // Protect mermaid SVGs from rehype processing (which corrupts <br /> in foreignObjects)
  const { html: protectedHtml, svgs } = protectMermaidSvgs(html);
  html = protectedHtml;

  // Apply syntax highlighting if enabled
  if (options.highlight) {
    const originalHtml = html;
    const highlightedHtml = await highlightCode(
      html,
      options.highlightTheme,
      options.highlightLangs,
    );
    html = napi.mergeHighlightedCodeBlocks(originalHtml, highlightedHtml);
  }

  // Restore protected SVGs
  html = restoreMermaidSvgs(html, svgs);

  // Generate JavaScript module code
  const code = generateModuleCode(html, frontmatter, toc, filePath, options);

  return {
    code,
    html,
    frontmatter,
    toc,
  };
}

/**
 * Parses YAML frontmatter from Markdown content.
 * Uses proper YAML parser for full nested object support.
 */
function parseFrontmatter(source: string): {
  content: string;
  frontmatter: Record<string, unknown>;
} {
  // Check for frontmatter delimiter
  if (!source.startsWith("---")) {
    return { content: source, frontmatter: {} };
  }

  // Find the closing delimiter
  const rest = source.slice(3);
  const endIndex = rest.indexOf("\n---");

  if (endIndex === -1) {
    return { content: source, frontmatter: {} };
  }

  const frontmatterStr = rest.slice(0, endIndex).trim();
  const content = rest.slice(endIndex + 4).trimStart();

  try {
    const frontmatter = YAML.parse(frontmatterStr) as Record<string, unknown>;
    return { content, frontmatter: frontmatter || {} };
  } catch (error) {
    console.warn("[ox-content] Failed to parse frontmatter:", error);
    return { content, frontmatter: {} };
  }
}

/**
 * Builds nested TOC tree from flat list.
 */
function buildTocTree(entries: TocEntry[]): TocEntry[] {
  const root: TocEntry[] = [];
  const stack: TocEntry[] = [];

  for (const entry of entries) {
    // Pop stack until we find a parent with smaller depth
    while (stack.length > 0 && stack[stack.length - 1].depth >= entry.depth) {
      stack.pop();
    }

    if (stack.length === 0) {
      root.push(entry);
    } else {
      stack[stack.length - 1].children.push(entry);
    }

    stack.push(entry);
  }

  return root;
}

/**
 * Generates the JavaScript module code.
 */
function generateModuleCode(
  html: string,
  frontmatter: Record<string, unknown>,
  toc: TocEntry[],
  filePath: string,
  _options: ResolvedOptions,
): string {
  const htmlJson = JSON.stringify(html);
  const frontmatterJson = JSON.stringify(frontmatter);
  const tocJson = JSON.stringify(toc);

  return `
// Generated by @ox-content/vite-plugin
// Source: ${filePath}

/**
 * Rendered HTML content.
 */
export const html = ${htmlJson};

/**
 * Parsed frontmatter.
 */
export const frontmatter = ${frontmatterJson};

/**
 * Table of contents.
 */
export const toc = ${tocJson};

/**
 * Default export with all data.
 */
export default {
  html,
  frontmatter,
  toc,
};

// HMR support
if (import.meta.hot) {
  import.meta.hot.accept((newModule) => {
    if (newModule) {
      // Trigger re-render with new content
      import.meta.hot.invalidate();
    }
  });
}
`;
}

/**
 * Extracts imports from Markdown content.
 *
 * Supports importing components for interactive islands.
 */
export function extractImports(content: string): string[] {
  const importRegex = /^import\s+.+\s+from\s+['"](.+)['"]/gm;
  const imports: string[] = [];
  let match;

  while ((match = importRegex.exec(content)) !== null) {
    imports.push(match[1]);
  }

  return imports;
}

/**
 * Generates an OG image SVG using the Rust-based generator.
 *
 * This function uses the Rust NAPI bindings to generate SVG-based
 * OG images for social media previews. The SVG can be served directly
 * or converted to PNG/JPEG for broader compatibility.
 *
 * In the future, custom JS templates can be provided to override
 * the default Rust-based template.
 *
 * @param data - OG image data (title, description, etc.)
 * @param config - Optional OG image configuration
 * @returns SVG string or null if NAPI bindings are unavailable
 */
export async function generateOgImageSvg(
  data: OgImageData,
  config?: OgImageConfig,
): Promise<string | null> {
  const napi = await loadNapiBindings();
  if (!napi) {
    return null;
  }

  // Convert config to NAPI format (camelCase to snake_case)
  const napiConfig = config
    ? {
        width: config.width,
        height: config.height,
        backgroundColor: config.backgroundColor,
        textColor: config.textColor,
        titleFontSize: config.titleFontSize,
        descriptionFontSize: config.descriptionFontSize,
      }
    : undefined;

  return napi.generateOgImageSvg(data, napiConfig);
}
