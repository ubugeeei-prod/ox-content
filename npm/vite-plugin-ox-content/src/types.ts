/**
 * Type definitions for @ox-content/vite-plugin
 */

import type { LanguageRegistration, ThemeRegistration } from "shiki";
import type { ThemeConfig, ResolvedThemeConfig } from "./theme";
import type { GitHubOptions, OgpOptions } from "./plugins";

// =============================================================================
// Entry Page Types (VitePress-like)
// =============================================================================

/**
 * Hero section action button.
 */
export interface HeroAction {
  /** Button theme: 'brand' (primary) or 'alt' (secondary) */
  theme?: "brand" | "alt";
  /** Button text */
  text: string;
  /** Link URL */
  link: string;
}

/**
 * Hero section image configuration.
 */
export interface HeroImage {
  /** Image source URL */
  src: string;
  /** Light mode image source URL */
  lightSrc?: string;
  /** Dark mode image source URL */
  darkSrc?: string;
  /** Alt text */
  alt?: string;
  /** Image width */
  width?: number;
  /** Image height */
  height?: number;
}

/**
 * Hero notice configuration.
 */
export interface HeroNotice {
  /** Notice title */
  title?: string;
  /** Notice paragraphs */
  body?: string[];
}

/**
 * Hero section configuration for entry page.
 */
export interface HeroConfig {
  /** Main title (large, gradient text) */
  name?: string;
  /** Secondary text (medium size) */
  text?: string;
  /** Tagline (smaller, muted) */
  tagline?: string;
  /** Notice shown near the top of the hero */
  notice?: HeroNotice;
  /** Hero image */
  image?: HeroImage;
  /** Action buttons */
  actions?: HeroAction[];
}

/**
 * Feature card for entry page.
 */
export interface FeatureConfig {
  /** Icon - supports: "mdi:icon-name" (Iconify), image URL, or emoji */
  icon?: string;
  /** Feature title */
  title: string;
  /** Feature description */
  details?: string;
  /** Optional link */
  link?: string;
  /** Link text */
  linkText?: string;
}

/**
 * Entry page frontmatter configuration.
 */
export interface EntryPageConfig {
  /** Layout type - set to 'entry' for entry page */
  layout: "entry";
  /** Hero section */
  hero?: HeroConfig;
  /** Feature cards */
  features?: FeatureConfig[];
}

/**
 * Navigation item for SSG sidebar rendering.
 */
export interface SsgNavigationItem {
  /** Display title */
  title: string;
  /**
   * Route path used for active-state matching.
   * Internal links should use site-relative paths such as `/getting-started`.
   */
  path?: string;
  /**
   * Final href used in the rendered HTML.
   * When omitted for internal links, ox-content derives it from `path`.
   */
  href?: string;
}

/**
 * Navigation group for SSG sidebar rendering.
 */
export interface SsgNavigationGroup {
  /** Group heading */
  title: string;
  /** Navigation items within this group */
  items: SsgNavigationItem[];
}

/**
 * SSG (Static Site Generation) options.
 */
export interface SsgOptions {
  /**
   * Enable SSG mode.
   * @default true
   */
  enabled?: boolean;

  /**
   * Output file extension.
   * @default '.html'
   */
  extension?: string;

  /**
   * Clean output directory before build.
   * @default false
   */
  clean?: boolean;

  /**
   * Bare HTML output (no navigation, no styles).
   * Useful for benchmarking or when using custom layouts.
   * @default false
   */
  bare?: boolean;

  /**
   * Site name for header and title suffix.
   */
  siteName?: string;

  /**
   * OG image URL for social sharing (static URL).
   * If generateOgImage is enabled, this serves as the fallback.
   */
  ogImage?: string;

  /**
   * Generate OG images per page using Rust-based generator.
   * When enabled, each page will have a unique OG image.
   * @default false
   */
  generateOgImage?: boolean;

  /**
   * Add each page's last git commit timestamp to the default theme.
   * @default false
   */
  lastUpdated?: boolean;

  /**
   * Site URL for generating absolute OG image URLs.
   * Required for proper SNS sharing.
   * Example: 'https://example.com'
   */
  siteUrl?: string;

  /**
   * Theme configuration for customizing the SSG output.
   * Use defineTheme() to create a theme configuration.
   */
  theme?: ThemeConfig;

