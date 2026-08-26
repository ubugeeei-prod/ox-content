import { describe, expect, it } from "vite-plus/test";
import { renderMarkdown } from "../render-markdown";
import { normalizeSelfClosingEmbeds, transformAllPlugins, transformBuiltinEmbeds } from ".";
import { isReservedBuiltinComponent } from "./embed-transform";

describe("native audio and video embeds", () => {
  it("renders HTTPS audio through transformAllPlugins", async () => {
    const html = await transformAllPlugins(
      '<Audio src="https://cdn.example.com/intro.mp3" title="Intro" />',
      { github: false, openGraph: false, mermaid: false, audio: true },
    );

    expect(html).toContain('class="ox-audio"');
    expect(html).toContain("<audio ");
    expect(html).toContain("controls");
    expect(html).toContain('aria-label="Intro"');
    expect(html).not.toMatch(/<\/Audio>/);
  });

  it("renders video poster, captions, and sizing", async () => {
    const html = await transformBuiltinEmbeds(
      '<Video src="/talk.mp4" poster="/talk.jpg" captions="/talk.en.vtt" width="1280" height="720" title="Talk"></Video>',
      { github: false, openGraph: false, video: true },
    );

    expect(html).toContain('class="ox-video"');
    expect(html).toContain('poster="/talk.jpg"');
    expect(html).toContain('width="1280"');
    expect(html).toContain("<track ");
    expect(html).toContain('src="/talk.en.vtt"');
  });

  it("leaves rejected or disabled sources as authored tags", async () => {
    const input = '<Audio src="javascript:alert(1)"></Audio>';
    const html = await transformBuiltinEmbeds(input, {
      github: false,
      openGraph: false,
      audio: true,
    });
    expect(html).toBe(input);

    const disabled = await transformBuiltinEmbeds(
      '<Audio src="https://cdn.example.com/intro.mp3"></Audio>',
      { github: false, openGraph: false },
    );
    expect(disabled).toContain("<Audio");
  });

  it("normalizes the self-closing authoring form", () => {
    expect(normalizeSelfClosingEmbeds('<Video src="/talk.mp4" />')).toBe(
      '<Video src="/talk.mp4"></Video>',
    );
  });

  it("reserves Audio and Video so MDX island lowering cannot swallow them", async () => {
    expect(isReservedBuiltinComponent("Audio")).toBe(true);
    expect(isReservedBuiltinComponent("Video")).toBe(true);

    const result = await renderMarkdown(
      '<Audio src="https://cdn.example.com/intro.mp3" />',
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
          audio: true,
        },
        ogViewer: false,
        search: false,
        toc: false,
        mdx: true,
      },
    );

    expect(result.html).toContain("ox-audio");
    expect(result.html).not.toContain('data-ox-island="Audio"');
    expect(result.html).not.toMatch(/<\/Audio>/);
  });
});
