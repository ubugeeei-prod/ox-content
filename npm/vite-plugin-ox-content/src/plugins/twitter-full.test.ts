import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { clearTweetCache } from "./twitter/fetch";
import { transformFetchedTweets } from "./twitter/transform";
import { renderTweetText } from "./twitter/text";
import type { TweetBodyData, TwitterEmbedOptions } from "./twitter/types";

const originalFetch = globalThis.fetch;
const POSTER = "https://pbs.twimg.com/amplify_video_thumb/1/img/poster.jpg";

afterEach(() => {
  globalThis.fetch = originalFetch;
  clearTweetCache();
});

describe("full-fidelity Tweet cards", () => {
  it("keeps compact markup by default and gates full chrome behind appearance", async () => {
    const compact = await renderCard({ text: "Hello", user: user() });
    expect(compact).toContain('class="ox-tweet ox-tweet--fetched"');
    expect(compact).not.toContain("ox-tweet--full");
    expect(compact).not.toContain("ox-tweet__actions");
    expect(compact).not.toContain("ox-tweet__badge");

    const full = await renderCard(
      { text: "Hello", id_str: "555", user: user() },
      { appearance: "full" },
    );
    expect(full).toContain('class="ox-tweet ox-tweet--fetched ox-tweet--full"');
    expect(full).toContain("ox-tweet__actions");
    expect(full).toContain("ox-tweet__replies");
    expect(full).not.toContain("<script");
  });

  it("lets a per-element appearance override the site default", async () => {
    const full = await renderCard(
      { text: "Hello", id_str: "555", user: user() },
      {},
      {
        html: '<XPost id="555" appearance="full" />',
      },
    );
    expect(full).toContain("ox-tweet--full");

    const compact = await renderCard(
      { text: "Hello", id_str: "555", user: user() },
      { appearance: "full" },
      {
        html: '<XPost id="555" appearance="compact" />',
      },
    );
    expect(compact).not.toContain("ox-tweet--full");
  });

  it("renders URL, hashtag, mention, and symbol entities in one UTF-16 pass", () => {
    const docs = "https://example.com/docs";
    const text = `👍See ${docs} #ox @ox_content $OX 日本語`;
    const html = renderTweetText(
      body(
        text,
        {
          urls: [
            {
              url: docs,
              expanded_url: docs,
              display_url: "example.com/docs",
              indices: span(text, docs),
            },
          ],
          hashtags: [{ text: "ox", indices: span(text, "#ox") }],
          user_mentions: [{ screen_name: "ox_content", indices: span(text, "@ox_content") }],
          symbols: [{ text: "OX", indices: span(text, "$OX") }],
        },
        2,
      ),
    );
    expect(html).toContain('href="https://example.com/docs"');
    expect(html).toContain('href="https://x.com/hashtag/ox"');
    expect(html).toContain('href="https://x.com/ox_content"');
    expect(html).toContain('href="https://x.com/search?q=%24OX"');
    expect(html).toContain("日本語");
    expect(html).not.toContain("👍");
  });

  it("drops unsafe mention hrefs and keeps JA/EN body text", () => {
    const text = "こんにちは <script> @bad";
    const html = renderTweetText(
      body(text, {
        user_mentions: [{ screen_name: '"><script>', indices: span(text, "@bad") }],
      }),
    );
    expect(html).toContain("こんにちは");
    expect(html).toContain("@bad");
    expect(html).toContain("&lt;script&gt;");
    expect(html).not.toContain('href="https://x.com/');
  });

  it("renders 1–4 photo grids, reply, quote+media, and video fallback", async () => {
    for (const count of [1, 2, 3, 4]) {
      const photos = await renderCard(photoPost(count), { appearance: "full" });
      expect(photos).toContain(`data-count="${count}"`);
      expect(photos.match(/ox-tweet__media-item/g)).toHaveLength(count);
    }

    const reply = await renderCard(
      {
        text: "Thanks",
        id_str: "555",
        user: user(),
        in_reply_to_screen_name: "alice",
        in_reply_to_status_id_str: "42",
      },
      { appearance: "full" },
    );
    expect(reply).toContain("Replying to @alice");

    const quoted = await renderCard(
      {
        text: "See this",
        id_str: "555",
        user: user(),
        quoted_tweet: {
          id_str: "99",
          text: "Quoted 写真",
          user: { name: "Other", screen_name: "other", is_blue_verified: true },
          mediaDetails: [
            { type: "photo", media_url_https: "https://pbs.twimg.com/media/quoted.jpg" },
          ],
        },
      },
      { appearance: "full" },
    );
    expect(quoted).toContain("Quoted 写真");
    expect(quoted).toContain("ox-tweet__badge--blue");
    expect(quoted).toContain("/tweets/555-quoted-media-1.jpg");

    const video = await renderCard(
      {
        text: "Watch",
        id_str: "555",
        user: user(),
        mediaDetails: [
          { type: "video", media_url_https: POSTER, original_info: { width: 16, height: 9 } },
        ],
      },
      { appearance: "full" },
    );
    expect(video).toContain("ox-tweet__media-fallback");
    expect(video).toContain("Watch on X");
    expect(video).not.toContain("video.twimg.com");
  });

  it("renders verified variants, intents, copy link, and reply counts", async () => {
    const blue = await renderCard(
      {
        text: "Hi",
        id_str: "555",
        favorite_count: 1500,
        conversation_count: 12,
        retweet_count: 44,
        quote_count: 3,
        view_count: "22000",
        user: { ...user(), is_blue_verified: true },
      },
      { appearance: "full" },
    );
    expect(blue).toContain("ox-tweet__badge--blue");
    expect(blue).toContain('href="https://x.com/intent/like?tweet_id=555"');
    expect(blue).toContain('href="https://x.com/intent/tweet?in_reply_to=555"');
    expect(blue).toContain("ox-tweet__action--copy");
    expect(blue).toContain("data-ox-tweet-copy");
    expect(blue).toContain('data-ox-tweet-copy-url="https://x.com/i/web/status/555"');
    expect(blue).toContain('aria-label="Copy link to post"');
    expect(blue).toContain(">Copy link</a>");
    expect(blue).toContain("1.5K");
    expect(blue).toContain("<strong>44</strong> reposts");
    expect(blue).toContain("<strong>3</strong> quotes");
    expect(blue).toContain("<strong>22.0K</strong> views");
    expect(blue).toContain("Read 12 replies");
    expect(blue).toContain("Follow");

    const gold = await renderCard(
      {
        text: "Hi",
        id_str: "555",
        conversation_count: 1,
        user: { ...user(), verified_type: "Business" },
      },
      { appearance: "full" },
    );
    expect(gold).toContain("ox-tweet__badge--gold");
    expect(gold).toContain("Read 1 reply");

    const gov = await renderCard(
      {
        text: "Hi",
        id_str: "555",
        user: { ...user(), verified_type: "Government" },
      },
      { appearance: "full" },
    );
    expect(gov).toContain("ox-tweet__badge--gray");
    expect(gov).toContain("Read more on X");
  });

  it("keeps a cheap link-only fallback when the post cannot be fetched", async () => {
    globalThis.fetch = async () => ({ ok: false, status: 404 }) as Response;
    const input = '<Tweet id="987654">fallback summary</Tweet>';
    await expect(
      transformFetchedTweets(input, { fetch: true, cache: false, appearance: "full" }),
    ).resolves.toBe(input);
  });
});

