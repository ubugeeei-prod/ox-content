import { describe, expect, it } from "vite-plus/test";
import { generateFeeds, resolveFeedCollectionName, resolveFeedsOptions } from "./feeds";

const sampleItems = [
  {
    title: "Older",
    description: "First post",
    loc: "https://example.com/blog/older/",
    date: "2024-01-01",
  },
  {
    title: "Newer",
    description: "Second post",
    loc: "https://example.com/blog/newer/",
    date: "2024-02-01",
  },
];

const enabled = {
  enabled: true,
  formats: ["rss", "atom", "json"] as const,
  limit: 20,
  path: "/",
};

describe("resolveFeedsOptions", () => {
  it("disables the feature by default", () => {
    expect(resolveFeedsOptions(undefined)).toEqual({
      enabled: false,
      formats: ["rss", "atom", "json"],
      limit: 20,
      path: "/",
    });
    expect(resolveFeedsOptions(false)).toEqual({
      enabled: false,
      formats: ["rss", "atom", "json"],
      limit: 20,
      path: "/",
    });
  });

  it("enables defaults when true", () => {
    expect(resolveFeedsOptions(true)).toEqual({
      enabled: true,
      formats: ["rss", "atom", "json"],
      limit: 20,
      path: "/",
    });
  });

  it("enables the feature from an object and overrides only set fields", () => {
    expect(
      resolveFeedsOptions({ formats: ["rss"], collection: "blog", limit: 5, path: "/feeds" }),
    ).toEqual({
      enabled: true,
      formats: ["rss"],
      collection: "blog",
      limit: 5,
      path: "/feeds",
    });
    expect(resolveFeedsOptions({})).toEqual({
      enabled: true,
      formats: ["rss", "atom", "json"],
      limit: 20,
      path: "/",
    });
  });
});

describe("resolveFeedCollectionName", () => {
  it("uses content or the first configured collection", () => {
    expect(resolveFeedCollectionName(undefined, ["blog", "docs"])).toBe("blog");
    expect(resolveFeedCollectionName(undefined, ["blog", "content"])).toBe("content");
    expect(resolveFeedCollectionName("docs", ["blog", "docs"])).toBe("docs");
  });
});

