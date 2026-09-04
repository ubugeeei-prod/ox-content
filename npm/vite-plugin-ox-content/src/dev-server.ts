/**
 * Dev server middleware for ox-content SSG.
 *
 * Serves fully-rendered HTML pages (with navigation, theme, etc.)
 * during `vite dev`, matching the SSG build output.
 */

import * as fs from "fs/promises";
import * as path from "path";
import type { Connect } from "vite";
import { transformMarkdown } from "./transform";
import { transformAllPlugins } from "./plugins";
import { resetTabGroupCounter } from "./plugins";
import { protectMermaidSvgs, restoreMermaidSvgs } from "./plugins/mermaid-protect";
import { transformIslands, hasIslands, resetIslandCounter } from "./island";
import {
  collectMarkdownFiles,
  buildNavItems,
  buildThemeNavItems,
  extractTitle,
  getUrlPath,
  getHref,
  generateHtmlPage,
  getPageLocale,
  formatTitle,
  normalizeRoutePrefix,
  parseSsgPagerOverride,
  resolveNavigationGroups,
} from "./ssg";
import type { NavGroup, SsgPageData, SsgEntryPageConfig } from "./ssg";
import type { OxContentOptions, ResolvedOptions } from "./types";
import type { HeroConfig, FeatureConfig } from "./types";
import { normalizeVitePressFrontmatter } from "./vitepress";
import { parsePageChromeFlags } from "./header-chrome";
import { buildLocalePaths } from "./locale-switcher";
import { localizeHeaderNavItems, localizeNavGroups } from "./locale-nav";
import { isMarkdownFilePath } from "./markdown";
import { remapNavGroups } from "./apply-permalinks";
import {
  buildMarkdownSourceIndex,
  injectMarkdownSourceAlternate,
  isMarkdownSourceRequest,
  markdownSourceHrefForPage,
  resolveMarkdownSourceRequest,
  type MarkdownSourceIndexEntry,
} from "./markdown-source";
import { resolveOptions } from "./resolve-options";

/** File extensions to skip in the middleware. */
const SKIP_EXTENSIONS = new Set([
  ".js",
  ".ts",
  ".css",
  ".scss",
  ".less",
  ".svg",
  ".png",
  ".jpg",
  ".jpeg",
  ".gif",
  ".webp",
  ".ico",
  ".woff",
  ".woff2",
  ".ttf",
  ".eot",
  ".json",
  ".map",
  ".mp4",
  ".webm",
  ".mp3",
  ".pdf",
]);

/** Vite internal URL prefixes to skip. */
const VITE_INTERNAL_PREFIXES = ["/@vite/", "/@fs/", "/@id/", "/__"];

/**
 * Check if a request URL should be skipped by the dev server middleware.
 */
function shouldSkip(url: string): boolean {
  // Skip Vite internal URLs
  for (const prefix of VITE_INTERNAL_PREFIXES) {
    if (url.startsWith(prefix)) return true;
  }

  // Skip node_modules
  if (url.includes("/node_modules/")) return true;

  // Skip requests with known static file extensions
  const extMatch = url.match(/\.([a-zA-Z0-9]+)(?:\?|$)/);
  if (extMatch) {
    const ext = "." + extMatch[1].toLowerCase();
    if (SKIP_EXTENSIONS.has(ext)) return true;
  }

  return false;
}

export interface OxContentRouteMatch {
  /** Request pathname before ox-content base / route-prefix handling. */
  originalPathname: string;
  /** Request pathname after `base` is stripped. */
  pathname: string;
  /** Markdown route pathname after `base` and `ssg.routePrefix` are stripped. */
  routePathname: string;
  /** Normalized Vite/base public prefix. */
  base: string;
  /** Normalized SSG route prefix without leading/trailing slashes. */
  routePrefix?: string;
}

function normalizeRouteBase(base: string): string {
  if (!base || base === "/") {
    return "/";
  }
  const withLeading = base.startsWith("/") ? base : `/${base}`;
  return withLeading.endsWith("/") ? withLeading : `${withLeading}/`;
}

function stripBasePathname(pathname: string, base: string): string | undefined {
  const normalizedBase = normalizeRouteBase(base);
  if (normalizedBase === "/") {
    return pathname;
  }
  const bareBase = normalizedBase.slice(0, -1);
  if (pathname === bareBase) {
    return "/";
  }
  if (!pathname.startsWith(normalizedBase)) {
    return undefined;
  }
  return `/${pathname.slice(normalizedBase.length)}`;
}

