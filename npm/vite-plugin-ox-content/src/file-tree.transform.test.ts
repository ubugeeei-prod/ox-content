import { describe, expect, it } from "vite-plus/test";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

describe("file-tree transform", () => {
  it("leaves file-tree fences literal unless opted in", async () => {
    const markdown = "```file-tree\n- src/\n  - index.ts **\n```\n";

    const defaultResult = await transformMarkdown(
      markdown,
      "docs/file-tree.md",
      createResolvedOptions(),
    );
    expect(defaultResult.html).not.toContain("ox-file-tree");

    const enabledResult = await transformMarkdown(
      markdown,
      "docs/file-tree.md",
      createResolvedOptions({
        fileTree: { enabled: true },
      }),
    );
    expect(enabledResult.html).toContain('class="ox-file-tree"');
    expect(enabledResult.html).toContain("ox-file-tree__dir");
    expect(enabledResult.html).toContain("ox-file-tree__highlight");
    expect(enabledResult.html).not.toContain("<script");
    expect(enabledResult.html).toContain("index.ts");
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
    math: { enabled: false },
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
