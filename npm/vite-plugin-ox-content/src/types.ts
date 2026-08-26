/**
 * Type definitions for @ox-content/vite-plugin
 */

import type { ThemeConfig, ResolvedThemeConfig } from "./theme";
import type {
  GitHubOptions,
  GraphvizOptions,
  OgpOptions,
  ProviderArticleEmbedOptions,
  ProviderPackageEmbedOptions,
  ProviderPlaygroundEmbedOptions,
  ProviderVideoEmbedOptions,
  ResolvedGraphvizOptions,
  RedditEmbedOptions,
  TwitterEmbedOptions,
} from "./plugins";
import type {
  CrossReferenceEntry,
  CrossReferencesOptions,
  ResolvedCrossReferencesOptions,
} from "./cross-references";
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
   * Mount generated page routes under this path, independent from `base` and
   * `outDir`.
   *
   * `blog`, `/blog`, and `/blog/` all mount under `/blog`. Page HTML and
   * page-level assets follow the prefix. Root host files (`_redirects`,
   * `_headers`, root feeds, sitemap index) stay at `outDir`. `base` remains
   * the public deployment prefix and is not used as an output mount.
   * Frontmatter `permalink` still wins when permalinks are enabled.
   *
   * Off when omitted.
   *
   * @default undefined
   */
  routePrefix?: string;

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
   * List unique git authors for each page.
   *
   * Off by default. `true` enables names only. An object enables the
   * feature and can set `ignore` and `avatars`. Missing `.git` (for
   * example a published tarball) yields an empty list and does not
   * fail the build.
   *
   * @default false
   */
  contributors?: boolean | ContributorsOptions;

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
   * Emit JSON-LD structured data (`TechArticle`, `WebSite`, and optional
   * `BreadcrumbList`) in the page `<head>`.
   *
   * Disabled when omitted or `false`. `true` enables the defaults. An object
   * enables the feature and can hide BreadcrumbList or supply a publisher.
   * Publisher fields the site does not set are not invented.
   *
   * @default false
   */
  jsonLd?: boolean | JsonLdOptions;

  /**
   * Validate custom page-head descriptors during SSG.
   *
   * `false` / omitted drops invalid values silently. `warn` logs them.
   * `strict` fails the build on unsafe URLs or invalid hreflang.
   *
   * @default false
   */
  headValidation?: false | "warn" | "strict";

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
   * Honor per-page frontmatter chrome flags (`sidebar`, `outline` / `aside`,
   * `footer`, `navbar`, `lastUpdated`, `editLink`).
   *
   * Disabled when omitted or `false`. `true` or `{}` enables the defaults:
   * omitted flags keep current chrome, and `false` hides that region.
   *
   * @default false
   */
  pageChrome?: boolean | Record<string, unknown>;

  /**
   * Publish the original Markdown beside each generated HTML page.
   *
   * Off by default. `true` writes a `.md` companion using the published URL
   * (permalink, locale, base, and output directory) and adds
   * `<link rel="alternate" type="text/markdown">`. An object enables the
   * feature and can turn the alternate link off, or opt in to the default
   * theme's Copy as Markdown control.
   *
   * The companion is a byte-for-byte copy of the source file, including
   * frontmatter. Draft and unlisted pages are never written.
   *
   * @default false
   */
  markdownSource?: boolean | MarkdownSourceOptions;

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
   * Render a static members card grid on pages with `layout: team`.
   *
   * Off by default. `true` enables an empty list. An object enables the
   * feature and supplies `members`. When the option is off, `layout: team`
   * is ignored and the page stays ordinary.
   *
   * @default false
   */
  team?: boolean | TeamOptions;

  /**
   * Opt-in blog index, authors, tags, reading time, and archive.
   *
   * Off by default. `true` enables defaults. An object enables the feature
   * and overrides only the fields you set. Top-level `blog` wins when both
   * are set.
   *
   * @default false
   */
  blog?: boolean | BlogOptions;

  /**
   * Generate a static index for directories that have child pages but no
   * `index.md` / `index.mdx`.
   *
   * Off by default. `true` enables card listings. An object enables the
   * feature and can switch the listing to `list`. Existing content indexes
   * are never overwritten.
   *
   * @default false
   */
  sectionIndex?: boolean | SectionIndexOptions;

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
 * Per-control flags for `ssg.jsonLd`.
 *
 * Omitted fields keep the defaults when the feature itself is enabled.
 */
export interface JsonLdOptions {
  /**
   * Emit `BreadcrumbList` when a visible breadcrumb trail exists.
   *
   * @default true
   */
  breadcrumbs?: boolean;

  /**
   * Optional publisher. Only configured `name` / `url` are written.
   * Logo and other Organization fields are never invented.
   */
  publisher?: JsonLdPublisherOptions;

  /**
   * Page `@type`. Defaults to `TechArticle`.
   */
  type?: JsonLdPageType;

  /**
   * Extra `@graph` nodes. Only objects are kept. The build does not invent
   * fields inside them.
   */
  graph?: Record<string, unknown>[];
}

/** JSON-LD page node `@type`. Unknown values fall back to `TechArticle`. */
export type JsonLdPageType = "TechArticle" | "BlogPosting" | "WebPage";

/**
 * Optional JSON-LD publisher. Empty or omitted fields are left out.
 */
export interface JsonLdPublisherOptions {
  /** Organization name. */
  name?: string;
  /** Organization URL. `javascript:` and other unsafe schemes are dropped. */
  url?: string;
}

/**
 * Resolved JSON-LD options. `false` means no `<script type="application/ld+json">`.
 */
export type ResolvedJsonLd =
  | false
  | {
      breadcrumbs: boolean;
      publisher?: {
        name?: string;
        url?: string;
      };
      type?: JsonLdPageType;
      graph?: Record<string, unknown>[];
    };

/**
 * Resolved SSG options.
 */