function stripRoutePrefixPathname(pathname: string, routePrefix?: string): string | undefined {
  const prefix = normalizeRoutePrefix(routePrefix);
  if (!prefix) {
    return pathname;
  }
  const mount = `/${prefix}`;
  if (pathname === mount) {
    return "/";
  }
  if (!pathname.startsWith(`${mount}/`)) {
    return undefined;
  }
  return `/${pathname.slice(mount.length + 1)}`;
}

function requestPathname(input: string | URL | Request): string {
  try {
    const pathname =
      typeof input === "string"
        ? new URL(input, "http://localhost").pathname
        : input instanceof URL
          ? input.pathname
          : new URL(input.url).pathname;
    return safeDecodePathname(pathname);
  } catch {
    return "/";
  }
}

function safeDecodePathname(pathname: string): string {
  try {
    return decodeURIComponent(pathname);
  } catch {
    return pathname;
  }
}

export function resolveOxContentRoute(
  input: string | URL | Request,
  options: Pick<ResolvedOptions, "base" | "ssg">,
): OxContentRouteMatch | undefined {
  const originalPathname = requestPathname(input);
  const base = normalizeRouteBase(options.base);
  const pathname = stripBasePathname(originalPathname, base);
  if (!pathname) {
    return undefined;
  }
  const routePrefix = normalizeRoutePrefix(options.ssg.routePrefix);
  const routePathname = stripRoutePrefixPathname(pathname, routePrefix);
  if (!routePathname) {
    return undefined;
  }
  return {
    originalPathname,
    pathname,
    routePathname,
    base,
    routePrefix,
  };
}

function containedSourcePath(srcDir: string, relativePath: string): string | undefined {
  if (!relativePath || relativePath.includes("\\") || relativePath.includes("\0")) {
    return undefined;
  }
  const segments = relativePath.split("/");
  if (segments.some((segment) => !segment || segment === "." || segment === "..")) {
    return undefined;
  }
  const root = path.resolve(srcDir);
  const filePath = path.resolve(root, ...segments);
  const prefix = root.endsWith(path.sep) ? root : `${root}${path.sep}`;
  return filePath.startsWith(prefix) ? filePath : undefined;
}

/**
 * Resolve a request URL to a markdown file path.
 * Returns null if no matching file exists.
 */
async function resolveMarkdownFile(
  url: string,
  srcDir: string,
  extensions: readonly string[],
): Promise<string | null> {
  // Remove query string and hash
  let pathname = url.split("?")[0].split("#")[0];

  // Remove trailing /index.html
  if (pathname.endsWith("/index.html")) {
    pathname = pathname.slice(0, -"/index.html".length) || "/";
  }

  // Remove trailing slash (except for root)
  if (pathname !== "/" && pathname.endsWith("/")) {
    pathname = pathname.slice(0, -1);
  }

  const routePath = pathname === "/" ? "" : pathname.slice(1);
  if (routePath.includes("\\") || routePath.includes("\0")) {
    return null;
  }

  const directCandidates =
    pathname === "/"
      ? extensions.map((extension) => `index${extension}`)
      : isMarkdownFilePath(routePath, extensions)
        ? [routePath]
        : extensions.map((extension) => `${routePath}${extension}`);

  for (const relativePath of directCandidates) {
    const filePath = containedSourcePath(srcDir, relativePath);
    if (!filePath) {
      continue;
    }
    try {
      await fs.access(filePath);
      return filePath;
    } catch {
      // Try the next extension.
    }
  }

  for (const extension of extensions) {
    const indexRelativePath = routePath ? `${routePath}/index${extension}` : `index${extension}`;
    const indexPath = containedSourcePath(srcDir, indexRelativePath);
    if (!indexPath) {
      continue;
    }
    try {
      await fs.access(indexPath);
      return indexPath;
    } catch {
      // Try the next extension.
    }
  }

  return null;
}

/**
 * Inject Vite HMR client script into the HTML.
 */
