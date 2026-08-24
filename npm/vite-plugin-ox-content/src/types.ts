/**
 * Type definitions for @ox-content/vite-plugin
 */

import type { ThemeConfig, ResolvedThemeConfig } from "./theme";
import type { GitHubOptions, OgpOptions, TwitterEmbedOptions } from "./plugins";
import type { ThemeComponent } from "./theme-renderer";

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
 * Static Site Generation options.
 *
 * These options control the HTML files emitted at build time and the matching
 * dev-server preview behavior. Pass `false` to the top-level `ssg` option to
 * disable the whole SSG pipeline, or pass an object to customize the defaults.
 */
export interface SsgOptions {
  /**
   * Enable the SSG pipeline.
   *
   * Keep this enabled when ox-content owns page rendering. Disable it only when
   * another framework integration will consume the Markdown modules directly.
   *
   * @default true
   */
  enabled?: boolean;

  /**
   * File extension used for generated routes.
   *
   * The value should include the leading dot. For example, `.html` emits
   * `guide.html`, while an empty string can be used by custom deployments that
   * map extensionless output themselves.
   *
   * @default '.html'
   */
  extension?: string;

  /**
   * Remove previously generated files from the output directory before writing
   * the new SSG result.
   *
   * Leave this disabled when the output directory also contains assets produced
   * by other Vite plugins or external build steps.
   *
   * @default false
   */
  clean?: boolean;

  /**
   * Emit bare HTML with only the rendered Markdown body.
   *
   * This skips the default navigation, layout shell, and theme styles. It is
   * mainly useful for benchmarking, fixture generation, or projects that wrap
   * the output in their own shell.
   *
   * @default false
   */
  bare?: boolean;

  /**
   * Site name shown in the default theme header and title suffix.
   *
   * When omitted, the renderer falls back to project metadata where available.
   *
   * @default undefined
   */
  siteName?: string;

  /**
   * Static Open Graph image URL used for social sharing.
   *
   * When `generateOgImage` is enabled, this value is still useful as a fallback
   * for pages that cannot produce a generated image.
   *
   * @default undefined
   */
  ogImage?: string;

  /**
   * Render each page with a JSX theme component instead of the built-in
   * renderer.
   *
   * The component owns the whole document, so `theme`, `bare` and the head
   * metadata options do not apply — everything from `<html>` down is yours.
   * Compose one per layout with `createTheme()`, and read the current page
   * through `usePageProps()` / `useSiteConfig()`.
   *
   * ```ts
   * ssg: { render: createTheme({ layouts: { default: DefaultLayout } }) }
   * ```
   *
   * @default undefined
   */
  render?: ThemeComponent;

  /**
   * `lang` attribute for the generated `<html>` element.
   *
   * Bare mode uses this verbatim; themed pages derive it from `i18n` instead.
   *
   * @default "en"
   */
  lang?: string;

  /**
   * Raw markup appended to `<head>`.
   *
   * Bare mode only — themed pages own their head. Use it for the stylesheet
   * your own build emits, or any tag the plugin does not generate.
   *
   * @default undefined
   */
  head?: string;

  /**
   * Raw markup inserted directly after `<body>`.
   *
   * Bare mode only. Use it for a site header that wraps the rendered page.
   *
   * @default undefined
   */
  bodyStart?: string;

  /**
   * Raw markup inserted directly before `</body>`.
   *
   * Bare mode only. Use it for a site footer, or scripts you inject yourself.
   *
   * @default undefined
   */
  bodyEnd?: string;

  /**
   * Generate one Open Graph image per page.
   *
   * Generated images are written alongside the SSG output and referenced from
   * each page's metadata. Configure rendering details with the top-level
   * `ogImageOptions` option.
   *
   * Under `bare`, the images are still written but nothing references them,
   * because bare output has no `<head>` to put the `<meta>` tags in — inject
   * them from your own shell.
   *
   * @default false
   */
  generateOgImage?: boolean;

  /**
   * Add each page's last git commit timestamp to the default theme.
   * @default false
   */
  lastUpdated?: boolean;

  /**
   * Show previous/next page links after the article.
   *
   * Disabled when omitted or `false`. `true` enables the default pager.
   * An object also enables the feature.
   *
   * @default false
   */
  pagination?: boolean | Record<string, unknown>;

  /**
   * Show a breadcrumb trail from the site root through sidebar ancestors.
   *
   * Disabled when omitted or `false`. `true` enables the default trail.
   * An object also enables the feature. Frontmatter `breadcrumbs: false`
   * hides the trail on that page.
   *
   * @default false
   */
  breadcrumbs?: boolean | Record<string, unknown>;

  /**
   * Opt-in copy buttons, outbound-link icons, and a back-to-top control.
   *
   * Disabled when omitted or `false`. `true` enables all three with defaults.
   * An object enables the feature and can turn one control off, for example
   * `{ copy: false }`.
   *
   * @default false
   */
  readerChrome?: boolean | ReaderChromeOptions;

  /**
   * Show a header locale switcher in the default theme.
   *
   * Disabled when omitted or `false`, even if `i18n.locales` is set.
   * `true` or an object enables the control when available locales are
   * non-empty. Links use the sibling page when it exists, otherwise the
   * locale root (`/{locale}/` or a configured root).
   *
   * @default false
   */
  localeSwitcher?: boolean | Record<string, unknown>;

  /**
   * Opt-in skip link and print styles.
   *
   * Disabled when omitted or `false`. `true` enables the default skip link
   * and print CSS. An object enables the feature and can override the label.
   *
   * @default false
   */
  a11y?: boolean | A11yOptions;

  /**
   * Write a themed 404 page during SSG.
   *
   * Off by default. `true` reads `404.md` from `srcDir` and writes `404.html`.
   * An object enables the feature and overrides only the fields you set.
   * When the source file is missing, a built-in "Page not found" page is
   * written instead. The page is omitted from the search index and sitemap.
   *
   * @default false
   */
  notFound?: boolean | NotFoundOptions;

  /**
   * Absolute site URL used when generating social metadata.
   *
   * Set this when pages need absolute Open Graph image URLs. Include the origin
   * and any deployment base path, without a trailing page path.
   *
   * @example
   * ```ts
   * siteUrl: 'https://example.com/docs'
   * ```
   *
   * @default undefined
   */
  siteUrl?: string;

  /**
   * Theme configuration for generated pages.
   *
   * Use `defineTheme()` to build this object so custom theme modules and the
   * default theme extension points keep their expected shape.
   *
   * An array composes layers left to right, which is how a skin package and a
   * color package are combined:
   *
   * ```ts
   * theme: [pixelSkin, tokyoNight, { footer: { copyright: "2026" } }]
   * ```
   *
   * @default defaultTheme
   */
  theme?: ThemeConfig | ThemeConfig[];

  /**
   * Sidebar navigation override.
   *
   * When omitted, ox-content derives navigation from the Markdown file tree.
   * Provide this when migrating from systems such as VitePress where navigation
   * is intentionally hand-authored.
   *
   * @default undefined
   */
  navigation?: SsgNavigationGroup[];
}

