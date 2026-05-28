/**
 * Vite Plugin for Ox Content
 *
 * Uses Vite's Environment API for SSG-focused Markdown processing.
 * Provides separate environments for client and server rendering.
 */

import * as path from "path";
import type { Plugin, ViteDevServer, ResolvedConfig } from "vite";
import { createMarkdownEnvironment } from "./environment";
import { transformMarkdown } from "./transform";
import { extractDocs, generateMarkdown, writeDocs, resolveDocsOptions } from "./docs";
import { buildSsg, resolveSsgOptions } from "./ssg";
import {
  resolveSearchOptions,
  buildSearchIndex,
  writeSearchIndex,
  generateSearchModule,
} from "./search";
import { resolveOgImageOptions } from "./og-image";
import {
  createDevServerMiddleware,
  createDevServerCache,
  invalidateNavCache,
  invalidatePageCache,
} from "./dev-server";
import { createOgViewerPlugin } from "./og-viewer";
import { resolveI18nOptions, createI18nPlugin } from "./i18n";
import { isMarkdownFilePath, normalizeMarkdownExtensions } from "./markdown";
import type { OxContentOptions, ResolvedOptions } from "./types";

export type { OxContentOptions } from "./types";
export type { LanguageRegistration, ThemeRegistration } from "shiki";
export type {
  CodeAnnotationSyntax,
  CodeAnnotationsOptions,
  ResolvedCodeAnnotationsOptions,
  DocsOptions,
  ResolvedDocsOptions,
  DocEntry,
  ParamDoc,
  ReturnDoc,
  ExtractedDocs,
  SsgOptions,
  ResolvedSsgOptions,
  SearchOptions,
  ResolvedSearchOptions,
  SearchDocument,
  SearchResult,
  // Entry page types
  HeroAction,
  HeroImage,
  HeroConfig,
  FeatureConfig,
  EntryPageConfig,
  SsgNavigationItem,
  SsgNavigationGroup,
  // i18n types
  I18nOptions,
  ResolvedI18nOptions,
  LocaleConfig,
  BuiltinEmbedOptions,
  ResolvedBuiltinEmbedOptions,
} from "./types";

/**
 * Creates the Ox Content Vite plugin.
 *
 * @example
 * ```ts
 * // vite.config.ts
 * import { defineConfig } from 'vite';
 * import { oxContent } from '@ox-content/vite-plugin';
 *
 * export default defineConfig({
 *   plugins: [
 *     oxContent({
 *       srcDir: 'content',
 *       gfm: true,
 *     }),
 *   ],
 * });
 * ```
 */
export function oxContent(options: OxContentOptions = {}): Plugin[] {
  const resolvedOptions = resolveOptions(options);
  let config: ResolvedConfig | undefined;
  const getRoot = () => config?.root || process.cwd();

  const ssgDevCache = createDevServerCache();
  const plugins: Plugin[] = [
    createMainPlugin(resolvedOptions, (resolvedConfig) => {
      config = resolvedConfig;
    }),
    createEnvironmentPlugin(resolvedOptions),
    createDocsPlugin(resolvedOptions, getRoot),
    createSsgPlugin(resolvedOptions, getRoot, ssgDevCache),
    createSearchPlugin(resolvedOptions, getRoot),
  ];

  if (resolvedOptions.i18n) {
    plugins.push(createI18nPlugin(resolvedOptions));
  }

  if (resolvedOptions.ogViewer) {
    plugins.push(createOgViewerPlugin(resolvedOptions));
  }

  return plugins;
}

async function regenerateDocs(resolvedOptions: ResolvedOptions, root: string): Promise<number> {
  const docsOptions = resolvedOptions.docs;
  if (!docsOptions || !docsOptions.enabled) {
    return 0;
  }

  const srcDirs = docsOptions.src.map((src) => path.resolve(root, src));
  const outDir = path.resolve(root, docsOptions.out);
  const extracted = await extractDocs(srcDirs, docsOptions);
  const generated = generateMarkdown(extracted, docsOptions);

  await writeDocs(generated, outDir, extracted, docsOptions);

  return Object.keys(generated).length;
}