function injectViteHmrClient(html: string): string {
  const hmrScript = `<script type="module" src="/@vite/client"></script>
<script type="module">
if (import.meta.hot) {
  const reexecuteBodyScripts = () => {
    const scripts = Array.from(document.body.querySelectorAll('script'));
    for (const script of scripts) {
      const nextScript = document.createElement('script');
      for (const attr of script.attributes) {
        nextScript.setAttribute(attr.name, attr.value);
      }
      nextScript.textContent = script.textContent;
      script.replaceWith(nextScript);
    }
  };

  const applyHotUpdate = async () => {
    const nextUrl = new URL(window.location.href);
    nextUrl.searchParams.set('__ox_hmr', String(Date.now()));

    const scrollX = window.scrollX;
    const scrollY = window.scrollY;
    const theme = document.documentElement.getAttribute('data-theme');

    const response = await fetch(nextUrl.toString(), {
      cache: 'no-store',
      headers: {
        'x-ox-content-hmr': '1',
      },
    });

    if (!response.ok) {
      throw new Error('Failed to fetch updated page');
    }

    const nextHtml = await response.text();
    const nextDocument = new DOMParser().parseFromString(nextHtml, 'text/html');

    if (!nextDocument.body) {
      throw new Error('Updated page is missing a body');
    }

    document.title = nextDocument.title;
    document.body.innerHTML = nextDocument.body.innerHTML;
    reexecuteBodyScripts();

    if (theme) {
      document.documentElement.setAttribute('data-theme', theme);
    }

    window.scrollTo({ left: scrollX, top: scrollY });
  };

  let pendingUpdate = Promise.resolve();

  import.meta.hot.on('ox-content:update', () => {
    pendingUpdate = pendingUpdate
      .then(() => applyHotUpdate())
      .catch((error) => {
        console.warn('[ox-content] HMR patch failed, falling back to reload.', error);
        location.reload();
      });
  });
}
</script>`;

  return html.replace("</head>", hmrScript + "\n</head>");
}

/**
 * Dev server state for caching.
 */
interface DevServerCache {
  /** Cached navigation groups. Invalidated on file add/unlink. */
  navGroups: NavGroup[] | null;
  /** Cached urlPath → href pairs for locale sibling lookup. */
  localePages: Array<{ path: string; href: string }> | null;
  /** Cached rendered HTML keyed by absolute file path. */
  pages: Map<string, string>;
  /** Cached site name. Computed once. */
  siteName: string | null;
  /** Companion URL → source bytes when `ssg.markdownSource` is on. */
  markdownSourceIndex: Map<string, MarkdownSourceIndexEntry> | null;
}

/**
 * Create a dev server cache instance.
 */
export function createDevServerCache(): DevServerCache {
  return {
    navGroups: null,
    localePages: null,
    pages: new Map(),
    siteName: null,
    markdownSourceIndex: null,
  };
}

/**
 * Invalidate navigation cache (called on file add/unlink).
 */
export function invalidateNavCache(cache: DevServerCache): void {
  cache.navGroups = null;
  cache.localePages = null;
  cache.markdownSourceIndex = null;
  // Also clear all page caches since navigation HTML is embedded in pages
  cache.pages.clear();
}

/**
 * Invalidate page cache for a specific file (called on file change).
 */
export function invalidatePageCache(cache: DevServerCache, filePath: string): void {
  cache.pages.delete(filePath);
  cache.pages.delete(pageCacheKeyFor(filePath, true));
  cache.pages.delete(pageCacheKeyFor(filePath, false));
  cache.markdownSourceIndex = null;
}

function pageCacheKeyFor(filePath: string, hmr: boolean): string {
  return `${hmr ? "hmr" : "static"}\0${filePath}`;
}

/**
 * Resolve site name from options or package.json.
 */
async function resolveSiteName(options: ResolvedOptions, root: string): Promise<string> {
  if (options.ssg.siteName) {
    return options.ssg.siteName;
  }

  try {
    const pkgPath = path.join(root, "package.json");
    const pkg = JSON.parse(await fs.readFile(pkgPath, "utf-8"));
    if (pkg.name) {
      return formatTitle(pkg.name);
    }
  } catch {
    // Use default
  }

  return "Documentation";
}

/**
 * Render a single markdown page to full HTML.
 */
