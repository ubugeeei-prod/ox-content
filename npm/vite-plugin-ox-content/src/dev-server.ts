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
  generateHtmlPage,
  formatTitle,
  resolveNavigationGroups,
} from "./ssg";
import type { NavGroup, SsgPageData, SsgEntryPageConfig } from "./ssg";
import type { ResolvedOptions } from "./types";
import type { HeroConfig, FeatureConfig } from "./types";
import { normalizeVitePressFrontmatter } from "./vitepress";
import { isMarkdownFilePath } from "./markdown";

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
  const directCandidates =
    pathname === "/"
      ? extensions.map((extension) => `index${extension}`)
      : isMarkdownFilePath(routePath, extensions)
        ? [routePath]
        : extensions.map((extension) => `${routePath}${extension}`);

  for (const relativePath of directCandidates) {
    const filePath = path.join(srcDir, relativePath);
    try {
      await fs.access(filePath);
      return filePath;
    } catch {
      // Try the next extension.
    }
  }

  for (const extension of extensions) {
    const indexPath = path.join(srcDir, routePath, `index${extension}`);
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
  /** Cached rendered HTML keyed by absolute file path. */
  pages: Map<string, string>;
  /** Cached site name. Computed once. */
  siteName: string | null;
}

/**
 * Create a dev server cache instance.
 */
export function createDevServerCache(): DevServerCache {
  return {
    navGroups: null,
    pages: new Map(),
    siteName: null,
  };
}

/**
 * Invalidate navigation cache (called on file add/unlink).
 */
export function invalidateNavCache(cache: DevServerCache): void {
  cache.navGroups = null;
  // Also clear all page caches since navigation HTML is embedded in pages
  cache.pages.clear();
}

/**
 * Invalidate page cache for a specific file (called on file change).
 */
export function invalidatePageCache(cache: DevServerCache, filePath: string): void {
  cache.pages.delete(filePath);
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
): Promise<string> {
  const srcDir = path.resolve(root, options.srcDir);

  // Reset counters for clean render
  resetTabGroupCounter();
  resetIslandCounter();

  // Read markdown content
  const content = await fs.readFile(filePath, "utf-8");

  // Transform markdown to HTML
  const result = await transformMarkdown(content, filePath, options, {
    convertMdLinks: true,
    baseUrl: base,
    sourcePath: filePath,
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
    stackBlitz: options.embeds.stackBlitz,
    twitter: options.embeds.twitter,
    bluesky: options.embeds.bluesky,
    webContainer: options.embeds.webContainer,
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
    path: getUrlPath(filePath, srcDir),
    href: getUrlPath(filePath, srcDir) || "/",
    entryPage,
  };

  // Generate full HTML page
  let html = await generateHtmlPage(
    pageData,
    navGroups,
    siteName,
    base,
    options.ssg.ogImage,
    options.ssg.theme,
  );

  // Inject Vite HMR client for live reload
  html = injectViteHmrClient(html);

  return html;
}

/**
 * Create the dev server middleware for SSG page serving.
 */
export function createDevServerMiddleware(
  options: ResolvedOptions,
  root: string,
  cache: DevServerCache,
): Connect.NextHandleFunction {
  const srcDir = path.resolve(root, options.srcDir);
  const base = options.base.endsWith("/") ? options.base : options.base + "/";

  return async (req, res, next) => {
    const url = req.url;
    if (!url) return next();

    // Strip base from URL for routing
    let routeUrl = url;
    if (base !== "/" && routeUrl.startsWith(base)) {
      routeUrl = "/" + routeUrl.slice(base.length);
    }

    // Skip non-page requests
    if (shouldSkip(routeUrl)) return next();

    // Resolve markdown file
    const filePath = await resolveMarkdownFile(routeUrl, srcDir, options.extensions);
    if (!filePath) return next();

    try {
      // Check page cache
      const cached = cache.pages.get(filePath);
      if (cached) {
        res.setHeader("Content-Type", "text/html");
        res.setHeader("Cache-Control", "no-cache");
        res.end(cached);
        return;
      }

      // Resolve site name (cached after first call)
      if (!cache.siteName) {
        cache.siteName = await resolveSiteName(options, root);
      }

      // Build navigation if not cached
      if (!cache.navGroups) {
        const markdownFiles = await collectMarkdownFiles(srcDir, options.extensions);
        cache.navGroups =
          resolveNavigationGroups(options.ssg.navigation, base, options.ssg.extension) ??
          (options.ssg.theme?.sidebar.length
            ? buildThemeNavItems(options.ssg.theme.sidebar, base, options.ssg.extension)
            : buildNavItems(markdownFiles, srcDir, base, options.ssg.extension));
      }

      // Render the page
      const html = await renderPage(filePath, options, cache.navGroups, cache.siteName, base, root);

      // Cache the result
      cache.pages.set(filePath, html);

      res.setHeader("Content-Type", "text/html");
      res.setHeader("Cache-Control", "no-cache");
      res.end(html);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error(`[ox-content:dev] Failed to render ${filePath}:`, message);
      next();
    }
  };
}
