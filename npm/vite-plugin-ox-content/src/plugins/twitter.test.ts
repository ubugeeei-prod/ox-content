import { Buffer } from "node:buffer";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { transformMediaEmbeds } from "./media";
import { clearTweetCache } from "./twitter/fetch";
import { resolveTwitterEmbedOptions, transformFetchedTweets } from "./twitter/transform";
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

  it("defaults tweet timestamps to UTC and accepts a valid timeZone", () => {
    expect(resolveTwitterEmbedOptions({}).timeZone).toBe("UTC");
    expect(resolveTwitterEmbedOptions({ timeZone: "Europe/London" }).timeZone).toBe(
      "Europe/London",
    );
    expect(resolveTwitterEmbedOptions({ timeZone: "Not/AZone" }).timeZone).toBe("UTC");
    expect(resolveTwitterEmbedOptions({ timeZone: "   " }).timeZone).toBe("UTC");
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
            conversation_count: 7,
            retweet_count: 5,
            quote_count: 2,
            favorite_count: 1500,
            view_count: "20000",
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
      expect(html).not.toContain("ox-tweet--full");
      expect(html).toContain("Ox &lt;Content&gt;");
      expect(html).toContain('href="https://example.com/docs"');
      expect(html).toContain(">example.com/docs</a>");
      expect(html).not.toContain(photo);
      expect(html).toContain('src="/tweets/123456-avatar.jpg"');
      expect(html).toContain('src="/tweets/123456-media-1.jpg"');
      expect(html).toContain('data-count="1"');
      expect(html).toContain("Open post");
      expect(html).toContain("<strong>7</strong> replies");
      expect(html).toContain("<strong>5</strong> reposts");
      expect(html).toContain("<strong>2</strong> quotes");
      expect(html).toContain("<strong>1.5K</strong> likes");
      expect(html).toContain("<strong>20.0K</strong> views");

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

  it("renders a plain quoted post and omits the trailing quote URL", async () => {
    const quoteLink = "https://t.co/quote";
    const text = `See this ${quoteLink}`;
    const html = await renderFetched({
      text,
      display_text_range: [0, text.length],
      entities: {
        urls: [
          {
            url: quoteLink,
            expanded_url: "https://x.com/other/status/99",
            display_url: "x.com/other/status/99",
            indices: [text.indexOf(quoteLink), text.indexOf(quoteLink) + quoteLink.length],
          },
        ],
      },
      user: tweetUser(),
      quoted_tweet: { id_str: "99", text: "Quoted body", user: tweetUser("Other User", "other") },
    });
    expect(html).toContain('class="ox-tweet__quote"');
    expect(html).toContain("See this");
    expect(html).toContain("Quoted body");
    expect(html).toContain("Other User");
    expect(html).toContain(">@other<");
    expect(html).toContain('href="https://x.com/other/status/99"');
    expect(html).not.toContain("https://t.co/quote");
    expect(html).not.toContain(">x.com/other/status/99<");
  });

  it("materializes quoted avatars and photos with a distinct basename", async () => {
    const html = await renderFetched({
      text: "Quote with photos",
      user: tweetUser(),
      quoted_tweet: {
        id_str: "99",
        text: "Photo quote",
        user: {
          name: "Other",
          screen_name: "other",
          profile_image_url_https: "https://pbs.twimg.com/profile_images/q_normal.jpg",
        },
        mediaDetails: [
          {
            type: "photo",
            media_url_https: "https://pbs.twimg.com/media/quoted.jpg",
            ext_alt_text: "Quoted chart",
            original_info: { width: 800, height: 400 },
          },
        ],
      },
    });
    expect(html).toContain("Photo quote");
    expect(html).toContain('src="/tweets/555-quoted-avatar.jpg"');
    expect(html).toContain('src="/tweets/555-quoted-media-1.jpg"');
    expect(html).toContain('alt="Quoted chart"');
  });

  it("renders reply metadata and ignores unsafe handles", async () => {
    const html = await renderFetched({
      text: "Thanks",
      user: tweetUser(),
      in_reply_to_screen_name: "alice",
      in_reply_to_status_id_str: "42",
    });
    expect(html).toContain("Replying to @alice");
    expect(html).toContain('href="https://x.com/alice/status/42"');

    const unsafe = await renderFetched({
      text: "Thanks",
      user: tweetUser(),
      in_reply_to_screen_name: '"><script>',
      in_reply_to_status_id_str: "42",
    });
    expect(unsafe).toContain("Thanks");
    expect(unsafe).not.toContain("Replying to");
    expect(unsafe).not.toContain("<script>");
  });

  it("keeps the root post when the quoted post is missing or deleted", async () => {
    const html = await renderFetched({
      text: "Root still here",
      user: tweetUser(),
      quoted_tweet: { text: "This Post was deleted." },
    });
    expect(html).toContain('class="ox-tweet ox-tweet--fetched"');
    expect(html).toContain("Root still here");
    expect(html).not.toContain("ox-tweet__quote");
    expect(html).not.toContain("was deleted");
  });

  it("drops malformed quotes and ignores nested quoted_tweet", async () => {
    const malformed = await renderFetched({
      text: "Root body",
      user: tweetUser(),
      quoted_tweet: { text: 1, user: { name: "x", screen_name: "x" } },
    });
    expect(malformed).toContain("Root body");
    expect(malformed).not.toContain("ox-tweet__quote");

    const nested = await renderFetched({
      text: "Root body",
      user: tweetUser(),
      quoted_tweet: {
        id_str: "99",
        text: "Level one",
        user: tweetUser("One", "one"),
        quoted_tweet: {
          id_str: "88",
          text: "Level two should hide",
          user: tweetUser("Two", "two"),
        },
      },
    });
    expect(nested).toContain("Level one");
    expect(nested).not.toContain("Level two should hide");
    expect(nested.match(/class="ox-tweet__quote"/g)).toHaveLength(1);
  });

  it("escapes entities and uses UTF-16 display_text_range indices", async () => {
    const quoteLink = "https://t.co/quote";
    const docs = "https://example.com/docs";
    const text = `👍See ${docs} <this> ${quoteLink}`;
    const html = await renderFetched({
      text,
      display_text_range: [2, text.length],
      entities: {
        urls: [
          {
            url: docs,
            expanded_url: docs,
            display_url: "example.com/docs",
            indices: [text.indexOf(docs), text.indexOf(docs) + docs.length],
          },
          {
            url: quoteLink,
            expanded_url: "https://x.com/other/status/99",
            display_url: "x.com/other/status/99",
            indices: [text.indexOf(quoteLink), text.indexOf(quoteLink) + quoteLink.length],
          },
        ],
      },
      user: tweetUser("Ox <Content>"),
      quoted_tweet: {
        id_str: "99",
        text: 'alert("xss")',
        user: tweetUser("Evil <img>", "other"),
      },
    });
    expect(html).toContain("See ");
    expect(html).toContain('href="https://example.com/docs"');
    expect(html).toContain("&lt;this&gt;");
    expect(html).toContain("Ox &lt;Content&gt;");
    expect(html).toContain("Evil &lt;img&gt;");
    expect(html).toContain("alert(&quot;xss&quot;)");
    expect(html).not.toContain("👍");
    expect(html).not.toContain("https://t.co/quote");
  });
});

function tweetUser(name = "Ox Content", screenName = "ox_content") {
  return { name, screen_name: screenName };
}

async function renderFetched(data: unknown, id = "555"): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), "ox-tweet-q-"));
  globalThis.fetch = async (input) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    if (url.startsWith("https://cdn.syndication.twimg.com/")) {
      return { ok: true, json: async () => data } as Response;
    }
    return { ok: true, arrayBuffer: async () => new Uint8Array([1, 2, 3]).buffer } as Response;
  };
  try {
    return await transformFetchedTweets(`<XPost id="${id}" />`, {
      fetch: true,
      cache: false,
      mediaOutputDir: path.join(root, "media"),
      mediaPublicPath: "/tweets",
    });
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}
