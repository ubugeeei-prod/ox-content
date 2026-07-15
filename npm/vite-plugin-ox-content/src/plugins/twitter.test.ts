import { Buffer } from "node:buffer";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { transformMediaEmbeds } from "./media";
import { clearTweetCache } from "./twitter/fetch";
import { transformFetchedTweets } from "./twitter/transform";
import { createSyndicationToken, parseTweetReference } from "./twitter/url";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  clearTweetCache();
});

describe("fetched Twitter embeds", () => {
  it("normalizes post references and creates the widgets.js token", () => {
    expect(parseTweetReference("https://mobile.twitter.com/ox_content/status/123456?s=20")).toEqual(
      {
        id: "123456",
        url: "https://x.com/ox_content/status/123456",
      },
    );
    expect(parseTweetReference("123456")).toEqual({
      id: "123456",
      url: "https://x.com/i/web/status/123456",
    });
    expect(createSyndicationToken("1234567890123456789")).toBe(
      ((Number("1234567890123456789") / 1e15) * Math.PI).toString(36).replaceAll(/(0+|\.)/g, ""),
    );
  });

  it("renders fetched content, rewrites links, downloads media, and reuses disk cache", async () => {
    const root = await mkdtemp(path.join(tmpdir(), "ox-content-twitter-"));
    const cacheDir = path.join(root, "cache");
    const mediaOutputDir = path.join(root, "public", "tweets");
    const text = "Hello https://t.co/docs https://t.co/photo";
    const link = "https://t.co/docs";
    const photo = "https://t.co/photo";
    let requests = 0;

    globalThis.fetch = async (input) => {
      requests += 1;
      const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
      if (url.startsWith("https://cdn.syndication.twimg.com/")) {
        return {
          ok: true,
          json: async () => ({
            text,
            display_text_range: [0, text.length],
            entities: {
              urls: [
                {
                  url: link,
                  expanded_url: "https://example.com/docs",
                  display_url: "example.com/docs",
                  indices: [text.indexOf(link), text.indexOf(link) + link.length],
                },
              ],
              media: [
                {
                  url: photo,
                  indices: [text.indexOf(photo), text.indexOf(photo) + photo.length],
                },
              ],
            },
            mediaDetails: [
              {
                type: "photo",
                media_url_https: "https://pbs.twimg.com/media/post.jpg",
                ext_alt_text: "A release chart",
                original_info: { width: 1200, height: 675 },
              },
            ],
            user: {
              name: "Ox <Content>",
              screen_name: "ox_content",
              profile_image_url_https: "https://pbs.twimg.com/profile_images/avatar_normal.jpg",
            },
            created_at: "Tue Jul 15 03:00:00 +0000 2026",
          }),
        } as Response;
      }
      return {
        ok: true,
        arrayBuffer: async () => new Uint8Array([1, 2, 3]).buffer,
      } as Response;
    };

    const input = '<XPost url="https://x.com/ox_content/status/123456?s=20" />';
    const options = {
      fetch: true,
      cacheDir,
      mediaOutputDir,
      mediaPublicPath: "/tweets",
    };

    try {
      const html = await transformMediaEmbeds(input, { twitter: options });
      expect(html).toContain('class="ox-tweet ox-tweet--fetched"');
      expect(html).toContain("Ox &lt;Content&gt;");
      expect(html).toContain('href="https://example.com/docs"');
      expect(html).toContain(">example.com/docs</a>");
      expect(html).not.toContain(photo);
      expect(html).toContain('src="/tweets/123456-avatar.jpg"');
      expect(html).toContain('src="/tweets/123456-media-1.jpg"');
      expect(html).toContain('data-count="1"');

      await expect(readFile(path.join(mediaOutputDir, "123456-avatar.jpg"))).resolves.toEqual(
        Buffer.from([1, 2, 3]),
      );
      await expect(readFile(path.join(cacheDir, "123456-en.json"), "utf8")).resolves.toContain(
        '"screen_name":"ox_content"',
      );
      expect(requests).toBe(3);

      clearTweetCache();
      await expect(transformFetchedTweets(input, options)).resolves.toContain(
        'class="ox-tweet ox-tweet--fetched"',
      );
      expect(requests).toBe(3);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it("preserves the source element when the post cannot be fetched", async () => {
    globalThis.fetch = async () => ({ ok: false, status: 404 }) as Response;
    const input = '<Tweet id="987654">fallback summary</Tweet>';
    await expect(transformFetchedTweets(input, { fetch: true, cache: false })).resolves.toBe(input);
  });
});
