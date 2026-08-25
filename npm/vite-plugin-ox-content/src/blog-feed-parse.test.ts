import { describe, expect, it } from "vite-plus/test";
import { parseBlogFeed } from "./blog-feed-parse";

const RSS = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Notes</title>
    <language>en</language>
    <item>
      <title>Older</title>
      <link>https://example.com/older/</link>
      <guid>https://example.com/older/</guid>
      <pubDate>Wed, 01 Mar 2024 10:00:00 +0900</pubDate>
      <description>Older summary</description>
    </item>
    <item>
      <title>Newer</title>
      <link>https://example.com/newer</link>
      <guid isPermaLink="false">urn:uuid:newer</guid>
      <pubDate>Wed, 01 Mar 2024 01:00:00 GMT</pubDate>
    </item>
    <item>
      <title></title>
      <link>https://example.com/missing</link>
    </item>
  </channel>
</rss>
`;

const ATOM = `<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xml:lang="ja">
  <title>Atom</title>
  <entry>
    <title>Atom post</title>
    <id>tag:example.com,2024:atom</id>
    <link rel="related" href="https://example.com/related"/>
    <link rel="alternate" href="https://example.com/atom-post"/>
    <published>2024-02-01T00:00:00Z</published>
    <summary>Atom summary</summary>
  </entry>
</feed>
`;

describe("parseBlogFeed", () => {
  it("normalizes RSS items and skips entries without a title or https link", () => {
    const items = parseBlogFeed(RSS);
    expect(items).toHaveLength(2);
    expect(items[0]).toMatchObject({
      title: "Older",
      link: "https://example.com/older",
      id: "https://example.com/older/",
      language: "en",
      summary: "Older summary",
    });
    expect(items[0]?.date?.unix).toBe(items[1]?.date?.unix);
    expect(items[1]).toMatchObject({
      title: "Newer",
      link: "https://example.com/newer",
      id: "urn:uuid:newer",
    });
  });

  it("prefers Atom alternate links and keeps a stable id", () => {
    const items = parseBlogFeed(ATOM);
    expect(items).toEqual([
      expect.objectContaining({
        title: "Atom post",
        link: "https://example.com/atom-post",
        id: "tag:example.com,2024:atom",
        language: "ja",
        summary: "Atom summary",
      }),
    ]);
  });

  it("rejects HTML pages and malformed XML", () => {
    expect(() => parseBlogFeed("<!DOCTYPE html><html><body>not a feed</body></html>")).toThrow(
      "not a feed",
    );
    expect(() => parseBlogFeed('{ "items": [] }')).toThrow("malformed XML");
  });
});