export interface ResolvedSsgOptions {
  enabled: boolean;
  extension: string;
  /**
   * Present after `resolveSsgOptions`. Omitted / empty means off.
   */
  routePrefix?: string;
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
  /**
   * Present after `resolveSsgOptions`. Omitted in hand-built fixtures means off.
   */
  contributors?: ResolvedContributors;
  pagination: boolean;
  breadcrumbs: boolean;
  jsonLd: ResolvedJsonLd;
  /**
   * Present after `resolveSsgOptions`. Omitted / `false` means off.
   */
  headValidation?: false | "warn" | "strict";
  readerChrome: ResolvedReaderChrome;
  localeSwitcher: boolean;
  a11y: ResolvedA11y;
  pageChrome: boolean;
  /**
   * Present after `resolveSsgOptions`. Omitted in hand-built fixtures means off.
   */
  markdownSource?: ResolvedMarkdownSourceOptions;
  /**
   * Present after `resolveSsgOptions`. Omitted in hand-built fixtures means off.
   */
  notFound?: ResolvedNotFoundOptions;
  /**
   * Present after `resolveSsgOptions`. Omitted in hand-built fixtures means off.
   */
  team?: ResolvedTeamOptions;
  /**
   * Present after `resolveSsgOptions`. Omitted in hand-built fixtures means off.
   */
  blog?: ResolvedBlogOptions;
  sectionIndex?: ResolvedSectionIndexOptions;
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
 * One link on a team member card.
 */
export interface TeamLink {
  /** Visible label. Escaped in HTML. */
  label: string;
  /** Destination. Only `https:` or a site-relative `/` path is emitted. */
  href: string;
}

/**
 * One person on the team page.
 */
export interface TeamMember {
  /** Display name. Escaped in HTML. */
  name: string;
  /** Optional role or title. Escaped in HTML. */
  role?: string;
  /** Avatar URL. Only `https:` or a site-relative `/` path is emitted. */
  avatar?: string;
  /** Optional profile or social links. */
  links?: TeamLink[];
}

/**
 * Opt-in team / members page.
 */
/**
 * Opt-in git contributor list.
 */
export interface ContributorsOptions {
  /**
   * Author names or emails to omit. Comparison is case-insensitive and
   * matches the full name or the full email.
   */
  ignore?: string[];
  /**
   * When true and a git author email is present, render a Gravatar
   * image from the MD5 of that email. The raw email is never written
   * into HTML. Default is names only.
   */
  avatars?: boolean;
}

/**
 * Resolved git contributor list. `false` means the feature is off.
 */
export type ResolvedContributors =
  | false
  | {
      ignore: string[];
      avatars: boolean;
    };

export interface TeamOptions {
  /**
   * People rendered as static cards on `layout: team` pages.
   * @default []
   */
  members?: TeamMember[];
}

/**
 * Resolved team page options.
 */
export interface ResolvedTeamOptions {
  enabled: boolean;
  members: TeamMember[];
}

/**
 * Listing style for a generated section index.
 */
export type SectionIndexStyle = "list" | "cards";

/**
 * Opt-in generated section index pages.
 */
export interface SectionIndexOptions {
  /**
   * How children are rendered. `cards` is the default when the feature is on.
   * @default "cards"
   */
  style?: SectionIndexStyle;
}

/**
 * Resolved generated section index options.
 */
export interface ResolvedSectionIndexOptions {
  enabled: boolean;
  style: SectionIndexStyle;
}

/**
 * Opt-in web app manifest and service worker written during SSG.
 *
 * Enabling `offline` (the default when the feature is on) injects a tiny
 * client script that registers `sw.js`. Set `offline: false` to keep the
 * manifest without that script.
 */
export interface PwaOptions {
  /**
   * Write `sw.js` and register it from themed pages.
   * @default true
   */
  offline?: boolean;

  /**
   * Manifest `name`. Falls back to `ssg.siteName` when omitted.
   */
  name?: string;

  /**
   * Manifest `short_name`. Falls back to `name` when omitted.
   */
  shortName?: string;

  /**
   * Manifest / meta theme color. Hex (`#rgb` / `#rrggbb`) or a CSS color name.
   * @default "#000000"
   */
  themeColor?: string;

  /**
   * Manifest background color. Hex or a CSS color name.
   * @default "#ffffff"
   */
  backgroundColor?: string;

  /**
   * Manifest `start_url`. Same-origin site paths only (`/`, `/docs/`).
   * Defaults to the Vite `base`.
   */
  startUrl?: string;
}

/**
 * Resolved PWA options.
 */
export interface ResolvedPwaOptions {
  enabled: boolean;
  offline: boolean;
  name?: string;
  shortName?: string;
  themeColor?: string;
  backgroundColor?: string;
  startUrl?: string;
}

/**
 * Opt-in self-hosted Iconify CSS for used icons.
 *
 * Off by default. When enabled, the SSG build resolves Iconify names from
 * installed `@iconify/json` or `@iconify-json/*` packages and emits CSS
 * masks so the published site does not request `api.iconify.design`.
 */
export interface IconsOptions {
  /**
   * CSS emission mode.
   * @default "css-mask"
   */
  mode?: "css-mask";

  /**
   * Class syntax. `"unocss"` emits `icon-[prefix--name]`.
   * @default "unocss"
   */
  syntax?: "unocss";

  /**
   * Glob patterns to scan, or explicit `prefix:name` icons.
   * Entries that look like Iconify names are used as-is (no scan).
   */
  include?: string[];

  /**
   * Iconify names that are always emitted, even when no source mentions them.
   */
  safelist?: string[];
}

/**
 * Resolved icon asset options.
 */
export interface ResolvedIconsOptions {
  enabled: boolean;
  mode: "css-mask";
  syntax: "unocss";
  include: string[];
  safelist: string[];
}

/**
 * Opt-in Markdown source companions written beside generated HTML.
 */
export interface MarkdownSourceOptions {
  /**
   * Add `<link rel="alternate" type="text/markdown">` to generated HTML.
   * @default true
   */
  alternate?: boolean;

  /**
   * Show a page-level Copy as Markdown control in the default theme.
   * The control copies or opens the published companion bytes, including
   * frontmatter. Off unless set, even when companions are enabled.
   * @default false
   */
  copy?: boolean;
}

/**
 * Resolved Markdown source-companion options.
 */
export interface ResolvedMarkdownSourceOptions {
  enabled: boolean;
  alternate: boolean;
  copy: boolean;
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
 * Host that consumes the generated `_redirects` file.
 *
 * Both values write the same `_redirects` body today. The distinct names
 * leave room for provider-specific limits and diagnostics later.
 */
export type RedirectProvider = "netlify" | "cloudflare";

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
   * Host that should receive a `_redirects` file.
   *
   * Omit the field to detect `CF_PAGES=1`, `WORKERS_CI=1`, or `NETLIFY=true`.
   * Local builds and GitHub Actions should set this explicitly. HTML redirect
   * pages are independent of this selector.
   */
  provider?: RedirectProvider;

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
   * Write static HTML fallback pages for ordinary redirect sources.
   *
   * Set `false` when the selected host should consume `_redirects` directly.
   * Wildcard sources never write HTML pages because they are host-rule syntax.
   * @default true
   */
  html?: boolean;

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
  provider?: RedirectProvider;
  headers: boolean;
  json: boolean;
  html: boolean;
  allowExternal: boolean;
}

/**
 * Feed file formats written during SSG.
 */
export type FeedFormat = "rss" | "atom" | "json";

/** One feed item author accepted by programmatic feeds. */
export interface FeedItemAuthor {
  name: string;
  url?: string;
}

export type FeedItemAuthorInput = string | FeedItemAuthor;

/** One JSON Feed / enclosure attachment accepted by programmatic feeds. */
export interface FeedItemAttachment {
  url: string;
  mimeType?: string;
  title?: string;
  sizeInBytes?: number;
  durationInSeconds?: number;
}

/** One collection or programmatic item considered for a generated feed. */
export interface FeedItemInput {
  title?: string;
  description?: string;
  content?: string;
  path?: string;
  loc?: string;
  url?: string;
  id?: string;
  date?: unknown;
  lastUpdated?: unknown;
  draft?: unknown;
  unlisted?: unknown;
  author?: FeedItemAuthorInput;
  authors?: readonly FeedItemAuthorInput[];
  image?: string;
  attachments?: readonly FeedItemAttachment[];
  language?: string;
  frontmatter?: Record<string, unknown>;
}

export interface FeedItemsResolveContext {
  name?: string;
  formats: readonly FeedFormat[];
  path: string;
  siteUrl?: string;
  siteName?: string;
  siteDescription?: string;
  base: string;
  outDir?: string;
}

export type FeedItemsSource =
  | readonly FeedItemInput[]
  | ((
      context: FeedItemsResolveContext,
    ) => readonly FeedItemInput[] | Promise<readonly FeedItemInput[]>);

/**
 * One feed's formats, source, output path, and channel metadata.
 */
export interface FeedChannelOptions {
  /**
   * Feed formats to write.
   * @default ["rss", "atom", "json"]
   */
  formats?: readonly FeedFormat[];

