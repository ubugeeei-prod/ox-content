import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import {
  feedDateFromFrontmatter,
  generateFeeds,
  resolveFeedsOptions,
  writeFeedFiles,
} from "./feeds";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

const sampleItems = [
  {
    title: "Old Post",
    loc: "https://example.com/old/",
    description: "Earlier",
    date: "2024-01-15",
  },
  {
    title: "New Post",
    loc: "https://example.com/new/",
    description: "Latest",
    date: "2024-03-01",
  },
];

const enabledOptions = {
  enabled: true,
  rss: true,
  atom: true,
  json: true,
  limit: 20,
  collection: "content",
  dateField: "date",
};

describe("resolveFeedsOptions", () => {
  it("disables the feature when omitted or false", () => {
    expect(resolveFeedsOptions(undefined).enabled).toBe(false);
    expect(resolveFeedsOptions(false).enabled).toBe(false);
    expect(resolveFeedsOptions(undefined)).toEqual({
      enabled: false,
      rss: true,
      atom: true,
      json: true,
      limit: 20,
      collection: "content",
      dateField: "date",
    });
  });

  it("enables defaults when true", () => {
    expect(resolveFeedsOptions(true).enabled).toBe(true);
    expect(resolveFeedsOptions(true)).toEqual({
      enabled: true,
      rss: true,
      atom: true,
      json: true,
      limit: 20,
      collection: "content",
      dateField: "date",
    });
  });

  it("enables the feature from an empty object", () => {
    expect(resolveFeedsOptions({}).enabled).toBe(true);
    expect(resolveFeedsOptions({})).toEqual({
      enabled: true,
      rss: true,
      atom: true,
      json: true,
      limit: 20,
      collection: "content",
      dateField: "date",
    });
  });

  it("overrides only the fields the site set", () => {
    expect(
      resolveFeedsOptions({
        rss: false,
        atom: false,
        limit: 5,
        collection: "blog",
        dateField: "published",
      }),
    ).toEqual({
      enabled: true,
      rss: false,
      atom: false,
      json: true,
      limit: 5,
      collection: "blog",
      dateField: "published",
    });
  });
});

describe("generateFeeds", () => {
  it("disabled_by_default", () => {
    expect(generateFeeds({ items: sampleItems })).toEqual({});
    expect(
      generateFeeds({
        options: { ...enabledOptions, enabled: false },
        siteUrl: "https://example.com",
        items: sampleItems,
      }),
    ).toEqual({});
  });

  it("missing_site_url_warns_and_writes_nothing", () => {
    expect(
      generateFeeds({
        options: enabledOptions,
        items: sampleItems,
      }),
    ).toEqual({
      warning:
        "[ox-content] feeds is enabled but ssg.siteUrl is not set; feed.xml, atom.xml, and feed.json were not written",
    });
    expect(
      generateFeeds({
        options: enabledOptions,
        siteUrl: "   ",
        items: sampleItems,
      }).warning,
    ).toBeDefined();
  });

  it("happy_path_rss_atom_json", () => {
    const output = generateFeeds({
      options: enabledOptions,
      siteUrl: "https://example.com",
      siteName: "Docs",
      siteDescription: "Example docs",
      homePageUrl: "https://example.com/",
      feedRssLoc: "https://example.com/feed.xml",
      feedAtomLoc: "https://example.com/atom.xml",
      feedJsonLoc: "https://example.com/feed.json",
      items: sampleItems,
    });

    expect(output.rssXml).toContain('<rss version="2.0">');
    expect(output.rssXml).toContain("<title>New Post</title>");
    expect(output.rssXml).toContain("<pubDate>Fri, 01 Mar 2024 00:00:00 +0000</pubDate>");
    expect(output.rssXml!.indexOf("New Post")).toBeLessThan(output.rssXml!.indexOf("Old Post"));
    expect(output.atomXml).toContain('<feed xmlns="http://www.w3.org/2005/Atom">');
    expect(output.atomXml).toContain("<updated>2024-03-01T00:00:00Z</updated>");
    expect(output.jsonFeed).toContain('"version": "https://jsonfeed.org/version/1.1"');
    expect(output.jsonFeed).toContain('"date_published": "2024-03-01T00:00:00Z"');
    expect(output.warning).toBeUndefined();
  });

  it("limit_truncates", () => {
    const output = generateFeeds({
      options: { ...enabledOptions, limit: 1 },
      siteUrl: "https://example.com",
      siteName: "Docs",
      items: [
        { title: "A", loc: "https://example.com/a/", date: "2024-01-01" },
        { title: "B", loc: "https://example.com/b/", date: "2024-03-01" },
        { title: "C", loc: "https://example.com/c/", date: "2024-02-01" },
      ],
    });

    expect(output.rssXml).toContain("<title>B</title>");
    expect(output.rssXml).not.toContain("<title>A</title>");
    expect(output.rssXml).not.toContain("<title>C</title>");
    expect(output.rssXml?.match(/<item>/g)).toHaveLength(1);
  });

  it("sort_by_date_descending", () => {
    const output = generateFeeds({
      options: enabledOptions,
      siteUrl: "https://example.com",
      siteName: "Docs",
      items: [
        { title: "Zeta", loc: "https://example.com/zeta/" },
        { title: "Old", loc: "https://example.com/old/", date: "2024-01-15" },
        { title: "Alpha", loc: "https://example.com/alpha/", date: "not-a-date" },
        { title: "New", loc: "https://example.com/new/", date: "2024-03-01T12:00:00Z" },
        { title: "Mid", loc: "https://example.com/mid/", date: "2024-02-01" },
      ],
    });
    const rss = output.rssXml ?? "";
    const newAt = rss.indexOf("<title>New</title>");
    const midAt = rss.indexOf("<title>Mid</title>");
    const oldAt = rss.indexOf("<title>Old</title>");
    const alphaAt = rss.indexOf("<title>Alpha</title>");
    const zetaAt = rss.indexOf("<title>Zeta</title>");

    expect(newAt).toBeGreaterThan(-1);
    expect(newAt).toBeLessThan(midAt);
    expect(midAt).toBeLessThan(oldAt);
    expect(oldAt).toBeLessThan(alphaAt);
    expect(alphaAt).toBeLessThan(zetaAt);
  });

  it("hostile_title_and_description_escaped", () => {
    const output = generateFeeds({
      options: enabledOptions,
      siteUrl: "https://example.com",
      siteName: "Docs",
      items: [
        {
          title: "</title></item><script>alert(1)</script>",
          loc: `https://example.com/x?a=1&b=2<>"'`,
          description: '">\n<img src=x onerror=alert(1)>&amp;',
          date: "2024-03-01",
        },
      ],
    });

    expect(output.rssXml).not.toContain("<script>");
    expect(output.atomXml).not.toContain("<script>");
    expect(output.jsonFeed).not.toContain("<script>");
    expect(output.jsonFeed).not.toContain("<img");
    expect(output.rssXml).toContain("&lt;");
    expect(output.rssXml).toContain("&amp;");
    expect(output.jsonFeed).toContain("\\u003c");
  });
});