async function renderPage(
  filePath: string,
  options: ResolvedOptions,
  navGroups: NavGroup[],
  siteName: string,
  base: string,
  root: string,
  localePages: Array<{ path: string; href: string }>,
  hmr: boolean,
): Promise<string> {
  const srcDir = path.resolve(root, options.srcDir);
  const routePrefix = normalizeRoutePrefix(options.ssg.routePrefix);
  const publicBase = routePrefix ? `${base}${routePrefix}/` : base;

  // Reset counters for clean render
  resetTabGroupCounter();
  resetIslandCounter();

  // Read markdown content
  const content = await fs.readFile(filePath, "utf-8");

  // Transform markdown to HTML
  const result = await transformMarkdown(content, filePath, options, {
    convertMdLinks: true,
    baseUrl: publicBase,
    sourcePath: filePath,
    srcDir,
  });
  const frontmatter = normalizeVitePressFrontmatter(result.frontmatter);

  let transformedHtml = result.html;

  // Protect mermaid SVGs from rehype processing
  const { html: protectedHtml, svgs: mermaidSvgs } = protectMermaidSvgs(transformedHtml);
  transformedHtml = protectedHtml;

  // Transform all plugins
  transformedHtml = await transformAllPlugins(transformedHtml, {
    tabs: true,
    youtube: true,
    github: options.embeds.github,
    openGraph: options.embeds.openGraph,
    pm: options.embeds.pm,
    spotify: options.embeds.spotify,
    appleMusic: options.embeds.appleMusic,
    speakerDeck: options.embeds.speakerDeck,
    audio: options.embeds.audio,
    video: options.embeds.video,
    stackBlitz: options.embeds.stackBlitz,
    twitter: options.embeds.twitter,
    reddit: options.embeds.reddit,
    bluesky: options.embeds.bluesky,
    googleMaps: options.embeds.googleMaps,
    qiita: options.embeds.qiita,
    zenn: options.embeds.zenn,
    discord: options.embeds.discord,
    fediverse: options.embeds.fediverse,
    facebook: options.embeds.facebook,
    threads: options.embeds.threads,
    instagram: options.embeds.instagram,
    webContainer: options.embeds.webContainer,
    loom: options.embeds.loom,
    asciinema: options.embeds.asciinema,
    figma: options.embeds.figma,
    note: options.embeds.note,
    googleSlides: options.embeds.googleSlides,
    mermaid: true,
    githubToken: process.env.GITHUB_TOKEN,
  });

  // Transform Island components
  if (hasIslands(transformedHtml)) {
    const islandResult = await transformIslands(transformedHtml);
    transformedHtml = islandResult.html;
  }

  // Restore protected mermaid SVGs
  transformedHtml = restoreMermaidSvgs(transformedHtml, mermaidSvgs);

  // Extract title
  const title = extractTitle(transformedHtml, frontmatter);
  const description = frontmatter.description as string | undefined;

  // Check if this is an entry page
  let entryPage: SsgEntryPageConfig | undefined;
  if (frontmatter.layout === "entry") {
    entryPage = {
      hero: frontmatter.hero as HeroConfig | undefined,
      features: frontmatter.features as FeatureConfig[] | undefined,
    };
  }

  // Build page data
  const pageData: SsgPageData = {
    title,
    description,
    content: transformedHtml,
    toc: result.toc,
    frontmatter,
    path: getUrlPath(filePath, srcDir, routePrefix),
    href: getHref(filePath, srcDir, base, options.ssg.extension, routePrefix),
    entryPage,
    prev: parseSsgPagerOverride(frontmatter.prev),
    next: parseSsgPagerOverride(frontmatter.next),
    breadcrumbs: frontmatter.breadcrumbs === false ? false : undefined,
    chrome: parsePageChromeFlags(frontmatter),
  };

  const i18n = options.i18n;
  const localePath =
    stripRoutePrefixPathname(`/${pageData.path}`, routePrefix)?.slice(1) ?? pageData.path;
  const locale = getPageLocale(localePath, i18n);
  const localeNav =
    i18n && locale
      ? {
          locale,
          locales: i18n.locales,
          defaultLocale: i18n.defaultLocale,
          hideDefaultLocale: i18n.hideDefaultLocale,
          pages: localePages,
          base,
        }
      : undefined;
  const localizedNav = localeNav ? localizeNavGroups(navGroups, localeNav) : navGroups;
  const theme = options.ssg.theme
    ? localeNav
      ? {
          ...options.ssg.theme,
          nav: localizeHeaderNavItems(options.ssg.theme.nav, localeNav),
        }
      : options.ssg.theme
    : undefined;
  const localePaths =
    options.ssg.localeSwitcher && i18n
      ? buildLocalePaths({
          currentPath: pageData.path,
          locales: i18n.locales,
          defaultLocale: i18n.defaultLocale,
          hideDefaultLocale: i18n.hideDefaultLocale,
          pages: localePages,
          base,
        })
      : undefined;

  const markdownSource = options.ssg.markdownSource?.enabled
    ? markdownSourceHrefForPage({
        source: filePath,
        fileUrl: pageData.path,
        frontmatter,
        base,
        permalinks: options.permalinks,
        cascade: options.cascade,
        publishState: options.publishState,
      })
    : undefined;
  if (options.ssg.markdownSource?.copy) {
    pageData.markdownSource = markdownSource;
  }

  // Generate full HTML page
  let html = await generateHtmlPage(
    pageData,
    localizedNav,
    siteName,
    base,
    options.ssg.ogImage,
    theme,
    locale,
    i18n ? i18n.locales : undefined,
    options.ssg.pagination,
    options.ssg.readerChrome,
    options.ssg.breadcrumbs,
    options.ssg.localeSwitcher,
    localePaths,
    options.ssg.a11y,
    options.ssg.team ?? { enabled: false, members: [] },
    options.ssg.pageChrome,
    undefined,
    options.ssg.jsonLd,
    options.ssg.siteUrl,
    options.ssg.headValidation,
    i18n ? i18n.defaultLocale : undefined,
    Boolean(options.icons?.enabled),
  );

  if (markdownSource && options.ssg.markdownSource?.alternate) {
    html = injectMarkdownSourceAlternate(html, markdownSource);
  }

  if (hmr) {
    html = injectViteHmrClient(html);
  }

  return html;
}