  /**
   * Named collection to publish. Defaults to `content`, or the first
   * configured collection when `content` is absent.
   */
  collection?: string;

  /**
   * Programmatic items for this channel. A channel may set either
   * `collection` or `items`, not both.
   */
  items?: FeedItemsSource;

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

  /** Channel title. Defaults to the SSG site name. */
  title?: string;

  /** Channel description. Defaults to the SSG site description. */
  description?: string;

  /** Channel language (`en`, `ja`, …). Omitted when unset. */
  language?: string;

  /** Channel image URL (RSS image / Atom logo / JSON Feed icon). */
  image?: string;

  /** Favicon URL (Atom icon / JSON Feed favicon). */
  favicon?: string;

  /** Copyright / rights notice. Omitted from JSON Feed. */
  copyright?: string;
}

/**
 * Opt-in RSS / Atom / JSON Feed files written during SSG.
 *
 * A single object is one default feed. A named record or array writes
 * multiple feeds with their own paths and channel metadata.
 */
export type FeedsOptions =
  | FeedChannelOptions
  | readonly FeedChannelOptions[]
  | { [name: string]: FeedChannelOptions };

/**
 * One resolved feed channel.
 */
export interface ResolvedFeedChannel {
  name?: string;
  formats: readonly FeedFormat[];
  collection?: string;
  items?: FeedItemsSource;
  limit: number;
  path: string;
  title?: string;
  description?: string;
  language?: string;
  image?: string;
  favicon?: string;
  copyright?: string;
}

/**
 * Resolved feed options.
 *
 * Legacy `true` / single-object configs keep one channel on the top-level
 * fields. A named record or array also sets `feeds` to every channel.
 */
export interface ResolvedFeedsOptions extends ResolvedFeedChannel {
  enabled: boolean;
  feeds?: ResolvedFeedChannel[];
}

/**
 * One person in the `blog.authors` map.
 */
export interface BlogAuthor {
  /** Display name. Escaped in HTML. */
  name: string;
  /** Optional short bio. Escaped in HTML. */
  bio?: string;
  /** Profile URL. Only `https:` or a site-relative `/` path is emitted. */
  url?: string;
}

/**
 * Opt-in blog index, authors, tags, reading time, and archive.
 */
export interface BlogOptions {
  /**
   * Named collection of posts. Defaults to a collection named `blog`, or
   * the only configured collection. Required when several collections exist
   * and none is named `blog`.
   */
  collection?: string;

  /**
   * Author records keyed by the frontmatter `author` / `authors` value.
   * @default {}
   */
  authors?: Record<string, BlogAuthor>;

  /**
   * Posts per index page, newest first.
   * @default 10
   */
  pageSize?: number;

  /**
   * External RSS / Atom sources merged into the blog index at build time.
   * Empty / omitted fetches nothing. Only these URLs are requested.
   * @default []
   */
  feeds?: Array<string | BlogFeedSource>;
}

/**
 * One configured external blog feed.
 */
export interface BlogFeedSource {
  /** Absolute `https:` feed URL. */
  url: string;
  /** Default language applied when an item omits one. */
  language?: string;
  /** Default author applied when an item omits one. */
  author?: string;
  /**
   * Failed fetch / parse handling for this source.
   * `warn` skips the source. `error` fails the build after other sources run.
   * @default "warn"
   */
  onError?: BlogFeedFailurePolicy;
}

/** How a failed external feed source is reported. */
export type BlogFeedFailurePolicy = "warn" | "error";

/**
 * Resolved blog options.
 */
export interface ResolvedBlogOptions {
  enabled: boolean;
  collection?: string;
  authors: Record<string, BlogAuthor>;
  pageSize: number;
  feeds: ResolvedBlogFeedSource[];
}

/**
 * Resolved external blog feed source.
 */
export interface ResolvedBlogFeedSource {
  url: string;
  language?: string;
  author?: string;
  onError: BlogFeedFailurePolicy;
}

/**
 * Opt-in term list pages, per-term pages, and related-page lists.
 */
export interface TaxonomiesOptions {
  /**
   * Frontmatter keys (and URL prefixes) to read terms from.
   * @default ["tags", "categories"]
   */
  taxonomies?: string[];

  /**
   * Maximum related pages injected into a source page.
   * @default 5
   */
  relatedLimit?: number;
}

/**
 * Resolved taxonomy options.
 */
export interface ResolvedTaxonomiesOptions {
  enabled: boolean;
  taxonomies: string[];
  relatedLimit: number;
}

/** Banner shown on pages that belong to one documented version. */
export type VersionBannerKind = "unreleased" | "unmaintained";

/**
 * One published or snapshot version of a docs tree.
 */
export interface VersionEntry {
  /** Stable id used as `versions.current`. */
  id: string;
  /** Header label. Escaped before it is rendered. */
  label: string;
  /**
   * URL prefix without slashes (`"2.90"`, `"next"`). Empty string is the
   * site root.
   */
  prefix: string;
  /**
   * Snapshot directory relative to the Vite root. Omitted entries use the
   * live `srcDir` and are not copied. Historical dirs are read-only.
   */
  dir?: string;
  /** Optional status banner for pages in this version. */
  banner?: VersionBannerKind | false;
}

/**
 * Opt-in documentation versioning.
 *
 * Off by default. `true` enables a single current entry. An object enables
 * the feature and overrides only the fields you set.
 */
export interface VersionsOptions {
  /** Id of the live tree being built from `srcDir`. */
  current?: string;
  /** Render the header version dropdown. @default true */
  switcher?: boolean;
  /** Show unreleased / unmaintained badges in the dropdown. @default true */
  badge?: boolean;
  /** Declared versions. Historical snapshots must set `dir`. */
  entries?: VersionEntry[];
}

/**
 * Resolved documentation versioning.
 */
export interface ResolvedVersionsOptions {
  enabled: boolean;
  current: string;
  switcher: boolean;
  badge: boolean;
  entries: ResolvedVersionEntry[];
}

/**
 * One resolved version after prefix and banner sanitization.
 */
export interface ResolvedVersionEntry {
  id: string;
  label: string;
  prefix: string;
  dir?: string;
  banner: VersionBannerKind | false;
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
   * Write a paginated blog index, tag pages, and yearly/monthly archive,
   * and inject author / reading-time chrome on posts.
   *
   * Off by default. `true` uses the `blog` collection when it exists,
   * otherwise the only configured collection, with pageSize 10.
   * An object enables the feature and overrides only the fields you set.
   * Also accepted as `ssg.blog`; the top-level option wins when both are set.
   *
   * @default false
   */
  blog?: boolean | BlogOptions;

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
   * Write a web app manifest and an optional service worker.
   *
   * Off by default. `true` writes `manifest.webmanifest` and `sw.js`, and
   * injects a tiny client script that registers the worker on themed pages.
   * An object enables the feature and can set `offline: false` to keep the
   * manifest without caching or that script. This adds client JavaScript
   * when offline is on. Requires `ssg.siteUrl`. When that is missing the
   * build continues and a warning is emitted instead of writing files.
   *
   * @default false
   */
  pwa?: boolean | PwaOptions;