describe("feedDateFromFrontmatter", () => {
  it("reads the configured date field and ignores empty values", () => {
    expect(feedDateFromFrontmatter({ date: "2024-03-01" }, "date")).toBe("2024-03-01");
    expect(feedDateFromFrontmatter({ published: " 2024-01-15 " }, "published")).toBe("2024-01-15");
    expect(feedDateFromFrontmatter({ date: "   " }, "date")).toBeUndefined();
    expect(feedDateFromFrontmatter({}, "date")).toBeUndefined();
  });
});

describe("writeFeedFiles", () => {
  it("does not write files when the feature is omitted", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-feeds-"));
    tempDirs.push(outDir);

    const result = await writeFeedFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/",
      items: sampleItems,
    });

    expect(result.files).toEqual([]);
    expect(result.warning).toBeUndefined();
    await expect(fs.access(path.join(outDir, "feed.xml"))).rejects.toThrow();
  });

  it("does not write files when siteUrl is missing", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-feeds-"));
    tempDirs.push(outDir);

    const result = await writeFeedFiles({
      outDir,
      base: "/",
      options: enabledOptions,
      items: sampleItems,
    });

    expect(result.files).toEqual([]);
    expect(result.warning).toContain("ssg.siteUrl is not set");
    await expect(fs.access(path.join(outDir, "feed.xml"))).rejects.toThrow();
  });

  it("writes rss atom and json next to generated HTML", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-feeds-"));
    tempDirs.push(outDir);

    const result = await writeFeedFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/docs/",
      siteName: "Docs",
      siteDescription: "Example docs",
      options: enabledOptions,
      items: sampleItems,
    });

    expect(result.files).toEqual([
      path.join(outDir, "feed.xml"),
      path.join(outDir, "atom.xml"),
      path.join(outDir, "feed.json"),
    ]);
    expect(await fs.readFile(path.join(outDir, "feed.xml"), "utf8")).toContain(
      "<title>New Post</title>",
    );
    expect(await fs.readFile(path.join(outDir, "atom.xml"), "utf8")).toContain(
      "https://example.com/docs/atom.xml",
    );
    expect(await fs.readFile(path.join(outDir, "feed.json"), "utf8")).toContain(
      "https://example.com/docs/feed.json",
    );
  });
});