async function serveMarkdownSource(
  routeUrl: string,
  options: ResolvedOptions,
  srcDir: string,
  cache: DevServerCache,
): Promise<string | "missing" | "hidden"> {
  if (!cache.markdownSourceIndex) {
    const files = await collectMarkdownFiles(srcDir, options.extensions);
    cache.markdownSourceIndex = await buildMarkdownSourceIndex({
      files,
      srcDir,
      permalinks: options.permalinks,
      cascade: options.cascade,
      publishState: options.publishState,
    });
  }
  const entry = resolveMarkdownSourceRequest(routeUrl, cache.markdownSourceIndex);
  if (!entry) {
    return "missing";
  }
  return entry.allowed ? entry.source : "hidden";
}

export interface OxContentRouterContext {
  request: Request;
  url: URL;
  match: OxContentRouteMatch;
  options: ResolvedOptions;
  root: string;
  srcDir: string;
  cache: DevServerCache;
  locals: Record<string, unknown>;
  render(): Promise<Response | undefined>;
}

export type OxContentRouterNext = () => Promise<Response | undefined>;

export type OxContentRouterMiddleware = (
  context: OxContentRouterContext,
  next: OxContentRouterNext,
) => Response | undefined | void | Promise<Response | undefined | void>;

export type OxContentRouterErrorHandler = (
  error: unknown,
  context: OxContentRouterContext,
) => Response | undefined | void | Promise<Response | undefined | void>;

export interface OxContentRouterInit {
  /** Existing cache to share with Vite's file watcher invalidation. */
  cache?: DevServerCache;
  /** Inject Vite HMR client into rendered pages. */
  hmr?: boolean;
  /** Middleware run before the built-in Markdown page renderer. */
  middleware?: readonly OxContentRouterMiddleware[];
  /** Custom response for render errors. Undefined falls through to the host. */
  onError?: OxContentRouterErrorHandler;
}

export interface OxContentRouter {
  readonly options: ResolvedOptions;
  readonly root: string;
  readonly cache: DevServerCache;
  readonly middleware: readonly OxContentRouterMiddleware[];
  resolve(input: string | URL | Request): OxContentRouteMatch | undefined;
  fetch(request: Request): Promise<Response | undefined>;
}

export type OxContentFetchMiddleware = (
  request: Request,
  next?: (request: Request) => Response | Promise<Response>,
) => Promise<Response | undefined>;