  /**
   * Generate self-hosted Iconify CSS for used and safelisted icons.
   *
   * Off by default. `true` or `{}` enables CSS-mask emission. Install
   * `@iconify/json` or individual `@iconify-json/*` packages so the build
   * can resolve collections without a network request.
   *
   * @default false
   */
  icons?: boolean | IconsOptions;

  /**
   * Write tag/category term pages and inject related-page lists.
   *
   * Off by default. `true` reads frontmatter `tags` and `categories` and
   * writes list pages, per-term pages, and up to 5 related links on pages
   * that share a term. An object enables the feature and overrides only
   * the fields you set. Term slugs are `[a-z0-9-]` and every label is
   * HTML-escaped.
   *
   * @default false
   */
  taxonomies?: boolean | TaxonomiesOptions;

  /**
   * Prefix live docs, emit frozen snapshot trees, and render a header
   * version dropdown.
   *
   * Off by default. `true` enables a single current entry. An object
   * enables the feature and lists additional versions. Historical
   * snapshot directories are read, never rewritten.
   *
   * @default false
   */
  versions?: boolean | VersionsOptions;

  /**
   * Enable GitHub Flavored Markdown extensions.
   * @default true
   */
  gfm?: boolean;

  /**
   * Enable MDX JSX, ESM, and expressions.
   *
   * When omitted, MDX is enabled for `.mdx` files only. Set `true` to enable
   * it for every configured extension or `false` to keep `.mdx` on the plain
   * Markdown path.
   * @default inferred from the source extension
   */
  mdx?: boolean;

  /**
   * Enable footnotes.
   * @default true
   */
  footnotes?: boolean;

  /**
   * Render footnotes as a semantic ordered section with numeric markers.
   *
   * Source identifiers are used only for lookup and slugs. Visible markers
   * are 1, 2, … in document order, and definitions emit as
   * `<section class="footnotes"><ol><li>…`.
   *
   * Off by default so current alpha HTML stays stable.
   * @default false
   */
  semanticFootnotes?: boolean;

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
   * native tree-sitter engine. Token colors are `--octc-syntax-*` custom
   * properties so theme-color packages resolve highlighting. Languages with no
   * native grammar stay unhighlighted.
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
   * Opt-in labeled cross-references for headings, figures/images, and tables.
   *
   * References such as `@sec-install`, `@fig-pipeline`, and `@tbl-options`
   * become links to matching `id` attributes generated by `attrs` or native
   * Markdown rendering. Missing labels, duplicate labels, and prefix/type
   * mismatches fail by default and can be downgraded to warnings.
   *
   * @default false
   */
  crossReferences?: boolean | CrossReferencesOptions;

  /**
   * Alias for `crossReferences`.
   *
   * @default false
   */
  xrefs?: boolean | CrossReferencesOptions;

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
   * Opt-in `<NotByAI />` authorship disclosure badge.
   *
   * Passing `true` or an options object emits the official Not By AI light/dark
   * artwork as static HTML. This is not a status badge — see `badges` for
   * `{badge:tip}` labels. Disabled when omitted. Fenced, indented, and inline
   * code plus HTML comments are skipped.
   *
   * @default false
   */
  notByAi?: boolean | NotByAiOptions;

  /**
   * Opt-in `{kbd:...}` inline keyboard keys.
   *
   * Passing `true` or an options object enables `{kbd:Ctrl+K}` and
   * `{kbd:Cmd Shift P}`. Key labels are HTML-escaped. Fenced, indented,
   * inline, and raw code, plus HTML comments, are skipped. Aliases come
   * from build config, not the runtime user agent.
   *
   * @default false
   */
  keyboardKeys?: boolean | KeyboardKeysOptions;

  /**
   * Opt-in abbreviation and glossary expansion.
   *
   * Passing `true` or an options object expands `*[LSP]: Language Server Protocol`
   * and config `terms` into `<abbr class="ox-abbr">`. Matching uses Unicode word
   * boundaries. Fenced, indented, inline, and raw code, HTML comments, and
   * existing links are skipped. There is no client JavaScript.
   *
   * @default false
   */
  abbreviations?: boolean | AbbreviationsOptions;

  /**
   * Opt-in PHP Markdown Extra / mdBook-style definition lists.
   *
   * Passing `true` or an options object turns
   * `Term` / `: definition` source into semantic `<dl>` markup.
   * Disabled when omitted. Fenced, indented, and inline code are skipped.
   * Invalid or ambiguous forms stay ordinary paragraphs or lists.
   *
   * @default false
   */
  definitionLists?: boolean | DefinitionListOptions;

  /**
   * Opt-in `{link:...}` rich magic links.
   *
   * Passing `true` or an options object enables GitHub-user, alias, and
   * explicit `label|url` forms. Attributes and text are HTML-escaped.
   * Fenced, indented, inline, and raw code, plus already-linked text, are
   * skipped. The transform does not make network requests unless an explicit
   * favicon template is enabled (still URL-only; no fetch at transform time).
   *
   * @default false
   */
  magicLinks?: boolean | MagicLinkOptions;

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
   * Opt-in static `::: gallery` image groups.
   *
   * Each non-empty line inside the block must be a Markdown image, optionally
   * as a list item. Image titles become item captions, and the block title or
   * caption metadata becomes the gallery caption. Passing `true` or `{}`
   * enables strict empty-gallery and missing-alt diagnostics.
   *
   * @default false
   */
  imageGalleries?: boolean | ImageGalleryOptions;

  /**
   * Opt-in static `::: timeline` milestone lists.
   *
   * Timeline blocks render dated or undated milestones from Markdown-only
   * `::: timeline` blocks. Items can carry `status`, `label`, and `href`
   * metadata while nested Markdown stays searchable and static.
   *
   * @default false
   */
  timelines?: boolean | TimelineOptions;

  /**
   * Opt-in static `::: if` / `::: else` blocks.
   *
   * Conditions are evaluated from `conditionalBlocks.values` and page
   * frontmatter before Markdown is parsed. Non-selected branches are excluded
   * from rendered HTML, TOC, and generated search payloads. The expression
   * language supports `==`, `!=`, `in`, `and`, `or`, parentheses, string /
   * number / boolean / null literals, and array literals. No JavaScript is
   * executed.
   *
   * @default false
   */
  conditionalBlocks?: boolean | ConditionalBlockOptions;