  /**
   * Override the auto-generated sidebar navigation.
   * Useful when migrating from tools with explicit navigation config such as VitePress.
   */
  navigation?: SsgNavigationGroup[];
}

/**
 * Resolved SSG options.
 */
export interface ResolvedSsgOptions {
  enabled: boolean;
  extension: string;
  clean: boolean;
  bare: boolean;
  siteName?: string;
  ogImage?: string;
  generateOgImage: boolean;
  lastUpdated: boolean;
  siteUrl?: string;
  theme?: ResolvedThemeConfig;
  navigation?: SsgNavigationGroup[];
}

/**
 * Plugin options.
 */
export interface OxContentOptions {
  /**
   * Source directory for Markdown files.
   * @default 'content'
   */
  srcDir?: string;

  /**
   * Output directory for built files.
   * @default 'dist'
   */
  outDir?: string;

  /**
   * Base path for the site.
   * @default '/'
   */
  base?: string;

  /**
   * Markdown-like file extensions to process.
   * @default ['.md', '.markdown', '.mdx']
   */
  extensions?: string[];

  /**
   * SSG (Static Site Generation) options.
   * Set to false to disable SSG completely.
   * @default { enabled: true }
   */
  ssg?: SsgOptions | boolean;

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
  highlightTheme?: string | ThemeRegistration;

  /**
   * Additional languages for syntax highlighting.
   * Accepts Shiki LanguageRegistration objects (e.g., TextMate grammars).
   * These are loaded alongside the built-in languages.
   */
  highlightLangs?: LanguageRegistration[];

  /**
   * Opt-in code block annotations for fenced code blocks.
   *
   * Supports the configurable attribute syntax by default, and can also opt
   * into VitePress-compatible fence metadata and inline notation.
   *
   * Example:
   * ` ```ts annotate="highlight:1,3-4;warning:6;error:7" `
   *
   * @default false
   */
  codeAnnotations?: boolean | CodeAnnotationsOptions;

  /**
   * Expand Obsidian-style `[[page]]` and `[[page|label]]` links.
   * @default false
   */
  wikiLinks?: boolean | WikiLinkOptions;

  /**
   * Expand `:shortcode:` emoji aliases to Unicode.
   * @default false
   */
  emojiShortcodes?: boolean | EmojiShortcodeOptions;

  /**
   * Enable markdown-it-attrs style `{#id .class key=value}` attributes.
   * @default false
   */
  attrs?: boolean | AttrsOptions;

  /**
   * Import source snippets into fences with `<<< @/path/to/file.ts{region}`.
   * @default false
   */
  codeImports?: boolean | CodeImportOptions;

  /**
   * Sanitize rendered HTML with safe defaults or explicit allow lists.
   * @default false
   */
  sanitize?: boolean | SanitizeOptions;

  /**
   * Append an "edit this page" link to rendered Markdown.
   * @default false
   */
  editThisPage?: boolean | EditThisPageOptions;

  /**
   * Recognize emphasis adjacent to CJK text. The native parser already supports
   * this behavior; the option documents the compatibility contract.
   * @default false
   */
  cjkEmphasis?: boolean;

  /**
   * Lint fenced code blocks during Markdown transforms.
   * @default false
   */
  codeBlockLint?: boolean | CodeBlockLintOptions;

  /**
   * Type-check TypeScript/TSX code fences via tsgo.
   * @default false
   */
  codeBlockTypecheck?: boolean | CodeBlockTypecheckOptions;

  /**
   * Extract runnable fenced examples for Vitest docs-as-tests harnesses.
   * @default false
   */
  docsTests?: boolean | DocsTestOptions;

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
   * Enable OG image generation.
   * @default false
   */
  ogImage?: boolean;

  /**
   * OG image generation options.
   */
  ogImageOptions?: OgImageOptions;

  /**
   * Custom AST transformers.
   */
  transformers?: MarkdownTransformer[];

  /**
   * Source documentation generation options.
   * Set to false to disable (opt-out).
   * @default { enabled: true }
   */
  docs?: DocsOptions | false;

  /**
   * Full-text search options.
   * Set to false to disable search.
   * @default { enabled: true }
   */
  search?: SearchOptions | boolean;

  /**
   * Enable OG Viewer dev tool.
   * Accessible at /__og-viewer during development.
   * @default true
   */
  ogViewer?: boolean;

  /**
   * Built-in static embeds rendered during Markdown transformation.
   * Set to `false` to disable all built-in embeds.
   * @default { github: true, openGraph: true }
   */
  embeds?: BuiltinEmbedOptions | false;