export function createOxContentRouter(
  inputOptions: OxContentOptions | ResolvedOptions,
  root = defaultRouterRoot(),
  init: OxContentRouterInit = {},
): OxContentRouter {
  const options = isResolvedOptions(inputOptions) ? inputOptions : resolveOptions(inputOptions);
  const cache = init.cache ?? createDevServerCache();
  const middleware = [...(init.middleware ?? [])];
  const hmr = init.hmr === true;

  const router: OxContentRouter = {
    options,
    root,
    cache,
    middleware,

    resolve(input) {
      return resolveOxContentRoute(input, options);
    },

    async fetch(request) {
      const method = request.method.toUpperCase();
      if (method !== "GET" && method !== "HEAD") {
        return undefined;
      }

      const match = resolveOxContentRoute(request, options);
      if (!match || shouldSkip(match.routePathname)) {
        return undefined;
      }

      const srcDir = path.resolve(root, options.srcDir);
      let context: OxContentRouterContext;
      context = {
        request,
        url: new URL(request.url),
        match,
        options,
        root,
        srcDir,
        cache,
        locals: {},
        render: () => renderRouteResponse(context, hmr),
      };

      try {
        const response = await dispatchRouterMiddleware(context, middleware, 0);
        if (!response) {
          return undefined;
        }
        if (method === "HEAD") {
          return new Response(null, {
            headers: response.headers,
            status: response.status,
            statusText: response.statusText,
          });
        }
        return response;
      } catch (error) {
        const handled = await init.onError?.(error, context);
        if (handled instanceof Response) {
          return handled;
        }
        const file = context.locals.filePath ? ` ${String(context.locals.filePath)}` : "";
        const message = error instanceof Error ? error.message : String(error);
        console.error(`[ox-content:router] Failed to render${file}:`, message);
        return undefined;
      }
    },
  };

  return router;
}

export function createOxContentMiddleware(
  inputOptions: OxContentOptions | ResolvedOptions,
  root = defaultRouterRoot(),
  init: OxContentRouterInit = {},
): OxContentFetchMiddleware {
  const router = createOxContentRouter(inputOptions, root, init);
  return async (request, next) => {
    const response = await router.fetch(request);
    if (response) {
      return response;
    }
    return next?.(request);
  };
}

export function createOxContentFetchHandler(
  inputOptions: OxContentOptions | ResolvedOptions,
  root = defaultRouterRoot(),
  init: OxContentRouterInit & { notFound?: Response | (() => Response) } = {},
): (request: Request) => Promise<Response> {
  const router = createOxContentRouter(inputOptions, root, init);
  return async (request) => {
    const response = await router.fetch(request);
    if (response) {
      return response;
    }
    if (typeof init.notFound === "function") {
      return init.notFound();
    }
    return init.notFound ?? new Response("Not Found", { status: 404 });
  };
}

async function dispatchRouterMiddleware(
  context: OxContentRouterContext,
  middleware: readonly OxContentRouterMiddleware[],
  index: number,
): Promise<Response | undefined> {
  const layer = middleware[index];
  if (!layer) {
    return context.render();
  }
  let nextCalled = false;
  let nextResponse: Response | undefined;
  const next = async () => {
    if (nextCalled) {
      throw new Error("[ox-content] Router middleware called next() more than once.");
    }
    nextCalled = true;
    nextResponse = await dispatchRouterMiddleware(context, middleware, index + 1);
    return nextResponse;
  };
  const response = await layer(context, next);
  return response instanceof Response ? response : nextResponse;
}

