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
import { generateCollectionsVirtualModule, resolveCollectionsOptions } from "./collections";
import type { BuiltinPmOptions, OxContentOptions, ResolvedOptions } from "./types";

export type { OxContentOptions } from "./types";
export type { LanguageRegistration, ThemeRegistration } from "shiki";
export type {
  CodeAnnotationSyntax,
  CodeAnnotationsOptions,
  ResolvedCodeAnnotationsOptions,
  WikiLinkOptions,
  ResolvedWikiLinkOptions,
  EmojiShortcodeOptions,
  ResolvedEmojiShortcodeOptions,
  AttrsOptions,
  ResolvedAttrsOptions,
  CodeImportOptions,
  ResolvedCodeImportOptions,
  SanitizeOptions,
  ResolvedSanitizeOptions,
  EditThisPageOptions,
  ResolvedEditThisPageOptions,
  CodeBlockLintOptions,
  ResolvedCodeBlockLintOptions,
  CodeBlockTypecheckOptions,
  ResolvedCodeBlockTypecheckOptions,
  DocsTestOptions,
  ResolvedDocsTestOptions,
  MarkdownDisplayFormat,
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
    autolinks: options.autolinks ?? options.gfm ?? true,
    highlight: options.highlight ?? false,
    highlightTheme: options.highlightTheme ?? "github-dark",
    highlightLangs: options.highlightLangs ?? [],
    codeAnnotations: resolveCodeAnnotationsOptions(options.codeAnnotations),
    wikiLinks: resolveWikiLinkOptions(options.wikiLinks, options.base ?? "/"),
    emojiShortcodes: resolveEmojiShortcodeOptions(options.emojiShortcodes),
    attrs: resolveAttrsOptions(options.attrs),
    codeImports: resolveCodeImportOptions(options.codeImports),
    sanitize: resolveSanitizeOptions(options.sanitize),
    editThisPage: resolveEditThisPageOptions(options.editThisPage),
    cjkEmphasis: options.cjkEmphasis ?? false,
    codeBlockLint: resolveCodeBlockLintOptions(options.codeBlockLint),
    codeBlockTypecheck: resolveCodeBlockTypecheckOptions(options.codeBlockTypecheck),
    docsTests: resolveDocsTestOptions(options.docsTests),
    mermaid: options.mermaid ?? false,
    frontmatter: options.frontmatter ?? true,
    toc: options.toc ?? true,
    tocMaxDepth: options.tocMaxDepth ?? 3,
    ogImage: options.ogImage ?? false,
    ogImageOptions: resolveOgImageOptions(options.ogImageOptions),
    transformers: options.transformers ?? [],
    docs: resolveDocsOptions(options.docs),
    search: resolveSearchOptions(options.search),
    collections: resolveCollectionsOptions(options.collections),
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
      pm: false,
      spotify: false,
      stackBlitz: false,
      twitter: false,
      bluesky: false,
      webContainer: false,
    };
  }

  return {
    github: resolveSingleEmbedOptions(options?.github),
    openGraph: resolveSingleEmbedOptions(options?.openGraph),
    pm: resolvePmOptions(options?.pm),
    spotify: options?.spotify === true,
    stackBlitz: options?.stackBlitz === true,
    twitter: options?.twitter === true,
    bluesky: options?.bluesky === true,
    webContainer: options?.webContainer === true,
  };
}

function resolveSingleEmbedOptions<T extends object>(options: boolean | T | undefined): T | false {
  if (options === false) return false;
  if (options === true || options === undefined) return {} as T;
  return options;
}

function resolvePmOptions(
  options: boolean | BuiltinPmOptions | undefined,
): BuiltinPmOptions | false {
  if (options === false || options === undefined) return false;
  if (options === true) return {};
  return options;
}

function resolveWikiLinkOptions(
  options: OxContentOptions["wikiLinks"],
  baseUrl: string,
): ResolvedOptions["wikiLinks"] {
  if (!options) return { enabled: false, baseUrl };
  if (options === true) return { enabled: true, baseUrl };
  return { enabled: true, baseUrl: options.baseUrl ?? baseUrl };
}

function resolveEmojiShortcodeOptions(
  options: OxContentOptions["emojiShortcodes"],
): ResolvedOptions["emojiShortcodes"] {
  if (!options) return { enabled: false, custom: {} };
  if (options === true) return { enabled: true, custom: {} };
  return { enabled: true, custom: options.custom ?? {} };
}

function resolveAttrsOptions(options: OxContentOptions["attrs"]): ResolvedOptions["attrs"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: options.enabled ?? true };
}

function resolveCodeImportOptions(
  options: OxContentOptions["codeImports"],
): ResolvedOptions["codeImports"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: true, rootDir: options.rootDir };
}

function resolveSanitizeOptions(
  options: OxContentOptions["sanitize"],
): ResolvedOptions["sanitize"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return {
    enabled: true,
    allowedTags: options.allowedTags,
    allowedAttributes: options.allowedAttributes,
    allowedUrlSchemes: options.allowedUrlSchemes,
  };
}

function resolveEditThisPageOptions(
  options: OxContentOptions["editThisPage"],
): ResolvedOptions["editThisPage"] {
  if (!options) return { enabled: false, branch: "main", label: "Edit this page" };
  if (options === true) return { enabled: false, branch: "main", label: "Edit this page" };
  return {
    enabled: Boolean(options.repoUrl),
    repoUrl: options.repoUrl,
    branch: options.branch ?? "main",
    rootDir: options.rootDir,
    label: options.label ?? "Edit this page",
  };
}

function resolveCodeBlockLintOptions(
  options: OxContentOptions["codeBlockLint"],
): ResolvedOptions["codeBlockLint"] {
  if (!options) {
    return { enabled: false, requireLanguage: false, trailingSpaces: true, mode: "warn" };
  }
  if (options === true) {
    return { enabled: true, requireLanguage: false, trailingSpaces: true, mode: "warn" };
  }
  return {
    enabled: true,
    languages: options.languages,
    requireLanguage: options.requireLanguage ?? false,
    trailingSpaces: options.trailingSpaces ?? true,
    mode: options.mode ?? "warn",
  };
}

function resolveCodeBlockTypecheckOptions(
  options: OxContentOptions["codeBlockTypecheck"],
): ResolvedOptions["codeBlockTypecheck"] {
  if (!options) {
    return {
      enabled: false,
      languages: ["ts", "tsx"],
      requireMeta: true,
      tsgoCommand: "tsgo",
      mode: "warn",
    };
  }
  if (options === true) {
    return {
      enabled: true,
      languages: ["ts", "tsx"],
      requireMeta: true,
      tsgoCommand: "tsgo",
      mode: "warn",
    };
  }
  return {
    enabled: true,
    languages: options.languages ?? ["ts", "tsx"],
    requireMeta: options.requireMeta ?? true,
    tsgoCommand: options.tsgoCommand ?? "tsgo",
    mode: options.mode ?? "warn",
  };
}

function resolveDocsTestOptions(
  options: OxContentOptions["docsTests"],
): ResolvedOptions["docsTests"] {
  if (!options) return { enabled: false, languages: ["js", "jsx", "ts", "tsx"], requireMeta: true };
  if (options === true) {
    return { enabled: true, languages: ["js", "jsx", "ts", "tsx"], requireMeta: true };
  }
  return {
    enabled: true,
    languages: options.languages ?? ["js", "jsx", "ts", "tsx"],
    requireMeta: options.requireMeta ?? true,
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