  /**
   * Opt-in page-bundle resources and build-time image processing.
   *
   * Off by default. `true` or `{}` treats each page directory as a bundle:
   * sibling images are addressable with relative URLs. Query-string
   * resize/crop/format transforms run at build time and are cached by
   * source mtime plus transform params. Paths that leave the page
   * directory or `srcDir` are rejected. Missing sources fail the build
   * when `missing` is `"error"` (the default when enabled).
   * `dedupe` is off unless set; it does not turn on with `true` / `{}`.
   *
   * This is separate from `images`, which only adds figures, captions,
   * and lazy-loading.
   *
   * @default false
   */
  resources?: boolean | ResourcesOptions;

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
   * Inline a parameterized Markdown partial with
   * `<!-- @partial: ./_partials/install.md package="ox-content" -->`.
   *
   * Disabled when omitted. `{{ name }}` substitutions are HTML-escaped.
   * Missing parameters stay literal unless `missing` is `"error"`. Existing
   * `<!-- @include: -->` behavior is unchanged.
   *
   * @default false
   */
  partials?: boolean | PartialsOptions;

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
   * Opt-in VitePress-style `::: code-group` fence groups.
   *
   * Passing `true` or `{}` enables rewriting labeled fences into the
   * existing no-JS tab widget. Omitted or `false` leaves the source on
   * the normal Markdown/container path.
   *
   * @default false
   */
  codeGroups?: boolean | CodeGroupOptions;

  /**
   * Opt-in static directory trees from `file-tree` fences.
   *
   * Passing `true` or `{}` enables the transform. Names are escaped and never
   * read from the filesystem. Directories with children open and close with
   * `<details>`. Icons are on by default and can be replaced from site config.
   *
   * @default false
   */
  fileTree?: boolean | FileTreeOptions;

  /**
   * Opt-in static tables from `csv-table` / `json-table` fences.
   *
   * Passing `true` or `{}` enables the transform. Inline CSV/JSON becomes a
   * semantic `<table>` with a responsive wrapper. `src` or a single path body
   * can import `@/data/options.csv` or `./options.json`. Paths cannot escape
   * the content/project root with `..`. Missing imports use `missing`.
   *
   * @default false
   */
  dataTables?: boolean | DataTableOptions;

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
   * Attach build-time TypeScript hover overlays to opted-in fences.
   *
   * Off by default. `true` or `{}` enables the feature. Only `ts` / `tsx`
   * fences tagged `twoslash` receive payloads. Types are generated during
   * the Markdown transform; no TypeScript compiler is shipped to the browser.
   *
   * @default false
   */
  typedHover?: boolean | TypedHoverOptions;

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
   * Render `dot` / `graphviz` fenced blocks to static SVG with Graphviz.
   * Pass an object to configure the renderer command and failure policy.
   *
   * @default false
   */
  graphviz?: boolean | GraphvizOptions;

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
   * Append a visible heading permalink (`<a class="header-anchor" href="#id">`).
   *
   * Reuses the generated heading id. Default off. Theme
   * `headingPermalink: "hover" | "always"` changes only CSS visibility.
   *
   * @default false
   */
  headingPermalinks?: boolean | HeadingPermalinksOptions;

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
  blog?: ResolvedBlogOptions;
  feeds?: ResolvedFeedsOptions;
  pwa?: ResolvedPwaOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  icons?: ResolvedIconsOptions;
  taxonomies?: ResolvedTaxonomiesOptions;
  versions?: ResolvedVersionsOptions;
  resources?: ResolvedResourcesOptions;
  gfm: boolean;
  mdx?: boolean;
  footnotes: boolean;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  semanticFootnotes?: boolean;
  tables: boolean;
  taskLists: boolean;
  strikethrough: boolean;
  autolinks: boolean;
  highlight: boolean;
  codeAnnotations: ResolvedCodeAnnotationsOptions;
  wikiLinks: ResolvedWikiLinkOptions;
  emojiShortcodes: ResolvedEmojiShortcodeOptions;
  attrs: ResolvedAttrsOptions;
  crossReferences: ResolvedCrossReferencesOptions;
  badges: ResolvedBadgeOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  notByAi?: ResolvedNotByAiOptions;
  keyboardKeys?: ResolvedKeyboardKeysOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  abbreviations?: ResolvedAbbreviationsOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  definitionLists?: ResolvedDefinitionListOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  magicLinks?: ResolvedMagicLinkOptions;
  containers: ResolvedContainerOptions;
  images: ResolvedImageOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  imageGalleries?: ResolvedImageGalleryOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  timelines?: ResolvedTimelineOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  conditionalBlocks?: ResolvedConditionalBlockOptions;
  codeImports: ResolvedCodeImportOptions;
  includes: ResolvedIncludeOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  partials?: ResolvedPartialsOptions;
  cards: ResolvedCardOptions;
  steps: ResolvedStepsOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  codeGroups?: ResolvedCodeGroupOptions;
  fileTree: ResolvedFileTreeOptions;
  dataTables: ResolvedDataTableOptions;
  sanitize: ResolvedSanitizeOptions;
  editThisPage: ResolvedEditThisPageOptions;
  cjkEmphasis: boolean;
  codeBlockLint: ResolvedCodeBlockLintOptions;
  codeBlockTypecheck: ResolvedCodeBlockTypecheckOptions;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  typedHover?: ResolvedTypedHoverOptions;
  docsTests: ResolvedDocsTestOptions;
  mermaid: boolean;
  graphviz: ResolvedGraphvizOptions | false;
  math: ResolvedMathOptions;
  frontmatter: boolean;
  toc: boolean;
  tocMaxDepth: number;
  /**
   * Present after `resolveOptions`. Omitted in hand-built fixtures means off.
   */
  headingPermalinks?: ResolvedHeadingPermalinksOptions;
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
   * Render `<AppleMusic url="https://music.apple.com/...">` iframes.
   * @default false
   */
  appleMusic?: boolean;

  /**
   * Render `<SpeakerDeck url="https://speakerdeck.com/...">` cards.
   * Player URLs and oEmbed-resolved share URLs render a lazy iframe plus
   * title/author metadata. Fetch or parse failures become a link card.
   * @default false
   */
  speakerDeck?: boolean;

  /**
   * Render `<Audio src="https://...">` native audio players.
   * @default false
   */
  audio?: boolean;

  /**
   * Render `<Video src="https://...">` native video players.
   * @default false
   */
  video?: boolean;

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
   * Render `<Reddit>` as a static post card.
   * Pass `{ fetch: false }` to skip metadata fetching and render a link-only card.
   * @default false
   */
  reddit?: boolean | RedditEmbedOptions;

  /**
   * Render `<Bluesky>` as static cards.
   * @default false
   */
  bluesky?: boolean;

  /**
   * Render `<GoogleMaps>` as static place cards.
   * @default false
   */
  googleMaps?: boolean;

  /**
   * Render `<Qiita>` as static article cards.
   * Pass `{ fetch: false }` to skip metadata fetching and render a link-only card.
   * @default false
   */
  qiita?: boolean | ProviderArticleEmbedOptions;

