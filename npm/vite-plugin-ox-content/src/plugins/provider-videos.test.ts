import { afterEach, describe, expect, it } from "vite-plus/test";
import {
  clearProviderVideoCache,
  enrichProviderVideoEmbeds,
  parseVideoProviderReference,
  type ProviderVideoFetch,
} from "./provider-videos";

const originalWarn = console.warn;

afterEach(() => {
  console.warn = originalWarn;
  clearProviderVideoCache();
});

describe("video provider cards", () => {
  it("parses supported Vimeo and Twitch URLs", () => {
    expect(parseVideoProviderReference("Vimeo", "https://vimeo.com/123456789")).toEqual({
      provider: "vimeo",
      kind: "video",
      canonicalUrl: "https://vimeo.com/123456789",
      title: "Vimeo video 123456789",
      apiUrl: "https://vimeo.com/api/oembed.json?url=https%3A%2F%2Fvimeo.com%2F123456789",
      vimeoId: "123456789",
    });
    expect(
      parseVideoProviderReference("Vimeo", "https://player.vimeo.com/video/123456789"),
    ).toMatchObject({
      provider: "vimeo",
      canonicalUrl: "https://vimeo.com/123456789",
    });
    expect(
      parseVideoProviderReference("Twitch", "https://www.twitch.tv/videos/40464143"),
    ).toMatchObject({
      provider: "twitch",
      kind: "video",
      twitchVideoId: "40464143",
    });
    expect(
      parseVideoProviderReference("Twitch", "https://www.twitch.tv/twitchdev/clip/FriendlySlug"),
    ).toMatchObject({
      provider: "twitch",
      kind: "clip",
      author: "twitchdev",
      twitchClip: "FriendlySlug",
    });
    expect(parseVideoProviderReference("Twitch", "https://www.twitch.tv/twitchdev")).toMatchObject({
      provider: "twitch",
      kind: "channel",
      title: "twitchdev on Twitch",
    });
  });

  it("rejects unsafe or unsupported video URLs", () => {
    expect(parseVideoProviderReference("Vimeo", "http://vimeo.com/123456789")).toBeNull();
    expect(parseVideoProviderReference("Vimeo", "https://vimeo.com.evil/123456789")).toBeNull();
    expect(
      parseVideoProviderReference("Twitch", "https://user:pass@www.twitch.tv/videos/40464143"),
    ).toBeNull();
    expect(parseVideoProviderReference("Twitch", "https://www.twitch.tv/directory")).toBeNull();
  });

  it("enriches Vimeo metadata and adds explicit lazy iframe URLs", async () => {
    const requests: string[] = [];
    const fetchImpl: ProviderVideoFetch = async (input) => {
      const url = requestUrl(input);
      requests.push(url);
      return okJson({
        title: "Fetched Vimeo",
        author_name: "Vimeo Staff",
        thumbnail_url: "https://i.vimeocdn.com/video/123.jpg",
        duration: 90,
      });
    };
    const input = [
      '<Vimeo url="https://vimeo.com/123456789"></Vimeo>',
      '<Twitch url="https://www.twitch.tv/videos/40464143"></Twitch>',
      '<Twitch url="https://clips.twitch.tv/FriendlySlug"></Twitch>',
    ].join("\n");

    const html = await enrichProviderVideoEmbeds(
      input,
      { vimeo: { iframe: true }, twitch: { iframe: true, parent: "docs.example.com" } },
      fetchImpl,
    );
    expect(html).toContain('title="Fetched Vimeo"');
    expect(html).toContain('author="Vimeo Staff"');
    expect(html).toContain('duration="1:30"');
    expect(html).toContain('embed="https://player.vimeo.com/video/123456789?dnt=1"');
    expect(html).toContain(
      'embed="https://player.twitch.tv/?video=v40464143&amp;parent=docs.example.com&amp;autoplay=false"',
    );
    expect(html).toContain(
      'embed="https://clips.twitch.tv/embed?clip=FriendlySlug&amp;parent=docs.example.com&amp;autoplay=false"',
    );

    await enrichProviderVideoEmbeds(input, { vimeo: {}, twitch: {} }, fetchImpl);
    expect(requests).toEqual([
      "https://vimeo.com/api/oembed.json?url=https%3A%2F%2Fvimeo.com%2F123456789",
    ]);
  });

  it("keeps Twitch iframes disabled when no parent is configured", async () => {
    const html = await enrichProviderVideoEmbeds(
      '<Twitch url="https://www.twitch.tv/videos/40464143"></Twitch>',
      { twitch: { iframe: true } },
    );

    expect(html).toContain('title="Twitch video 40464143"');
    expect(html).not.toContain('embed="https://player.twitch.tv/');
  });

  it("keeps deterministic card attributes when Vimeo metadata is unavailable", async () => {
    const warnings: string[] = [];
    console.warn = (message?: unknown) => {
      warnings.push(String(message));
    };
    const html = await enrichProviderVideoEmbeds(
      '<Vimeo url="https://vimeo.com/123456789"></Vimeo>',
      { vimeo: {} },
      async () => new Response("{}", { status: 429 }),
    );

    expect(html).toContain('title="Vimeo video 123456789"');
    expect(warnings[0]).toContain("429");
    expect(warnings[0]).toContain("link-only video card");
  });
});

function requestUrl(input: Parameters<typeof fetch>[0]): string {
  if (typeof input === "string") return input;
  if (input instanceof URL) return input.href;
  return input.url;
}

function okJson(value: unknown): Response {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "content-type": "application/json" },
  });
}
