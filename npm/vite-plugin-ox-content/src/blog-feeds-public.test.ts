import { describe, expect, it } from "vite-plus/test";
import { loadBlogFeedEntries, type BlogFeedNetwork } from ".";

const RSS = `<?xml version="1.0"?><rss version="2.0"><channel>
<item><title>Remote</title><link>https://news.example.com/remote</link>
<guid>remote-id</guid><pubDate>Thu, 01 Feb 2024 00:00:00 GMT</pubDate></item>
</channel></rss>`;

function network(handler: (url: string) => Response | Promise<Response>): BlogFeedNetwork {
  return {
    lookup: async () => ["1.1.1.1"],
    limits: { timeoutMs: 40, maxBytes: 2048, maxRedirects: 2 },
    fetch: async (url) => handler(url),
  };
}

describe("public blog feed entries", () => {
  it("does not fetch when no public sources are configured", async () => {
    let calls = 0;
    const loaded = await loadBlogFeedEntries({
      network: network(() => {
        calls += 1;
        return new Response(RSS, { headers: { "content-type": "application/rss+xml" } });
      }),
    });

    expect(calls).toBe(0);
    expect(loaded).toEqual({ entries: [], warnings: [], fatals: [] });
  });

  it("coalesces duplicate source URLs and preserves successful entries with fatal diagnostics", async () => {
    const urls: string[] = [];
    const loaded = await loadBlogFeedEntries({
      sources: [
        "https://feeds.example.com/rss.xml",
        { url: "https://feeds.example.com/rss.xml", language: "en" },
        { url: "https://feeds.example.com/bad.xml", onError: "error" },
      ],
      network: network((url) => {
        urls.push(url);
        if (url.endsWith("bad.xml")) {
          throw new Error("feed unavailable");
        }
        return new Response(RSS, { headers: { "content-type": "application/rss+xml" } });
      }),
    });

    expect(urls).toEqual([
      "https://feeds.example.com/rss.xml",
      "https://feeds.example.com/bad.xml",
    ]);
    expect(loaded.entries).toEqual([
      expect.objectContaining({
        title: "Remote",
        url: "https://news.example.com/remote",
        id: "remote-id",
        external: true,
        sourceUrl: "https://feeds.example.com/rss.xml",
      }),
      expect.objectContaining({
        title: "Remote",
        url: "https://news.example.com/remote",
        id: "remote-id",
        language: "en",
        external: true,
      }),
    ]);
    expect(loaded.warnings).toEqual([]);
    expect(loaded.fatals[0]).toContain("feed unavailable");
  });
});
