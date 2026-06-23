import { importNapiModuleSync } from "./napi";
import type { ResolvedOptions, TocEntry } from "./types";

export type FrameworkRenderTarget = "html" | "native";
export type FrameworkCodegenTarget = "react" | "vue" | "svelte";
export type FrameworkCodegenMode = "innerHtml" | "expression" | "renderFunction" | "component";

export interface FrameworkMarkdownOptions {
  srcDir: string;
  outDir: string;
  base: string;
  extensions: string[];
  gfm: boolean;
  frontmatter?: boolean;
  toc: boolean;
  tocMaxDepth: number;
  codeAnnotations?: {
    enabled?: boolean;
    metaKey?: string;
  };
  embeds?: {
    github?: ResolvedOptions["embeds"]["github"];
    openGraph?: ResolvedOptions["embeds"]["openGraph"];
  };
}

export interface FrameworkComponentIsland {
  name: string;
  props: Record<string, unknown>;
  id: string;
  content?: string;
}

export interface FrameworkTransformData {
  html: string;
  frontmatter: Record<string, unknown>;
  toc: TocEntry[];
}

export function createFrameworkMarkdownOptions(options: FrameworkMarkdownOptions): ResolvedOptions {
  return {
    srcDir: options.srcDir,
    outDir: options.outDir,
    base: options.base,
    extensions: options.extensions,
    ssg: {
      enabled: false,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
    },
    gfm: options.gfm,
    frontmatter: options.frontmatter ?? false,
    toc: options.toc,
    tocMaxDepth: options.tocMaxDepth,
    codeAnnotations: {
      enabled: options.codeAnnotations?.enabled ?? false,
      notation: "attribute",
      metaKey: options.codeAnnotations?.metaKey ?? "annotate",
      defaultLineNumbers: false,
    },
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    highlight: false,
    highlightTheme: "github-dark",
    highlightLangs: [],
    mermaid: false,
    ogImage: false,
    ogImageOptions: {
      vuePlugin: "vitejs",
      width: 1200,
      height: 630,
      cache: true,
      concurrency: 1,
    },
    transformers: [],
    docs: false,
    ogViewer: false,
    search: {
      enabled: false,
      limit: 10,
      prefix: true,
      placeholder: "Search...",
      hotkey: "k",
    },
    collections: { enabled: false, collections: {} },
    embeds: {
      github: options.embeds?.github ?? {},
      openGraph: options.embeds?.openGraph ?? {},
      pm: false,
      spotify: false,
      stackBlitz: false,
      twitter: false,
      bluesky: false,
      webContainer: false,
    },
    i18n: false,
    wikiLinks: { enabled: false, baseUrl: options.base },
    emojiShortcodes: { enabled: false, custom: {} },
    attrs: { enabled: false },
    codeImports: { enabled: false },
    sanitize: { enabled: false },
    editThisPage: { enabled: false, branch: "main", label: "Edit this page" },
    cjkEmphasis: false,
    codeBlockLint: { enabled: false, requireLanguage: false, trailingSpaces: true, mode: "warn" },
    codeBlockTypecheck: {
      enabled: false,
      languages: ["ts", "tsx"],
      requireMeta: true,
      tsgoCommand: "tsgo",
      mode: "warn",
    },
    docsTests: {
      enabled: false,
      languages: ["js", "jsx", "ts", "tsx"],
      requireMeta: true,
    },
  } as ResolvedOptions;
}

export function renderHtmlToReactCreateElement(
  html: string,
  islands: readonly FrameworkComponentIsland[] = [],
): string {
  return renderHtmlToFrameworkCode(html, "react", "expression", islands);
}

export function renderHtmlToVueH(
  html: string,
  islands: readonly FrameworkComponentIsland[] = [],
): string {
  return renderHtmlToFrameworkCode(html, "vue", "expression", islands);
}

export function renderHtmlToFrameworkCode(
  html: string,
  target: FrameworkCodegenTarget,
  mode: FrameworkCodegenMode,
  islands: readonly FrameworkComponentIsland[] = [],
): string {
  return importNapiModuleSync().renderFrameworkComponentCode(
    html,
    target,
    toNapiIslands(islands),
    mode,
  );
}

export function renderHtmlToReactComponent(
  html: string,
  islands: readonly FrameworkComponentIsland[] = [],
): string {
  return renderHtmlToFrameworkCode(html, "react", "component", islands);
}

export function renderHtmlToVueComponent(
  html: string,
  islands: readonly FrameworkComponentIsland[] = [],
): string {
  return renderHtmlToFrameworkCode(html, "vue", "component", islands);
}

export function renderHtmlToSvelteComponent(html: string): string {
  return renderHtmlToFrameworkCode(html, "svelte", "component");
}

export function escapeSvelteMarkup(html: string): string {
  return importNapiModuleSync().escapeSvelteMarkup(html);
}

function toNapiIslands(islands: readonly FrameworkComponentIsland[]) {
  return islands.map((island) => ({
    name: island.name,
    props: island.props,
    id: island.id,
    content: island.content,
  }));
}