/**
 * Per-control flags for `ssg.readerChrome`.
 *
 * Omitted fields stay on when the feature itself is enabled.
 */
export interface ReaderChromeOptions {
  /**
   * Copy button on fenced code blocks. The clipboard is read in the browser,
   * never at build time.
   *
   * @default true
   */
  copy?: boolean;

  /**
   * Icon and `rel="noopener noreferrer"` on outbound `http(s)` links.
   * Relative, hash, and same-document links are left alone.
   *
   * @default true
   */
  externalLinks?: boolean;

  /**
   * Back-to-top control that appears after the page is scrolled.
   *
   * @default true
   */
  backToTop?: boolean;
}

/**
 * Resolved reader chrome. `false` means no extra markup or JS.
 */
export type ResolvedReaderChrome =
  | false
  | {
      copy: boolean;
      externalLinks: boolean;
      backToTop: boolean;
    };

/**
 * Per-control flags for `ssg.a11y`.
 *
 * Omitted fields keep the defaults when the feature itself is enabled.
 */
export interface A11yOptions {
  /**
   * Visible label for the skip link. Escaped in HTML.
   *
   * @default "Skip to content"
   */
  skipLinkLabel?: string;
}

/**
 * Resolved skip-link / print styles. `false` means no extra markup or CSS.
 */
export type ResolvedA11y =
  | false
  | {
      skipLinkLabel: string;
    };

/**
 * Resolved SSG options.
 */
export interface ResolvedSsgOptions {
  enabled: boolean;
  extension: string;
  clean: boolean;
  bare: boolean;
  render?: ThemeComponent;
  lang?: string;
  head?: string;
  bodyStart?: string;
  bodyEnd?: string;
  siteName?: string;
  ogImage?: string;
  generateOgImage: boolean;
  lastUpdated: boolean;
  pagination: boolean;
  breadcrumbs: boolean;
  readerChrome: ResolvedReaderChrome;
  localeSwitcher: boolean;
  a11y: ResolvedA11y;
  /**
   * Present after `resolveSsgOptions`. Omitted in hand-built fixtures means off.
   */
  notFound?: ResolvedNotFoundOptions;
  siteUrl?: string;
  theme?: ResolvedThemeConfig;
  navigation?: SsgNavigationGroup[];
}

/**
 * Opt-in custom 404 page written during SSG.
 */
export interface NotFoundOptions {
  /**
   * Markdown source relative to `srcDir`.
   * @default "404.md"
   */
  source?: string;

  /**
   * Output file relative to `outDir`.
   * @default "404.html"
   */
  output?: string;
}

/**
 * Resolved custom 404 options.
 */
export interface ResolvedNotFoundOptions {
  enabled: boolean;
  source: string;
  output: string;
}

/**
 * Opt-in crawl manifests written during SSG.
 */
export interface SiteMapsOptions {
  /**
   * Write `robots.txt` with a Sitemap line.
   * @default true
   */
  robots?: boolean;

  /**
   * Write `llms.txt` with the site title, description, and page URLs.
   * @default true
   */
  llms?: boolean;
}

/**
 * Resolved crawl-manifest options.
 */
export interface ResolvedSiteMapsOptions {
  enabled: boolean;
  robots: boolean;
  llms: boolean;
}

/**
 * Opt-in draft / unlisted / scheduled page filtering.
 */
export interface PublishStateOptions {
  /**
   * When `false`, frontmatter publish fields are ignored.
   * @default true when the option is an object
   */
  enabled?: boolean;

  /**
   * Injected ISO-8601 clock compared against `scheduled`, `date`, and `expiry`.
   * Invalid values fall back to the system clock.
   */
  now?: string;

  /**
   * Keep draft and not-yet-scheduled pages in output. The dev server sets this.
   * @default false
   */
  includeDrafts?: boolean;
}

/**
 * Resolved publish-state options.
 */
export interface ResolvedPublishStateOptions {
  enabled: boolean;
  now?: string;
  includeDrafts: boolean;
}

/**
 * Opt-in frontmatter `permalink` / `slug` routing.
 *
 * `false` or omitted stays off. `true` or `{}` enables defaults.
 * Set `enabled: false` on the object to turn the feature back off.
 */
export interface PermalinksOptions {
  /**
   * Enable permalink / slug routing.
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved permalink options.
 */
export interface ResolvedPermalinksOptions {
  enabled: boolean;
}

/**
 * Opt-in `_index` directory frontmatter cascade.
 *
 * `false` or omitted stays off. `true` or `{}` enables defaults.
 * Set `enabled: false` on the object to turn the feature back off.
 */
export interface CascadeOptions {
  /**
   * Enable directory-level frontmatter inheritance.
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved cascade options.
 */
export interface ResolvedCascadeOptions {
  enabled: boolean;
}

/**
 * Opt-in static redirects, aliases, and path rewrites.
 *
 * A path map such as `{ "/old-guide": "/guide" }` is also accepted in place
 * of this object and enables the feature with that map.
 */
export interface RedirectsOptions {
  /**
   * Old path to new path. Destinations must be same-origin (`/` but not `//`)
   * unless `allowExternal` is set.
   * @default {}
   */
  map?: Record<string, string>;

  /**
   * Write a Netlify / Cloudflare `_redirects` file next to the HTML pages.
   * @default false
   */
  netlify?: boolean;

  /**
   * Write a `_headers` Location map next to the HTML pages.
   * @default false
   */
  headers?: boolean;

  /**
   * Write a machine-readable `redirects.json` map.
   * @default false
   */
  json?: boolean;

  /**
   * Allow `http://` and `https://` destinations. `javascript:`, `data:`, and
   * protocol-relative `//` targets stay rejected.
   * @default false
   */
  allowExternal?: boolean;
}

/**
 * Resolved redirect options.
 */
export interface ResolvedRedirectsOptions {
  enabled: boolean;
  map: Record<string, string>;
  netlify: boolean;
  headers: boolean;
  json: boolean;
  allowExternal: boolean;
}

/**
 * Feed file formats written during SSG.
 */
export type FeedFormat = "rss" | "atom" | "json";

/**
 * Opt-in RSS / Atom / JSON Feed files written during SSG.
 */
export interface FeedsOptions {
  /**
   * Feed formats to write.
   * @default ["rss", "atom", "json"]
   */
  formats?: FeedFormat[];

  /**
   * Named collection to publish. Defaults to `content`, or the first
   * configured collection when `content` is absent.
   */
  collection?: string;

  /**
   * Maximum number of published items, newest first.
   * @default 20
   */
  limit?: number;

