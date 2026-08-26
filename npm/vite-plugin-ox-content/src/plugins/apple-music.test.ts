import { describe, expect, it } from "vite-plus/test";
import { renderMarkdown } from "../render-markdown";
import { normalizeSelfClosingEmbeds, transformAllPlugins, transformBuiltinEmbeds } from ".";

describe("Apple Music media embed", () => {
  it("renders a localized share URL through transformAllPlugins", async () => {
    const html = await transformAllPlugins(
      '<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989" />',
      { github: false, openGraph: false, mermaid: false, appleMusic: true },
    );

    expect(html).toContain('class="ox-apple-music"');
    expect(html).toContain(
      "https://embed.music.apple.com/gb/album/1989-taylors-version/1708308989",
    );
    expect(html).toContain('title="Apple Music"');
    expect(html).toContain('loading="lazy"');
    expect(html).not.toMatch(/<\/AppleMusic>/i);
  });

  it("leaves rejected URLs as the authored tag", async () => {
    const input =
      '<AppleMusic url="https://music.apple.com.evil.com/us/album/folklore/1524801260"></AppleMusic>';
    const html = await transformBuiltinEmbeds(input, {
      github: false,
      openGraph: false,
      appleMusic: true,
    });
    expect(html).toBe(input);
  });

  it("normalizes the self-closing authoring form", () => {
    expect(
      normalizeSelfClosingEmbeds(
        '<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989" />',
      ),
    ).toBe(
      '<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989"></AppleMusic>',
    );
  });

  it("renders AppleMusic in .mdx as a built-in embed, not an island", async () => {
    const result = await renderMarkdown(
      '<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989" />',
      "/virtual/article.mdx",
      {
        ssg: false,
        frontmatter: false,
        highlight: false,
        embeds: {
          github: false,
          openGraph: false,
          twitter: false,
          bluesky: false,
          appleMusic: true,
        },
        ogViewer: false,
        search: false,
        toc: false,
        mdx: true,
      },
    );

    expect(result.html).toContain("ox-apple-music");
    expect(result.html).not.toContain('data-ox-island="AppleMusic"');
    expect(result.html).not.toMatch(/<\/AppleMusic>/i);
  });
});