async function renderRouteResponse(
  context: OxContentRouterContext,
  hmr: boolean,
): Promise<Response | undefined> {
  const { options, root, srcDir, cache, match } = context;

  if (options.ssg.markdownSource?.enabled && isMarkdownSourceRequest(match.routePathname)) {
    const served = await serveMarkdownSource(match.routePathname, options, srcDir, cache);
    if (served === "missing") {
      return undefined;
    }
    if (served === "hidden") {
      return new Response(null, { status: 404 });
    }
    return new Response(served, {
      headers: {
        "Cache-Control": "no-cache",
        "Content-Type": "text/markdown; charset=utf-8",
      },
    });
  }

  const filePath = await resolveMarkdownFile(match.routePathname, srcDir, options.extensions);
  if (!filePath) {
    return undefined;
  }
  context.locals.filePath = filePath;

  const pageCacheKey = pageCacheKeyFor(filePath, hmr);
  const cached = cache.pages.get(pageCacheKey);
  if (cached) {
    return htmlResponse(cached);
  }

  if (!cache.siteName) {
    cache.siteName = await resolveSiteName(options, root);
  }

  if (!cache.navGroups || !cache.localePages) {
    const markdownFiles = await collectMarkdownFiles(srcDir, options.extensions);
    cache.localePages = markdownFiles.map((file) => ({
      path: getUrlPath(file, srcDir, options.ssg.routePrefix),
      href: getHref(file, srcDir, match.base, options.ssg.extension, options.ssg.routePrefix),
    }));
    const generatedNav = buildNavItems(markdownFiles, srcDir, match.base, options.ssg.extension);
    cache.navGroups =
      resolveNavigationGroups(options.ssg.navigation, match.base, options.ssg.extension) ??
      (options.ssg.theme?.sidebar.length
        ? buildThemeNavItems(options.ssg.theme.sidebar, match.base, options.ssg.extension)
        : remapNavGroups(
            generatedNav,
            markdownFiles.map((file) => ({
              fileUrl: getUrlPath(file, srcDir),
              urlPath: getUrlPath(file, srcDir, options.ssg.routePrefix),
              href: getHref(
                file,
                srcDir,
                match.base,
                options.ssg.extension,
                options.ssg.routePrefix,
              ),
            })),
            [],
          ));
  }

  const navGroups = cache.navGroups;
  const localePages = cache.localePages;
  if (!navGroups || !localePages) {
    return undefined;
  }

  const html = await renderPage(
    filePath,
    options,
    navGroups,
    cache.siteName,
    match.base,
    root,
    localePages,
    hmr,
  );

  cache.pages.set(pageCacheKey, html);
  return htmlResponse(html);
}

function htmlResponse(html: string): Response {
  return new Response(html, {
    headers: {
      "Cache-Control": "no-cache",
      "Content-Type": "text/html",
    },
  });
}

/**
 * Create the dev server middleware for SSG page serving.
 */
export function createDevServerMiddleware(
  options: ResolvedOptions,
  root: string,
  cache: DevServerCache,
): Connect.NextHandleFunction {
  const router = createOxContentRouter(options, root, { cache, hmr: true });

  return async (req, res, next) => {
    const request = connectRequestToRequest(req);
    if (!request) return next();
    const response = await router.fetch(request);
    if (!response) return next();
    await writeConnectResponse(response, res);
  };
}

function connectRequestToRequest(req: Parameters<Connect.NextHandleFunction>[0]): Request | null {
  if (!req.url) {
    return null;
  }
  return new Request(new URL(req.url, "http://localhost"), {
    method: req.method ?? "GET",
    headers: connectRequestHeaders(req.headers),
  });
}

function connectRequestHeaders(
  headers: Parameters<Connect.NextHandleFunction>[0]["headers"] | undefined,
) {
  const result = new Headers();
  if (!headers) {
    return result;
  }
  for (const [name, value] of Object.entries(headers)) {
    if (Array.isArray(value)) {
      for (const item of value) {
        result.append(name, item);
      }
    } else if (value !== undefined) {
      result.set(name, value);
    }
  }
  return result;
}

async function writeConnectResponse(
  response: Response,
  res: Parameters<Connect.NextHandleFunction>[1],
): Promise<void> {
  res.statusCode = response.status;
  response.headers.forEach((value, name) => {
    res.setHeader(name, value);
  });
  res.end(await response.text());
}

function defaultRouterRoot(): string {
  const runtime = globalThis as {
    Deno?: { cwd?: () => string };
    process?: { cwd?: () => string };
  };
  if (typeof runtime.process?.cwd === "function") {
    return runtime.process.cwd();
  }
  if (typeof runtime.Deno?.cwd === "function") {
    return runtime.Deno.cwd();
  }
  return ".";
}

function isResolvedOptions(
  options: OxContentOptions | ResolvedOptions,
): options is ResolvedOptions {
  const candidate = options as Partial<ResolvedOptions>;
  return (
    typeof candidate.outDir === "string" &&
    Array.isArray(candidate.extensions) &&
    typeof candidate.gfm === "boolean" &&
    Boolean(candidate.ssg && typeof candidate.ssg === "object" && "enabled" in candidate.ssg)
  );
}
