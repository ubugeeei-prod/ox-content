import { describe, expect, it } from "vite-plus/test";
import { generateFeeds } from "./feeds";
import { loadExternalBlogPosts, mergeBlogPosts } from "./blog-feeds";
import type { BlogSourcePage } from "./blog-html";
import type { BlogFeedNetwork } from "./blog-feed-fetch";
import type { ResolvedBlogFeedSource } from "./types";

const RSS = `<?xml version="1.0"?><rss version="2.0"><channel>
<item><title>Remote</title><link>https://news.example.com/remote</link>
<guid>https://news.example.com/remote</guid>
<pubDate>Thu, 01 Feb 2024 00:00:00 GMT</pubDate></item>
</channel></rss>`;

const ATOM = `<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom">
<entry><title>Atom remote</title><id>urn:atom:1</id>
<link href="https://news.example.com/atom"/><published>2024-03-01T00:00:00Z</published>
</entry></feed>`;

const DUP = `<?xml version="1.0"?><rss version="2.0"><channel>
<item><title>First</title><link>https://news.example.com/same</link><guid>urn:same</guid>
<pubDate>Mon, 01 Jan 2024 00:00:00 GMT</pubDate></item>
<item><title>Second</title><link>https://news.example.com/same/</link><guid>urn:other</guid>
<pubDate>Tue, 02 Jan 2024 00:00:00 GMT</pubDate></item>
</channel></rss>`;

function source(url: string, onError: "warn" | "error" = "warn"): ResolvedBlogFeedSource {
  return { url, onError };
}

function xmlResponse(body: string, status = 200, headers: Record<string, string> = {}): Response {
  return new Response(body, {
    status,
    headers: { "content-type": "application/rss+xml", ...headers },
  });
}

function network(
  handler: (url: string) => Response | Promise<Response>,
  lookup: (hostname: string) => Promise<string[]> = async () => ["1.1.1.1"],
): BlogFeedNetwork {
  return {
    lookup,
    limits: { timeoutMs: 40, maxBytes: 2048, maxRedirects: 2 },
    fetch: async (url) => handler(url),
  };
}

function localPage(title: string, href: string, date: string, id?: string): BlogSourcePage {
  return {
    title,
    inputPath: `/${title}.md`,
    transformedHtml: "",
    routePaths: { href },
    frontmatter: { date, ...(id ? { id } : {}) },
  };
}

describe("loadExternalBlogPosts", () => {
  it("does not fetch when no sources are configured", async () => {
    let calls = 0;
    const loaded = await loadExternalBlogPosts(
      [],
      network(() => {
        calls += 1;
        return xmlResponse(RSS);
      }),
    );
    expect(calls).toBe(0);
    expect(loaded).toEqual({ pages: [], warnings: [], fatals: [] });
  });

  it("fetches each unique URL once and marks items external", async () => {
    const urls: string[] = [];
    const loaded = await loadExternalBlogPosts(
      [
        {
          url: "https://feeds.example.com/rss.xml",
          language: "en",
          author: "ada",
          onError: "warn",
        },
        { url: "https://feeds.example.com/rss.xml", onError: "warn" },
        { url: "https://feeds.example.com/atom.xml", onError: "warn" },
      ],
      network((url) => {
        urls.push(url);
        return xmlResponse(url.endsWith("atom.xml") ? ATOM : RSS);
      }),
    );
    expect(urls).toEqual([
      "https://feeds.example.com/rss.xml",
      "https://feeds.example.com/atom.xml",
    ]);
    expect(loaded.pages.map((page) => page.title)).toEqual(["Remote", "Remote", "Atom remote"]);
    expect(loaded.pages.every((page) => page.external === true)).toBe(true);
    expect(loaded.pages[0]?.routePaths.href).toBe("https://news.example.com/remote");
    expect(loaded.pages[0]?.frontmatter).toMatchObject({
      external: true,
      language: "en",
      author: "ada",
    });
  });

  it("keeps successful sources in warn mode and fails the source in error mode", async () => {
    const warn = await loadExternalBlogPosts(
      [source("https://feeds.example.com/ok.xml"), source("https://feeds.example.com/bad.xml")],
      network((url) => {
        if (url.endsWith("bad.xml")) {
          return xmlResponse("<html><body>nope</body></html>", 200, {
            "content-type": "text/html",
          });
        }
        return xmlResponse(RSS);
      }),
    );
    expect(warn.pages).toHaveLength(1);
    expect(warn.fatals).toEqual([]);
    expect(warn.warnings[0]).toContain("https://feeds.example.com/bad.xml");

    const fatal = await loadExternalBlogPosts(
      [
        source("https://feeds.example.com/ok.xml"),
        source("https://feeds.example.com/bad.xml", "error"),
      ],
      network((url) => {
        if (url.endsWith("bad.xml")) {
          throw new Error("boom");
        }
        return xmlResponse(RSS);
      }),
    );
    expect(fatal.pages).toHaveLength(1);
    expect(fatal.warnings).toEqual([]);
    expect(fatal.fatals[0]).toContain("boom");
  });

  it("rejects timeouts, unsafe URLs, private DNS, and oversized bodies", async () => {
    const timeout = await loadExternalBlogPosts([source("https://feeds.example.com/slow.xml")], {
      lookup: async () => ["1.1.1.1"],
      limits: { timeoutMs: 20, maxBytes: 64, maxRedirects: 1 },
      fetch: async (_url, init) =>
        new Promise<Response>((_, reject) => {
          init?.signal?.addEventListener("abort", () => {
            const error = new Error("aborted");
            error.name = "AbortError";
            reject(error);
          });
        }),
    });
    expect(timeout.warnings[0]).toContain("timeout");

    const unsafe = await loadExternalBlogPosts(
      [source("http://example.com/rss.xml"), source("https://127.0.0.1/rss.xml")],
      network(() => xmlResponse(RSS)),
    );
    expect(unsafe.warnings.every((message) => message.includes("unsafe URL"))).toBe(true);

    const privateNet = await loadExternalBlogPosts(
      [source("https://feeds.example.com/rss.xml")],
      network(
        () => xmlResponse(RSS),
        async () => ["10.0.0.1"],
      ),
    );
    expect(privateNet.warnings[0]).toContain("private network");

    const oversized = await loadExternalBlogPosts(
      [source("https://feeds.example.com/big.xml")],
      network(() => xmlResponse(RSS, 200, { "content-length": "999999" })),
    );
    expect(oversized.warnings[0]).toContain("oversized");

    const redirects = await loadExternalBlogPosts(
      [source("https://feeds.example.com/loop.xml")],
      network(
        (url) =>
          new Response(null, {
            status: 302,
            headers: { location: url },
          }),
      ),
    );
    expect(redirects.warnings[0]).toContain("too many redirects");
  });
});

