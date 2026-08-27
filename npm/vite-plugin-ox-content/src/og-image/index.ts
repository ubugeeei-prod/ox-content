/**
 * Public API for OG image generation.
 *
 * Orchestrates renderer lifecycle, template resolution, caching,
 * and batch rendering with concurrency control.
 */
import * as path from "path";
import * as crypto from "crypto";
import { availableParallelism } from "os";
import { openBrowser } from "./browser";
import type { OgBrowserSession } from "./browser";
import { renderHtmlToPngWithSatori } from "./satori-renderer";
import { getDefaultSatoriTemplate, getDefaultTemplate } from "./template";
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
} from "./types";

export type { OgBrowserSession } from "./browser";

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
    width: options?.width ?? 1200,
    height: options?.height ?? 630,
    cache: options?.cache ?? true,
    concurrency: options?.concurrency ?? defaultConcurrency(),
    satori: {
      fonts: options?.satori?.fonts ?? [],
      systemFontFallback: options?.satori?.systemFontFallback ?? true,
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
 * Resolves the template function from options.
 *
 * Dispatches by file extension:
 * - `.vue`  → Vue SFC (SSR via vue/server-renderer)
 * - `.svelte` → Svelte SFC (SSR via svelte/server)
 * - `.tsx`/`.jsx` → React Server Component (SSR via react-dom/server)
 * - others → TypeScript template (direct function export)
 */
async function resolveTemplate(
  options: ResolvedOgImageOptions,
  root: string,
): Promise<OgImageTemplateFn> {
  if (!options.template) {
    return options.renderer === "satori" ? getDefaultSatoriTemplate() : getDefaultTemplate();
  }

  const templatePath = path.resolve(root, options.template);

  // Verify file exists
  const fs = await import("fs/promises");
  try {
    await fs.access(templatePath);
  } catch {
    throw new Error(`[ox-content:og-image] Template file not found: ${templatePath}`);
  }

  const ext = path.extname(templatePath).toLowerCase();

  switch (ext) {
    case ".vue":
      return resolveVueTemplate(templatePath, options, root);
    case ".svelte":
      return resolveSvelteTemplate(templatePath, root);
    case ".tsx":
    case ".jsx":
      return resolveReactTemplate(templatePath, root);
    default:
      return resolveTsTemplate(templatePath, options, root);
  }
}

/**
 * Matches this package and every subpath it exports.
 *
 * A template's natural runtime is whatever renders it, and for the
 * framework-less kinds that is this package: `renderToString`, `raw`, `when`
 * and `each` live at its root, and the JSX runtime under `./jsx-runtime`.
 * Inlining them instead drags the entire plugin — chokidar, fsevents and all
 * — into the template bundle, which is what made importing it fail outright.
 */
const OX_CONTENT_PACKAGE = /^@ox-content\/vite-plugin(\/.*)?$/;

/**
 * Whether `id` is a bare specifier, and so resolvable at runtime rather than
 * something the template bundle has to inline.
 *
 * Template bundles are written to `<root>/.cache/og-images/` and imported
 * from there, so Node resolves anything left external against the project's
 * own `node_modules`. Relative and absolute imports still bundle, which is
 * what a template actually needs — its own components travel with it.
 */
export function isBareSpecifier(id: string): boolean {
  if (id.startsWith(".") || id.startsWith("/") || id.startsWith("\0")) {
    return false;
  }
  // Windows drive letters and rolldown's virtual-module prefixes.
  return !/^[a-zA-Z]:[\\/]/.test(id);
}

/**
 * Rolldown input options for a `.ts` template bundle.
 *
 * A `.ts` template is the framework-less kind, so it has no single runtime to
 * externalize the way the `.vue`, `.svelte` and `.tsx` paths do — anything
 * from `node_modules` is better resolved at import time than inlined. Nothing
 * on this path has a compiler plugin, so nothing here needed bundling to be
 * loadable in the first place.
 */
export function tsTemplateBundleOptions(templatePath: string) {
  return {
    input: templatePath,
    platform: "node" as const,
    external: (id: string) => isBareSpecifier(id),
  };
}

/**
 * Resolves a plain TypeScript template (existing behavior).
 */
async function resolveTsTemplate(
  templatePath: string,
  options: ResolvedOgImageOptions,
  root: string,
): Promise<OgImageTemplateFn> {
  const fs = await import("fs/promises");
  const { rolldown } = await import("rolldown");
  const cacheDir = path.join(root, ".cache", "og-images");
  await fs.mkdir(cacheDir, { recursive: true });

  const outfile = path.join(cacheDir, "_template.mjs");

  const bundle = await rolldown(tsTemplateBundleOptions(templatePath));
  await bundle.write({
    file: outfile,
    format: "esm",
  });
  await bundle.close();

  const mod = await import(`${outfile}?t=${Date.now()}`);
  const templateFn = mod.default;

  if (typeof templateFn !== "function") {
    throw new Error(
      `[ox-content:og-image] Template must default-export a function: ${options.template}`,
    );
  }

  return templateFn as OgImageTemplateFn;
}

/**
 * Resolves a Vue SFC template via SSR.
 *
 * Compiles the SFC with @vue/compiler-sfc (or @vizejs/vite-plugin),
 * bundles with rolldown, then wraps with createSSRApp + renderToString.
 */
async function resolveVueTemplate(
  templatePath: string,
  options: ResolvedOgImageOptions,
  root: string,
): Promise<OgImageTemplateFn> {
  const fs = await import("fs/promises");
  const { rolldown } = await import("rolldown");
  const cacheDir = path.join(root, ".cache", "og-images");
  await fs.mkdir(cacheDir, { recursive: true });

  const outfile = path.join(cacheDir, "_template_vue.mjs");

  const plugins =
    options.vuePlugin === "vizejs" ? await getVizejsPlugin() : [createVueCompilerPlugin()];

  const bundle = await rolldown({
    input: templatePath,
    platform: "node",
    external: ["vue", "vue/server-renderer", OX_CONTENT_PACKAGE],
    plugins,
  });
  await bundle.write({
    file: outfile,
    format: "esm",
  });
  await bundle.close();

  const mod = await import(`${outfile}?t=${Date.now()}`);
  const Component = mod.default;

  if (!Component) {
    throw new Error(
      `[ox-content:og-image] Vue template must have a default export: ${templatePath}`,
    );
  }

  // Extract CSS from SFC <style> blocks (Vue SSR does not include styles).
  // OG image templates render in complete isolation, so scoping is unnecessary.
  // We use raw CSS content to avoid scope ID mismatches between compilers
  // (e.g., vizejs and @vue/compiler-sfc may produce different scope hashes).
  let extractedCss = ((mod as Record<string, unknown>).__vize_css__ as string) || "";
  if (!extractedCss) {
    try {
      let compilerSfc: typeof import("@vue/compiler-sfc");
      try {
        compilerSfc = await import("@vue/compiler-sfc");
      } catch {
        compilerSfc = null as never;
      }
      if (compilerSfc) {
        const sfcSource = await fs.readFile(templatePath, "utf-8");
        const { descriptor } = compilerSfc.parse(sfcSource, { filename: templatePath });
        for (const style of descriptor.styles) {
          extractedCss += style.content;
        }
      }
    } catch {
      // CSS extraction is best-effort
    }
  }

  // Import Vue SSR utilities
  const { createSSRApp } = await import("vue");
  const { renderToString } = await import("vue/server-renderer");

  return async (props) => {
    const app = createSSRApp(Component, props);
    const html = await renderToString(app);
    if (extractedCss) {
      return `<style>${extractedCss}</style>${html}`;
    }
    return html;
  };
}

/**
 * Creates a rolldown plugin that compiles Vue SFCs using @vue/compiler-sfc.
 */
function createVueCompilerPlugin(): import("rolldown").Plugin {
  return {
    name: "ox-content-vue-sfc",
    async transform(code, id) {
      if (!id.endsWith(".vue")) return null;

      let compilerSfc: typeof import("@vue/compiler-sfc");
      try {
        compilerSfc = await import("@vue/compiler-sfc");
      } catch {
        throw new Error(
          "[ox-content:og-image] @vue/compiler-sfc is required for .vue templates. " +
            "Install it with: pnpm add -D @vue/compiler-sfc",
        );
      }

      const { descriptor } = compilerSfc.parse(code, { filename: id });

      // Compile <script setup> or <script>
      let scriptCode: string;
      if (descriptor.scriptSetup || descriptor.script) {
        const compiled = compilerSfc.compileScript(descriptor, {
          id,
          inlineTemplate: true,
        });
        scriptCode = compiled.content;
      } else {
        // Template-only SFC: compile template separately
        if (!descriptor.template) {
          throw new Error(
            `[ox-content:og-image] Vue SFC must have a <template> or <script>: ${id}`,
          );
        }
        const templateResult = compilerSfc.compileTemplate({
          source: descriptor.template.content,
          filename: id,
          id,
        });
        if (templateResult.errors.length > 0) {
          throw new Error(
            `[ox-content:og-image] Vue template compilation errors in ${id}: ${templateResult.errors.map(String).join(", ")}`,
          );
        }
        scriptCode = `${templateResult.code}\nexport default { render }`;
      }

      // Determine if the compiled output contains TypeScript
      const isTs = !!(descriptor.scriptSetup?.lang === "ts" || descriptor.script?.lang === "ts");

      return { code: scriptCode, moduleType: isTs ? "ts" : "js" };
    },
  };
}

/**
 * Loads @vizejs/vite-plugin as a rolldown plugin for Vue SFC compilation.
 */
async function getVizejsPlugin(): Promise<import("rolldown").Plugin[]> {
  try {
    const vizejs = await import("@vizejs/vite-plugin");
    const plugin = vizejs.default?.() ?? vizejs;
    return Array.isArray(plugin) ? plugin : [plugin];
  } catch {
    throw new Error(
      "[ox-content:og-image] @vizejs/vite-plugin is required when vuePlugin is 'vizejs'. " +
        "Install it with: pnpm add -D @vizejs/vite-plugin",
    );
  }
}

/**
 * Resolves a Svelte SFC template via SSR.
 *
 * Compiles the SFC with svelte/compiler (server mode + runes),
 * bundles with rolldown, then wraps with svelte/server render().
 */
async function resolveSvelteTemplate(
  templatePath: string,
  root: string,
): Promise<OgImageTemplateFn> {
  const fs = await import("fs/promises");
  const { rolldown } = await import("rolldown");
  const cacheDir = path.join(root, ".cache", "og-images");
  await fs.mkdir(cacheDir, { recursive: true });

  const outfile = path.join(cacheDir, "_template_svelte.mjs");

  const bundle = await rolldown({
    input: templatePath,
    platform: "node",
    external: [
      "svelte",
      "svelte/server",
      "svelte/internal",
      "svelte/internal/server",
      OX_CONTENT_PACKAGE,
    ],
    plugins: [createSvelteCompilerPlugin()],
  });
  await bundle.write({
    file: outfile,
    format: "esm",
  });
  await bundle.close();

  const mod = await import(`${outfile}?t=${Date.now()}`);
  const Component = mod.default;

  if (!Component) {
    throw new Error(
      `[ox-content:og-image] Svelte template must have a default export: ${templatePath}`,
    );
  }

  // Import Svelte SSR utility
  const { render } = (await import("svelte/server")) as {
    render: (component: unknown, options: { props: Record<string, unknown> }) => { body: string };
  };

  return async (props) => {
    const { body } = render(Component, { props });
    return body;
  };
}

/**
 * Creates a rolldown plugin that compiles Svelte SFCs using svelte/compiler.
 */
function createSvelteCompilerPlugin(): import("rolldown").Plugin {
  return {
    name: "ox-content-svelte-sfc",
    async transform(code, id) {
      if (!id.endsWith(".svelte")) return null;

      let svelteCompiler: typeof import("svelte/compiler");
      try {
        svelteCompiler = await import("svelte/compiler");
      } catch {
        throw new Error(
          "[ox-content:og-image] svelte is required for .svelte templates. " +
            "Install it with: pnpm add -D svelte",
        );
      }

      const result = svelteCompiler.compile(code, {
        generate: "server",
        runes: true,
        filename: id,
      });

      return { code: result.js.code };
    },
  };
}

/**
 * Resolves a React (.tsx/.jsx) template via SSR.
 *
 * Bundles with rolldown (JSX transform), then wraps with
 * react-dom/server renderToReadableStream for async Server Component support.
 */
async function resolveReactTemplate(
  templatePath: string,
  root: string,
): Promise<OgImageTemplateFn> {
  const fs = await import("fs/promises");
  const { rolldown } = await import("rolldown");
  const cacheDir = path.join(root, ".cache", "og-images");
  await fs.mkdir(cacheDir, { recursive: true });

  const outfile = path.join(cacheDir, "_template_react.mjs");

  const bundle = await rolldown({
    input: templatePath,
    platform: "node",
    external: [
      "react",
      "react/jsx-runtime",
      "react/jsx-dev-runtime",
      "react-dom",
      "react-dom/server",
      OX_CONTENT_PACKAGE,
    ],
    transform: {
      jsx: "react-jsx",
    },
  });
  await bundle.write({
    file: outfile,
    format: "esm",
  });
  await bundle.close();

  const mod = await import(`${outfile}?t=${Date.now()}`);
  const Component = mod.default;

  if (!Component) {
    throw new Error(
      `[ox-content:og-image] React template must have a default export: ${templatePath}`,
    );
  }

  // Import React SSR utilities
  let React: typeof import("react");
  let ReactDOMServer: typeof import("react-dom/server");
  try {
    React = await import("react");
    ReactDOMServer = await import("react-dom/server");
  } catch {
    throw new Error(
      "[ox-content:og-image] react and react-dom are required for .tsx/.jsx templates. " +
        "Install them with: pnpm add -D react react-dom",
    );
  }

  return async (props) => {
    const element = React.createElement(Component, props);
    // Use renderToReadableStream for async Server Component support
    const stream = await ReactDOMServer.renderToReadableStream(element);
    const reader = stream.getReader();
    const chunks: Uint8Array[] = [];
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(value);
    }
    const decoder = new TextDecoder();
    return (
      chunks.map((chunk) => decoder.decode(chunk, { stream: true })).join("") + decoder.decode()
    );
  };
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
): Promise<string> {
  let baseSource: string;
  if (!options.template) {
    baseSource = options.renderer === "satori" ? "__default_satori_v1__" : "__default__";
  } else {
    const fs = await import("fs/promises");
    const templatePath = path.resolve(root, options.template);
    const content = await fs.readFile(templatePath, "utf-8");
    baseSource = crypto.createHash("sha256").update(content).digest("hex");
  }

  if (options.renderer !== "satori") {
    return baseSource;
  }

  const fontSource = await computeSatoriFontSource(options, root);
  return crypto
    .createHash("sha256")
    .update(JSON.stringify({ baseSource, renderer: options.renderer, fontSource }))
    .digest("hex");
}

