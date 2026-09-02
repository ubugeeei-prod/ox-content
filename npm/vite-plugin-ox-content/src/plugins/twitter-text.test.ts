import { describe, expect, it } from "vite-plus/test";
import { renderTweetText } from "./twitter/text";
import type { TweetBodyData } from "./twitter/types";

describe("Tweet text entities", () => {
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

  it("keeps decoded entity ranges aligned after decoding syndication text", () => {
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