function createMainPlugin(
  resolvedOptions: ResolvedOptions,
  setConfig: (config: ResolvedConfig) => void,
): Plugin {
  return {
    name: "ox-content",

    configResolved: setConfig,

    configureServer(devServer) {
      devServer.middlewares.use(async (req, res, next) => {
        const url = req.url;
        if (!url || !isMarkdownFilePath(url, resolvedOptions.extensions)) {
          return next();
        }

        next();
      });
    },

    resolveId(id) {
      if (id.startsWith("virtual:ox-content/")) {
        return "\0" + id;
      }

      if (isMarkdownFilePath(id, resolvedOptions.extensions)) {
        return id;
      }

      return null;
    },

    async load(id) {
      if (id.startsWith("\0virtual:ox-content/")) {
        const virtualPath = id.slice("\0virtual:ox-content/".length);
        return generateVirtualModule(virtualPath, resolvedOptions);
      }

      return null;
    },

    async transform(code, id) {
      if (!isMarkdownFilePath(id, resolvedOptions.extensions)) {
        return null;
      }

      const result = await transformMarkdown(code, id, resolvedOptions);
      return {
        code: result.code,
        map: null,
      };
    },

    async handleHotUpdate({ file, server }) {
      if (!isMarkdownFilePath(file, resolvedOptions.extensions)) {
        return;
      }

      server.ws.send({
        type: "custom",
        event: "ox-content:update",
        data: { file },
      });

      const modules = server.moduleGraph.getModulesByFile(file);
      return modules ? Array.from(modules) : [];
    },
  };
}

function createEnvironmentPlugin(resolvedOptions: ResolvedOptions): Plugin {
  return {
    name: "ox-content:environment",

    config() {
      return {
        environments: {
          markdown: createMarkdownEnvironment(resolvedOptions),
        },
      };
    },
  };
}

function createDocsPlugin(resolvedOptions: ResolvedOptions, getRoot: () => string): Plugin {
  return {
    name: "ox-content:docs",

    async buildStart() {
      const docsOptions = resolvedOptions.docs;
      if (!docsOptions || !docsOptions.enabled) {
        return;
      }

      try {
        const count = await regenerateDocs(resolvedOptions, getRoot());
        console.log(`[ox-content] Generated ${count} documentation files to ${docsOptions.out}`);
      } catch (err) {
        console.warn("[ox-content] Failed to generate documentation:", err);
      }
    },

    configureServer(devServer) {
      const docsOptions = resolvedOptions.docs;
      if (!docsOptions || !docsOptions.enabled) {
        return;
      }

      const root = getRoot();
      const srcDirs = docsOptions.src.map((src) => path.resolve(root, src));
      for (const srcDir of srcDirs) {
        devServer.watcher.add(srcDir);
      }

      devServer.watcher.on("all", async (event, file) => {
        if (event !== "add" && event !== "change" && event !== "unlink") {
          return;
        }

        const isSourceFile = srcDirs.some(
          (srcDir) => file.startsWith(srcDir) && (file.endsWith(".ts") || file.endsWith(".tsx")),
        );
        if (!isSourceFile) {
          return;
        }

        try {
          await regenerateDocs(resolvedOptions, root);
        } catch {
          // Ignore errors during dev.
        }
      });
    },
  };
}

