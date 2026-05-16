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
  let config: ResolvedConfig;
  let _server: ViteDevServer | undefined;

  async function regenerateDocs(root: string): Promise<number> {
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

  const mainPlugin: Plugin = {
    name: "ox-content",

    configResolved(resolvedConfig) {
      config = resolvedConfig;
    },

    configureServer(devServer) {
      _server = devServer;

      // Add middleware for serving Markdown files
      devServer.middlewares.use(async (req, res, next) => {
        const url = req.url;
        if (!url || !isMarkdownFilePath(url, resolvedOptions.extensions)) {
          return next();
        }

        // Let Vite handle the transformation
        next();
      });
    },

    resolveId(id) {
      // Handle virtual modules for Markdown imports
      if (id.startsWith("virtual:ox-content/")) {
        return "\0" + id;
      }

      // Resolve Markdown files
      if (isMarkdownFilePath(id, resolvedOptions.extensions)) {
        return id;
      }

      return null;
    },

    async load(id) {
      // Handle virtual modules
      if (id.startsWith("\0virtual:ox-content/")) {
        const path = id.slice("\0virtual:ox-content/".length);
        return generateVirtualModule(path, resolvedOptions);
      }

      return null;
    },

    async transform(code, id) {
      if (!isMarkdownFilePath(id, resolvedOptions.extensions)) {
        return null;
      }

      // Transform Markdown to JavaScript module
      const result = await transformMarkdown(code, id, resolvedOptions);

      return {
        code: result.code,
        map: null,
      };
    },

    // Hot Module Replacement support
    async handleHotUpdate({ file, server }) {
      if (isMarkdownFilePath(file, resolvedOptions.extensions)) {
        // Notify client about the update
        server.ws.send({
          type: "custom",
          event: "ox-content:update",
          data: { file },
        });

        // Return empty array to prevent default HMR
        // We handle it ourselves
        const modules = server.moduleGraph.getModulesByFile(file);
        return modules ? Array.from(modules) : [];
      }
    },
  };

  // Environment API plugin for SSG
  const environmentPlugin: Plugin = {
    name: "ox-content:environment",

    config() {
      return {
        environments: {
          // Markdown processing environment
          markdown: createMarkdownEnvironment(resolvedOptions),
        },
      };
    },
  };

  // Docs generation plugin (builtin, opt-out)
  const docsPlugin: Plugin = {
    name: "ox-content:docs",

    async buildStart() {
      const docsOptions = resolvedOptions.docs;
      if (!docsOptions || !docsOptions.enabled) {
        return;
      }

      const root = config?.root || process.cwd();

      try {
        const count = await regenerateDocs(root);
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

      // Watch source directories for changes
      const root = config?.root || process.cwd();
      const srcDirs = docsOptions.src.map((src) => path.resolve(root, src));

      for (const srcDir of srcDirs) {
        devServer.watcher.add(srcDir);
      }

      // Regenerate docs on file changes
      devServer.watcher.on("all", async (event, file) => {
        if (event !== "add" && event !== "change" && event !== "unlink") {
          return;
        }

        const isSourceFile = srcDirs.some(
          (srcDir) => file.startsWith(srcDir) && (file.endsWith(".ts") || file.endsWith(".tsx")),
        );

        if (isSourceFile) {
          try {
            await regenerateDocs(root);
          } catch {
            // Ignore errors during dev
          }
        }
      });
    },
  };

  // SSG plugin (builtin, opt-in by default)
  const ssgDevCache = createDevServerCache();
  const ssgPlugin: Plugin = {
    name: "ox-content:ssg",

    configureServer(devServer) {
      const ssgOptions = resolvedOptions.ssg;
      if (!ssgOptions.enabled) return;

      const root = config?.root || process.cwd();
      const srcDir = path.resolve(root, resolvedOptions.srcDir);

      // Register dev server middleware
      devServer.middlewares.use(createDevServerMiddleware(resolvedOptions, root, ssgDevCache));

      // Watch for .md file add/unlink to invalidate nav cache
      devServer.watcher.on("add", (file: string) => {
        if (file.startsWith(srcDir) && isMarkdownFilePath(file, resolvedOptions.extensions)) {
          invalidateNavCache(ssgDevCache);
          devServer.ws.send({
            type: "custom",
            event: "ox-content:update",
            data: { file, type: "add" },
          });
        }
      });
      devServer.watcher.on("unlink", (file: string) => {
        if (file.startsWith(srcDir) && isMarkdownFilePath(file, resolvedOptions.extensions)) {
          invalidateNavCache(ssgDevCache);
          devServer.ws.send({
            type: "custom",
            event: "ox-content:update",
            data: { file, type: "unlink" },
          });
        }
      });

      // Watch for .md file changes to invalidate page cache
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

      const root = config?.root || process.cwd();

      try {
        const result = await buildSsg(resolvedOptions, root);

        if (result.files.length > 0) {
          console.log(`[ox-content] Generated ${result.files.length} output files`);
        }

        if (result.errors.length > 0) {
          for (const error of result.errors) {
            console.warn(`[ox-content] ${error}`);
          }
        }
      } catch (err) {
        console.error("[ox-content] SSG build failed:", err);
      }
    },
  };

  // Search plugin
  let searchIndexJson = "";
  const searchPlugin: Plugin = {
    name: "ox-content:search",

    resolveId(id) {
      if (id === "virtual:ox-content/search") {
        return "\0virtual:ox-content/search";
      }
      return null;
    },

    async load(id) {
      if (id === "\0virtual:ox-content/search") {
        const searchOptions = resolvedOptions.search;
        if (!searchOptions.enabled) {
          return "export const search = () => []; export const searchOptions = { enabled: false }; export default { search, searchOptions };";
        }

        const indexPath = resolvedOptions.base + "search-index.json";
        return generateSearchModule(searchOptions, indexPath);
      }
      return null;
    },

    async buildStart() {
      const searchOptions = resolvedOptions.search;
      if (!searchOptions.enabled) {
        return;
      }

      const root = config?.root || process.cwd();
      const srcDir = path.resolve(root, resolvedOptions.srcDir);

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

      const root = config?.root || process.cwd();
      const outDir = path.resolve(root, resolvedOptions.outDir);

      try {
        await writeSearchIndex(searchIndexJson, outDir);
        console.log("[ox-content] Search index written to", path.join(outDir, "search-index.json"));
      } catch (err) {
        console.warn("[ox-content] Failed to write search index:", err);
      }
    },
  };

  const plugins: Plugin[] = [mainPlugin, environmentPlugin, docsPlugin, ssgPlugin, searchPlugin];

  if (resolvedOptions.i18n) {
    plugins.push(createI18nPlugin(resolvedOptions));
  }

  if (resolvedOptions.ogViewer) {
    plugins.push(createOgViewerPlugin(resolvedOptions));
  }

  return plugins;
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
    i18n: resolveI18nOptions(options.i18n),
  };
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
  collectGitHubRepos,
  prefetchGitHubRepos,
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
