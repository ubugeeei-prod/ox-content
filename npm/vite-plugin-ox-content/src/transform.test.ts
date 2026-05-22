import { describe, expect, it } from "vite-plus/test";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

describe("transformMarkdown", () => {
  it("uses Rust frontmatter parsing and Rust-built TOC trees", async () => {
    const result = await transformMarkdown(
      "---\ntitle: Guide\nmeta:\n  tags:\n    - rust\n  draft: false\n---\n# Intro\n\n## Install\n\n### CLI\n",
      "docs/guide.md",
      createResolvedOptions(),
    );

    expect(result.frontmatter).toEqual({
      title: "Guide",
      meta: { tags: ["rust"], draft: false },
    });
    expect(result.html).toContain('<h1 id="intro">Intro</h1>');
    expect(result.html).not.toContain("<hr>");
    expect(result.toc).toEqual([
      {
        depth: 1,
        text: "Intro",
        slug: "intro",
        children: [
          {
            depth: 2,
            text: "Install",
            slug: "install",
            children: [{ depth: 3, text: "CLI", slug: "cli", children: [] }],
          },
        ],
      },
    ]);
  });

  it("keeps malformed frontmatter behavior on the Rust path", async () => {
    const result = await transformMarkdown(
      "---\ntitle: [broken\n---\n# Body",
      "docs/broken.md",
      createResolvedOptions(),
    );

    expect(result.frontmatter).toEqual({});
    expect(result.html).toContain('<h1 id="body">Body</h1>');
    expect(result.html).not.toContain("title: [broken");
  });
});

function createResolvedOptions(): ResolvedOptions {
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
    highlight: false,
    highlightTheme: "github-dark",
    highlightLangs: [],
    codeAnnotations: {
      enabled: false,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    },
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
    },
    i18n: false,
  };
}