describe("mergeBlogPosts", () => {
  it("prefers local posts, dedupes by canonical URL or id, and sorts deterministically", () => {
    const local = [
      localPage("Local same url", "/local-remote/", "2024-01-15", undefined),
      localPage("Local reprint", "/reprint/", "2024-01-10"),
    ];
    local[0]!.frontmatter.canonical = "https://news.example.com/remote";
    local[1]!.frontmatter.id = "urn:atom:1";
    const external: BlogSourcePage[] = [
      {
        title: "Remote",
        inputPath: "external:remote",
        transformedHtml: "",
        external: true,
        routePaths: { href: "https://news.example.com/remote" },
        frontmatter: { external: true, id: "https://news.example.com/remote", date: "2024-02-01" },
      },
      {
        title: "Atom remote",
        inputPath: "external:atom",
        transformedHtml: "",
        external: true,
        routePaths: { href: "https://news.example.com/atom" },
        frontmatter: { external: true, id: "urn:atom:1", date: "2024-03-01" },
      },
      {
        title: "Only remote",
        inputPath: "external:only",
        transformedHtml: "",
        external: true,
        routePaths: { href: "https://news.example.com/only" },
        frontmatter: { external: true, id: "https://news.example.com/only", date: "2024-01-20" },
      },
    ];
    expect(mergeBlogPosts(local, external).map((page) => page.title)).toEqual([
      "Only remote",
      "Local same url",
      "Local reprint",
    ]);
  });

  it("dedupes duplicate items inside one feed by canonical URL", async () => {
    const loaded = await loadExternalBlogPosts(
      [source("https://feeds.example.com/dup.xml")],
      network(() => xmlResponse(DUP)),
    );
    expect(mergeBlogPosts([], loaded.pages).map((page) => page.title)).toEqual(["First"]);
  });
});

describe("generated feeds exclude external items", () => {
  it("omits frontmatter.external entries from outbound RSS and Atom", () => {
    const output = generateFeeds({
      options: { enabled: true, formats: ["rss", "atom"], limit: 20, path: "/" },
      siteUrl: "https://site.example",
      siteName: "Site",
      items: [
        { title: "Local", loc: "https://site.example/local/", date: "2024-02-01" },
        {
          title: "Remote",
          loc: "https://news.example.com/remote",
          date: "2024-03-01",
          frontmatter: { external: true },
        },
      ],
    });
    expect(output.rssXml).toContain("Local");
    expect(output.rssXml).not.toContain("Remote");
    expect(output.atomXml).toContain("Local");
    expect(output.atomXml).not.toContain("Remote");
  });
});