  /**
   * Render `<Zenn>` as static article cards.
   * Pass `{ fetch: false }` to skip metadata fetching and render a link-only card.
   * @default false
   */
  zenn?: boolean | ProviderArticleEmbedOptions;

  /**
   * Render `<NpmPackage>`, `<CratesIo>`, `<PyPI>`, and `<DockerHub>` as static cards.
   * Pass `{ fetch: false }` to skip metadata fetching and render link-only cards.
   * @default false
   */
  packageRegistry?: boolean | ProviderPackageEmbedOptions;

  /**
   * Render `<CodePen>`, `<JSFiddle>`, and `<Observable>` as static playground cards.
   * Pass `{ iframe: true }` to include lazy iframe URLs where supported.
   * @default false
   */
  playgrounds?: boolean | ProviderPlaygroundEmbedOptions;

  /**
   * Render `<Vimeo>` as static video cards.
   * Pass `{ iframe: true }` to include lazy player iframe URLs.
   * @default false
   */
  vimeo?: boolean | ProviderVideoEmbedOptions;

  /**
   * Render `<Twitch>` as static video, clip, and channel cards.
   * Pass `{ iframe: true, parent: "example.com" }` to include Twitch iframes.
   * @default false
   */
  twitch?: boolean | ProviderVideoEmbedOptions;

  /**
   * Render `<Discord>` as static invite/message cards.
   * @default false
   */
  discord?: boolean;

  /**
   * Render `<Fediverse>`, `<Mastodon>`, `<Misskey>`, and `<Mixi2>` as static cards.
   * @default false
   */
  fediverse?: boolean;

  /**
   * Render `<Facebook>` as static post cards.
   * @default false
   */
  facebook?: boolean;

  /**
   * Render `<Threads>` as static post cards.
   * @default false
   */
  threads?: boolean;

  /**
   * Render `<Instagram>` as static post cards.
   * @default false
   */
  instagram?: boolean;

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
  appleMusic: boolean;
  speakerDeck: boolean;
  audio?: boolean;
  video?: boolean;
  stackBlitz: boolean;
  twitter: TwitterEmbedOptions | false;
  reddit?: RedditEmbedOptions | false;
  bluesky: boolean;
  googleMaps?: boolean;
  qiita?: ProviderArticleEmbedOptions | false;
  zenn?: ProviderArticleEmbedOptions | false;
  packageRegistry?: ProviderPackageEmbedOptions | false;
  playgrounds?: ProviderPlaygroundEmbedOptions | false;
  vimeo?: ProviderVideoEmbedOptions | false;
  twitch?: ProviderVideoEmbedOptions | false;
  discord?: boolean;
  fediverse?: boolean;
  facebook?: boolean;
  threads?: boolean;
  instagram?: boolean;
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
 * Options for opt-in PHP Markdown Extra / mdBook-style definition lists.
 */
export interface DefinitionListOptions {
  /**
   * Enable the definition-list transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved definition-list transform options.
 */
export interface ResolvedDefinitionListOptions {
  enabled: boolean;
}

/**
 * Options for the opt-in `<NotByAI />` authorship badge.
 */
export interface NotByAiOptions {
  /**
   * Enable the badge transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
  /**
   * Accessible label for the badge link.
   *
   * @default "Written by human, not by AI"
   */
  label?: string;
  /**
   * Destination URL. Unsafe values fall back to `https://notbyai.fyi`.
   *
   * @default "https://notbyai.fyi"
   */
  href?: string;
}

/**
 * Resolved NotByAI authorship-badge options.
 */
export interface ResolvedNotByAiOptions {
  enabled: boolean;
  label: string;
  href: string;
}

/**
 * Options for opt-in `{kbd:...}` inline keyboard keys.
 */
export interface KeyboardKeysOptions {
  /**
   * Enable the keyboard-key transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
  /**
   * Build-time aliases. Keys are matched case-insensitively and override
   * the built-in `cmd` / `ctrl` table.
   */
  aliases?: Record<string, string>;
  /**
   * Built-in alias labels. `"words"` emits `Command`; `"symbols"` emits `⌘`.
   *
   * @default "words"
   */
  style?: "words" | "symbols";
}

/**
 * Resolved inline keyboard-key transform options.
 */
export interface ResolvedKeyboardKeysOptions {
  enabled: boolean;
  aliases: Record<string, string>;
  style: "words" | "symbols";
}

/**
 * Options for opt-in abbreviation and glossary expansion.
 */
export interface AbbreviationsOptions {
  /**
   * Enable the transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
  /**
   * Central glossary. Keys are matched with Unicode word boundaries.
   */
  terms?: Record<string, string>;
  /**
   * Wrap only the first occurrence of each term.
   *
   * @default false
   */
  firstUseOnly?: boolean;
}

/**
 * Resolved abbreviation / glossary transform options.
 */
export interface ResolvedAbbreviationsOptions {
  enabled: boolean;
  terms: Record<string, string>;
  firstUseOnly: boolean;
}

/**
 * Options for opt-in `{link:...}` rich magic links.
 */
export interface MagicLinkOptions {
  /**
   * Enable the magic-link transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
  /**
   * Named aliases. A string value is treated as `{ href }`.
   */
  aliases?: Record<string, string | MagicLinkAlias>;
  /**
   * Emit a favicon URL when a link has no image.
   *
   * `true` uses `https://{host}/favicon.ico`. Pass `{ template }` to override.
   * The transform never fetches; the browser may load the URL later.
   *
   * @default false
   */
  favicon?: boolean | { template?: string };
  /**
   * Replace the resolved image for matching hrefs.
   */
  imageOverrides?: MagicLinkImageOverride[];
}

/**
 * One configured magic-link target.
 */
export interface MagicLinkAlias {
  href: string;
  label?: string;
  image?: string;
}

/**
 * Replace the image for an exact href or prefix.
 */
export interface MagicLinkImageOverride {
  href?: string;
  prefix?: string;
  image: string;
}

/**
 * Resolved magic-link transform options.
 */
export interface ResolvedMagicLinkOptions {
  enabled: boolean;
  aliases: Record<string, MagicLinkAlias>;
  favicon: boolean;
  faviconTemplate?: string;
  imageOverrides: MagicLinkImageOverride[];
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
 * Options for opt-in static image galleries.
 */
export interface ImageGalleryOptions {
  /**
   * Enable `::: gallery` blocks.
   *
   * @default true when the options object is supplied.
   */
  enabled?: boolean;

  /**
   * Add `loading="lazy"` to gallery images.
   *
   * @default follows `images.lazy`, or true when `images` is disabled.
   */
  lazy?: boolean;

  /**
   * Diagnostics for image items without alt text.
   *
   * @default "error"
   */
  missingAlt?: "error" | "warn" | "ignore";