  /**
   * i18n (internationalization) options.
   * Set to false to disable i18n.
   * @default false
   */
  i18n?: I18nOptions | false;
}

/**
 * Resolved options with all defaults applied.
 */
export interface ResolvedOptions {
  srcDir: string;
  outDir: string;
  base: string;
  extensions: string[];
  ssg: ResolvedSsgOptions;
  gfm: boolean;
  footnotes: boolean;
  tables: boolean;
  taskLists: boolean;
  strikethrough: boolean;
  highlight: boolean;
  highlightTheme: string | ThemeRegistration;
  highlightLangs: LanguageRegistration[];
  codeAnnotations: ResolvedCodeAnnotationsOptions;
  wikiLinks: ResolvedWikiLinkOptions;
  emojiShortcodes: ResolvedEmojiShortcodeOptions;
  attrs: ResolvedAttrsOptions;
  codeImports: ResolvedCodeImportOptions;
  sanitize: ResolvedSanitizeOptions;
  editThisPage: ResolvedEditThisPageOptions;
  cjkEmphasis: boolean;
  codeBlockLint: ResolvedCodeBlockLintOptions;
  codeBlockTypecheck: ResolvedCodeBlockTypecheckOptions;
  docsTests: ResolvedDocsTestOptions;
  mermaid: boolean;
  frontmatter: boolean;
  toc: boolean;
  tocMaxDepth: number;
  ogImage: boolean;
  ogImageOptions: ResolvedOgImageOptions;
  transformers: MarkdownTransformer[];
  docs: ResolvedDocsOptions | false;
  search: ResolvedSearchOptions;
  ogViewer: boolean;
  embeds: ResolvedBuiltinEmbedOptions;
  i18n: ResolvedI18nOptions | false;
}

/**
 * Built-in embed configuration.
 */
export interface BuiltinEmbedOptions {
  /**
   * Render `<GitHub repo="owner/name" />` repository cards.
   * Pass an options object to configure fetching.
   * @default true
   */
  github?: boolean | GitHubOptions;

  /**
   * Render `<OgCard url="https://example.com" />` Open Graph link cards.
   * Pass an options object to configure fetching.
   * @default true
   */
  openGraph?: boolean | OgpOptions;

  /**
   * Expand `<pm>npm install …</pm>` blocks into npm/pnpm/yarn/bun install tabs.
   *
   * Accepts a boolean to toggle the feature, or an options object to opt in to
   * synced tab groups. Synced groups are OFF by default; when enabled with
   * `{ sync: true }`, selecting a package manager in one block selects it in
   * every other package-manager block on the page (persisted in localStorage).
   * @default false
   */
  pm?: boolean | BuiltinPmOptions;

  /**
   * Render `<Spotify url="https://open.spotify.com/track/...">` iframes.
   * @default false
   */
  spotify?: boolean;

  /**
   * Render `<StackBlitz url="https://stackblitz.com/edit/...">` iframes.
   * @default false
   */
  stackBlitz?: boolean;

  /**
   * Render `<Tweet>` / `<XPost>` as static privacy-conscious cards.
   * @default false
   */
  twitter?: boolean;

  /**
   * Render `<Bluesky>` as static cards.
   * @default false
   */
  bluesky?: boolean;

  /**
   * Render `<WebContainer>` lazy placeholders with isolation metadata.
   * @default false
   */
  webContainer?: boolean;
}

/**
 * Options for the package-manager install-tab transform.
 */
export interface BuiltinPmOptions {
  /**
   * Enable opt-in synced package-manager tab groups.
   * @default false
   */
  sync?: boolean;
}

/**
 * Resolved built-in embed configuration.
 */
export interface ResolvedBuiltinEmbedOptions {
  github: GitHubOptions | false;
  openGraph: OgpOptions | false;
  pm: BuiltinPmOptions | false;
  spotify: boolean;
  stackBlitz: boolean;
  twitter: boolean;
  bluesky: boolean;
  webContainer: boolean;
}

export interface WikiLinkOptions {
  baseUrl?: string;
}

export interface ResolvedWikiLinkOptions {
  enabled: boolean;
  baseUrl: string;
}

export interface EmojiShortcodeOptions {
  custom?: Record<string, string>;
}

export interface ResolvedEmojiShortcodeOptions {
  enabled: boolean;
  custom: Record<string, string>;
}

