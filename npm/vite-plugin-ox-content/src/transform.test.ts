import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

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

  it("runs opt-in native transforms without changing default behavior", async () => {
    const result = await transformMarkdown(
      [
        "# Guide {.lead}",
        "",
        "See [[install|Install guide]] :rocket:",
        "",
        '<a href="javascript:alert(1)" onclick="alert(1)">bad</a>',
      ].join("\n"),
      "docs/guide.md",
      createResolvedOptions({
        wikiLinks: { enabled: true, baseUrl: "/docs" },
        emojiShortcodes: { enabled: true, custom: {} },
        attrs: { enabled: true },
        sanitize: { enabled: true },
      }),
    );

    expect(result.html).toContain('class="lead"');
    expect(result.html).toContain('<a href="/docs/install">Install guide</a>');
    expect(result.html).toContain("\u{1F680}");
    expect(result.html).toContain("<a>bad</a>");
    expect(result.html).not.toContain("javascript:");
    expect(result.html).not.toContain("onclick");
  });

  it("can append edit links and import source snippets when opted in", async () => {
    const result = await transformMarkdown(
      "<<< @/README.md{1-1}",
      resolve(repoRoot, "docs/import.md"),
      createResolvedOptions({
        codeImports: {
          enabled: true,
          rootDir: repoRoot,
        },
        editThisPage: {
          enabled: true,
          repoUrl: "https://github.com/ubugeeei-prod/ox-content",
          branch: "main",
          rootDir: repoRoot,
          label: "Suggest an edit",
        },
      }),
    );

    expect(result.html).toContain("<pre><code");
    expect(result.html).toContain("Suggest an edit");
    expect(result.html).toContain(
      "https://github.com/ubugeeei-prod/ox-content/edit/main/docs/import.md",
    );
  });

  it("keeps package-manager tabs disabled unless opted in", async () => {
    const markdown = "<pm>npm install -D vite</pm>";

    const defaultResult = await transformMarkdown(
      markdown,
      "docs/package-manager.md",
      createResolvedOptions(),
    );
    expect(defaultResult.html).toContain("<pm>npm install -D vite</pm>");

    const optInResult = await transformMarkdown(
      markdown,
      "docs/package-manager.md",
      createResolvedOptions({
        embeds: {
          github: {},
          openGraph: {},
          pm: {},
          spotify: false,
          stackBlitz: false,
          twitter: false,
          bluesky: false,
          webContainer: false,
        },
      }),
    );
    expect(optInResult.html).toContain("ox-tabs");
    expect(optInResult.html).toContain("pnpm add -D vite");
  });

  it("sanitizes after opt-in embeds are rendered", async () => {
    const result = await transformMarkdown(
      [
        '<Spotify url="https://open.spotify.com/track/abc123"></Spotify>',
        '<script>alert("bad")</script>',
      ].join("\n"),
      "docs/safe-embed.md",
      createResolvedOptions({
        sanitize: { enabled: true },
        embeds: {
          github: {},
          openGraph: {},
          pm: false,
          spotify: true,
          stackBlitz: false,
          twitter: false,
          bluesky: false,
          webContainer: false,
        },
      }),
    );

    expect(result.html).toContain("ox-spotify");
    expect(result.html).toContain("https://open.spotify.com/embed/track/abc123");
    expect(result.html).not.toContain("<script");
    expect(result.html).not.toContain("alert");
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
    highlight: false,
    highlightTheme: "github-dark",
    highlightLangs: [],
    codeAnnotations: {
      enabled: false,
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