  /**
   * Site-relative directory for the generated files.
   * @default "/"
   */
  path?: string;
}

/**
 * Resolved feed options.
 */
export interface ResolvedFeedsOptions {
  enabled: boolean;
  formats: FeedFormat[];
  collection?: string;
  limit: number;
  path: string;
}

/**
 * Options for the core `oxContent()` Vite plugin.
 *
 * The top-level options describe where content lives, which Markdown features
 * are enabled, and which build-time features should run. Feature toggles that
 * accept `boolean | Options` follow the same convention:
 *
 * - `false` disables the feature.
 * - `true` enables the feature with its documented defaults.
 * - an object enables the feature and overrides only the provided fields.
 */
export interface OxContentOptions {
  /**
   * Directory containing Markdown source files.
   *
   * The path is resolved from the Vite project root. SSG, search indexing, and
   * dev-server routing all use this directory as the content root.
   *
   * @default 'content'
   */
  srcDir?: string;

  /**
   * Directory where generated files are written.
   *
   * SSG HTML, search indexes, and generated assets are emitted under this
   * directory during production builds.
   *
   * @default 'dist'
   */
  outDir?: string;

  /**
   * Base path prepended to generated internal URLs.
   *
   * Use this when the site is deployed below a sub-path, such as GitHub Pages or
   * a documentation route inside a larger application.
   *
   * @default '/'
   */
  base?: string;

  /**
   * Markdown-like file extensions to process.
   *
   * Extensions are normalized with a leading dot and matched case-insensitively.
   * Add custom extensions when another authoring format is compiled to Markdown
   * before ox-content sees it.
   *
   * @default ['.md', '.markdown', '.mdx']
   */
  extensions?: string[];

  /**
   * Static Site Generation options.
   *
   * Passing `true` or omitting this option enables SSG with defaults. Passing
   * `false` disables the SSG plugin while still allowing Markdown module
   * transforms to run.
   *
   * @default { enabled: true }
   */
  ssg?: SsgOptions | boolean;

  /**
   * Write crawl manifests next to generated HTML.
   *
   * Off by default. `true` writes `sitemap.xml`, `robots.txt`, and `llms.txt`.
   * An object enables the feature and overrides only the fields you set.
   * Requires `ssg.siteUrl`. When that is missing the build continues and a
   * warning is emitted instead of writing files.
   *
   * @default false
   */
  siteMaps?: boolean | SiteMapsOptions;

  /**
   * Honor frontmatter draft / unlisted / scheduled publish states.
   *
   * Off by default. `true` omits drafts and future-scheduled pages from
   * production HTML, search, and sitemaps. Unlisted pages still build and
   * remain reachable by URL. An object enables the feature and can inject
   * `now` for a deterministic build-time clock.
   *
   * @default false
   */
  publishState?: boolean | PublishStateOptions;

  /**
   * Honor frontmatter `permalink` / `slug` when resolving page URLs.
   *
   * Off by default. `true` or `{}` replaces the file-tree URL with
   * `permalink`, or the last path segment with `slug`. Path escape
   * (`../`, absolute filesystem paths, `javascript:`, protocol-relative
   * `//`) is rejected and the file-tree URL is kept. Two pages that
   * resolve to the same URL produce an error; the first page is kept and
   * the later page is skipped.
   *
   * @default false
   */
  permalinks?: boolean | PermalinksOptions;

  /**
   * Inherit missing frontmatter keys from ancestor `_index` files.
   *
   * Off by default. `true` or `{}` fills keys a child does not set.
   * `permalink` and `slug` are never inherited.
   *
   * @default false
   */
  cascade?: boolean | CascadeOptions;

  /**
   * Write static HTML redirect pages for frontmatter aliases and a config map.
   *
   * Off by default. `true` or `{}` enables empty defaults. A path map such as
   * `{ "/old-guide": "/guide" }` enables the feature with that map. Destinations
   * must be same-origin paths (`/` but not `//`) unless `allowExternal` is set.
   * `javascript:`, `data:`, and protocol-relative URLs are ignored.
   * Overlapping sources last-win after trailing slashes are folded.
   *
   * @default false
   */
  redirects?: boolean | RedirectsOptions | Record<string, string>;

  /**
   * Write RSS, Atom, and/or JSON Feed files from a named collection.
   *
   * Off by default. `true` writes all three formats from the `content`
   * collection (or the first configured collection) with a 20-item limit.
   * An object enables the feature and overrides only the fields you set.
   * Requires `ssg.siteUrl`. When that is missing the build continues and a
   * warning is emitted instead of writing files.
   *
   * @default false
   */
  feeds?: boolean | FeedsOptions;

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
   * Enable GFM autolinks and linkify bare URLs.
   * @default true
   */
  autolinks?: boolean;

  /**
   * Enable syntax highlighting for code blocks.
   *
   * When true, fenced and language-tagged inline code is highlighted with the
   * native tree-sitter engine. Token colors are `--octc-shiki-*` custom
   * properties (the `shiki` prefix is historical) so theme-color packages keep
   * working. Languages with no native grammar stay unhighlighted.
   *
   * @default false
   */
  highlight?: boolean;

  /**
   * Code block line annotations for fenced code blocks.
   *
   * This feature is opt-in because it changes rendered code-block markup. Pass
   * `true` to enable ox-content's attribute syntax, or pass an options object to
   * change the meta key or enable VitePress-compatible notation.
   *
   * @example
   * ~~~md
   * ```ts annotate="highlight:1,3-4;warning:6;error:7"
   * const value = compute()
   * ```
   * ~~~
   *
   * @default false
   */
  codeAnnotations?: boolean | CodeAnnotationsOptions;

  /**
   * Expand Obsidian-style `[[page]]` and `[[page|label]]` links.
   *
   * Use this for knowledge-base style content where authors prefer short,
   * document-relative link syntax. Pass an object to override the base URL used
   * when resolving generated hrefs.
   *
   * @default false
   */
  wikiLinks?: boolean | WikiLinkOptions;

  /**
   * Expand `:shortcode:` emoji aliases to Unicode.
   *
   * Built-in aliases cover common emoji names. Provide `custom` entries for
   * project-specific aliases or to override a built-in mapping.
   *
   * @default false
   */
  emojiShortcodes?: boolean | EmojiShortcodeOptions;

  /**
   * Enable markdown-it-attrs style `{#id .class key=value}` attributes.
   *
   * Attribute blocks can be attached to headings, paragraphs, links, images, and
   * other supported Markdown nodes depending on parser context.
   *
   * @default false
   */
  attrs?: boolean | AttrsOptions;

  /**
   * Opt-in `{badge:variant}` inline badges.
   *
   * Passing `true` or an options object enables the built-in variants.
   * Badge text is HTML-escaped. Fenced, indented, and inline code are skipped.
   *
   * @default false
   */
  badges?: boolean | BadgeOptions;

  /**
   * Opt-in `::: tip` custom containers.
   *
   * GitHub-style `> [!NOTE]` callouts stay available without this option.
   * Passing `true` enables the built-in types. Pass an object to register extra
   * types or override titles.
   *
   * @default false
   */
  containers?: boolean | ContainerOptions;

  /**
   * Opt-in figures, captions, and lazy-loaded images.
   *
   * Title text becomes a `<figcaption>`. Optional `{width=N height=M}` on the
   * image is consumed by this feature and does not require `attrs`. Passing
   * `true` or `{}` enables defaults (`lazy: true`).
   *
   * @default false
   */
  images?: boolean | ImageOptions;