export interface AttrsOptions {
  enabled?: boolean;
}

export interface ResolvedAttrsOptions {
  enabled: boolean;
}

export interface CodeImportOptions {
  rootDir?: string;
}

export interface ResolvedCodeImportOptions {
  enabled: boolean;
  rootDir?: string;
}

export interface SanitizeOptions {
  allowedTags?: string[];
  allowedAttributes?: string[];
  allowedUrlSchemes?: string[];
}

export interface ResolvedSanitizeOptions {
  enabled: boolean;
  allowedTags?: string[];
  allowedAttributes?: string[];
  allowedUrlSchemes?: string[];
}

export interface EditThisPageOptions {
  repoUrl: string;
  branch?: string;
  rootDir?: string;
  label?: string;
}

export interface ResolvedEditThisPageOptions {
  enabled: boolean;
  repoUrl?: string;
  branch: string;
  rootDir?: string;
  label: string;
}

export interface CodeBlockLintOptions {
  languages?: string[];
  requireLanguage?: boolean;
  trailingSpaces?: boolean;
  mode?: "warn" | "error";
}

export interface ResolvedCodeBlockLintOptions {
  enabled: boolean;
  languages?: string[];
  requireLanguage: boolean;
  trailingSpaces: boolean;
  mode: "warn" | "error";
}

export interface CodeBlockTypecheckOptions {
  languages?: string[];
  requireMeta?: boolean;
  tsgoCommand?: string;
  mode?: "warn" | "error";
}

export interface ResolvedCodeBlockTypecheckOptions {
  enabled: boolean;
  languages: string[];
  requireMeta: boolean;
  tsgoCommand: string;
  mode: "warn" | "error";
}

export interface DocsTestOptions {
  languages?: string[];
  requireMeta?: boolean;
}

export interface ResolvedDocsTestOptions {
  enabled: boolean;
  languages: string[];
  requireMeta: boolean;
}

/**
 * Supported line annotation kinds for code blocks.
 */
export type CodeAnnotationKind = "highlight" | "warning" | "error";

/**
 * Supported code annotation syntaxes.
 */
export type CodeAnnotationSyntax = "attribute" | "vitepress" | "both";

/**
 * Opt-in code annotation configuration.
 */
export interface CodeAnnotationsOptions {
  /**
   * Annotation syntax to enable.
   *
   * - `attribute`: custom attribute syntax like `annotate="highlight:1,3-4"`
   * - `vitepress`: VitePress-compatible syntax like `{1,3-4}` and `[!code warning]`
   * - `both`: enables both syntaxes
   *
   * @default "attribute"
   */
  notation?: CodeAnnotationSyntax;

  /**
   * Attribute name read from the code fence meta string.
   *
   * Example: `annotate="highlight:1,3-4;warning:6"`
   *
   * @default "annotate"
   */
  metaKey?: string;

  /**
   * Enable line numbers for all code blocks by default.
   *
   * In `vitepress` or `both` mode, fenced code blocks can override this with
   * `:line-numbers`, `:line-numbers=<start>`, or `:no-line-numbers`.
   *
   * @default false
   */
  defaultLineNumbers?: boolean;
}

/**
 * Resolved code annotation configuration.
 */
export interface ResolvedCodeAnnotationsOptions {
  enabled: boolean;
  notation: CodeAnnotationSyntax;
  metaKey: string;
  defaultLineNumbers: boolean;
}

/**
 * OG image generation options.
 * Uses Chromium-based rendering with customizable templates.
 */
export interface OgImageOptions {
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
}

/**
 * Resolved OG image options with all defaults applied.
 */
export interface ResolvedOgImageOptions {
  template?: string;
  vuePlugin: "vitejs" | "vizejs";
  width: number;
  height: number;
  cache: boolean;
  concurrency: number;
}

/**
 * Custom AST transformer.
 */
export interface MarkdownTransformer {
  /**
   * Transformer name.
   */
  name: string;

  /**
   * Transform function.
   */
  transform: (ast: MarkdownNode, context: TransformContext) => MarkdownNode | Promise<MarkdownNode>;
}

/**
 * Transform context passed to transformers.
 */
export interface TransformContext {
  /**
   * File path being processed.
   */
  filePath: string;

  /**
   * Frontmatter data.
   */
  frontmatter: Record<string, unknown>;

  /**
   * Resolved plugin options.
   */
  options: ResolvedOptions;
}

