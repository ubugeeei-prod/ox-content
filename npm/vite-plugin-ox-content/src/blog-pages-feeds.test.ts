import * as os from "node:os";
import * as path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { resolveBlogOptions } from "./blog-options";
import { BlogFeedError } from "./blog-feeds";
import { appendBlogPages } from "./blog-pages";
import type { BlogSourcePage } from "./blog-html";
import type { BlogFeedNetwork } from "./blog-feed-fetch";

const RSS = `<?xml version="1.0"?><rss version="2.0"><channel>
<item><title>Remote Essay</title><link>https://news.example.com/essay</link>
<guid>https://news.example.com/essay</guid>
<pubDate>Fri, 01 Mar 2024 00:00:00 GMT</pubDate></item>
</channel></rss>`;

function localPage(title: string, href: string, date: string): BlogSourcePage {
  return {
    title,
    inputPath: `/src/${title}.md`,
    transformedHtml: "",
    routePaths: { href },
    frontmatter: { date },
  };
}

function network(handler: (url: string) => Response): BlogFeedNetwork {
  return {
    lookup: async () => ["1.1.1.1"],
    fetch: async (url) => handler(url),
  };
}

describe("appendBlogPages external feeds", () => {
  it("does not fetch when feeds are empty", async () => {
    let calls = 0;
    const generated: Array<{ inputPath: string; outputPath: string; html: string }> = [];
    await appendBlogPages({
      generatedPages: generated,
      listedPages: [localPage("Hello", "/hello/", "2024-01-15")],
      options: resolveBlogOptions(true),
      srcDir: "/src",
      outDir: path.join(os.tmpdir(), "ox-blog-pages-off"),
      base: "/",
      errors: [],
      feedNetwork: network(() => {
        calls += 1;
        return new Response(RSS);
      }),
      render: async (page) => page.content,
    });
    expect(calls).toBe(0);
    expect(generated[0]?.html).toContain("Hello");
    expect(generated[0]?.html).not.toContain("ox-blog-external");
  });

  it("merges remote items with an external marker and keeps the canonical href", async () => {
    const urls: string[] = [];
    const generated: Array<{ inputPath: string; outputPath: string; html: string }> = [];
    await appendBlogPages({
      generatedPages: generated,
      listedPages: [localPage("Hello", "/hello/", "2024-01-15")],
      options: resolveBlogOptions({ feeds: ["https://feeds.example.com/rss.xml"] }),
      srcDir: "/src",
      outDir: path.join(os.tmpdir(), "ox-blog-pages-on"),
      base: "/",
      errors: [],
      feedNetwork: network((url) => {
        urls.push(url);
        return new Response(RSS, { headers: { "content-type": "application/rss+xml" } });
      }),
      render: async (page) => page.content,
    });
    expect(urls).toEqual(["https://feeds.example.com/rss.xml"]);
    const html = generated[0]?.html ?? "";
    expect(html).toContain("Remote Essay");
    expect(html).toContain("Hello");
    expect(html).toContain('href="https://news.example.com/essay"');
    expect(html).toContain("ox-blog-external");
    expect(html).toContain('rel="external noopener noreferrer"');
    expect(html).not.toContain('href="/news.example.com/essay');
  });

  it("throws BlogFeedError when onError is error", async () => {
    await expect(
      appendBlogPages({
        generatedPages: [],
        listedPages: [localPage("Hello", "/hello/", "2024-01-15")],
        options: resolveBlogOptions({
          feeds: [{ url: "https://feeds.example.com/rss.xml", onError: "error" }],
        }),
        srcDir: "/src",
        outDir: path.join(os.tmpdir(), "ox-blog-pages-err"),
        base: "/",
        errors: [],
        feedNetwork: {
          lookup: async () => ["1.1.1.1"],
          fetch: async () => {
            throw new Error("offline");
          },
        },
        render: async (page) => page.content,
      }),
    ).rejects.toBeInstanceOf(BlogFeedError);
  });
});