function createSsgPlugin(
  resolvedOptions: ResolvedOptions,
  getRoot: () => string,
  ssgDevCache: ReturnType<typeof createDevServerCache>,
): Plugin {
  return {
    name: "ox-content:ssg",

    configureServer(devServer) {
      const ssgOptions = resolvedOptions.ssg;
      if (!ssgOptions.enabled) return;

      const root = getRoot();
      const srcDir = path.resolve(root, resolvedOptions.srcDir);
      devServer.middlewares.use(createDevServerMiddleware(resolvedOptions, root, ssgDevCache));

      devServer.watcher.on("add", (file: string) => {
        notifySsgFileAddedOrRemoved(devServer, resolvedOptions, ssgDevCache, srcDir, file, "add");
      });
      devServer.watcher.on("unlink", (file: string) => {
        notifySsgFileAddedOrRemoved(
          devServer,
          resolvedOptions,
          ssgDevCache,
          srcDir,
          file,
          "unlink",
        );
      });
      devServer.watcher.on("change", (file: string) => {
        if (file.startsWith(srcDir) && isMarkdownFilePath(file, resolvedOptions.extensions)) {
          invalidatePageCache(ssgDevCache, file);
        }
      });
    },

    async closeBundle() {
      const ssgOptions = resolvedOptions.ssg;
      if (!ssgOptions.enabled) {
        return;
      }

      try {
        const result = await buildSsg(resolvedOptions, getRoot());
        if (result.files.length > 0) {
          console.log(`[ox-content] Generated ${result.files.length} output files`);
        }

        for (const error of result.errors) {
          console.warn(`[ox-content] ${error}`);
        }
      } catch (err) {
        console.error("[ox-content] SSG build failed:", err);
      }
    },
  };
}

function notifySsgFileAddedOrRemoved(
  devServer: ViteDevServer,
  resolvedOptions: ResolvedOptions,
  ssgDevCache: ReturnType<typeof createDevServerCache>,
  srcDir: string,
  file: string,
  type: "add" | "unlink",
): void {
  if (!file.startsWith(srcDir) || !isMarkdownFilePath(file, resolvedOptions.extensions)) {
    return;
  }

  invalidateNavCache(ssgDevCache);
  devServer.ws.send({
    type: "custom",
    event: "ox-content:update",
    data: { file, type },
  });
}

function createSearchPlugin(resolvedOptions: ResolvedOptions, getRoot: () => string): Plugin {
  let searchIndexJson = "";

  return {
    name: "ox-content:search",

    resolveId(id) {
      if (id === "virtual:ox-content/search") {
        return "\0virtual:ox-content/search";
      }
      return null;
    },

    async load(id) {
      if (id !== "\0virtual:ox-content/search") {
        return null;
      }

      const searchOptions = resolvedOptions.search;
      if (!searchOptions.enabled) {
        return "export const search = () => []; export const searchOptions = { enabled: false }; export default { search, searchOptions };";
      }

      const indexPath = resolvedOptions.base + "search-index.json";
      return generateSearchModule(searchOptions, indexPath);
    },

    async buildStart() {
      const searchOptions = resolvedOptions.search;
      if (!searchOptions.enabled) {
        return;
      }

      const srcDir = path.resolve(getRoot(), resolvedOptions.srcDir);
      try {
        searchIndexJson = await buildSearchIndex(
          srcDir,
          resolvedOptions.base,
          resolvedOptions.extensions,
        );
        console.log("[ox-content] Search index built");
      } catch (err) {
        console.warn("[ox-content] Failed to build search index:", err);
      }
    },

    async closeBundle() {
      const searchOptions = resolvedOptions.search;
      if (!searchOptions.enabled || !searchIndexJson) {
        return;
      }

      const outDir = path.resolve(getRoot(), resolvedOptions.outDir);
      try {
        await writeSearchIndex(searchIndexJson, outDir);
        console.log("[ox-content] Search index written to", path.join(outDir, "search-index.json"));
      } catch (err) {
        console.warn("[ox-content] Failed to write search index:", err);
      }
    },
  };
}

/**
 * Resolves plugin options with defaults.
 */
function resolveOptions(options: OxContentOptions): ResolvedOptions {
  return {
    srcDir: options.srcDir ?? "content",
    outDir: options.outDir ?? "dist",
    base: options.base ?? "/",
    extensions: normalizeMarkdownExtensions(options.extensions),
    ssg: resolveSsgOptions(options.ssg),
    gfm: options.gfm ?? true,
    footnotes: options.footnotes ?? true,
    tables: options.tables ?? true,
    taskLists: options.taskLists ?? true,
    strikethrough: options.strikethrough ?? true,
    highlight: options.highlight ?? false,
    highlightTheme: options.highlightTheme ?? "github-dark",
    highlightLangs: options.highlightLangs ?? [],
    codeAnnotations: resolveCodeAnnotationsOptions(options.codeAnnotations),
    mermaid: options.mermaid ?? false,
    frontmatter: options.frontmatter ?? true,
    toc: options.toc ?? true,
    tocMaxDepth: options.tocMaxDepth ?? 3,
    ogImage: options.ogImage ?? false,
    ogImageOptions: resolveOgImageOptions(options.ogImageOptions),
    transformers: options.transformers ?? [],
    docs: resolveDocsOptions(options.docs),
    search: resolveSearchOptions(options.search),
    ogViewer: options.ogViewer ?? true,
    embeds: resolveBuiltinEmbedOptions(options.embeds),
    i18n: resolveI18nOptions(options.i18n),
  };
}