/**
 * Markdown AST node (simplified for TypeScript).
 */
export interface MarkdownNode {
  type: string;
  children?: MarkdownNode[];
  value?: string;
  [key: string]: unknown;
}

/**
 * Transform result.
 */
export interface TransformResult {
  /**
   * Generated JavaScript code.
   */
  code: string;

  /**
   * Source map (null means no source map).
   */
  map?: null;

  /**
   * Rendered HTML.
   */
  html: string;

  /**
   * Parsed frontmatter.
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
  /**
   * Heading depth (1-6).
   */
  depth: number;

  /**
   * Heading text.
   */
  text: string;

  /**
   * Slug/ID for linking.
   */
  slug: string;

  /**
   * Child entries.
   */
  children: TocEntry[];
}

// ============================================
// Source Documentation Types
// ============================================

/**
 * Public API entry point for grouped documentation.
 */
export type DocsEntryPoint =
  | string
  | {
      path: string;
      name?: string;
    };

export type MarkdownDisplayFormat = "none" | "list" | "table";

export type DocsSortStrategy =
  | "source-order"
  | "alphabetical"
  | "alphabetical-ignoring-documents"
  | "enum-value-ascending"
  | "enum-value-descending"
  | "static-first"
  | "instance-first"
  | "visibility"
  | "required-first"
  | "kind"
  | "external-last"
  | "documents-first"
  | "documents-last";

/**
 * Resolved public API entry point.
 */
export interface ResolvedDocsEntryPoint {
  path: string;
  name?: string;
}

/**
 * Options for source documentation generation.
 */
export interface DocsOptions {
  /**
   * Enable/disable docs generation.
   * @default true (opt-out)
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
   * Glob patterns for files to include.
   * @default ['**\/*.ts', '**\/*.tsx', '**\/*.js', '**\/*.jsx', '**\/*.mts', '**\/*.mjs', '**\/*.cts', '**\/*.cjs']
   */
  include?: string[];

  /**
   * Glob patterns for files to exclude.
   * @default ['**\/*.test.*', '**\/*.spec.*', 'node_modules']
   */
  exclude?: string[];

  /**
   * Public API entry points used to group re-exported docs.
   */
  entryPoints?: DocsEntryPoint[];

  /**
   * Output format.
   * @default 'markdown'
   */
  format?: "markdown" | "json" | "html";

  /**
   * Include private members in documentation.
   * @default false
   */
  private?: boolean;

  /**
   * Include internal members in documentation.
   * @default false
   */
  internal?: boolean;

  /**
   * Generate table of contents for each file.
   * @default true
   */
  toc?: boolean;

  /**
   * Group documentation by file or category.
   * @default 'file'
   */
  groupBy?: "file" | "category";

  /**
   * GitHub repository URL for source code links.
   * When provided, generated documentation will include links to source code.
   * Example: 'https://github.com/ubugeeei-prod/ox-content'
   */
  githubUrl?: string;

  /**
   * Internal documentation link style.
   * @default 'markdown'
   */
  linkStyle?: "markdown" | "clean";

  /**
   * Route prefix used by generated documentation links and nav metadata.
   * Nav metadata falls back to '/api' when this is not set.
   */
  basePath?: string;

  /**
   * Generated Markdown output path strategy.
   * @default 'flat'
   */
  pathStrategy?: "flat" | "typedoc";

  /**
   * Rendering style for generated API Markdown.
   *
   * - `'html'` (default): HTML-laced Markdown with collapsible entries, stat
   *   blocks and member tables (ox-content theme).
   * - `'markdown'`: pure Markdown (headings, tables, fenced code) with no raw
   *   HTML scaffolding, suitable for plain Markdown hosts such as VitePress.
   * @default 'html'
   */
  renderStyle?: "html" | "markdown";

  /**
   * Display format for index items.
   * @default 'none'
   */
  indexFormat?: MarkdownDisplayFormat;

  /**
   * Display format for value and type parameters.
   * @default 'none'
   */
  parametersFormat?: MarkdownDisplayFormat;

  /**
   * Display format for interface property groups.
   * @default 'none'
   */
  interfacePropertiesFormat?: MarkdownDisplayFormat;

  /**
   * Display format for class property groups.
   * @default 'none'
   */
  classPropertiesFormat?: MarkdownDisplayFormat;

  /**
   * Display format for type alias property groups.
   * @default 'none'
   */
  typeAliasPropertiesFormat?: MarkdownDisplayFormat;

