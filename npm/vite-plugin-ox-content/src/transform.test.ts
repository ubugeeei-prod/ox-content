import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { transformMarkdown } from "./transform";
import type { ResolvedOptions } from "./types";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
describe("transformMarkdown", () => {
  it("enables MDX from the resource id and honors explicit overrides", async () => {
    const source = "import Card from './Card'\n\n<Card>Visible copy</Card>\n";
    const inferred = await transformMarkdown(source, "docs/Guide.MDX?raw", createResolvedOptions());
    const optedOut = await transformMarkdown(
      source,
      "docs/guide.mdx",
      createResolvedOptions({ mdx: false }),
    );
    const optedIn = await transformMarkdown(
      source,
      "docs/guide.md",
      createResolvedOptions({ mdx: true }),
    );

    expect(inferred.html).toContain('data-ox-island="Card"');
    expect(inferred.html).not.toContain("import Card");
    expect(optedOut.html).toContain("import Card");
    expect(optedIn.html).toContain('data-ox-island="Card"');
  });

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

  it("keeps attrs on inline links and transformed images", async () => {
    const result = await transformMarkdown(
      [
        "[slides](https://example.com){#deck .text-xl data-kind=deck}",
        "",
        "![alt](./image.png){#hero .w-1/2 .mx-auto width=480 height=320}",
      ].join("\n"),
      "docs/attrs-images.md",
      createResolvedOptions({
        attrs: { enabled: true },
        images: { enabled: true, lazy: true },
      }),
    );

    expect(result.html).toContain('<p><a href="https://example.com"');
    expect(result.html).toContain('id="deck" class="text-xl" data-kind="deck">slides</a></p>');
    expect(result.html).toContain(
      '<img src="./image.png" alt="alt" id="hero" class="w-1/2 mx-auto" loading="lazy" width="480" height="320">',
    );
    expect(result.html).not.toContain('<p id="deck"');
    expect(result.html).not.toContain("{#hero");
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

  it("leaves {badge} markup literal unless opted in", async () => {
    const markdown = "{badge:tip}Beta{/badge}";

    const defaultResult = await transformMarkdown(
      markdown,
      "docs/badges.md",
      createResolvedOptions(),
    );
    expect(defaultResult.html).not.toContain("ox-badge");
    expect(defaultResult.html).toContain("{badge:tip}Beta{/badge}");

    const enabledResult = await transformMarkdown(
      markdown,
      "docs/badges.md",
      createResolvedOptions({
        badges: { enabled: true },
      }),
    );
    expect(enabledResult.html).toContain('<span class="ox-badge ox-badge--tip">Beta</span>');
    expect(enabledResult.html).not.toContain("{badge:tip}");
  });

  it("leaves {kbd} markup literal unless opted in", async () => {
    const markdown = "{kbd:Ctrl+K}";

    const defaultResult = await transformMarkdown(
      markdown,
      "docs/keyboard-keys.md",
      createResolvedOptions(),
    );
    expect(defaultResult.html).not.toContain("ox-kbd");
    expect(defaultResult.html).toContain("{kbd:Ctrl+K}");

    const enabledResult = await transformMarkdown(
      markdown,
      "docs/keyboard-keys.md",
      createResolvedOptions({
        keyboardKeys: { enabled: true, aliases: {}, style: "words" },
      }),
    );
    expect(enabledResult.html).toContain('<kbd class="ox-kbd ox-kbd--combo">');
    expect(enabledResult.html).toContain('<kbd class="ox-kbd__key">Ctrl</kbd>');
    expect(enabledResult.html).not.toContain("{kbd:");
  });

  it("leaves {link} markup literal unless opted in", async () => {
    const markdown = "{link:@ryoppippi}";

    const defaultResult = await transformMarkdown(
      markdown,
      "docs/magic-links.md",
      createResolvedOptions(),
    );
    expect(defaultResult.html).not.toContain("ox-magic-link");
    expect(defaultResult.html).toContain("{link:@ryoppippi}");

    const enabledResult = await transformMarkdown(
      markdown,
      "docs/magic-links.md",
      createResolvedOptions({
        magicLinks: { enabled: true, aliases: {}, favicon: false, imageOverrides: [] },
      }),
    );
    expect(enabledResult.html).toContain('class="ox-magic-link ox-magic-link--github"');
    expect(enabledResult.html).toContain('href="https://github.com/ryoppippi"');
    expect(enabledResult.html).not.toContain("{link:@ryoppippi}");
  });

  it("leaves ::: containers literal unless opted in", async () => {
    const markdown = "::: tip\nHello **world**\n:::\n";

    const defaultResult = await transformMarkdown(
      markdown,
      "docs/containers.md",
      createResolvedOptions(),
    );
    expect(defaultResult.html).not.toContain("ox-container");

    const enabledResult = await transformMarkdown(
      markdown,
      "docs/containers.md",
      createResolvedOptions({
        containers: { enabled: true, types: {} },
      }),
    );
    expect(enabledResult.html).toContain('class="ox-container ox-container--tip"');
    expect(enabledResult.html).toContain("<strong>world</strong>");
    expect(enabledResult.html).not.toContain(":::");
  });

  it("drops hostile container titles and attributes", async () => {
    const result = await transformMarkdown(
      "::: tip[<img src=x onerror=alert(1)>]{#ok .ok onclick=alert(1)}\nBody\n:::\n",
      "docs/containers-xss.md",
      createResolvedOptions({
        containers: { enabled: true, types: {} },
      }),
    );

    expect(result.html).toContain('id="ok"');
    expect(result.html).toContain("ox-container--tip ok");
    expect(result.html).not.toContain("onclick");
    expect(result.html).not.toContain("<img");
    expect(result.html).toMatch(/(&lt;|&#x3C;|&#60;)img/);
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
          appleMusic: false,
          speakerDeck: false,
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
          appleMusic: false,
          speakerDeck: false,
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
  return createDocsResolvedOptions({ highlight: false, ...overrides });
}