export function resolveBuiltinEmbedOptions(
  options: OxContentOptions["embeds"],
): ResolvedOptions["embeds"] {
  if (options === false) {
    return {
      github: false,
      openGraph: false,
    };
  }

  return {
    github: resolveSingleEmbedOptions(options?.github),
    openGraph: resolveSingleEmbedOptions(options?.openGraph),
  };
}

function resolveSingleEmbedOptions<T extends object>(options: boolean | T | undefined): T | false {
  if (options === false) return false;
  if (options === true || options === undefined) return {} as T;
  return options;
}

function resolveCodeAnnotationsOptions(
  options: OxContentOptions["codeAnnotations"],
): ResolvedOptions["codeAnnotations"] {
  if (!options) {
    return {
      enabled: false,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    };
  }

  if (options === true) {
    return {
      enabled: true,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    };
  }

  return {
    enabled: true,
    notation: options.notation ?? "attribute",
    metaKey: options.metaKey ?? "annotate",
    defaultLineNumbers: options.defaultLineNumbers ?? false,
  };
}

/**
 * Generates virtual module content.
 */
export function generateVirtualModule(path: string, options: ResolvedOptions): string {
  if (path === "config") {
    return `export default ${JSON.stringify(options)};`;
  }

  if (path === "runtime") {
    const base = normalizeRuntimeBase(options.base);
    return `
      export const base = ${JSON.stringify(base)};
      export const runtimeConfig = { base };

      export function isExternalUrl(value) {
        return /^(?:https?:)?\\/\\//i.test(value) || /^(?:mailto|tel):/i.test(value);
      }

      export function withBase(pathname = "") {
        const value = String(pathname);
        if (!value || value === "/") return base;
        if (value.startsWith("#") || isExternalUrl(value)) return value;
        return base + (value.startsWith("/") ? value.slice(1) : value);
      }

      export function withoutBase(pathname = "") {
        const value = String(pathname);
        if (base === "/" || value.startsWith("#") || isExternalUrl(value)) return value;
        const bareBase = base.slice(0, -1);
        if (value === bareBase) return "/";
        if (value.startsWith(base)) return "/" + value.slice(base.length);
        return value;
      }

      export function useMarkdown() {
        return {
          base,
          withBase,
          withoutBase,
          render: (content) => {
            return content;
          },
        };
      }
    `;
  }

  return "export default {};";
}

function normalizeRuntimeBase(base: string): string {
  const trimmed = base.trim();
  if (!trimmed || trimmed === "/") return "/";
  const withLeading = trimmed.startsWith("/") ? trimmed : `/${trimmed}`;
  return withLeading.endsWith("/") ? withLeading : `${withLeading}/`;
}