describe("generateFeeds", () => {
  it("writes nothing when the feature is omitted or disabled", () => {
    expect(generateFeeds({ items: sampleItems })).toEqual({});
    expect(
      generateFeeds({
        options: { enabled: false, formats: ["rss", "atom", "json"], limit: 20, path: "/" },
        siteUrl: "https://example.com",
        items: sampleItems,
      }),
    ).toEqual({});
  });

  it("warns and writes nothing when siteUrl is missing", () => {
    expect(
      generateFeeds({
        options: { ...enabled },
        items: sampleItems,
      }),
    ).toEqual({
      warning:
        "[ox-content] feeds is enabled but ssg.siteUrl is not set; RSS, Atom, and JSON feeds were not written",
    });
    expect(
      generateFeeds({
        options: { ...enabled },
        siteUrl: "   ",
        items: sampleItems,
      }).warning,
    ).toBeDefined();
  });

  it("writes a sorted RSS, Atom, and JSON feed", () => {
    const output = generateFeeds({
      options: { ...enabled },
      siteUrl: "https://example.com",
      siteName: "Docs",
      siteDescription: "Example docs",
      items: sampleItems,
    });

    expect(output.rssXml).toBe(`\
<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Docs</title>
    <link>https://example.com/</link>
    <description>Example docs</description>
    <item>
      <title>Newer</title>
      <link>https://example.com/blog/newer/</link>
      <guid>https://example.com/blog/newer/</guid>
      <description>Second post</description>
      <pubDate>Thu, 01 Feb 2024 00:00:00 +0000</pubDate>
    </item>
    <item>
      <title>Older</title>
      <link>https://example.com/blog/older/</link>
      <guid>https://example.com/blog/older/</guid>
      <description>First post</description>
      <pubDate>Mon, 01 Jan 2024 00:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>
`);
    expect(output.atomXml).toBe(`\
<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Docs</title>
  <link href="https://example.com/atom.xml" rel="self"/>
  <link href="https://example.com/" rel="alternate"/>
  <id>https://example.com/</id>
  <updated>2024-02-01T00:00:00Z</updated>
  <subtitle>Example docs</subtitle>
  <entry>
    <title>Newer</title>
    <link href="https://example.com/blog/newer/"/>
    <id>https://example.com/blog/newer/</id>
    <updated>2024-02-01T00:00:00Z</updated>
    <summary>Second post</summary>
  </entry>
  <entry>
    <title>Older</title>
    <link href="https://example.com/blog/older/"/>
    <id>https://example.com/blog/older/</id>
    <updated>2024-01-01T00:00:00Z</updated>
    <summary>First post</summary>
  </entry>
</feed>
`);
    expect(output.jsonFeed).toBe(`\
{
  "version": "https://jsonfeed.org/version/1.1",
  "title": "Docs",
  "home_page_url": "https://example.com/",
  "feed_url": "https://example.com/feed.json",
  "description": "Example docs",
  "items": [
    {
      "id": "https://example.com/blog/newer/",
      "url": "https://example.com/blog/newer/",
      "title": "Newer",
      "content_text": "Second post",
      "date_published": "2024-02-01T00:00:00Z"
    },
    {
      "id": "https://example.com/blog/older/",
      "url": "https://example.com/blog/older/",
      "title": "Older",
      "content_text": "First post",
      "date_published": "2024-01-01T00:00:00Z"
    }
  ]
}
`);
    expect(output.warning).toBeUndefined();
  });

  it("honors object overrides for formats, collection, and limit", () => {
    const output = generateFeeds({
      options: {
        enabled: true,
        formats: ["rss"],
        collection: "blog",
        limit: 1,
        path: "/",
      },
      siteUrl: "https://example.com",
      siteName: "Docs",
      collections: {
        blog: sampleItems,
        docs: [{ title: "Guide", loc: "https://example.com/guide/", date: "2024-06-01" }],
      },
      collectionNames: ["blog", "docs"],
    });

    expect(output.rssXml).toContain("Newer");
    expect(output.rssXml).not.toContain("Older");
    expect(output.rssXml).not.toContain("Guide");
    expect(output.atomXml).toBeUndefined();
    expect(output.jsonFeed).toBeUndefined();
  });

  it("omits draft items", () => {
    const output = generateFeeds({
      options: { ...enabled },
      siteUrl: "https://example.com",
      siteName: "Docs",
      items: [
        { title: "Secret", loc: "https://example.com/secret/", date: "2024-03-01", draft: true },
        {
          title: "Also secret",
          loc: "https://example.com/hidden/",
          date: "2024-04-01",
          frontmatter: { draft: true },
        },
        { title: "Public", loc: "https://example.com/public/", date: "2024-01-01" },
      ],
    });

    expect(output.rssXml).toContain("https://example.com/public/");
    expect(output.rssXml).not.toContain("secret");
    expect(output.rssXml).not.toContain("hidden");
    expect(output.jsonFeed).toContain("Public");
    expect(output.jsonFeed).not.toContain("Secret");
  });

  it("omits unlisted items", () => {
    const output = generateFeeds({
      options: { ...enabled },
      siteUrl: "https://example.com",
      siteName: "Docs",
      items: [
        { title: "Hidden", loc: "https://example.com/hidden/", date: "2024-03-01", unlisted: true },
        {
          title: "Also hidden",
          loc: "https://example.com/unlisted/",
          date: "2024-04-01",
          frontmatter: { unlisted: true },
        },
        { title: "Public", loc: "https://example.com/public/", date: "2024-01-01" },
      ],
    });

    expect(output.rssXml).toContain("https://example.com/public/");
    expect(output.rssXml).not.toContain("hidden");
    expect(output.rssXml).not.toContain("unlisted");
    expect(output.jsonFeed).toContain("Public");
    expect(output.jsonFeed).not.toContain("Hidden");
  });

  it("escapes hostile titles and descriptions", () => {
    const output = generateFeeds({
      options: { ...enabled },
      siteUrl: "https://example.com",
      siteName: "Docs",
      items: [
        {
          loc: `https://example.com/x?a=1&b=2<>"'`,
          title: "</title></channel><script>alert(1)</script>",
          description: '">\n<img src=x onerror=alert(1)>',
          date: "2024-01-01",
        },
      ],
    });

    expect(output.rssXml).not.toContain("<script>");
    expect(output.rssXml).toContain("&amp;");
    expect(output.rssXml).toContain("&lt;");
    expect(output.atomXml).not.toContain("<img");
    expect(output.jsonFeed).not.toContain("</title>");
    expect(output.jsonFeed).not.toContain("<script>");
    expect(output.jsonFeed).toContain("\\n");
  });

  it("sorts by date then lastUpdated and applies the limit", () => {
    const output = generateFeeds({
      options: { ...enabled, limit: 3 },
      siteUrl: "https://example.com",
      siteName: "Docs",
      items: [
        { title: "By lastUpdated", loc: "https://example.com/c/", lastUpdated: "2024-03-01" },
        {
          title: "Oldest date",
          loc: "https://example.com/a/",
          date: "2024-01-01",
          lastUpdated: "2024-04-01",
        },
        { title: "Newest date", loc: "https://example.com/b/", date: "2024-02-01" },
        { title: "Tied date z", loc: "https://example.com/z/", date: "2024-02-01" },
        { title: "No date", loc: "https://example.com/m/" },
      ],
    });

    const rss = output.rssXml ?? "";
    expect(rss.indexOf("https://example.com/c/")).toBeLessThan(
      rss.indexOf("https://example.com/b/"),
    );
    expect(rss.indexOf("https://example.com/b/")).toBeLessThan(
      rss.indexOf("https://example.com/z/"),
    );
    expect(rss).not.toContain("https://example.com/a/");
    expect(rss).not.toContain("https://example.com/m/");
  });
});
