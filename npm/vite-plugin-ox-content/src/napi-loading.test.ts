import { describe, expect, it } from "vite-plus/test";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

// This file's first use of the plugin has to be the concurrent one, which is
// why it is a file of its own: the NAPI load is cached per module instance,
// so any earlier call here would warm it and hide what is being tested.
describe("loading the NAPI bindings", () => {
  it("serves every caller that arrives while the first load is in flight", async () => {
    // A build starts transforming several pages at once, and all of them reach
    // the loader before any of them has finished loading. Each used to see the
    // "already attempted" flag with the result still unset, decide the bindings
    // were unavailable, and throw — so the first page rendered and the rest
    // failed.
    const pages = Array.from({ length: 16 }, (_, i) =>
      transformMarkdown(
        `# page ${i}\n\nText for page ${i}.\n`,
        `docs/page-${i}.md`,
        createResolvedOptions(),
      ),
    );

    const settled = await Promise.allSettled(pages);
    const rejected = settled.filter((result) => result.status === "rejected");

    expect(rejected).toEqual([]);
    for (const [index, result] of settled.entries()) {
      expect(result.status).toBe("fulfilled");
      if (result.status === "fulfilled") {
        expect(result.value.html).toContain(`page ${index}`);
      }
    }
  });
});

function createResolvedOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return {
    srcDir: "content",
    outDir: "dist",
    base: "/",
    extensions: [".md", ".markdown", ".mdx"],
    ssg: {
      enabled: true,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
      pagination: false,
      breadcrumbs: false,
      readerChrome: false,
      localeSwitcher: false,
      a11y: false,
    },
    gfm: true,
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    autolinks: true,
    highlight: false,
    codeAnnotations: {
      enabled: false,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    },
    wikiLinks: { enabled: false, baseUrl: "/" },
    emojiShortcodes: { enabled: false, custom: {} },
    attrs: { enabled: false },
    badges: { enabled: false },
    containers: { enabled: false, types: {} },
    images: { enabled: false, lazy: true },
    codeImports: { enabled: false },
    includes: { enabled: false },
    cards: { enabled: false },
    steps: { enabled: false },
    fileTree: { enabled: false },
    sanitize: { enabled: false },
    editThisPage: { enabled: false, branch: "main", label: "Edit this page" },
    cjkEmphasis: false,
    codeBlockLint: {
      enabled: false,
      requireLanguage: false,
      trailingSpaces: true,
      mode: "warn",
    },
    codeBlockTypecheck: {
      enabled: false,
      languages: ["ts", "tsx"],
      requireMeta: true,
      tsgoCommand: "tsgo",
      mode: "warn",
    },
    docsTests: { enabled: false, languages: ["js", "jsx", "ts", "tsx"], requireMeta: true },
    mermaid: false,
    math: { enabled: false },
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    ogImage: false,
    ogImageOptions: {
      width: 1200,
      height: 630,
      cache: true,
      concurrency: 1,
      vuePlugin: "vitejs",
    },
    transformers: [],
    docs: false,
    search: {
      enabled: true,
      limit: 10,
      prefix: true,
      placeholder: "Search documentation...",
      hotkey: "/",
    },
    collections: { enabled: false, collections: {} },
    ogViewer: false,
    embeds: {
      github: {},
      openGraph: {},
      pm: false,
      spotify: false,
      stackBlitz: false,
      twitter: false,
      bluesky: false,
      webContainer: false,
    },
    i18n: false,
    ...overrides,
  };
}
