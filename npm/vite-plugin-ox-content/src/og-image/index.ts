/**
 * Public API for OG image generation.
 *
 * Orchestrates renderer lifecycle, template resolution, caching,
 * and batch rendering with concurrency control.
 */
import * as path from "path";
import * as crypto from "crypto";
import { availableParallelism } from "os";
import { openBrowser, openBrowserPool } from "./browser";
import type { OgBrowserSession } from "./browser";
import { createSatoriRenderer } from "./satori-renderer";
import { createTakumiRenderer } from "./takumi-renderer";
import { resolveTemplate } from "./template-loader";
import { templateSources } from "./template-source";
export { isBareSpecifier, tsTemplateBundleOptions } from "./template-loader";
import { computeCacheKey, getCached, isCached, writeCache } from "./cache";
import type {
  OgImageOptions,
  ResolvedOgImageOptions,
  OgImageTemplateProps,
  OgImageTemplateFn,
} from "./types";

export type {
  OgImageOptions,
  ResolvedOgImageOptions,
  OgImageTemplateProps,
  OgImageTemplateFn,
  OgImageRenderer,
  OgImageSatoriFont,
  OgImageSatoriFontWeight,
  OgImageSatoriOptions,
  OgImageTakumiOptions,
} from "./types";

export type { OgBrowserPool, OgBrowserSession } from "./browser";
export { openBrowser, openBrowserPool } from "./browser";

/**
 * Resolves user-provided OG image options with defaults.
 */
/**
 * How many pages to render at once when the caller does not say.
 *
 * Rendering was serial by default, so a site paid one Chromium page render per
 * page, end to end. Pages are cheap next to the browser itself, but they are
 * not free — this stays well under the core count so a build machine keeps
 * room for everything else.
 */
function defaultConcurrency(): number {
  const cores = availableParallelism();
  return Math.max(1, Math.min(4, cores - 1));
}

export function resolveOgImageOptions(options: OgImageOptions | undefined): ResolvedOgImageOptions {
  return {
    renderer: options?.renderer ?? "chromium",
    template: options?.template,
    vuePlugin: options?.vuePlugin ?? "vitejs",
    jsxFramework: options?.jsxFramework ?? "react",
    width: options?.width ?? 1200,
    height: options?.height ?? 630,
    cache: options?.cache ?? true,
    concurrency: options?.concurrency ?? defaultConcurrency(),
    satori: {
      fonts: options?.satori?.fonts ?? [],
      systemFontFallback: options?.satori?.systemFontFallback ?? true,
    },
    takumi: {
      fonts: options?.takumi?.fonts ?? [],
      systemFontFallback: options?.takumi?.systemFontFallback ?? true,
    },
  };
}

/**
 * A single page entry for batch OG image generation.
 */
export interface OgImagePageEntry {
  /** Props to pass to the template */
  props: OgImageTemplateProps;
  /** Absolute path to write the output PNG */
  outputPath: string;
}

/**
 * Result of OG image generation for a single page.
 */
export interface OgImageResult {
  outputPath: string;
  cached: boolean;
  error?: string;
}

/**
 * Computes a stable template source identifier for cache keys.
 *
 * For custom templates, hashes the file content so cache invalidates
 * when the template changes. For the default template, returns a fixed string.
 */
async function computeTemplateSource(
  options: ResolvedOgImageOptions,
  root: string,
  templateFn: OgImageTemplateFn,
): Promise<string> {
  let baseSource: string;
  if (!options.template) {
    baseSource = options.renderer !== "chromium" ? "__default_satori_v1__" : "__default__";
  } else {
    const fs = await import("fs/promises");
    const templatePath = path.resolve(root, options.template);
    const content = templateSources.get(templateFn) ?? (await fs.readFile(templatePath, "utf-8"));
    baseSource = crypto.createHash("sha256").update(content).digest("hex");
  }

  const fontSource = await computeSatoriFontSource(options, root);
  return crypto
    .createHash("sha256")
    .update(
      JSON.stringify({
        baseSource,
        renderer: options.renderer,
        jsxFramework: options.jsxFramework,
        vuePlugin: options.vuePlugin,
        fontSource,
      }),
    )
    .digest("hex");
}