  /**
   * Import source snippets into fences with `<<< @/path/to/file.ts{region}`.
   *
   * This is useful for documentation that must stay synchronized with examples
   * in the repository. Use `rootDir` when snippets should resolve from a
   * directory other than the Vite project root.
   *
   * @default false
   */
  codeImports?: boolean | CodeImportOptions;

  /**
   * Inline another Markdown file with `<!-- @include: ./path.md -->`.
   *
   * Expansion happens before Markdown is parsed, so included headings and
   * lists become part of the host document. Relative paths resolve from the
   * current file. `@/` and `/` resolve from `rootDir`. Paths outside
   * `rootDir` are rejected and reported as transform errors.
   *
   * @default false
   */
  includes?: boolean | IncludeOptions;

  /**
   * Opt-in `::: card` / `::: link-card` / `::: card-grid` blocks.
   *
   * Passing `true` enables the defaults. Pass an object to keep the option
   * shape while overriding `enabled`.
   *
   * @default false
   */
  cards?: boolean | CardOptions;

  /**
   * Restyle a `::: steps` wrapper around an ordered list.
   *
   * Disabled when omitted or `false`. `true` and `{}` enable the default
   * step-list markup. Ordinary ordered lists outside `::: steps` are unchanged.
   *
   * @default false
   */
  steps?: boolean | StepsOptions;

  /**
   * Opt-in static directory trees from `file-tree` fences.
   *
   * Passing `true` or `{}` enables the transform. Names are escaped and never
   * read from the filesystem.
   *
   * @default false
   */
  fileTree?: boolean | FileTreeOptions;

  /**
   * Sanitize rendered HTML with safe defaults or explicit allow lists.
   *
   * Enable this for untrusted Markdown. The default allow lists are conservative;
   * pass an options object only when the content model intentionally needs extra
   * tags, attributes, or URL schemes.
   *
   * @default false
   */
  sanitize?: boolean | SanitizeOptions;

  /**
   * Append an "edit this page" link to rendered Markdown.
   *
   * The feature is enabled only when `repoUrl` is provided in the options object.
   * Passing `true` keeps the feature disabled because there is not enough
   * repository information to generate valid links.
   *
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
   *
   * Use this as a lightweight authoring check for missing languages or trailing
   * whitespace inside fences. For project-wide linting, prefer the exported
   * `lintCodeBlocks()` helper or the Markdown lint APIs.
   *
   * @default false
   */
  codeBlockLint?: boolean | CodeBlockLintOptions;

  /**
   * Type-check TypeScript/TSX code fences via tsgo.
   *
   * By default only fences with explicit opt-in metadata are checked. This keeps
   * incidental examples cheap while allowing docs-as-code snippets to fail the
   * build when configured with `mode: 'error'`.
   *
   * @default false
   */
  codeBlockTypecheck?: boolean | CodeBlockTypecheckOptions;

  /**
   * Extract runnable fenced examples for Vitest docs-as-tests harnesses.
   *
   * Collected examples can be written by the docs test helpers and executed as
   * part of a normal Vitest suite.
   *
   * @default false
   */
  docsTests?: boolean | DocsTestOptions;

  /**
   * Enable mermaid diagram rendering.
   * @default false
   */
  mermaid?: boolean;

  /**
   * Enable `$…$` inline and `$$…$$` block math.
   *
   * Currency-like `$` runs, fenced code, indented code, and inline code stay
   * literal. TeX is HTML-escaped into accessible MathML `mtext`.
   *
   * @default false
   */
  math?: boolean | MathOptions;

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
   * Ignored unless `ogImage` or `ssg.generateOgImage` is enabled.
   * @default { vuePlugin: 'vitejs', width: 1200, height: 630, cache: true, concurrency: 1 }
   */
  ogImageOptions?: OgImageOptions;

