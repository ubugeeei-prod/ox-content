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

  it("decodes syndication HTML entities once before escaping tweet text", () => {
    const html = renderTweetText({
      text: "0.13.4 -&gt; 0.13.5 &#45; &lt;b&gt; &amp;lt;i&amp;gt;",
      user: user(),
    });
    expect(html).toContain("0.13.4 -&gt; 0.13.5 - &lt;b&gt; &amp;lt;i&amp;gt;");
    expect(html).not.toContain("-&amp;gt;");
    expect(html).not.toContain("<b>");
    expect(html).not.toContain("<i>");
  });

  it("remaps raw syndication entity ranges after decoding tweet text", () => {
    const url = "https://t.co/docs";
    const raw =
      "0.13.4 -&gt; 0.13.5 @sanxiaozhizi @boshen_c @TheAlexLichter " +
      `${url} #ox $OX 😀 &amp; @after_emoji`;
    const html = renderTweetText({
      text: raw,
      display_text_range: [0, raw.length],
      user: user(),
      entities: {
        urls: [
          {
            url,
            expanded_url: "https://example.com/docs",
            display_url: "example.com/docs",
            indices: span(raw, url),
          },
        ],
        hashtags: [{ text: "ox", indices: span(raw, "#ox") }],
        user_mentions: [
          { screen_name: "sanxiaozhizi", indices: span(raw, "@sanxiaozhizi") },
          { screen_name: "boshen_c", indices: span(raw, "@boshen_c") },
          { screen_name: "TheAlexLichter", indices: span(raw, "@TheAlexLichter") },
          { screen_name: "after_emoji", indices: span(raw, "@after_emoji") },
        ],
        symbols: [{ text: "OX", indices: span(raw, "$OX") }],
      },
    });

    expect(html).toContain("0.13.4 -&gt; 0.13.5");
    expect(html).not.toContain("-&amp;gt;");
    expect(html).toContain('href="https://example.com/docs"');
    expect(html).toContain(">example.com/docs</a>");
    expect(html).toContain('href="https://x.com/hashtag/ox"');
    expect(html).toContain(">#ox</a>");
    expect(html).toContain('href="https://x.com/search?q=%24OX"');
    expect(html).toContain(">$OX</a>");
    expect(html).toContain('href="https://x.com/sanxiaozhizi"');
    expect(html).toContain(">@sanxiaozhizi</a>");
    expect(html).toContain('href="https://x.com/boshen_c"');
    expect(html).toContain(">@boshen_c</a>");
    expect(html).toContain('href="https://x.com/TheAlexLichter"');
    expect(html).toContain(">@TheAlexLichter</a>");
    expect(html).toContain('href="https://x.com/after_emoji"');
    expect(html).toContain(">@after_emoji</a>");
    expect(html).not.toContain("nxiaozhizi @b");
    expect(html).not.toContain("shen_c @T");
  });

  it("remaps raw syndication ranges in full root and quoted tweet text", async () => {
    const rootText = "Root &amp; @root_user";
    const quoteText = "Quote -&gt; @quoted_user";
    const html = await renderCard(
      {
        text: rootText,
        id_str: "555",
        display_text_range: [0, rootText.length],
        user: user(),
        in_reply_to_screen_name: "source_user",
        in_reply_to_status_id_str: "42",
        entities: {
          user_mentions: [{ screen_name: "root_user", indices: span(rootText, "@root_user") }],
        },
        quoted_tweet: {
          text: quoteText,
          id_str: "99",
          display_text_range: [0, quoteText.length],
          user: { name: "Quoted", screen_name: "quoted" },
          entities: {
            user_mentions: [
              { screen_name: "quoted_user", indices: span(quoteText, "@quoted_user") },
            ],
          },
        },
      },
      { appearance: "full" },
    );
    expect(html).toContain("Root &amp;");
    expect(html).toContain('href="https://x.com/root_user"');
    expect(html).toContain(">@root_user</a>");
    expect(html).toContain("Replying to @source_user");
    expect(html).toContain("Quote -&gt;");
    expect(html).toContain('href="https://x.com/quoted_user"');
    expect(html).toContain(">@quoted_user</a>");
  });

  it("keeps entity ranges aligned after decoding syndication text", () => {
    const docs = "https://t.co/docs";
    const decoded = `Read & share ${docs} #ox @ox_content`;
    const encoded = `Read &amp; share ${docs} #ox @ox_content`;
    const html = renderTweetText({
      text: encoded,
      display_text_range: [0, decoded.length],
      user: user(),
      entities: {
        urls: [
          {
            url: docs,
            expanded_url: "https://example.com/docs",
            display_url: "example.com/docs",
            indices: span(decoded, docs),
          },
        ],
        hashtags: [{ text: "ox", indices: span(decoded, "#ox") }],
        user_mentions: [{ screen_name: "ox_content", indices: span(decoded, "@ox_content") }],
      },
    });
    expect(html).toContain("Read &amp; share");
    expect(html).toContain('href="https://example.com/docs"');
    expect(html).toContain(">example.com/docs</a>");
    expect(html).toContain('href="https://x.com/hashtag/ox"');
    expect(html).toContain(">#ox</a>");
    expect(html).toContain('href="https://x.com/ox_content"');
    expect(html).toContain(">@ox_content</a>");
    expect(html).not.toContain("&amp;amp;");
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
          text: "Quoted &lt;photo&gt; &#45; safe",
          user: { name: "Other", screen_name: "other", is_blue_verified: true },
          mediaDetails: [
            { type: "photo", media_url_https: "https://pbs.twimg.com/media/quoted.jpg" },
          ],
        },
      },
      { appearance: "full" },
    );
    expect(quoted).toContain("Quoted &lt;photo&gt; - safe");
    expect(quoted).not.toContain("&amp;lt;photo&amp;gt;");
    expect(quoted).not.toContain("<photo>");
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
    expect(blue.match(/ox-tweet__action-icon/g)).toHaveLength(3);
    expect(blue.match(/ox-tweet__action-text/g)).toHaveLength(4);
    expect(blue).toContain('aria-label="Like. This Tweet has 1.5K likes"');
    expect(blue).toContain('aria-label="Reply to this Tweet on Twitter"');
    expect(blue).toContain("data-ox-tweet-copy");
    expect(blue).toContain('href="https://x.com/i/web/status/555"');
    expect(blue).toContain('data-ox-tweet-copy-url="https://x.com/i/web/status/555"');
    expect(blue).toContain('aria-label="Copy link"');
    expect(blue).toContain("ox-tweet__copy-text");
    expect(blue).toContain("Copy link");
    expect(blue).toContain("ox-tweet__copied-text");
    expect(blue).toContain("Copied!");
    expect(blue).not.toContain("<script");
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