  /**
   * Display format for enum member groups.
   * @default 'none'
   */
  enumMembersFormat?: MarkdownDisplayFormat;

  /**
   * Display format for property-owned object literal members.
   * @default 'none'
   */
  propertyMembersFormat?: MarkdownDisplayFormat;

  /**
   * Display format for return type declaration members.
   * @default 'none'
   */
  typeDeclarationFormat?: MarkdownDisplayFormat;

  /**
   * Opt in to TSDoc-style type-parameter documentation.
   *
   * When enabled, declaration type parameters (`<T extends C = D>`) are
   * extracted into a structured "Type Parameters" section and `@typeParam` /
   * `@template` tags are merged in (and removed from the generic tag list).
   * `@typeParam` is a TSDoc feature, so this is off by default (JSDoc semantics).
   * @default false
   */
  typeParameters?: boolean;

  /**
   * Emit the stats summary line on generated index pages.
   * @default true
   */
  renderStats?: boolean;

  /**
   * Emit the generated-by attribution on generated root index pages.
   * @default true
   */
  renderGeneratedBy?: boolean;

  /**
   * TypeDoc-style group order for module index sections and nav groups.
   */
  groupOrder?: string[];

  /**
   * TypeDoc-style sort strategies applied to entries and members.
   */
  sort?: DocsSortStrategy[];

  /**
   * Preserve caller-provided entry point order when false.
   * @default true
   */
  sortEntryPoints?: boolean;

  /**
   * TypeDoc-style declaration kind ranking for module sections and nav groups.
   */
  kindSortOrder?: string[];

  /**
   * Single-entry root handling for TypeDoc-style generated docs.
   *
   * When set to `'flatten'`, a single TypeDoc entry point uses the root
   * `index.md` as its landing page and omits the extra module level from
   * generated nav metadata. Symbol page paths stay under the entry point.
   * @default 'preserve'
   */
  singleEntryRoot?: "preserve" | "flatten";

  /**
   * Generate navigation metadata file.
   * @default true
   */
  generateNav?: boolean;
}

/**
 * Resolved docs options with all defaults applied.
 */
export interface ResolvedDocsOptions {
  enabled: boolean;
  src: string[];
  out: string;
  include: string[];
  exclude: string[];
  entryPoints?: ResolvedDocsEntryPoint[];
  format: "markdown" | "json" | "html";
  private: boolean;
  internal: boolean;
  toc: boolean;
  groupBy: "file" | "category";
  githubUrl?: string;
  linkStyle: "markdown" | "clean";
  basePath?: string;
  pathStrategy: "flat" | "typedoc";
  renderStyle: "html" | "markdown";
  indexFormat: MarkdownDisplayFormat;
  parametersFormat: MarkdownDisplayFormat;
  interfacePropertiesFormat: MarkdownDisplayFormat;
  classPropertiesFormat: MarkdownDisplayFormat;
  typeAliasPropertiesFormat: MarkdownDisplayFormat;
  enumMembersFormat: MarkdownDisplayFormat;
  propertyMembersFormat: MarkdownDisplayFormat;
  typeDeclarationFormat: MarkdownDisplayFormat;
  typeParameters: boolean;
  renderStats: boolean;
  renderGeneratedBy: boolean;
  groupOrder?: string[];
  sort?: DocsSortStrategy[];
  sortEntryPoints: boolean;
  kindSortOrder?: string[];
  singleEntryRoot: "preserve" | "flatten";
  generateNav: boolean;
}

/**
 * A single documentation entry extracted from source.
 */
export interface DocEntry {
  name: string;
  kind: "function" | "class" | "interface" | "type" | "enum" | "variable" | "module";
  description: string;
  params?: ParamDoc[];
  returns?: ReturnDoc;
  examples?: string[];
  tags?: Record<string, string>;
  private?: boolean;
  file: string;
  line: number;
  endLine: number;
  signature?: string; // Full function/type signature (for functions and type aliases)
  members?: DocMember[];
}

/**
 * A member belonging to a class, interface, type alias, or enum entry.
 */
export interface DocMember {
  name: string;
  kind: "property" | "method" | "constructor" | "getter" | "setter" | "enumMember";
  description: string;
  signature?: string;
  type?: string;
  default?: string;
  params?: ParamDoc[];
  returns?: ReturnDoc;
  optional?: boolean;
  readonly?: boolean;
  static?: boolean;
  private?: boolean;
  tags?: Record<string, string>;
  line: number;
  endLine: number;
}