  /**
   * Custom AST transformers.
   * Transformers run after parsing and before the final JavaScript module is emitted.
   * @default []
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
   * Markdown collection query options.
   *
   * Collections are exposed through `virtual:ox-content/collections`. The
   * default collection is metadata-only and is built by the native Rust
   * manifest builder without rendering every document; add `include` fields
   * only for routes that need raw or rendered content in the query payload.
   *
   * @default content collection for all Markdown files
   */
  collections?: CollectionsOptions | boolean;

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
  siteMaps?: ResolvedSiteMapsOptions;
  publishState?: ResolvedPublishStateOptions;
  permalinks?: ResolvedPermalinksOptions;
  cascade?: ResolvedCascadeOptions;
  redirects?: ResolvedRedirectsOptions;
  feeds?: ResolvedFeedsOptions;
  gfm: boolean;
  footnotes: boolean;
  tables: boolean;
  taskLists: boolean;
  strikethrough: boolean;
  autolinks: boolean;
  highlight: boolean;
  codeAnnotations: ResolvedCodeAnnotationsOptions;
  wikiLinks: ResolvedWikiLinkOptions;
  emojiShortcodes: ResolvedEmojiShortcodeOptions;
  attrs: ResolvedAttrsOptions;
  badges: ResolvedBadgeOptions;
  containers: ResolvedContainerOptions;
  images: ResolvedImageOptions;
  codeImports: ResolvedCodeImportOptions;
  includes: ResolvedIncludeOptions;
  cards: ResolvedCardOptions;
  steps: ResolvedStepsOptions;
  fileTree: ResolvedFileTreeOptions;
  sanitize: ResolvedSanitizeOptions;
  editThisPage: ResolvedEditThisPageOptions;
  cjkEmphasis: boolean;
  codeBlockLint: ResolvedCodeBlockLintOptions;
  codeBlockTypecheck: ResolvedCodeBlockTypecheckOptions;
  docsTests: ResolvedDocsTestOptions;
  mermaid: boolean;
  math: ResolvedMathOptions;
  frontmatter: boolean;
  toc: boolean;
  tocMaxDepth: number;
  ogImage: boolean;
  ogImageOptions: ResolvedOgImageOptions;
  transformers: MarkdownTransformer[];
  docs: ResolvedDocsOptions | false;
  search: ResolvedSearchOptions;
  collections: ResolvedCollectionsOptions;
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
   * Pass `{ fetch: true }` to fetch the post body, author, and self-hosted
   * media at build time. Fetch failures fall back to the link-only card.
   * @default false
   */
  twitter?: boolean | TwitterEmbedOptions;

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
  twitter: TwitterEmbedOptions | false;
  bluesky: boolean;
  webContainer: boolean;
}

/**
 * Options for opt-in `{badge:variant}` inline badges.
 */
export interface BadgeOptions {
  /**
   * Enable the badge transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved inline-badge transform options.
 */
export interface ResolvedBadgeOptions {
  enabled: boolean;
}

/**
 * Options for opt-in `::: type` custom containers.
 */
export interface ContainerOptions {
  /**
   * Enable the container transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
  /**
   * Extra or overriding container types.
   *
   * Keys must be ASCII identifiers (`[A-Za-z0-9_-]+`). Unknown hostile names
   * are ignored.
   */
  types?: Record<string, ContainerTypeOptions>;
}

/**
 * Per-type container presentation.
 */
export interface ContainerTypeOptions {
  /** Title used when the opener does not set one. */
  title?: string;
  /** `"details"` renders `<details>`/`<summary>`; anything else is a `<div>`. */
  tag?: "div" | "details";
}

/**
 * Resolved custom-container transform options.
 */
export interface ResolvedContainerOptions {
  enabled: boolean;
  types: Record<string, ContainerTypeOptions>;
}

/**
 * Options for opt-in figures, captions, and lazy images.
 */
export interface ImageOptions {
  /**
   * Add `loading="lazy"` to transformed images.
   *
   * @default true
   */
  lazy?: boolean;
}

/**
 * Resolved image transform options.
 */
export interface ResolvedImageOptions {
  enabled: boolean;
  lazy: boolean;
}

/**
 * Options for expanding Obsidian-style wiki links.
 *
 * The transform accepts `[[target]]` and `[[target|label]]` syntax and rewrites
 * it to regular links before rendering. It is intentionally small: path
 * resolution is based on the configured base URL rather than a full backlink
 * graph.
 */
export interface WikiLinkOptions {
  /**
   * Base URL prepended to resolved wiki-link targets.
   *
   * When omitted, the top-level `base` option is used.
   *
   * @default options.base
   */
  baseUrl?: string;
}

/**
 * Resolved wiki-link transform options.
 */
export interface ResolvedWikiLinkOptions {
  enabled: boolean;
  baseUrl: string;
}

/**
 * Options for expanding `:shortcode:` emoji aliases.
 *
 * The transform replaces recognized shortcode tokens with their Unicode emoji
 * equivalents during Markdown transformation. Unknown shortcodes are left
 * untouched so colon-delimited text can still be used by other tools.
 */
export interface EmojiShortcodeOptions {
  /**
   * Custom shortcode map merged with the built-in emoji aliases.
   *
   * Keys should omit the surrounding colons.
   *
   * @example
   * ```ts
   * custom: { shipit: '\u{1F6A2}' }
   * ```
   *
   * @default {}
   */
  custom?: Record<string, string>;
}

/**
 * Resolved emoji-shortcode transform options.
 */
export interface ResolvedEmojiShortcodeOptions {
  enabled: boolean;
  custom: Record<string, string>;
}

/**
 * Options for opt-in `$…$` / `$$…$$` math.
 */
export interface MathOptions {
  /**
   * Enable the math transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved math transform options.
 */
export interface ResolvedMathOptions {
  enabled: boolean;
}

/**
 * Options for markdown-it-attrs style attribute blocks.
 *
 * Attribute blocks let authors attach IDs, classes, and key/value attributes to
 * nearby Markdown nodes with syntax such as `{#install .lead}`.
 */
export interface AttrsOptions {
  /**
   * Enable the attrs transform when an options object is supplied.
   *
   * Set to `false` to keep the object shape while disabling the transform.
   * This is mainly useful for config merging where callers want to preserve a
   * stable object structure.
   *
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved attrs transform options.
 */
export interface ResolvedAttrsOptions {
  enabled: boolean;
}

/**
 * Options for importing source snippets into code fences.
 *
 * The transform resolves `<<<` imports before code highlighting and other
 * code-block features run. Imported snippets therefore behave like ordinary
 * fenced code in later stages.
 */
export interface CodeImportOptions {
  /**
   * Directory used to resolve `<<<` imports.
   *
   * When omitted, imports resolve from the Vite project root and configured aliases.
   *
   * @example
   * ```ts
   * rootDir: 'examples'
   * ```
   *
   * @default undefined
   */
  rootDir?: string;
}

/**
 * Resolved code-import transform options.
 */
export interface ResolvedCodeImportOptions {
  enabled: boolean;
  rootDir?: string;
}

/**
 * Options for inlining Markdown files with `<!-- @include: PATH -->`.
 *
 * Relative paths resolve from the current file. `@/` and leading `/` resolve
 * from `rootDir`. After canonicalize, paths outside `rootDir` are rejected.
 */
export interface IncludeOptions {
  /**
   * Directory used to resolve `@/` and absolute include paths.
   *
   * When omitted, includes resolve from the Vite project root.
   *
   * @default undefined
   */
  rootDir?: string;
}

/**
 * Resolved Markdown-include transform options.
 */
export interface ResolvedIncludeOptions {
  enabled: boolean;
  rootDir?: string;
}

/**
 * Options for opt-in `::: card` / `::: link-card` / `::: card-grid` blocks.
 */
export interface CardOptions {
  /**
   * Enable the card transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved card transform options.
 */
export interface ResolvedCardOptions {
  enabled: boolean;
}

/**
 * Options for opt-in `::: steps` ordered lists.
 */
export interface StepsOptions {
  /**
   * Enable the steps transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved step-list transform options.
 */
export interface ResolvedStepsOptions {
  enabled: boolean;
}

/**
 * Options for opt-in `file-tree` fences.
 */
export interface FileTreeOptions {
  /**
   * Enable the file-tree transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved file-tree transform options.
 */
export interface ResolvedFileTreeOptions {
  enabled: boolean;
}

/**
 * Options for sanitizing rendered HTML.
 *
 * Sanitization happens after Markdown is rendered to HTML. This makes it useful
 * for user-authored content, but consumers should avoid enabling extra tags or
 * schemes unless the rendered output explicitly requires them.
 */
export interface SanitizeOptions {
  /**
   * Allowed HTML tag names. Omit to use the built-in safe tag allow list.
   *
   * Provide a full replacement list, not a list of additions.
   *
   * @default undefined
   */
  allowedTags?: string[];

  /**
   * Allowed HTML attribute names. Omit to use the built-in safe attribute allow list.
   *
   * Provide a full replacement list, not a list of additions.
   *
   * @default undefined
   */
  allowedAttributes?: string[];

  /**
   * Allowed URL schemes for link-like attributes.
   *
   * Omit to use the built-in safe scheme allow list.
   *
   * @default undefined
   */
  allowedUrlSchemes?: string[];
}

/**
 * Resolved sanitize transform options.
 */
export interface ResolvedSanitizeOptions {
  enabled: boolean;
  allowedTags?: string[];
  allowedAttributes?: string[];
  allowedUrlSchemes?: string[];
}

/**
 * Options for appending an "edit this page" link.
 *
 * The generated link points at the source Markdown file rather than the emitted
 * HTML route. Configure `branch` and `rootDir` to match the repository layout
 * users should edit.
 */
export interface EditThisPageOptions {
  /**
   * Repository URL used to build edit links.
   *
   * The transform is enabled only when this value is provided.
   *
   * @example
   * ```ts
   * repoUrl: 'https://github.com/owner/project'
   * ```
   */
  repoUrl: string;

