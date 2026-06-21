import { describe, expect, it } from "vite-plus/test";
import type { ResolvedOptions } from "./types";
import { transformMarkdown } from "./transform";

describe("code annotations", () => {
  it("preserves pre and line classes after syntax highlighting", async () => {
    const markdown = `\`\`\`ts annotate="highlight:1;warning:2;error:3"
const first = 1;
const second = 2;
throw new Error("boom");
\`\`\`
`;

    const result = await transformMarkdown(
      markdown,
      "docs/code-annotations.md",
      createResolvedOptions(),
    );

    expect(result.html).toMatchSnapshot();
  });

  it("keeps non-annotated highlighted blocks unchanged", async () => {
    const markdown = `\`\`\`ts
const value = 1;
const next = 2;
\`\`\`
`;

    const result = await transformMarkdown(markdown, "docs/plain-code.md", createResolvedOptions());

    expect(result.html).toMatchSnapshot();
  });

  it("supports VitePress-style fence metadata", async () => {
    const markdown = `\`\`\`ts:line-numbers=7 {1,3} [config.ts]
const first = true;
const second = false;
const third = true;
\`\`\`
`;

    const result = await transformMarkdown(
      markdown,
      "docs/vitepress-meta.md",
      createResolvedOptions({
        codeAnnotations: {
          enabled: true,
          notation: "vitepress",
          metaKey: "annotate",
          defaultLineNumbers: false,
        },
      }),
    );

    expect(result.html).toMatchSnapshot();
  });

  it("supports VitePress-style inline directives", async () => {
    const markdown = `\`\`\`ts
// [!code focus:2]
const first = true;
const second = false;
console.log("before") // [!code --]
console.log("after") // [!code ++]
console.warn("careful") // [!code warning]
throw new Error("boom") // [!code error]
\`\`\`
`;

    const result = await transformMarkdown(
      markdown,
      "docs/vitepress-inline.md",
      createResolvedOptions({
        codeAnnotations: {
          enabled: true,
          notation: "vitepress",
          metaKey: "annotate",
          defaultLineNumbers: false,
        },
      }),
    );

    expect(result.html).toMatchSnapshot();
  });

  it("supports VitePress-style escape-next-line directives", async () => {
    const markdown = `\`\`\`ts
// [!code escape]
console.warn(literal) // [!code warning]
console.warn(annotated) // [!code warning]
\`\`\`
`;

    const result = await transformMarkdown(
      markdown,
      "docs/vitepress-escape.md",
      createResolvedOptions({
        highlight: false,
        codeAnnotations: {
          enabled: true,
          notation: "vitepress",
          metaKey: "annotate",
          defaultLineNumbers: false,
        },
      }),
    );

    expect(result.html).toMatchSnapshot();
    expect(result.html.match(/ox-code-line--warning/g)?.length ?? 0).toBe(1);
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
    },
    gfm: true,
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    highlight: true,
    highlightTheme: "github-dark",
    highlightLangs: [],
    codeAnnotations: {
      enabled: true,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    },
    wikiLinks: { enabled: false, baseUrl: "/" },
    emojiShortcodes: { enabled: false, custom: {} },
    attrs: { enabled: false },
    codeImports: { enabled: false },
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
