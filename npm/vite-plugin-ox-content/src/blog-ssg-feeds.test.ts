import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { BlogFeedError, resolveBlogOptions } from "./blog";
import { installBlogFeedNetwork, resetBlogFeedNetwork } from "./blog-feed-fetch";
import { resolveFeedsOptions } from "./feeds";
import { buildSsg } from "./ssg";

const RSS = `<?xml version="1.0"?><rss version="2.0"><channel>
<item><title>Remote Essay</title><link>https://news.example.com/essay</link>
<guid>https://news.example.com/essay</guid>
<pubDate>Fri, 01 Mar 2024 00:00:00 GMT</pubDate></item>
</channel></rss>`;

const tempDirs: string[] = [];

afterEach(async () => {
  resetBlogFeedNetwork();
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-blog-feeds-"));
  tempDirs.push(root);
  const srcDir = path.join(root, "content");
  await fs.mkdir(srcDir, { recursive: true });
  for (const [relative, body] of Object.entries(files)) {
    const filePath = path.join(srcDir, relative);
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, body, "utf8");
  }
  return root;
}

describe("buildSsg blog external feeds", () => {
  it("does not fetch when blog feeds are omitted", async () => {
    let calls = 0;
    installBlogFeedNetwork({
      lookup: async () => {
        calls += 1;
        return ["1.1.1.1"];
      },
      fetch: async () => {
        calls += 1;
        return new Response(RSS);
      },
    });
    const root = await makeSite({
      "hello.md": "---\ntitle: Hello\ndate: 2024-01-15\n---\n\n# Hello\n",
    });
    await buildSsg(createDocsResolvedOptions({ blog: resolveBlogOptions(true) }), root);
    expect(calls).toBe(0);
    const index = await fs.readFile(path.join(root, "dist", "blog", "index.html"), "utf8");
    expect(index).toContain("Hello");
    expect(index).not.toContain("ox-blog-external");
  });

  it("merges fixture items onto the index with an external marker and keeps them out of outbound feeds", async () => {
    const urls: string[] = [];
    installBlogFeedNetwork({
      lookup: async () => ["1.1.1.1"],
      fetch: async (url) => {
        urls.push(url);
        return new Response(RSS, { headers: { "content-type": "application/rss+xml" } });
      },
    });
    const root = await makeSite({
      "hello.md": "---\ntitle: Hello\ndate: 2024-01-15\n---\n\n# Hello\n",
    });
    await buildSsg(
      createDocsResolvedOptions({
        blog: resolveBlogOptions({
          feeds: [{ url: "https://feeds.example.com/rss.xml", language: "en" }],
        }),
        feeds: resolveFeedsOptions({ formats: ["rss", "atom"], collection: "content" }),
        ssg: { ...createDocsResolvedOptions().ssg, siteUrl: "https://site.example" },
        collections: {
          enabled: true,
          collections: {
            content: { name: "content", source: ["**/*.md"], include: [] },
          },
        },
      }),
      root,
    );

    expect(urls).toEqual(["https://feeds.example.com/rss.xml"]);
    const index = await fs.readFile(path.join(root, "dist", "blog", "index.html"), "utf8");
    expect(index).toContain("Remote Essay");
    expect(index).toContain("Hello");
    expect(index).toContain('href="https://news.example.com/essay"');
    expect(index).toContain("ox-blog-external");
    expect(index).toContain('rel="external noopener noreferrer"');
    expect(index).not.toContain('href="/news.example.com/essay');

    const feed = await fs.readFile(path.join(root, "dist", "feed.xml"), "utf8");
    expect(feed).toContain("Hello");
    expect(feed).not.toContain("Remote Essay");
  });

  it("fails the build when a source uses onError error", async () => {
    installBlogFeedNetwork({
      lookup: async () => ["1.1.1.1"],
      fetch: async () => {
        throw new Error("offline");
      },
    });
    const root = await makeSite({
      "hello.md": "---\ntitle: Hello\ndate: 2024-01-15\n---\n\n# Hello\n",
    });
    await expect(
      buildSsg(
        createDocsResolvedOptions({
          blog: resolveBlogOptions({
            feeds: [{ url: "https://feeds.example.com/rss.xml", onError: "error" }],
          }),
        }),
        root,
      ),
    ).rejects.toBeInstanceOf(BlogFeedError);
    await expect(fs.access(path.join(root, "dist", "blog", "index.html"))).rejects.toThrow();
  });
});