  /**
   * Branch used in generated edit links.
   *
   * Use the branch that accepts documentation changes, not necessarily the
   * branch that produced the deployed site.
   *
   * @default 'main'
   */
  branch?: string;

  /**
   * Source root inside the repository, used before the page path.
   *
   * Set this when `srcDir` is nested in a package or docs workspace.
   *
   * @default undefined
   */
  rootDir?: string;

  /**
   * Link text rendered in the page footer.
   *
   * Keep this short; the default theme renders it as a compact footer action.
   *
   * @default 'Edit this page'
   */
  label?: string;
}

/**
 * Resolved edit-link transform options.
 */
export interface ResolvedEditThisPageOptions {
  enabled: boolean;
  repoUrl?: string;
  branch: string;
  rootDir?: string;
  label: string;
}

/**
 * Options for linting fenced code blocks during Markdown transforms.
 *
 * These checks are intentionally local to each fence. They do not execute code
 * or parse a project graph, so they are safe to run during normal Markdown
 * transformation.
 */
export interface CodeBlockLintOptions {
  /**
   * Languages to lint. Omit to lint every fenced block language.
   *
   * Language names are compared case-insensitively.
   *
   * @default undefined
   */
  languages?: string[];

  /**
   * Require every fenced code block to declare a language.
   *
   * This is helpful for documentation sites where every example should be
   * highlighted and searchable by language.
   *
   * @default false
   */
  requireLanguage?: boolean;

  /**
   * Report trailing whitespace inside fenced code blocks.
   *
   * The check reports the exact line and column range inside the fence content.
   *
   * @default true
   */
  trailingSpaces?: boolean;

  /**
   * Diagnostic severity for lint failures.
   *
   * Use `'error'` when code-block lint failures should fail the build.
   *
   * @default 'warn'
   */
  mode?: "warn" | "error";
}

/**
 * Resolved code-block lint options.
 */
export interface ResolvedCodeBlockLintOptions {
  enabled: boolean;
  languages?: string[];
  requireLanguage: boolean;
  trailingSpaces: boolean;
  mode: "warn" | "error";
}

/**
 * Options for type-checking TypeScript and TSX fenced code blocks.
 *
 * Type-checking writes matching snippets to a temporary directory and invokes
 * `tsgo`. It is best suited for concise examples that should stay synchronized
 * with the public TypeScript API.
 */
export interface CodeBlockTypecheckOptions {
  /**
   * Fence languages to type-check.
   *
   * Language names are compared case-insensitively.
   *
   * @default ['ts', 'tsx']
   */
  languages?: string[];

  /**
   * Require an opt-in fence meta marker before type-checking.
   *
   * When enabled, only fences with metadata such as `typecheck` or `twoslash`
   * are checked.
   *
   * @default true
   */
  requireMeta?: boolean;

  /**
   * Command used to run the TypeScript checker.
   *
   * Override this for package-manager scripts or workspace-local binaries.
   *
   * @default 'tsgo'
   */
  tsgoCommand?: string;

  /**
   * Diagnostic severity for type-check failures.
   *
   * Use `'error'` to fail the Markdown transform on broken snippets.
   *
   * @default 'warn'
   */
  mode?: "warn" | "error";
}

/**
 * Resolved code-block type-check options.
 */
export interface ResolvedCodeBlockTypecheckOptions {
  enabled: boolean;
  languages: string[];
  requireMeta: boolean;
  tsgoCommand: string;
  mode: "warn" | "error";
}

/**
 * Options for extracting fenced examples into docs-as-tests fixtures.
 *
 * The extractor collects code fences that can be written into test files and
 * executed by the exported docs test harness helpers.
 */
export interface DocsTestOptions {
  /**
   * Fence languages to collect as runnable examples.
   *
   * Language names are compared case-insensitively.
   *
   * @default ['js', 'jsx', 'ts', 'tsx']
   */
  languages?: string[];

  /**
   * Require an opt-in fence meta marker before collecting an example.
   *
   * When enabled, only fences marked with metadata such as `test`, `runnable`,
   * `vitest`, or `docs-test` are collected.
   *
   * @default true
   */
  requireMeta?: boolean;
}

/**
 * Resolved docs-as-tests extraction options.
 */
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
 *
 * The generator extracts JSDoc/TSDoc comments from JavaScript and TypeScript
 * source files, normalizes the declarations, and writes Markdown plus optional
 * navigation metadata. The defaults are optimized for documenting a package's
 * public `src` tree without exposing private implementation details.
 */
export interface DocsOptions {
  /**
   * Enable source documentation generation.
   *
   * The top-level `docs` option is opt-out: omitting it enables docs generation
   * with defaults, while `docs: false` disables the docs plugin entirely.
   *
   * @default true
   */
  enabled?: boolean;

  /**
   * Source directories to scan for documentation.
   *
   * Paths are resolved from the Vite project root before applying `include` and
   * `exclude` patterns.
   *
   * @default ['./src']
   */
  src?: string[];

  /**
   * Output directory for generated documentation.
   *
   * The path is resolved from the Vite project root. Markdown pages, `docs.json`,
   * and generated navigation metadata are written under this directory.
   *
   * @default 'docs/api'
   */
  out?: string;

  /**
   * Glob patterns for files to include.
   *
   * Patterns are evaluated inside each `src` directory.
   *
   * @default ['**\/*.ts', '**\/*.tsx', '**\/*.js', '**\/*.jsx', '**\/*.mts', '**\/*.mjs', '**\/*.cts', '**\/*.cjs']
   */
  include?: string[];

  /**
   * Glob patterns for files to exclude.
   *
   * Excludes run after `include` matching and should cover tests, generated
   * files, and implementation-only entry points.
   *
   * @default ['**\/*.test.*', '**\/*.spec.*', 'node_modules']
   */
  exclude?: string[];

  /**
   * Public API entry points used to group re-exported docs.
   *
   * When omitted, docs are generated from the discovered source files without
   * entry-point grouping.
   *
   * Use entry points when a package exposes a smaller public surface than its
   * source tree. Re-exported declarations are grouped under the entry point that
   * exposes them.
   *
   * @default undefined
   */
  entryPoints?: DocsEntryPoint[];

  /**
   * Output format.
   *
   * `markdown` is the primary supported format. `json` and `html` are reserved
   * for consumers that want to post-process extracted documentation data.
   *
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
   * Reserved for future use; current generated API pages do not emit this TOC.
   * @default false
   */
  toc?: boolean;

  /**
   * Group documentation by file or category.
   * @default 'file'
   */
  groupBy?: "file" | "category";

  /**
   * GitHub repository URL for source code links.
   *
   * When provided, generated documentation includes links back to the source
   * declaration lines.
   *
   * @example
   * ```ts
   * githubUrl: 'https://github.com/ubugeeei-prod/ox-content'
   * ```
   *
   * @default undefined
   */
  githubUrl?: string;

