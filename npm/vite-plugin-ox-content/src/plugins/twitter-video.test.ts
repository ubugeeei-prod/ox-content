import { access, mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { clearTweetCache } from "./twitter/fetch";
import { transformFetchedTweets } from "./twitter/transform";
import { selectBestMp4Url } from "./twitter/video";
import type { TwitterEmbedOptions } from "./twitter/types";

const originalFetch = globalThis.fetch;
const LOW = "https://video.twimg.com/amplify_video/1/vid/low.mp4";
const MID = "https://video.twimg.com/amplify_video/1/vid/mid.mp4";
const HIGH = "https://video.twimg.com/amplify_video/1/vid/high.mp4";
const HLS = "https://video.twimg.com/amplify_video/1/pl/stream.m3u8";
const POSTER = "https://pbs.twimg.com/amplify_video_thumb/1/img/poster.jpg";
const GIF = "https://video.twimg.com/tweet_video/cat.mp4";

afterEach(() => {
  globalThis.fetch = originalFetch;
  clearTweetCache();
});

describe("selectBestMp4Url", () => {
  it("picks the highest-bitrate video/mp4 deterministically", () => {
    expect(
      selectBestMp4Url([
        { content_type: "application/x-mpegURL", url: HLS },
        { content_type: "video/mp4", url: LOW, bitrate: 256000 },
        { content_type: "video/mp4", url: HIGH, bitrate: 2176000 },
        { content_type: "video/mp4", url: MID, bitrate: 832000 },
      ]),
    ).toBe(HIGH);
  });

  it("skips empty, non-mp4, and disallowed hosts", () => {
    expect(selectBestMp4Url([])).toBeUndefined();
    expect(selectBestMp4Url([{ content_type: "application/x-mpegURL", url: HLS }])).toBeUndefined();
    expect(
      selectBestMp4Url([
        { content_type: "video/mp4", url: "https://example.com/clip.mp4", bitrate: 1 },
      ]),
    ).toBeUndefined();
  });
});

describe("fetched Twitter video media", () => {
  it("downloads the highest-bitrate mp4 and renders native video", async () => {
    await withTweet(
      [videoDetails()],
      { downloadVideo: true },
      async ({ html, requested, mediaDir }) => {
        expect(requested).toContain(HIGH);
        expect(requested).not.toContain(LOW);
        expect(requested).not.toContain(MID);
        expect(html).toContain(
          '<video class="ox-tweet__media-item" src="/tweets/555-media-1.mp4" poster="/tweets/555-media-1-poster.jpg" width="1604" height="1252" controls playsinline preload="none">',
        );
        expect(html).toContain(">Watch on X</a></video>");
        expect(html).toContain('data-count="1"');
        expect(html).not.toContain("video.twimg.com");
        expect(html).not.toContain(" muted loop");
        await expect(readFile(path.join(mediaDir, "555-media-1.mp4"))).resolves.toHaveLength(3);
      },
    );
  });

  it("renders animated GIFs as muted looping video", async () => {
    await withTweet(
      [
        {
          type: "animated_gif",
          media_url_https: POSTER,
          video_info: { variants: [{ content_type: "video/mp4", url: GIF, bitrate: 0 }] },
        },
      ],
      { downloadVideo: true },
      ({ html }) => {
        expect(html).toContain('src="/tweets/555-media-1.mp4"');
        expect(html).toContain(' controls playsinline preload="none" muted loop>');
        expect(html).toContain("Watch on X");
        expect(html).not.toContain("video.twimg.com");
      },
    );
  });

  it("keeps a poster and permalink when downloadVideo is false", async () => {
    await withTweet([videoDetails()], {}, ({ html, requested }) => {
      expect(requested.some((url) => url.includes("video.twimg.com"))).toBe(false);
      expect(html).not.toContain("<video");
      expect(html).toContain('src="/tweets/555-media-1-poster.jpg"');
      expect(html).toContain("ox-tweet__media-fallback");
      expect(html).toContain("Watch on X");
      expect(html).not.toContain("video.twimg.com");
    });
  });

  it("falls back when variants are empty or invalid", async () => {
    await withTweet(
      [{ type: "video", media_url_https: POSTER, video_info: { variants: [] } }],
      { downloadVideo: true },
      ({ html, requested }) => {
        expect(requested.some((url) => url.includes("video.twimg.com"))).toBe(false);
        expect(html).not.toContain("<video");
        expect(html).toContain('src="/tweets/555-media-1-poster.jpg"');
        expect(html).toContain("Watch on X");
      },
    );
    await withTweet(
      [
        {
          type: "video",
          media_url_https: POSTER,
          video_info: { variants: [{ content_type: "application/x-mpegURL", url: HLS }] },
        },
      ],
      { downloadVideo: true },
      ({ html }) => {
        expect(html).not.toContain("<video");
        expect(html).toContain("Watch on X");
      },
    );
  });

  it("rejects oversized videos without failing the build", async () => {
    await withTweet(
      [videoDetails()],
      { downloadVideo: true, maxVideoBytes: 2 },
      async ({ html, mediaDir }) => {
        expect(html).not.toContain("<video");
        expect(html).toContain("Watch on X");
        await expect(access(path.join(mediaDir, "555-media-1.mp4"))).rejects.toMatchObject({
          code: "ENOENT",
        });
      },
      (url) =>
        url.startsWith("https://video.twimg.com/")
          ? ({
              ok: true,
              headers: headerMap({ "content-type": "video/mp4", "content-length": "99" }),
              arrayBuffer: async () => {
                throw new Error("should not read an oversized body");
              },
            } as Response)
          : undefined,
    );
    await withTweet(
      [videoDetails()],
      { downloadVideo: true, maxVideoBytes: 2 },
      ({ html }) => {
        expect(html).not.toContain("<video");
        expect(html).toContain("Watch on X");
      },
      (url) =>
        url.startsWith("https://video.twimg.com/")
          ? ({
              ok: true,
              headers: headerMap({ "content-type": "video/mp4" }),
              arrayBuffer: async () => Uint8Array.from([1, 2, 3]).buffer,
            } as Response)
          : undefined,
    );
  });

  it("falls back on wrong MIME types and failed downloads", async () => {
    await withTweet(
      [videoDetails()],
      { downloadVideo: true },
      ({ html }) => {
        expect(html).not.toContain("<video");
        expect(html).toContain("Watch on X");
        expect(html).not.toContain("video.twimg.com");
      },
      (url) =>
        url.startsWith("https://video.twimg.com/")
          ? ({
              ok: true,
              headers: headerMap({ "content-type": "video/webm" }),
              arrayBuffer: async () => Uint8Array.from([1, 2, 3]).buffer,
            } as Response)
          : undefined,
    );
    await withTweet(
      [videoDetails()],
      { downloadVideo: true },
      ({ html }) => {
        expect(html).not.toContain("<video");
        expect(html).toContain("Watch on X");
      },
      (url) =>
        url.startsWith("https://video.twimg.com/")
          ? ({ ok: false, status: 500 } as Response)
          : undefined,
    );
    await withTweet(
      [videoDetails()],
      { downloadVideo: true },
      ({ html }) => {
        expect(html).not.toContain("<video");
        expect(html).toContain("Watch on X");
      },
      (url) => {
        if (url.startsWith("https://video.twimg.com/")) throw new Error("network down");
        return undefined;
      },
    );
  });
});

function videoDetails(): Record<string, unknown> {
  return {
    type: "video",
    media_url_https: POSTER,
    original_info: { width: 1604, height: 1252 },
    video_info: {
      variants: [
        { content_type: "application/x-mpegURL", url: HLS },
        { content_type: "video/mp4", url: LOW, bitrate: 256000 },
        { content_type: "video/mp4", url: MID, bitrate: 832000 },
        { content_type: "video/mp4", url: HIGH, bitrate: 2176000 },
      ],
    },
  };
}

function headerMap(values: Record<string, string>): { get: (name: string) => string | null } {
  const map = new Map(Object.entries(values).map(([key, value]) => [key.toLowerCase(), value]));
  return { get: (name: string) => map.get(name.toLowerCase()) ?? null };
}

function videoBytes(): Response {
  return {
    ok: true,
    headers: headerMap({ "content-type": "video/mp4", "content-length": "3" }),
    arrayBuffer: async () => Uint8Array.from([1, 2, 3]).buffer,
  } as Response;
}

async function withTweet(
  mediaDetails: unknown[],
  twitter: TwitterEmbedOptions,
  assert: (ctx: { html: string; requested: string[]; mediaDir: string }) => Promise<void> | void,
  respond?: (url: string) => Response | undefined,
): Promise<void> {
  const root = await mkdtemp(path.join(tmpdir(), "ox-content-twitter-video-"));
  const mediaDir = path.join(root, "media");
  const requested: string[] = [];
  globalThis.fetch = async (input) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    requested.push(url);
    if (url.startsWith("https://cdn.syndication.twimg.com/")) {
      return {
        ok: true,
        json: async () => ({
          text: "Watch this",
          display_text_range: [0, 10],
          mediaDetails,
          user: { name: "Ox", screen_name: "ox_content" },
        }),
      } as Response;
    }
    const custom = respond?.(url);
    if (custom) return custom;
    if (url.startsWith("https://video.twimg.com/")) return videoBytes();
    return { ok: true, arrayBuffer: async () => Uint8Array.from([1, 2, 3]).buffer } as Response;
  };

  try {
    const html = await transformFetchedTweets(
      '<XPost url="https://x.com/ox_content/status/555" />',
      {
        fetch: true,
        cache: false,
        mediaOutputDir: mediaDir,
        mediaPublicPath: "/tweets",
        ...twitter,
      },
    );
    await assert({ html, requested, mediaDir });
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}
