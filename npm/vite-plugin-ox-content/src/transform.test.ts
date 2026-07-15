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
    expect(result.html).toMatchSnapshot();
  });

  it("keeps malformed frontmatter behavior on the Rust path", async () => {
    const result = await transformMarkdown(
      "---\ntitle: [broken\n---\n# Body",
      "docs/broken.md",
      createResolvedOptions(),
    );

    expect(result.frontmatter).toEqual({});
    expect(result.html).toMatchSnapshot();
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

    expect(result.html).toMatchSnapshot();
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

    expect(result.html).toMatchSnapshot();
  });

  it("keeps package-manager tabs disabled unless opted in", async () => {
    const markdown = "<pm>npm install -D vite</pm>";

    const defaultResult = await transformMarkdown(
      markdown,
      "docs/package-manager.md",
      createResolvedOptions(),
    );
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
    expect({ defaultHtml: defaultResult.html, optInHtml: optInResult.html }).toMatchSnapshot();
  });

  it("preserves wrapped continuation lines inside list items", async () => {
    const result = await transformMarkdown(
      [
        "- [Blacksmith](https://www.blacksmith.sh/) for sponsoring CI and",
        "  Testbox infrastructure across projects.",
        "- [Mates Inc.](https://eng.mates.education/) for supporting OSS and",
        "  adopting Vize in production.",
      ].join("\n"),
      "docs/credits.md",
      createResolvedOptions(),
    );

    expect(result.html).toMatchSnapshot();
  });

  it("forwards the autolinks option to bare URL rendering", async () => {
    const markdown = "See https://example.com/foo here.";
    const enabled = await transformMarkdown(
      markdown,
      "docs/autolinks.md",
      createResolvedOptions({ autolinks: true }),
    );
    const disabled = await transformMarkdown(
      markdown,
      "docs/autolinks.md",
      createResolvedOptions({ autolinks: false }),
    );

    expect(enabled.html).toContain('<a href="https://example.com/foo"');
    expect(disabled.html).toBe("<p>See https://example.com/foo here.</p>\n");
  });

  it("preserves safe raw media tags when sanitizing", async () => {
    const result = await transformMarkdown(
      [
        '<video controls muted playsinline poster="/poster.jpg">',
        '  <source src="/demo.webm" type="video/webm">',
        '  <track src="/captions.vtt" kind="captions" srclang="en" label="English" default>',
        "  Fallback",
        "</video>",
        '<picture><source media="(min-width: 800px)" srcset="/hero-large.jpg 2x, /hero.jpg 1x"><img src="/hero.jpg" alt="Hero"></picture>',
      ].join("\n"),
      "docs/media.md",
      createResolvedOptions({ sanitize: { enabled: true } }),
    );

    expect(result.html).toMatchSnapshot();
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

    expect(result.html).toMatchSnapshot();
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
    autolinks: true,
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