  /**
   * Diagnostics for galleries without image items.
   *
   * @default "error"
   */
  empty?: "error" | "warn" | "ignore";
}

/**
 * Resolved image gallery transform options.
 */
export interface ResolvedImageGalleryOptions {
  enabled: boolean;
  lazy?: boolean;
  missingAlt: "error" | "warn" | "ignore";
  empty: "error" | "warn" | "ignore";
}

/**
 * Options for opt-in static timelines.
 */
export interface TimelineOptions {
  /**
   * Enable `::: timeline` blocks.
   *
   * @default true when the options object is supplied.
   */
  enabled?: boolean;

  /**
   * Render timelines as ordered lists unless a block overrides it.
   *
   * @default true
   */
  ordered?: boolean;

  /**
   * Diagnostics for malformed `YYYY`, `YYYY-MM`, or `YYYY-MM-DD` item dates.
   *
   * @default "error"
   */
  invalidDate?: "error" | "warn" | "ignore";

  /**
   * Diagnostics for unsupported item metadata.
   *
   * @default "error"
   */
  unknownMeta?: "error" | "warn" | "ignore";

  /**
   * Diagnostics for timeline blocks without items.
   *
   * @default "error"
   */
  empty?: "error" | "warn" | "ignore";
}

/**
 * Resolved timeline transform options.
 */
export interface ResolvedTimelineOptions {
  enabled: boolean;
  ordered: boolean;
  invalidDate: "error" | "warn" | "ignore";
  unknownMeta: "error" | "warn" | "ignore";
  empty: "error" | "warn" | "ignore";
}

/**
 * Options for opt-in static conditional blocks.
 */
export interface ConditionalBlockOptions {
  /**
   * Enable `::: if` / `::: else` blocks.
   *
   * @default true when the options object is supplied.
   */
  enabled?: boolean;

  /**
   * Build-time values available as `config.*` or bare identifiers. Page
   * frontmatter wins for bare identifiers; use `config.name` to force config.
   */
  values?: Record<string, unknown>;
}

/**
 * Resolved conditional-block transform options.
 */
export interface ResolvedConditionalBlockOptions {
  enabled: boolean;
  values: Record<string, unknown>;
}

/**
 * Options for opt-in page-bundle resources and image processing.
 */
export interface ResourcesOptions {
  /**
   * Allowed output formats for `?format=`.
   *
   * `jpg` is treated as `jpeg`. Pixel transforms encode `png` and `jpeg`.
   * `webp` is copied when the source is already webp and no pixel
   * transform is requested.
   *
   * @default ["png", "jpeg", "webp"]
   */
  formats?: string[];

  /**
   * Allowed `?width=` / `?w=` values. An empty list allows any positive
   * width.
   *
   * @default []
   */
  widths?: number[];

  /**
   * What to do when a relative resource is missing.
   *
   * @default "error"
   */
  missing?: "error" | "warn";

  /**
   * Emit identical bytes once as `/assets/content/<sha256>.<ext>` and
   * rewrite `src`, `poster`, and relevant `href` to that URL.
   *
   * Off unless this is `true`. `resources: true` and `{}` leave it off.
   *
   * @default false
   */
  dedupe?: boolean;
}

/**
 * Resolved page-resource options.
 */
export interface ResolvedResourcesOptions {
  enabled: boolean;
  formats: string[];
  widths: number[];
  missing: "error" | "warn";
  dedupe: boolean;
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
 *
 * Delimiter parsing lives in the native transform. Typesetting uses KaTeX at
 * build time when the optional `katex` peer is installed. Sites that omit
 * `math` do not need that package.
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
 * Opt-in visible heading permalinks.
 *
 * Headings already have stable `id`s. Enabling this appends a real
 * `<a class="header-anchor" href="#id">` using that exact id. Off by
 * default so existing HTML stays byte-stable.
 */
export interface HeadingPermalinksOptions {
  /**
   * Emit the permalink control.
   *
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved heading permalink options.
 */
export interface ResolvedHeadingPermalinksOptions {
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
 * Options for parameterized Markdown partials with `<!-- @partial: PATH k="v" -->`.
 *
 * Bare names resolve under `root` (`_partials` by default). Relative `./` and
 * `../` paths resolve from the current file. `@/` and leading `/` resolve from
 * `rootDir`. After canonicalize, paths outside `rootDir` are rejected.
 */
export interface PartialsOptions {
  /**
   * Enable the transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
  /**
   * Directory used to resolve `@/` and absolute partial paths.
   *
   * @default undefined
   */
  rootDir?: string;
  /**
   * Directory used for bare names such as `install.md`.
   *
   * @default "_partials"
   */
  root?: string;
  /**
   * Missing `{{ name }}` substitutions stay literal, or report a diagnostic.
   *
   * @default "literal"
   */
  missing?: "literal" | "error";
}

/**
 * Resolved parameterized-partial transform options.
 */
export interface ResolvedPartialsOptions {
  enabled: boolean;
  rootDir?: string;
  root: string;
  missing: "literal" | "error";
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
 * Options for opt-in `::: code-group` fence groups.
 */
export interface CodeGroupOptions {
  /**
   * Enable the code-group transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
}

/**
 * Resolved code-group transform options.
 */
export interface ResolvedCodeGroupOptions {
  enabled: boolean;
}

/**
 * Replaceable file-tree icons. Values are trusted site-config SVG markup or
 * CSS class tokens, never fence content.
 */
export interface FileTreeIconOptions {
  /** Collapsed folder icon. */
  folder?: string;
  /** Open folder icon. */
  folderOpen?: string;
  /** Default file icon. */
  file?: string;
  /** File icons keyed by extension (`ts`, `.json`). */
  files?: Record<string, string>;
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
  /**
   * Open directory `<details>` by default.
   *
   * @default true
   */
  defaultOpen?: boolean;
  /**
   * Render folder and file icons. Pass an object to replace the defaults.
   *
   * @default true
   */
  icons?: boolean | FileTreeIconOptions;
}

/**
 * Resolved file-tree transform options.
 */
export interface ResolvedFileTreeOptions {
  enabled: boolean;
  defaultOpen: boolean;
  icons: boolean;
  iconFolder?: string;
  iconFolderOpen?: string;
  iconFile?: string;
  iconFiles?: Record<string, string>;
}

/**
 * Options for opt-in `csv-table` / `json-table` fences.
 */
export interface DataTableOptions {
  /**
   * Enable the data-table transform when an options object is supplied.
   *
   * @default true
   */
  enabled?: boolean;
  /**
   * Directory used to resolve `@/` and absolute import paths.
   *
   * When omitted, imports resolve from the Vite project root.
   *
   * @default undefined
   */
  rootDir?: string;
  /**
   * What to do when an imported CSV/JSON file is missing.
   *
   * @default "error"
   */
  missing?: "error" | "warn";
}

/**
 * Resolved data-table transform options.
 */
export interface ResolvedDataTableOptions {
  enabled: boolean;
  rootDir?: string;
  missing: "error" | "warn";
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
 * Options for opt-in typed hover overlays on TypeScript fences.
 *
 * Hover strings are computed at build time with the same TypeScript compiler
 * family used by `codeBlockTypecheck` (`tsgo` / `typescript`). The browser
 * only receives JSON payloads and a tiny overlay script.
 */
export interface TypedHoverOptions {
  /**
   * Enable typed hover overlays.
   *
   * @default true when the object form is used
   */
  enabled?: boolean;