async function computeSatoriFontSource(
  options: ResolvedOgImageOptions,
  root: string,
): Promise<unknown> {
  const fs = await import("fs/promises");
  const fonts = await Promise.all(
    options.satori.fonts.map(async (font) => {
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
    systemFontFallback: options.satori.systemFontFallback,
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
  const templateSource = await computeTemplateSource(options, root);

  // Cache directory
  const cacheDir = path.join(root, ".cache", "og-images");

  const keyed = withCacheKeys(pages, templateSource, options);

  // Try to serve all from cache first if caching is enabled
  if (options.cache) {
    const allCached = await tryServeAllFromCache(keyed, cacheDir);
    if (allCached) return allCached;
  }

  if (options.renderer === "satori") {
    return renderSatoriPages(keyed, templateFn, options, cacheDir, root);
  }

  // Launch browser
  await using session = await openBrowser();
  if (!session) {
    return pages.map((p) => ({
      outputPath: p.outputPath,
      cached: false,
      error: "Chromium not available",
    }));
  }

  const results: OgImageResult[] = [];

  // Resolve public directory for serving local assets in templates
  const publicDir = path.join(root, "public");

  // Process pages with concurrency control
  const concurrency = Math.max(1, options.concurrency);

  for (let i = 0; i < keyed.length; i += concurrency) {
    const batch = keyed.slice(i, i + concurrency);
    const batchResults = await Promise.all(
      batch.map((entry) =>
        renderSinglePage(entry, templateFn, options, cacheDir, session, publicDir),
      ),
    );
    results.push(...batchResults);
  }

  return results;
}

async function renderSatoriPages(
  pages: KeyedPageEntry[],
  templateFn: OgImageTemplateFn,
  options: ResolvedOgImageOptions,
  cacheDir: string,
  root: string,
): Promise<OgImageResult[]> {
  const results: OgImageResult[] = [];
  const concurrency = Math.max(1, options.concurrency);

  for (let i = 0; i < pages.length; i += concurrency) {
    const batch = pages.slice(i, i + concurrency);
    const batchResults = await Promise.all(
      batch.map((entry) => renderSingleSatoriPage(entry, templateFn, options, cacheDir, root)),
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
async function renderSingleSatoriPage(
  entry: KeyedPageEntry,
  templateFn: OgImageTemplateFn,
  options: ResolvedOgImageOptions,
  cacheDir: string,
  root: string,
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
    const png = await renderHtmlToPngWithSatori(html, options, root);

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
