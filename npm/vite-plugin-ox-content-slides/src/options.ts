import {
  resolveOgImageOptions,
  resolveSsgOptions,
  type ResolvedOptions,
} from "@ox-content/vite-plugin";
import { toNapiTheme } from "./napi";
import type { ResolvedSlidesPluginOptions } from "./internal-types";
import { normalizeExtension, normalizeRouteSegment, DEFAULT_EXTENSIONS } from "./path-utils";
import { resolveSlidePdfOptions } from "./pdf-options";
import type { OxContentSlidesOptions } from "./public-types";

/**
 * Resolves user-facing plugin options into runtime defaults.
 */
export function resolveOptions(options: OxContentSlidesOptions): ResolvedSlidesPluginOptions {
  const base = options.base ?? "/";
  const routeBase = normalizeRouteSegment(options.routeBase ?? "slides") || "slides";
  const resolved = {
    srcDir: options.srcDir ?? "slides",
    outDir: options.outDir ?? "dist",
    base,
    baseHref: base.endsWith("/") ? base : `${base}/`,
    routeBase,
    routePrefix: `/${routeBase}`,
    animations: options.animations ?? true,
    presenter: options.presenter ?? true,
    separator: options.separator ?? "---",
    extensions: [
      ...new Set([...(options.extensions ?? DEFAULT_EXTENSIONS)].map(normalizeExtension)),
    ],
    renderers: options.renderers ?? {},
    ssg: resolveSsgOptions(options.ssg ?? true),
    ogImageOptions: resolveOgImageOptions(options.ogImageOptions),
    pdf: resolveSlidePdfOptions(options.pdf),
    theme: options.theme ?? {},
    napiTheme: toNapiTheme(options.theme ?? {}, options.animations ?? true),
    markdown: {} as ResolvedOptions,
    gfm: options.gfm ?? true,
    footnotes: options.footnotes ?? true,
    tables: options.tables ?? true,
    taskLists: options.taskLists ?? true,
    strikethrough: options.strikethrough ?? true,
    highlight: options.highlight ?? false,
    highlightTheme: options.highlightTheme ?? "github-dark",
    highlightLangs: options.highlightLangs ?? [],
    mermaid: options.mermaid ?? false,
  } satisfies ResolvedSlidesPluginOptions;

  resolved.markdown = createMarkdownOptions(resolved);
  return resolved;
}

/**
 * Creates the Markdown transform options shared across slide rendering.
 */
export function createMarkdownOptions(options: ResolvedSlidesPluginOptions): ResolvedOptions {
  return {
    srcDir: options.srcDir,
    outDir: options.outDir,
    base: options.base,
    extensions: options.extensions,
    ssg: options.ssg,
    gfm: options.gfm,
    footnotes: options.footnotes,
    tables: options.tables,
    taskLists: options.taskLists,
    strikethrough: options.strikethrough,
    highlight: options.highlight,
    highlightTheme: options.highlightTheme,
    highlightLangs: options.highlightLangs,
    codeAnnotations: {
      enabled: false,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    },
    mermaid: options.mermaid,
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    ogImage: false,
    ogImageOptions: options.ogImageOptions,
    transformers: [],
    docs: false,
    search: {
      enabled: false,
      limit: 10,
      prefix: true,
      placeholder: "Search",
      hotkey: "k",
    },
    ogViewer: false,
    embeds: { github: false, openGraph: false },
    i18n: false,
  };
}