  /**
   * Fence languages that can receive hover payloads.
   *
   * Language names are compared case-insensitively.
   *
   * @default ['ts', 'tsx']
   */
  languages?: string[];

  /**
   * Path to the `tsgo` binary used to compute hover types.
   *
   * When omitted, the bundled `@typescript/native-preview` executable is used.
   */
  tsgoCommand?: string;
}

/**
 * Resolved typed-hover options.
 */
export interface ResolvedTypedHoverOptions {
  enabled: boolean;
  languages: string[];
  tsgoCommand?: string;
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
 * OG image generation options.
 * Uses Chromium or Satori rendering with customizable templates.
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
 * How a specifier was imported from an MDX `import` statement.
 */
export type MdxImportSpecifierKind = "default" | "named" | "namespace";

/**
 * One binding created by an MDX `import` statement.
 */
export interface MdxImportSpecifier {
  /** Imported name (`default`, `*`, or the named export). */
  imported: string;
  /** Local binding name. */
  local: string;
  /** Specifier kind. */
  kind: MdxImportSpecifierKind;
}

/**
 * One MDX `import` statement collected from the AST.
 */
export interface MdxImport {
  /** Module specifier string. */
  source: string;
  /** Bindings created by the import. */
  specifiers: MdxImportSpecifier[];
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

  /**
   * MDX `import` statements (empty when MDX is off or no ESM nodes).
   */
  imports: MdxImport[];

  /**
   * Export names from MDX ESM (empty when MDX is off or no exports).
   */
  exports: string[];

  /**
   * Unique JSX component names in document order (empty when none).
   */
  components: string[];

  /**
   * Labeled cross-reference targets collected during the Markdown transform.
   */
  crossReferences: CrossReferenceEntry[];
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
   * Local OpenAPI 3.0/3.1 JSON or YAML files to render as static REST API docs.
   *
   * Generated pages are written under `out/openapi/<spec>/` and use the same
   * Markdown, stale-file cleanup, SSG, and search pipeline as source docs.
   *
   * @default false
   */
  openapi?: OpenApiDocsSource | OpenApiDocsSource[] | OpenApiDocsOptions | false;

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
  openapi: ResolvedOpenApiDocsOptions | false;
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

/** OpenAPI docs shorthand accepted by `docs.openapi`. */
export type OpenApiDocsSource = string | OpenApiDocsInput;

/** One local OpenAPI file consumed by generated REST API docs. */
export interface OpenApiDocsInput {
  /** JSON or YAML file path, resolved from the Vite project root. */
  path: string;
  /** Optional display name. Defaults to `info.title` or the file name. */
  name?: string;
  /** Fail on unresolved or remote `$ref` values. Defaults to `true`. */
  failOnUnresolvedRefs?: boolean;
}

/** Object form for configuring generated OpenAPI docs. */
export interface OpenApiDocsOptions {
  /** Local OpenAPI files to render. */
  src?: OpenApiDocsSource | OpenApiDocsSource[];
  /** Route prefix used by generated OpenAPI nav metadata. Defaults to `basePath` or `/api`. */
  basePath?: string;
  /** Default unresolved `$ref` policy for sources. Defaults to `true`. */
  failOnUnresolvedRefs?: boolean;
}

/** Resolved local OpenAPI file input. */
export interface ResolvedOpenApiDocsInput {
  path: string;
  name?: string;
  failOnUnresolvedRefs: boolean;
}

/** Resolved generated OpenAPI docs options. */
export interface ResolvedOpenApiDocsOptions {
  src: ResolvedOpenApiDocsInput[];
  basePath?: string;
}

/** Navigation item emitted for generated docs sidebars. */
export interface DocsNavigationItem {
  title: string;
  path: string;
  children?: DocsNavigationItem[];
}

/** Generated OpenAPI Markdown pages and sidebar metadata. */
export interface GeneratedOpenApiDocs {
  pages: Record<string, string>;
  nav: DocsNavigationItem[];
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
   * Enable fuzzy typo-tolerant matching.
   *
   * Fuzzy matching is off by default so large static indexes keep the fastest
   * exact/prefix path. When enabled, local BM25 also considers near matches
   * for tokens with at least three characters.
   *
   * @default false
   */
  fuzzy?: boolean;

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

  /**
   * Search backend used by `virtual:ox-content/search`.
   *
   * `"local"` (the default) keeps the static BM25 `search-index.json` client.
   * `"hosted"` sends queries to a remote index with a public search-only key.
   * Hosted search is used only when this is set to `"hosted"`.
   *
   * @default 'local'
   */
  provider?: "local" | "hosted";

  /**
   * Hosted search application id.
   *
   * Required when `provider` is `"hosted"`. Also read from
   * `OX_CONTENT_SEARCH_APP_ID` when omitted here.
   */
  appId?: string;

  /**
   * Hosted search index name.
   *
   * Required when `provider` is `"hosted"`. Also read from
   * `OX_CONTENT_SEARCH_INDEX_NAME` when omitted here.
   */
  indexName?: string;

  /**
   * Public search-only key for the hosted provider.
   *
   * Write and admin keys are rejected. Also read from `OX_CONTENT_SEARCH_KEY`
   * when omitted here.
   */
  searchKey?: string;

  /**
   * Alias for `searchKey`.
   *
   * Also read from `OX_CONTENT_SEARCH_PUBLIC_KEY` when omitted here.
   */
  publicKey?: string;

  /**
   * HTTP endpoint that receives hosted search queries.
   *
   * Also read from `OX_CONTENT_SEARCH_ENDPOINT`. Defaults to `/search` when
   * hosted credentials are present.
   */
  endpoint?: string;
}

/**
 * Resolved search options.
 */
export interface ResolvedSearchOptions {
  enabled: boolean;
  limit: number;
  prefix: boolean;
  fuzzy: boolean;
  placeholder: string;
  hotkey: string;
  provider?: "local" | "hosted";
  appId?: string;
  indexName?: string;
  searchKey?: string;
  publicKey?: string;
  endpoint?: string;
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

/**
 * One host-owned page for composable SSG outputs (`ssg: false`).
 *
 * The host renders HTML. Ox Content plans and emits resources, Markdown
 * companions, feeds, and sitemap metadata from these fields.
 */
export interface SsgOutputPageInput {
  /** Source file used for git lastmod and companion identity. */
  inputPath: string;
  /** Published URL path (`guide` or `/`). */
  urlPath: string;
  /** Filesystem path of the host-rendered HTML page. */
  outputPath?: string;
  /** Host-rendered HTML. Required for resource fingerprinting. */
  html?: string;
  /** Already-read Markdown source bytes for companions. */
  source?: string;
  title?: string;
  description?: string;
  /** Absolute page URL. When omitted, `siteUrl` + `base` + `urlPath` is used. */
  loc?: string;
  /** Git commit time in milliseconds, or a host-supplied timestamp. */
  lastUpdated?: number;
  draft?: boolean;
  unlisted?: boolean;
  frontmatter?: Record<string, unknown>;
}
