/**
 * Vite Plugin for Ox Content
 *
 * Uses Vite's Environment API for SSG-focused Markdown processing.
 * Provides separate environments for client and server rendering.
 */

import * as path from "path";
import type { Plugin, ViteDevServer, ResolvedConfig } from "vite";
import "./virtual";
import { createMarkdownEnvironment } from "./environment";
import { transformMarkdown } from "./transform";
import { extractDocs, generateMarkdown, writeDocs } from "./docs";
import { buildSsg } from "./ssg";
import { notFoundSearchExcludeIds } from "./not-found";
import { BlogFeedError } from "./blog";
import { PageResourceError } from "./resources";
import { createKatexAssetsPlugin } from "./plugins/math-assets";
import { buildSearchIndex, writeSearchIndex, generateSearchModule } from "./search";
import {
  createDevServerMiddleware,
  createDevServerCache,
  invalidateNavCache,
  invalidatePageCache,
} from "./dev-server";
import { createOgViewerPlugin } from "./og-viewer";
import { createI18nPlugin } from "./i18n";
import { isMarkdownFilePath } from "./markdown";
import { generateCollectionsVirtualModule } from "./collections";
import type { OxContentOptions, ResolvedOptions } from "./types";
import { resolveOptions } from "./resolve-options";

export type { OxContentOptions } from "./types";
export type { TwitterEmbedOptions } from "./plugins";
export type {
  CodeAnnotationSyntax,
  CodeAnnotationsOptions,
  ResolvedCodeAnnotationsOptions,
  WikiLinkOptions,
  ResolvedWikiLinkOptions,
  EmojiShortcodeOptions,
  ResolvedEmojiShortcodeOptions,
  MathOptions,
  ResolvedMathOptions,
  AttrsOptions,
  ResolvedAttrsOptions,
  BadgeOptions,
  ResolvedBadgeOptions,
  MagicLinkOptions,
  MagicLinkAlias,
  MagicLinkImageOverride,
  ResolvedMagicLinkOptions,
  ContainerOptions,
  ContainerTypeOptions,
  ResolvedContainerOptions,
  ImageOptions,
  ResolvedImageOptions,
  ResourcesOptions,
  ResolvedResourcesOptions,
  CodeImportOptions,
  ResolvedCodeImportOptions,
  IncludeOptions,
  ResolvedIncludeOptions,
  CardOptions,
  ResolvedCardOptions,
  StepsOptions,
  ResolvedStepsOptions,
  FileTreeIconOptions,
  FileTreeOptions,
  ResolvedFileTreeOptions,
  SanitizeOptions,
  ResolvedSanitizeOptions,
  EditThisPageOptions,
  ResolvedEditThisPageOptions,
  CodeBlockLintOptions,
  ResolvedCodeBlockLintOptions,
  CodeBlockTypecheckOptions,
  ResolvedCodeBlockTypecheckOptions,
  TypedHoverOptions,
  ResolvedTypedHoverOptions,
  DocsTestOptions,
  ResolvedDocsTestOptions,
  OgImageRenderer,
  OgImageSatoriFont,
  OgImageSatoriFontWeight,
  OgImageSatoriOptions,
  MarkdownDisplayFormat,
  DocsOptions,
  ResolvedDocsOptions,
  DocEntry,
  ParamDoc,
  ReturnDoc,
  ExtractedDocs,
  SsgOptions,
  ResolvedSsgOptions,
  MarkdownSourceOptions,
  ResolvedMarkdownSourceOptions,
  JsonLdOptions,
  JsonLdPublisherOptions,
  ResolvedJsonLd,
  A11yOptions,
  ResolvedA11y,
  ReaderChromeOptions,
  ResolvedReaderChrome,
  NotFoundOptions,
  ResolvedNotFoundOptions,
  TeamLink,
  TeamMember,
  TeamOptions,
  ResolvedTeamOptions,
  ContributorsOptions,
  ResolvedContributors,
  SiteMapsOptions,
  ResolvedSiteMapsOptions,
  PublishStateOptions,
  ResolvedPublishStateOptions,
  PermalinksOptions,
  ResolvedPermalinksOptions,
  CascadeOptions,
  ResolvedCascadeOptions,
  RedirectsOptions,
  ResolvedRedirectsOptions,
  BlogAuthor,
  BlogFeedFailurePolicy,
  BlogFeedSource,
  BlogOptions,
  ResolvedBlogFeedSource,
  ResolvedBlogOptions,
  FeedFormat,
  FeedsOptions,
  ResolvedFeedsOptions,
  PwaOptions,
  ResolvedPwaOptions,
  TaxonomiesOptions,
  ResolvedTaxonomiesOptions,
  SearchOptions,
  ResolvedSearchOptions,
  SearchDocument,
  SearchResult,
  CollectionEntry,
  CollectionOptions,
  CollectionsOptions,
  ResolvedCollectionOptions,
  ResolvedCollectionsOptions,
  CollectionIncludeField,
  CollectionManifest,
  CollectionQueryBuilder,
  CollectionQueryOperator,
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
  BuiltinPmOptions,
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
    createCollectionsPlugin(resolvedOptions, getRoot),
    createSearchPlugin(resolvedOptions, getRoot),
  ];

  if (resolvedOptions.math.enabled) {
    plugins.push(createKatexAssetsPlugin());
  }

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
      if (id === "virtual:ox-content/config" || id === "virtual:ox-content/runtime") {
        return "\0" + id;
      }

      if (isMarkdownFilePath(id, resolvedOptions.extensions)) {
        return id;
      }

      return null;
    },

    async load(id) {
      if (id === "\0virtual:ox-content/config" || id === "\0virtual:ox-content/runtime") {
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

function createCollectionsPlugin(resolvedOptions: ResolvedOptions, getRoot: () => string): Plugin {
  const moduleId = "\0virtual:ox-content/collections";
  let moduleCode: Promise<string> | undefined;

  const invalidate = (devServer: ViteDevServer) => {
    moduleCode = undefined;
    const mod = devServer.moduleGraph.getModuleById(moduleId);
    if (mod) {
      devServer.moduleGraph.invalidateModule(mod);
      devServer.ws.send({ type: "full-reload" });
    }
  };

  return {
    name: "ox-content:collections",

    resolveId(id) {
      return id === "virtual:ox-content/collections" ? moduleId : null;
    },

    async load(id) {
      if (id !== moduleId) {
        return null;
      }
      moduleCode ??= generateCollectionsVirtualModule(getRoot(), resolvedOptions);
      return moduleCode;
    },

    configureServer(devServer) {
      if (!resolvedOptions.collections.enabled) {
        return;
      }

      const srcDir = path.resolve(getRoot(), resolvedOptions.srcDir);
      devServer.watcher.add(srcDir);
      devServer.watcher.on("all", (_event, file) => {
        if (file.startsWith(srcDir) && isMarkdownFilePath(file, resolvedOptions.extensions)) {
          invalidate(devServer);
        }
      });
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
        if (err instanceof PageResourceError || err instanceof BlogFeedError) {
          throw err;
        }
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

function searchPublishState(
  resolvedOptions: ResolvedOptions,
  command: "build" | "serve",
): ResolvedOptions["publishState"] {
  const publishState = resolvedOptions.publishState ?? {
    enabled: false,
    includeDrafts: false,
  };
  return {
    ...publishState,
    includeDrafts: publishState.includeDrafts || command === "serve",
  };
}

function createSearchPlugin(resolvedOptions: ResolvedOptions, getRoot: () => string): Plugin {
  let searchIndexJson = "";
  let command: "build" | "serve" = "build";

  return {
    name: "ox-content:search",

    config(_config, env) {
      command = env.command;
    },

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
          searchPublishState(resolvedOptions, command),
          notFoundSearchExcludeIds(resolvedOptions.ssg.notFound),
          resolvedOptions.mdx,
        );
        console.log("[ox-content] Search index built");
      } catch (err) {
        console.warn("[ox-content] Failed to build search index:", err);
      }
    },

    configureServer(devServer) {
      const searchOptions = resolvedOptions.search;
      if (!searchOptions.enabled) {
        return;
      }

      // The index is only written to disk by the static build (closeBundle);
      // without a dev handler the client's fetch falls through to the html
      // fallback and search reports the index unavailable. Serve it from
      // memory, rebuilt lazily after a Markdown change.
      const srcDir = path.resolve(getRoot(), resolvedOptions.srcDir);
      let stale = false;
      devServer.watcher.on("all", (event, file) => {
        if (event !== "add" && event !== "change" && event !== "unlink") {
          return;
        }
        const relative = path.relative(srcDir, file);
        const isInsideSrcDir =
          relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
        if (isInsideSrcDir && isMarkdownFilePath(file, resolvedOptions.extensions)) {
          stale = true;
        }
      });

      const indexPath = resolvedOptions.base + "search-index.json";
      devServer.middlewares.use(async (req, res, next) => {
        if (req.url?.split("?")[0] !== indexPath) {
          return next();
        }
        try {
          if (stale || !searchIndexJson) {
            searchIndexJson = await buildSearchIndex(
              srcDir,
              resolvedOptions.base,
              resolvedOptions.extensions,
              searchPublishState(resolvedOptions, command),
              notFoundSearchExcludeIds(resolvedOptions.ssg.notFound),
              resolvedOptions.mdx,
            );
            stale = false;
          }
          res.setHeader("Content-Type", "application/json; charset=utf-8");
          res.end(searchIndexJson);
        } catch (err) {
          next(err);
        }
      });
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

export {
  resolveBadgeOptions,
  resolveBuiltinEmbedOptions,
  resolveMathOptions,
} from "./resolve-options";
export { resolveCardOptions } from "./card-options";
export { resolveIncludeOptions } from "./include-options";
export { resolveStepsOptions } from "./step-options";
export { resolveFileTreeOptions } from "./file-tree-options";
export { resolveHeadingPermalinksOptions } from "./heading-permalinks-options";
export { resolveTypedHoverOptions } from "./typed-hover";

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
export {
  IncrementalMarkdownParser,
  IncrementalMarkdownRenderer,
  createIncrementalMarkdownParser,
  createIncrementalMarkdownRenderer,
  renderMarkdownStream,
  type IncrementalMarkdownParseAppendOptions,
  type IncrementalMarkdownParseResult,
  type IncrementalMarkdownParserOptions,
  type IncrementalMarkdownRenderAppendOptions,
  type IncrementalMarkdownRenderResult,
  type IncrementalMarkdownRendererOptions,
  type MarkdownChunkSource,
} from "./incremental";
export { transformMarkdown } from "./transform";
export { createMarkdownProcessor, renderMarkdown } from "./render-markdown";
export type { MarkdownProcessor } from "./render-markdown";
export { isMdxFilePath, resolveMdxForFilePath } from "./markdown";
export {
  collectMdxIslandNamesFromHtml,
  collectMdxJsxNamesFromAst,
  discoverRegisteredMdxComponents,
  intersectHydratableComponentNames,
  intersectRegisteredComponentNames,
  isRegisteredComponent,
  type ComponentRegistry,
  type DiscoverRegisteredMdxComponentsInput,
} from "./mdx-islands";
export {
  resolveContentRootPath,
  resolveDocumentComponentImports,
  stripViteQuery,
  type DocumentImportDiagnostic,
  type DocumentImportDiagnosticCode,
  type ResolveDocumentComponentImportsInput,
  type ResolveDocumentComponentImportsResult,
  type ResolvedDocumentComponentImport,
} from "./document-imports";
export {
  discoverDocumentMdxIslands,
  type DiscoverDocumentMdxIslandsInput,
  type DiscoverDocumentMdxIslandsResult,
} from "./document-islands";
export {
  renderIslandComponentImports,
  type GlobalComponentMap,
  type RenderIslandComponentImportsInput,
} from "./island-codegen";
export { applyIslandSsrHtml, type RenderIslandFn } from "./island-ssr";
export { resolveImageOptions } from "./resolve-image-options";
export {
  createFrameworkMarkdownOptions,
  escapeSvelteMarkup,
  renderHtmlToFrameworkCode,
  renderHtmlToReactCreateElement,
  renderHtmlToReactComponent,
  renderHtmlToSvelteComponent,
  renderHtmlToVueComponent,
  renderHtmlToVueH,
  type FrameworkCodegenMode,
  type FrameworkCodegenTarget,
  type FrameworkComponentIsland,
  type FrameworkMarkdownOptions,
  type FrameworkRenderTarget,
  type FrameworkTransformData,
} from "./framework";
export {
  extractCodeBlocks,
  extractDocsTests,
  lintCodeBlocks,
  typecheckCodeBlocks,
  type CodeBlockDiagnostic,
  type ExtractedCodeBlock,
  type TypecheckCodeBlockOptions,
} from "./code-blocks";
export {
  collectDocsTests,
  DocsTestRunError,
  runDocsTests,
  writeDocsTestFiles,
  type CollectedDocsTest,
  type DocsTestFileOptions,
  type DocsTestHarnessOptions,
  type DocsTestRunResult,
  type DocsTestSource,
  type DocsTestWriteResult,
  type RunDocsTestsOptions,
  type WrittenDocsTestFile,
} from "./docs-tests";
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
export { renderHead, resolveHeadValidation } from "./page-head";
export type {
  HeadAlternate,
  HeadDiagnostic,
  HeadInput,
  HeadJsonLd,
  HeadLink,
  HeadMeta,
  HeadValidationMode,
  RenderedHead,
  SiteHead,
} from "./page-head";
export { resolveNotFoundOptions } from "./not-found";
export { resolveSiteMapsOptions } from "./site-maps";
export { resolveMarkdownSourceOptions } from "./markdown-source";
export {
  classifyPublishState,
  resolvePublishStateOptions,
  partitionPublishedPages,
} from "./publish-state";
export { resolvePermalinksOptions, resolveCascadeOptions } from "./permalinks";
export { resolveRedirectsOptions } from "./redirects";
export { resolveFeedsOptions } from "./feeds";
export {
  BlogFeedError,
  resolveBlogOptions,
  resolveBlogCollectionName,
  readingTimeMinutes,
} from "./blog";
export { resolvePwaOptions } from "./pwa";
export { resolveTaxonomiesOptions } from "./taxonomies";
export { resolveVersionsOptions } from "./versions";
export { resolveResourcesOptions, PageResourceError } from "./resources";
export { resolveTeamOptions } from "./team";
export { resolveSectionIndexOptions } from "./section-index";
export { resolveSearchOptions, buildSearchIndex, writeSearchIndex } from "./search";
export {
  buildCollectionManifest,
  defineCollection,
  defineCollections,
  generateCollectionsVirtualModule,
  resolveCollectionsOptions,
} from "./collections";
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
  ThemeTokens,
  SocialLinks,
  ThemeEmbed,
  ResolvedThemeConfig,
  HeaderNavItem,
  LocaleLabel,
  SidebarItem,
  ThemeAnnouncement,
} from "./theme";
export type { PageChromeFlags } from "./header-chrome";
export {
  parsePageChromeFlags,
  resolveHeaderNavItems,
  resolveLocaleLabel,
  resolvePageChromeOption,
} from "./header-chrome";
export * from "./types";

// JSX Runtime
export { jsx, jsxs, Fragment, renderToString, raw, when, each } from "./jsx-html";
export type { JSXNode, JSXChild, JSXProps, JSXElementType } from "./jsx-html";

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
  GitHubSourceCommit,
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
