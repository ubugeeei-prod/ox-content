import { describe, expect, it } from "vite-plus/test";
import { renderMarkdown } from "../render-markdown";
import { normalizeSelfClosingEmbeds, transformAllPlugins, transformBuiltinEmbeds } from ".";
import { enrichSpeakerDeckEmbeds } from "./speaker-deck";

const PLAYER = "abcdef1234567890";
const SHARE = "https://speakerdeck.com/jane/my-cool-talk";
const PLAYER_URL = `https://speakerdeck.com/player/${PLAYER}`;

describe("Speaker Deck media embed", () => {
  it("renders a player URL with title and author", async () => {
    const html = await transformAllPlugins(
      `<SpeakerDeck url="${PLAYER_URL}" title="My Talk" author="Jane Doe" />`,
      { github: false, openGraph: false, mermaid: false, speakerDeck: true },
    );

    expect(html).toContain('class="ox-speaker-deck"');
    expect(html).toContain(PLAYER_URL);
    expect(html).toContain("My Talk");
    expect(html).toContain("Jane Doe");
    expect(html).toContain('loading="lazy"');
    expect(html).toContain("sandbox=");
    expect(html).toContain('referrerpolicy="strict-origin-when-cross-origin"');
    expect(html).not.toMatch(/<\/SpeakerDeck>/i);
  });

  it("renders a fallback link card when oEmbed fetch fails", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async () => {
      throw new Error("offline");
    }) as typeof fetch;

    try {
      const html = await transformBuiltinEmbeds(`<SpeakerDeck url="${SHARE}"></SpeakerDeck>`, {
        github: false,
        openGraph: false,
        speakerDeck: true,
      });
      expect(html).toContain("ox-speaker-deck--fallback");
      expect(html).toContain(SHARE);
      expect(html).toContain("My Cool Talk");
      expect(html).not.toContain("<iframe");
    } finally {
      globalThis.fetch = originalFetch;
    }
  });

  it("enriches a share URL from oEmbed metadata", async () => {
    const html = await enrichSpeakerDeckEmbeds(`<SpeakerDeck url="${SHARE}"></SpeakerDeck>`, () =>
      Promise.resolve(
        new Response(
          JSON.stringify({
            title: "My Talk",
            author_name: "Jane Doe",
            html: `<iframe src="${PLAYER_URL}"></iframe>`,
            thumbnail_url: "https://files.speakerdeck.com/presentations/slide.jpg",
          }),
          { status: 200, headers: { "content-type": "application/json" } },
        ),
      ),
    );

    expect(html).toContain(`player="${PLAYER}"`);
    expect(html).toContain('title="My Talk"');
    expect(html).toContain('author="Jane Doe"');
    expect(html).toContain('preview="https://files.speakerdeck.com/presentations/slide.jpg"');
  });

  it("fetches share metadata concurrently while preserving output order", async () => {
    const calls: string[] = [];
    const resolvers: Array<(response: Response) => void> = [];
    const pending = enrichSpeakerDeckEmbeds(
      `<SpeakerDeck url="${SHARE}"></SpeakerDeck><SpeakerDeck url="https://speakerdeck.com/jane/second-talk"></SpeakerDeck>`,
      async (input) => {
        calls.push(input);
        return new Promise<Response>((resolve) => {
          resolvers.push(resolve);
        });
      },
    );

    await Promise.resolve();
    expect(calls).toHaveLength(2);

    resolvers[1]!(
      new Response(
        JSON.stringify({
          title: "Second",
          html: '<iframe src="https://speakerdeck.com/player/22222222"></iframe>',
        }),
      ),
    );
    resolvers[0]!(
      new Response(
        JSON.stringify({
          title: "First",
          html: '<iframe src="https://speakerdeck.com/player/11111111"></iframe>',
        }),
      ),
    );

    const html = await pending;
    expect(html.indexOf('title="First"')).toBeLessThan(html.indexOf('title="Second"'));
  });

  it("leaves rejected URLs as the authored tag", async () => {
    const input = '<SpeakerDeck url="javascript:alert(1)"></SpeakerDeck>';
    const html = await transformBuiltinEmbeds(input, {
      github: false,
      openGraph: false,
      speakerDeck: true,
    });
    expect(html).toBe(input);
  });

  it("normalizes the self-closing authoring form", () => {
    expect(normalizeSelfClosingEmbeds(`<SpeakerDeck url="${PLAYER_URL}" />`)).toBe(
      `<SpeakerDeck url="${PLAYER_URL}"></SpeakerDeck>`,
    );
  });

  it("renders SpeakerDeck in .mdx as a built-in embed, not an island", async () => {
    const result = await renderMarkdown(
      `<SpeakerDeck url="${PLAYER_URL}" title="My Talk" />`,
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
          speakerDeck: true,
        },
        ogViewer: false,
        search: false,
        toc: false,
        mdx: true,
      },
    );

    expect(result.html).toContain("ox-speaker-deck");
    expect(result.html).not.toContain('data-ox-island="SpeakerDeck"');
    expect(result.html).not.toMatch(/<\/SpeakerDeck>/i);
  });
});