async function computeSatoriFontSource(
  options: ResolvedOgImageOptions,
  root: string,
): Promise<unknown> {
  const fs = await import("fs/promises");
  const fontOptions = options.renderer === "takumi" ? options.takumi : options.satori;
  const fonts = await Promise.all(
    (fontOptions?.fonts ?? []).map(async (font) => {
      const fontPath = path.isAbsolute(font.path) ? font.path : path.resolve(root, font.path);
      try {
        const stat = await fs.stat(fontPath);
        return {
          path: fontPath,
          name: font.name,
          weight: font.weight,
          style: font.style,
          size: stat.size,
          mtimeMs: stat.mtimeMs,
        };
      } catch {
        return {
          path: fontPath,
          name: font.name,
          weight: font.weight,
          style: font.style,
          missing: true,
        };
      }
    }),
  );

  return {
    fonts,
    systemFontFallback: fontOptions?.systemFontFallback ?? true,
  };
}

/**
 * Generates OG images for a batch of pages.
 *
 * Manages the full lifecycle: resolve template → select renderer →
 * render each page (with caching and concurrency).
 *
 * All errors are non-fatal: failures are reported in results but never throw.
 */
export async function generateOgImages(
  pages: OgImagePageEntry[],
  options: ResolvedOgImageOptions,
  root: string,
): Promise<OgImageResult[]> {
  if (pages.length === 0) return [];

  // Resolve template
  const templateFn = await resolveTemplate(options, root);

  // Compute template source for cache key
  const templateSource = await computeTemplateSource(options, root, templateFn);

  // Cache directory
  const cacheDir = path.join(root, ".cache", "og-images");

  const keyed = withCacheKeys(pages, templateSource, options);

  // Try to serve all from cache first if caching is enabled
  if (options.cache) {
    const allCached = await tryServeAllFromCache(keyed, cacheDir);
    if (allCached) return allCached;
  }

  if (options.renderer !== "chromium") {
    return renderBrowserlessPages(keyed, templateFn, options, cacheDir, root);
  }

  // One browser per worker: rendering does not parallelize inside a single
  // browser, so this is where the concurrency option actually buys something.
  const concurrency = Math.max(1, Math.min(options.concurrency, keyed.length));
  await using pool = await openBrowserPool(concurrency);
  if (!pool) {
    return pages.map((p) => ({
      outputPath: p.outputPath,
      cached: false,
      error: "Chromium not available",
    }));
  }

  // Resolve public directory for serving local assets in templates
  const publicDir = path.join(root, "public");

  // Awaited, not returned: `await using` disposes when this function returns,
  // and returning the promise would close every browser before the first
  // render finished.
  return await mapWithSessions(keyed, pool.sessions, (entry, session) =>
    renderSinglePage(entry, templateFn, options, cacheDir, session, publicDir),
  );
}

/**
 * Runs `render` over `items`, one at a time per session, in input order.
 *
 * A shared cursor rather than fixed batches: a batch waits for its slowest
 * member before the next one starts, so one heavy card idled every other
 * worker until it finished. Results are written by index, so the order the
 * caller gets back does not depend on which session finished first.
 */
export async function mapWithSessions<Item, Result>(
  items: readonly Item[],
  sessions: readonly OgBrowserSession[],
  render: (item: Item, session: OgBrowserSession) => Promise<Result>,
): Promise<Result[]> {
  const results: Result[] = new Array(items.length);
  let next = 0;

  await Promise.all(
    sessions.map(async (session) => {
      for (;;) {
        const index = next++;
        if (index >= items.length) {
          return;
        }
        results[index] = await render(items[index] as Item, session);
      }
    }),
  );

  return results;
}

async function renderBrowserlessPages(
  pages: KeyedPageEntry[],
  templateFn: OgImageTemplateFn,
  options: ResolvedOgImageOptions,
  cacheDir: string,
  root: string,
): Promise<OgImageResult[]> {
  const results: OgImageResult[] = [];
  const concurrency = Math.max(1, options.concurrency);
  let render: (html: string) => Promise<Buffer>;
  try {
    render = await (options.renderer === "takumi"
      ? createTakumiRenderer(options, root)
      : createSatoriRenderer(options, root));
  } catch (error) {
    return pages.map(({ outputPath }) => ({ outputPath, cached: false, error: String(error) }));
  }

  for (let i = 0; i < pages.length; i += concurrency) {
    const batch = pages.slice(i, i + concurrency);
    const batchResults = await Promise.all(
      batch.map((entry) =>
        renderSingleBrowserlessPage(entry, templateFn, options, cacheDir, render),
      ),
    );
    results.push(...batchResults);
  }

  return results;
}