  /**
   * Internal documentation link style.
   *
   * Use `markdown` for generated `.md` targets and `clean` for route-style links
   * consumed by static-site frameworks.
   *
   * @default 'markdown'
   */
  linkStyle?: "markdown" | "clean";

  /**
   * Route prefix used by generated documentation links and nav metadata.
   *
   * Nav metadata falls back to `/api` when this is not set.
   *
   * @default undefined
   */
  basePath?: string;

  /**
   * Generated Markdown output path strategy.
   *
   * `flat` emits one page per source module or category. `typedoc` emits
   * TypeDoc-like module, kind, and symbol pages for larger API references.
   *
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
   * Use `*` as the insertion point for unlisted groups.
   * @default undefined
   */
  groupOrder?: string[];

  /**
   * TypeDoc-style sort strategies applied to entries and members.
   * Strategies run in order; later strategies break ties from earlier ones.
   * @default undefined
   */
  sort?: DocsSortStrategy[];

  /**
   * Preserve caller-provided entry point order when false.
   * @default true
   */
  sortEntryPoints?: boolean;

  /**
   * TypeDoc-style declaration kind ranking for module sections and nav groups.
   * @default undefined
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
 *
 * Entries represent top-level declarations such as functions, classes,
 * interfaces, type aliases, enums, variables, and modules. Members of compound
 * declarations are stored in `members`.
 */
export interface DocEntry {
  /** Exported or declared symbol name. */
  name: string;

  /** Normalized declaration kind used for grouping and rendering. */
  kind: "function" | "class" | "interface" | "type" | "enum" | "variable" | "module";

  /** Main prose extracted from the leading JSDoc/TSDoc block. */
  description: string;

  /** Function, method, or constructor parameter documentation. */
  params?: ParamDoc[];

  /** Return value documentation for callable declarations. */
  returns?: ReturnDoc;

  /** Exceptions/errors documented with `@throws` / `@exception`. */
  throws?: ThrowsDoc[];

  /** Code examples collected from `@example` tags. */
  examples?: string[];

  /** Additional tags preserved by tag name after known tags are normalized. */
  tags?: Record<string, string>;

  /** True when the entry is marked private or matched by private filtering. */
  private?: boolean;

  /** Source file path relative to the extraction root when available. */
  file: string;

  /** 1-based start line of the declaration in the source file. */
  line: number;

  /** 1-based end line of the declaration in the source file. */
  endLine: number;

  /** Full declaration signature, when the renderer can extract one. */
  signature?: string;

  /** Members belonging to classes, interfaces, object types, and enums. */
  members?: DocMember[];
}

/**
 * A member belonging to a class, interface, type alias, or enum entry.
 */
export interface DocMember {
  /** Member name as it appears in the containing declaration. */
  name: string;

  /** Normalized member kind used for rendering and sorting. */
  kind: "property" | "method" | "constructor" | "getter" | "setter" | "enumMember";

  /** Main prose extracted from the member's documentation comment. */
  description: string;

  /** Full member signature, when available. */
  signature?: string;

  /** Rendered TypeScript type text for properties and enum members. */
  type?: string;

  /** Default value extracted from syntax or `@default` tags. */
  default?: string;

  /** Parameter documentation for methods and constructors. */
  params?: ParamDoc[];

  /** Return value documentation for methods and accessors. */
  returns?: ReturnDoc;

  /** Exceptions/errors documented with `@throws` / `@exception`. */
  throws?: ThrowsDoc[];

  /** True when the member is optional in the source declaration. */
  optional?: boolean;

  /** True when the member is declared readonly. */
  readonly?: boolean;

  /** True when the member is static. */
  static?: boolean;

  /** True when the member is marked private or matched by private filtering. */
  private?: boolean;

  /** Additional tags preserved by tag name after known tags are normalized. */
  tags?: Record<string, string>;

  /** 1-based start line of the member declaration. */
  line: number;

  /** 1-based end line of the member declaration. */
  endLine: number;
}

/**
 * Parameter documentation.
 */
export interface ParamDoc {
  /** Parameter name, including dotted names for destructured properties. */
  name: string;

  /** Rendered TypeScript type text. */
  type: string;

  /** Prose extracted from `@param` / `@arg` documentation. */
  description: string;

  /** True when the parameter is optional. */
  optional?: boolean;

  /** Default value extracted from syntax or `@default` tags. */
  default?: string;
}

/**
 * Return type documentation.
 */
export interface ReturnDoc {
  /** Rendered TypeScript type text for the return value. */
  type: string;

  /** Prose extracted from `@returns` / `@return` documentation. */
  description: string;
}

/**
 * Exception/error documentation.
 */
export interface ThrowsDoc {
  /** Rendered TypeScript type text for the thrown value, when documented. */
  type?: string;

  /** Prose extracted from `@throws` / `@exception` documentation. */
  description: string;
}

/**
 * Extracted documentation for a single file.
 */
export interface ExtractedDocs {
  /** Source module or file identifier used by generated output. */
  file: string;

  /** Optional module-level description extracted from a file header comment. */
  description?: string;

  /** Absolute source path, when available for source links and diagnostics. */
  sourcePath?: string;

  /** Module-level examples collected from a file header comment. */
  examples?: string[];

  /** Module-level tags preserved by tag name. */
  tags?: Record<string, string>;

  /** Top-level documented declarations found in this module. */
  entries: DocEntry[];
}

/**
 * Summary counts emitted with generated documentation data.
 */
export interface DocsSummary {
  /** Number of modules included in the generated payload. */
  modules: number;

  /** Number of top-level entries across all modules. */
  entries: number;

  /** Entry counts grouped by normalized declaration kind. */
  byKind: Record<string, number>;

  /** Number of documented parameters. */
  params: number;

  /** Number of documented return values. */
  returns: number;

  /** Number of collected examples. */
  examples: number;

  /** Number of entries or members marked with `@deprecated`. */
  deprecated: number;
}

/**
 * Machine-readable payload emitted alongside generated docs.
 */
export interface GeneratedDocsData {
  /** Payload schema version. Increment when the JSON shape changes incompatibly. */
  version: 1;

  /** ISO timestamp for the generation run. */
  generatedAt: string;

  /** Aggregate counts useful for dashboards and generated index pages. */
  summary: DocsSummary;

  /** Extracted documentation modules in render order. */
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
// Collection Types
// ============================================

/**
 * Extra payload fields embedded into collection entries.
 *
 * Keep this list small for large sites. By default collection entries contain
 * only route metadata and frontmatter. `body`, `html`, and `toc` increase the
 * virtual module size, and `html`/`toc` require a native Markdown transform.
 */
export type CollectionIncludeField = "body" | "html" | "toc";

/**
 * Collection source configuration.
 */
export interface CollectionOptions {
  /**
   * Glob pattern(s) resolved from `srcDir`.
   *
   * Patterns are filtered by the configured Markdown extensions. Numeric route
   * prefixes such as `1.guide/2.install.md` are stripped from generated `path`.
   *
   * @default all Markdown files
   */
  source?: string | readonly string[];