/**
 * Parameter documentation.
 */
export interface ParamDoc {
  name: string;
  type: string;
  description: string;
  optional?: boolean;
  default?: string;
}

/**
 * Return type documentation.
 */
export interface ReturnDoc {
  type: string;
  description: string;
}

/**
 * Extracted documentation for a single file.
 */
export interface ExtractedDocs {
  file: string;
  description?: string;
  sourcePath?: string;
  examples?: string[];
  tags?: Record<string, string>;
  entries: DocEntry[];
}

/**
 * Summary counts emitted with generated documentation data.
 */
export interface DocsSummary {
  modules: number;
  entries: number;
  byKind: Record<string, number>;
  params: number;
  returns: number;
  examples: number;
  deprecated: number;
}

/**
 * Machine-readable payload emitted alongside generated docs.
 */
export interface GeneratedDocsData {
  version: 1;
  generatedAt: string;
  summary: DocsSummary;
  modules: ExtractedDocs[];
}

/**
 * Navigation item for sidebar navigation.
 */
export interface NavItem {
  /**
   * Display title for the navigation item.
   */
  title: string;

  /**
   * Path to the documentation page.
   */
  path: string;

  /**
   * Child navigation items (optional).
   */
  children?: NavItem[];
}

// ============================================
// Search Types
// ============================================

/**
 * Options for full-text search.
 */
export interface SearchOptions {
  /**
   * Enable search functionality.
   * @default true
   */
  enabled?: boolean;

  /**
   * Maximum number of search results.
   * @default 10
   */
  limit?: number;

  /**
   * Enable prefix matching for autocomplete.
   * @default true
   */
  prefix?: boolean;

  /**
   * Placeholder text for the search input.
   * @default 'Search documentation...'
   */
  placeholder?: string;

  /**
   * Keyboard shortcut to focus search (without modifier).
   * @default '/'
   */
  hotkey?: string;
}

/**
 * Resolved search options.
 */
export interface ResolvedSearchOptions {
  enabled: boolean;
  limit: number;
  prefix: boolean;
  placeholder: string;
  hotkey: string;
}

/**
 * Search document structure.
 */
export interface SearchDocument {
  id: string;
  title: string;
  url: string;
  body: string;
  headings: string[];
  code: string[];
}

/**
 * Search result structure.
 */
export interface SearchResult {
  id: string;
  title: string;
  url: string;
  score: number;
  matches: string[];
  snippet: string;
  scopes?: string[];
}

/**
 * Parsed search query with optional scope prefixes.
 */
export interface ScopedSearchQuery {
  text: string;
  scopes: string[];
}

// ============================================
// i18n Types
// ============================================

/**
 * Locale configuration.
 */
export interface LocaleConfig {
  /** BCP 47 locale tag (e.g., 'en', 'ja', 'zh-Hans'). */
  code: string;
  /** Display name for this locale (e.g., 'English', '日本語'). */
  name: string;
  /** Text direction. @default 'ltr' */
  dir?: "ltr" | "rtl";
}

/**
 * i18n (internationalization) options.
 */
export interface I18nOptions {
  /**
   * Enable i18n.
   * @default false
   */
  enabled?: boolean;

  /**
   * Path to i18n dictionary directory (relative to project root).
   * @default 'content/i18n'
   */
  dir?: string;

  /**
   * Default locale tag.
   * @default 'en'
   */
  defaultLocale?: string;

  /**
   * Available locales.
   */
  locales?: LocaleConfig[];

  /**
   * Hide default locale prefix in URLs.
   * When true, `/page` serves the default locale and `/ja/page` serves Japanese.
   * When false, all locales get prefixed: `/en/page`, `/ja/page`.
   * @default true
   */
  hideDefaultLocale?: boolean;

  /**
   * Run i18n checks during build.
   * @default true
   */
  check?: boolean;

  /**
   * Translation function names to detect in source code.
   * @default ['t', '$t']
   */
  functionNames?: string[];
}

/**
 * Resolved i18n options with all defaults applied.
 */
export interface ResolvedI18nOptions {
  enabled: boolean;
  dir: string;
  defaultLocale: string;
  locales: LocaleConfig[];
  hideDefaultLocale: boolean;
  check: boolean;
  functionNames: string[];
}