/**
 * Serves every page from cache when all of them are present.
 *
 * The probe is an existence check per key, not a read: on a partial hit this
 * used to read and write every cached page before discovering the miss, then
 * throw that work away and let the render loop redo it. Returns null when any
 * page is missing, which is the signal that a renderer has to start.
 */
async function tryServeAllFromCache(
  pages: KeyedPageEntry[],
  cacheDir: string,
): Promise<OgImageResult[] | null> {
  for (const entry of pages) {
    if (!(await isCached(cacheDir, entry.key))) return null;
  }

  const results: OgImageResult[] = [];
  for (const entry of pages) {
    const cached = await getCached(cacheDir, entry.key);
    // Raced with a cache eviction between the probe and the read.
    if (!cached) return null;
    await writeOutput(entry.outputPath, cached);
    results.push({ outputPath: entry.outputPath, cached: true });
  }
  return results;
}

/** A page entry with its cache key computed once. */
interface KeyedPageEntry extends OgImagePageEntry {
  key: string;
}

/**
 * Attaches the cache key to each entry.
 *
 * The key is a SHA-256 over the template source and the page props. It used to
 * be recomputed three times per page — once to probe, once to read, once to
 * write — over props that can be a whole frontmatter object.
 */
function withCacheKeys(
  pages: OgImagePageEntry[],
  templateSource: string,
  options: ResolvedOgImageOptions,
): KeyedPageEntry[] {
  return pages.map((entry) => ({
    ...entry,
    key: computeCacheKey(
      templateSource,
      entry.props as unknown as Record<string, unknown>,
      options.width,
      options.height,
    ),
  }));
}

async function writeOutput(outputPath: string, png: Buffer): Promise<void> {
  const fs = await import("fs/promises");
  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(outputPath, png);
}

/**
 * Renders a single page to PNG, with cache support.
 */
async function renderSinglePage(
  entry: KeyedPageEntry,
  templateFn: OgImageTemplateFn,
  options: ResolvedOgImageOptions,
  cacheDir: string,
  session: OgBrowserSession,
  publicDir?: string,
): Promise<OgImageResult> {
  try {
    if (options.cache) {
      const cached = await getCached(cacheDir, entry.key);
      if (cached) {
        await writeOutput(entry.outputPath, cached);
        return { outputPath: entry.outputPath, cached: true };
      }
    }

    // Render template to HTML (may be async for SFC templates)
    const html = await templateFn(entry.props);

    // Render HTML to PNG via session (page create/close handled internally)
    const png = await session.renderPage(html, options.width, options.height, publicDir);

    await writeOutput(entry.outputPath, png);

    if (options.cache) {
      await writeCache(cacheDir, entry.key, png);
    }

    return { outputPath: entry.outputPath, cached: false };
  } catch (err) {
    return {
      outputPath: entry.outputPath,
      cached: false,
      error: err instanceof Error ? err.message : String(err),
    };
  }
}

/**
 * Renders a single page to PNG using Satori, with cache support.
 */
async function renderSingleBrowserlessPage(
  entry: KeyedPageEntry,
  templateFn: OgImageTemplateFn,
  options: ResolvedOgImageOptions,
  cacheDir: string,
  render: (html: string) => Promise<Buffer>,
): Promise<OgImageResult> {
  try {
    if (options.cache) {
      const cached = await getCached(cacheDir, entry.key);
      if (cached) {
        await writeOutput(entry.outputPath, cached);
        return { outputPath: entry.outputPath, cached: true };
      }
    }

    const html = await templateFn(entry.props);
    const png = await render(html);

    await writeOutput(entry.outputPath, png);

    if (options.cache) {
      await writeCache(cacheDir, entry.key, png);
    }

    return { outputPath: entry.outputPath, cached: false };
  } catch (err) {
    return {
      outputPath: entry.outputPath,
      cached: false,
      error: err instanceof Error ? err.message : String(err),
    };
  }
}