function user() {
  return { name: "Ox Content", screen_name: "ox_content" };
}

function body(
  text: string,
  entities: NonNullable<TweetBodyData["entities"]>,
  start = 0,
): TweetBodyData {
  return { text, display_text_range: [start, text.length], user: user(), entities };
}

function span(text: string, value: string): [number, number] {
  const start = text.indexOf(value);
  return [start, start + value.length];
}

function photoPost(count: number) {
  return {
    text: "Photos",
    id_str: "555",
    user: user(),
    mediaDetails: Array.from({ length: count }, (_, index) => ({
      type: "photo",
      media_url_https: `https://pbs.twimg.com/media/p${index}.jpg`,
    })),
  };
}

async function renderCard(
  data: unknown,
  twitter: TwitterEmbedOptions = {},
  options?: { html?: string },
): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), "ox-tweet-full-"));
  globalThis.fetch = async (input) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    if (url.startsWith("https://cdn.syndication.twimg.com/")) {
      return { ok: true, json: async () => data } as Response;
    }
    return { ok: true, arrayBuffer: async () => new Uint8Array([1, 2, 3]).buffer } as Response;
  };
  try {
    return await transformFetchedTweets(options?.html ?? '<XPost id="555" />', {
      fetch: true,
      cache: false,
      mediaOutputDir: path.join(root, "media"),
      mediaPublicPath: "/tweets",
      ...twitter,
    });
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}