  /**
   * Optional fields to include in each entry.
   *
   * The default is metadata-only for performance. Use `body` for stripped raw
   * Markdown, `html` for native rendered HTML, and `toc` for the parsed table
   * of contents.
   *
   * @default []
   */
  include?: readonly CollectionIncludeField[];
}

/**
 * Top-level collection definitions.
 */
export type CollectionsOptions = Record<string, CollectionOptions | string | readonly string[]>;

/**
 * Resolved collection definition.
 */
export interface ResolvedCollectionOptions {
  name: string;
  source: string[];
  include: CollectionIncludeField[];
}

/**
 * Resolved collection options.
 */
export interface ResolvedCollectionsOptions {
  enabled: boolean;
  collections: Record<string, ResolvedCollectionOptions>;
}

/**
 * Queryable Markdown collection entry.
 */
export interface CollectionEntry {
  [key: string]: unknown;
  id: string;
  collection: string;
  path: string;
  stem: string;
  source: string;
  extension: string;
  title: string;
  description?: string;
  frontmatter: Record<string, unknown>;
  body?: string;
  html?: string;
  toc?: TocEntry[];
}

/**
 * Generated collection manifest.
 */
export interface CollectionManifest {
  collections: Record<string, CollectionEntry[]>;
}

export type CollectionQueryOperator =
  | "="
  | "=="
  | "!="
  | "<>"
  | ">"
  | ">="
  | "<"
  | "<="
  | "IN"
  | "NOT IN"
  | "BETWEEN"
  | "NOT BETWEEN"
  | "IS NULL"
  | "IS NOT NULL"
  | "LIKE"
  | "NOT LIKE";

export interface CollectionQueryBuilder<T extends CollectionEntry = CollectionEntry> {
  path(path: string): CollectionQueryBuilder<T>;
  select<K extends keyof T>(...fields: K[]): CollectionQueryBuilder<Pick<T, K> & CollectionEntry>;
  where(field: keyof T | string, operator: CollectionQueryOperator, value?: unknown): this;
  where(field: keyof T | string, value: unknown): this;
  andWhere(factory: (query: CollectionQueryBuilder<T>) => void): this;
  orWhere(factory: (query: CollectionQueryBuilder<T>) => void): this;
  order(field: keyof T | string, direction?: "ASC" | "DESC"): this;
  limit(limit: number): this;
  skip(skip: number): this;
  all(): Promise<T[]>;
  first(): Promise<T | null>;
  count(): Promise<number>;
}

// ============================================
// Search Types
// ============================================

/**
 * Options for full-text search.
 *
 * Search indexes are built from Markdown content at build time and loaded by
 * the client runtime from `search-index.json`. Pass `false` to the top-level
 * `search` option to disable both index generation and the virtual search
 * module.
 */
export interface SearchOptions {
  /**
   * Enable search functionality.
   *
   * Set this to `false` when config merging requires an object shape but search
   * should be disabled.
   *
   * @default true
   */
  enabled?: boolean;

  /**
   * Maximum number of search results.
   *
   * This controls client-side result truncation, not the number of documents in
   * the generated index.
   *
   * @default 10
   */
  limit?: number;

  /**
   * Enable prefix matching for autocomplete.
   *
   * Prefix matching applies to the final query token, which keeps normal terms
   * precise while still supporting typeahead-style interactions.
   *
   * @default true
   */
  prefix?: boolean;

  /**
   * Placeholder text for the search input.
   *
   * This value is embedded in the virtual search module for UI consumers.
   *
   * @default 'Search documentation...'
   */
  placeholder?: string;

  /**
   * Keyboard shortcut to focus search (without modifier).
   *
   * Use an empty string to let the UI opt out of registering a shortcut.
   *
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
  /** Stable document identifier used by the search index. */
  id: string;

  /** Human-readable document title. */
  title: string;

  /** URL returned to search consumers. */
  url: string;

  /** Plain-text body content used for scoring and snippets. */
  body: string;

  /** Headings extracted from the document. */
  headings: string[];

  /** Code block text extracted from the document. */
  code: string[];
}

/**
 * Search result structure.
 */
export interface SearchResult {
  /** Matching document identifier. */
  id: string;

  /** Matching document title. */
  title: string;

  /** URL to open when the result is selected. */
  url: string;

  /** Relevance score returned by the BM25 search engine. */
  score: number;

  /** Query terms that matched the document. */
  matches: string[];

  /** Context snippet with highlighted terms when available. */
  snippet: string;

  /** Hierarchical scopes derived from the result URL or document id. */
  scopes?: string[];
}

/**
 * Parsed search query with optional scope prefixes.
 */
export interface ScopedSearchQuery {
  /** Query text after `@scope` prefixes have been removed. */
  text: string;

  /** Deduplicated lowercase scope prefixes requested by the query. */
  scopes: string[];
}

// ============================================
// i18n Types
// ============================================

/**
 * Locale configuration.
 *
 * Locales define the routing and display metadata used by the i18n plugin.
 */
export interface LocaleConfig {
  /** BCP 47 locale tag (e.g., 'en', 'ja', 'zh-Hans'). */
  code: string;

  /** Display name for this locale (e.g., 'English', '日本語'). */
  name: string;

  /**
   * Text direction for rendered pages.
   *
   * @default 'ltr'
   */
  dir?: "ltr" | "rtl";
}

/**
 * i18n (internationalization) options.
 *
 * i18n is opt-in because it changes routing and build-time validation. Set
 * `enabled: true` and configure at least `defaultLocale` / `locales` when the
 * same content tree should serve multiple languages.
 */
export interface I18nOptions {
  /**
   * Enable i18n.
   *
   * The resolver returns `false` unless this is explicitly set to `true`.
   *
   * @default false
   */
  enabled?: boolean;

  /**
   * Path to i18n dictionary directory (relative to project root).
   *
   * Dictionary files are watched in development and checked during builds when
   * `check` is enabled.
   *
   * @default 'content/i18n'
   */
  dir?: string;

  /**
   * Default locale tag.
   *
   * The default locale is added to `locales` automatically when omitted from the
   * list.
   *
   * @default 'en'
   */
  defaultLocale?: string;

  /**
   * Available locales.
   *
   * When omitted, ox-content creates a single locale from `defaultLocale`.
   *
   * @default [{ code: defaultLocale, name: defaultLocale }]
   */
  locales?: LocaleConfig[];

  /**
   * Hide default locale prefix in URLs.
   *
   * When true, `/page` serves the default locale and `/ja/page` serves Japanese.
   * When false, all locales get prefixed: `/en/page`, `/ja/page`.
   *
   * @default true
   */
  hideDefaultLocale?: boolean;

  /**
   * Run i18n checks during build.
   *
   * Checks validate dictionary coverage and translation function usage when the
   * native i18n checker is available.
   *
   * @default true
   */
  check?: boolean;

  /**
   * Translation function names to detect in source code.
   *
   * Add framework-specific wrappers here so build-time checks can find all
   * translation keys.
   *
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