// Re-export types and utilities
export { createMarkdownEnvironment } from "./environment";
export { transformMarkdown } from "./transform";
export { extractDocs, generateMarkdown, writeDocs, resolveDocsOptions } from "./docs";
export { lintMarkdown, lintMarkdownAsync } from "./lint";
export { lintMarkdownFile, lintMarkdownFiles, shouldLintMarkdownFile } from "./lint-files";
export type {
  MarkdownLintDiagnostic,
  MarkdownLintDictionaryOptions,
  MarkdownLintLanguage,
  MarkdownLintOptions,
  MarkdownLintResult,
  MarkdownLintRuleOptions,
  MarkdownLintSeverity,
  MarkdownLintStandardDictionaryOptions,
} from "./lint";
export type {
  MarkdownLintFileDiagnostic as MarkdownLintBatchDiagnostic,
  MarkdownLintFileDiagnostic,
  MarkdownLintFileOptions,
  MarkdownLintFileResult,
  MarkdownLintFilesResult,
  MarkdownLintFileOptions as MarkdownLintProjectOptions,
} from "./lint-files";
export { buildSsg, resolveSsgOptions, DEFAULT_HTML_TEMPLATE } from "./ssg";
export { resolveSearchOptions, buildSearchIndex, writeSearchIndex } from "./search";
export {
  DEFAULT_MARKDOWN_EXTENSIONS,
  normalizeMarkdownExtensions,
  isMarkdownFilePath,
  stripMarkdownExtension,
} from "./markdown";
export { defineTheme, defaultTheme, mergeThemes, resolveTheme } from "./theme";
export {
  fromVitePressConfig,
  generateVitePressMigrationConfig,
  convertVitePressSidebar,
  convertVitePressNav,
  normalizeVitePressFrontmatter,
} from "./vitepress";
export type {
  GenerateVitePressMigrationConfigOptions,
  VitePressConfig,
  VitePressThemeConfig,
  VitePressSidebar,
  VitePressSidebarItem,
  VitePressNavItem,
  VitePressSocialLink,
  VitePressFooter,
  VitePressLogo,
} from "./vitepress";
export type {
  ThemeConfig,
  ThemeColors,
  ThemeLayout,
  ThemeFonts,
  ThemeEntryPage,
  ThemeHeader,
  ThemeFooter,
  SocialLinks,
  ThemeEmbed,
  ResolvedThemeConfig,
} from "./theme";
export * from "./types";

// JSX Runtime
export { jsx, jsxs, Fragment, renderToString, raw, when, each } from "./jsx-runtime";
export type { JSXNode, JSXChild, JSXProps, JSXElementType } from "./jsx-runtime";

// Page Context
export {
  usePageProps,
  useSiteConfig,
  useRenderContext,
  useNav,
  useIsActive,
  setRenderContext,
  clearRenderContext,
  generateFrontmatterTypes,
  inferType,
} from "./page-context";
export type {
  BasePageProps,
  PageProps,
  SiteConfig,
  NavGroup,
  NavItem,
  RenderContext,
  FrontmatterSchema,
} from "./page-context";

// Theme Renderer
export {
  renderPage,
  renderAllPages,
  generateTypes,
  DefaultTheme,
  createTheme,
} from "./theme-renderer";
export type { ThemeComponent, ThemeProps, PageData, ThemeRenderOptions } from "./theme-renderer";

// Built-in Plugins (No-JS First)
export {
  transformTabs,
  generateTabsCSS,
  transformYouTube,
  extractVideoId,
  transformGitHub,
  fetchRepoData,
  fetchGitHubSource,
  collectGitHubRepos,
  collectGitHubSources,
  prefetchGitHubRepos,
  prefetchGitHubSources,
  parseGitHubPermalink,
  parseGitHubLineRange,
  transformOgp,
  fetchOgpData,
  collectOgpUrls,
  prefetchOgpData,
  transformMermaidStatic,
  mermaidClientScript,
  transformAllPlugins,
} from "./plugins";
export type {
  YouTubeOptions,
  GitHubRepoData,
  GitHubSourceData,
  GitHubSourceRef,
  GitHubLineRange,
  GitHubOptions,
  OgpData,
  OgpOptions,
  MermaidOptions,
  TransformAllOptions,
} from "./plugins";

// Island Architecture
export { transformIslands, hasIslands, extractIslandInfo, generateHydrationScript } from "./island";
export type { LoadStrategy, IslandInfo, ParseIslandsResult } from "./island";

// OG Image
export { resolveOgImageOptions, generateOgImages } from "./og-image";
export { resolveI18nOptions, createI18nPlugin } from "./i18n";
export type {
  OgImageOptions as OgImagePluginOptions,
  ResolvedOgImageOptions,
  OgImageTemplateProps,
  OgImageTemplateFn,
  OgImagePageEntry,
  OgImageResult,
  OgBrowserSession,
} from "./og-image";
