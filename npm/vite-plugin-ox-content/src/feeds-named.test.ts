import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { generateFeeds, renderFeedFiles, resolveFeedsOptions, writeFeedFiles } from "./feeds";
import type { FeedChannelOptions } from "./types";

const blogItems = [
  {
    title: "Blog post",
    description: "From blog",
    loc: "https://example.com/blog/one/",
    date: "2024-02-01",
  },
];

const mediaItems = [
  {
    title: "Clip",
    description: "From media",
    loc: "https://example.com/works/media/clip/",
    date: "2024-03-01",
  },
];

const namedFeeds: Record<string, FeedChannelOptions> = {
  blog: {
    formats: ["rss"],
    collection: "blog",
    path: "/",
    title: "blog | example.com",
    description: "Technical articles",
    language: "en",
    image: "https://example.com/icon.png",
    favicon: "https://example.com/icon.png",
    copyright: "© 2026 example.com",
  },
  media: {
    formats: ["rss"],
    collection: "media",
    path: "/works/media",
    title: "Media | example.com",
    language: "ja",
  },
};

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("resolveFeedsOptions named feeds", () => {
  it("keeps a legacy single object as one default feed", () => {
    expect(
      resolveFeedsOptions({ formats: ["rss"], collection: "blog", limit: 5, path: "/" }),
    ).toEqual({
      enabled: true,
      formats: ["rss"],
      collection: "blog",
      limit: 5,
      path: "/",
    });
  });

  it("resolves a named record into two channels", () => {
    const resolved = resolveFeedsOptions(namedFeeds);
    expect(resolved.enabled).toBe(true);
    expect(resolved.feeds).toEqual([
      {
        name: "blog",
        formats: ["rss"],
        collection: "blog",
        limit: 20,
        path: "/",
        title: "blog | example.com",
        description: "Technical articles",
        language: "en",
        image: "https://example.com/icon.png",
        favicon: "https://example.com/icon.png",
        copyright: "© 2026 example.com",
      },
      {
        name: "media",
        formats: ["rss"],
        collection: "media",
        limit: 20,
        path: "/works/media",
        title: "Media | example.com",
        language: "ja",
      },
    ]);
  });

  it("resolves an array of feed definitions", () => {
    const resolved = resolveFeedsOptions([
      { formats: ["atom"], collection: "blog", path: "/blog", title: "Blog" },
      { formats: ["json"], collection: "media", path: "/media", language: "ja" },
    ]);
    expect(resolved.enabled).toBe(true);
    expect(resolved.feeds?.map((feed) => feed.path)).toEqual(["/blog", "/media"]);
    expect(resolved.feeds?.[0]?.title).toBe("Blog");
    expect(resolved.feeds?.[1]?.language).toBe("ja");
  });

  it("rejects channels that set both collection and items", () => {
    expect(() =>
      resolveFeedsOptions({
        formats: ["rss"],
        collection: "blog",
        items: [],
      }),
    ).toThrow("cannot set both collection and items");
    expect(() =>
      resolveFeedsOptions({
        media: {
          formats: ["rss"],
          collection: "media",
          items: [],
        },
      }),
    ).toThrow('"media"');
  });
});

describe("generateFeeds channel metadata", () => {
  it("writes title, description, language, image, favicon, and copyright", () => {
    const output = generateFeeds({
      options: {
        enabled: true,
        formats: ["rss", "atom", "json"],
        limit: 20,
        path: "/",
        title: "blog | example.com",
        description: "Technical articles",
        language: "en",
        image: "https://example.com/icon.png",
        favicon: "https://example.com/favicon.png",
        copyright: "© 2026 example.com",
      },
      siteUrl: "https://example.com",
      siteName: "Docs",
      siteDescription: "Site summary",
      items: blogItems,
    });

    expect(output.rssXml).toContain("<title>blog | example.com</title>");
    expect(output.rssXml).toContain("<description>Technical articles</description>");
    expect(output.rssXml).toContain("<language>en</language>");
    expect(output.rssXml).toContain("<copyright>© 2026 example.com</copyright>");
    expect(output.rssXml).toContain("<url>https://example.com/icon.png</url>");
    expect(output.rssXml).not.toContain("<title>Docs</title>");

    expect(output.atomXml).toContain('xml:lang="en"');
    expect(output.atomXml).toContain("<title>blog | example.com</title>");
    expect(output.atomXml).toContain("<subtitle>Technical articles</subtitle>");
    expect(output.atomXml).toContain("<icon>https://example.com/favicon.png</icon>");
    expect(output.atomXml).toContain("<logo>https://example.com/icon.png</logo>");
    expect(output.atomXml).toContain("<rights>© 2026 example.com</rights>");

    expect(output.jsonFeed).toContain('"title": "blog | example.com"');
    expect(output.jsonFeed).toContain('"description": "Technical articles"');
    expect(output.jsonFeed).toContain('"language": "en"');
    expect(output.jsonFeed).toContain('"icon": "https://example.com/icon.png"');
    expect(output.jsonFeed).toContain('"favicon": "https://example.com/favicon.png"');
  });
});

describe("writeFeedFiles named feeds", () => {
  it("writes one feed at the default path for a legacy object", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-feeds-legacy-"));
    tempDirs.push(outDir);

    const result = await writeFeedFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/",
      options: resolveFeedsOptions({ formats: ["rss"], collection: "blog", path: "/" }),
      collections: { blog: blogItems },
      collectionNames: ["blog"],
    });

    expect(result.files).toEqual([path.join(outDir, "feed.xml")]);
    expect(await fs.readFile(path.join(outDir, "feed.xml"), "utf8")).toContain("Blog post");
    await expect(fs.access(path.join(outDir, "works/media/feed.xml"))).rejects.toThrow();
  });

  it("writes two named feeds to different paths with distinct titles and languages", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-feeds-named-"));
    tempDirs.push(outDir);

    const result = await writeFeedFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/",
      siteName: "example.com",
      options: resolveFeedsOptions(namedFeeds),
      collections: { blog: blogItems, media: mediaItems },
      collectionNames: ["blog", "media"],
    });

    expect(result.files).toEqual([
      path.join(outDir, "feed.xml"),
      path.join(outDir, "works/media/feed.xml"),
    ]);
    const blog = await fs.readFile(path.join(outDir, "feed.xml"), "utf8");
    const media = await fs.readFile(path.join(outDir, "works/media/feed.xml"), "utf8");
    expect(blog).toContain("<title>blog | example.com</title>");
    expect(blog).toContain("<language>en</language>");
    expect(blog).toContain("Blog post");
    expect(blog).not.toContain("Clip");
    expect(media).toContain("<title>Media | example.com</title>");
    expect(media).toContain("<language>ja</language>");
    expect(media).toContain("Clip");
    expect(media).not.toContain("Blog post");
  });

  it("rejects duplicate named feed output paths before writing files", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-feeds-named-"));
    tempDirs.push(outDir);

    const input = {
      siteUrl: "https://example.com",
      base: "/",
      siteName: "example.com",
      options: resolveFeedsOptions([
        { formats: ["rss"], collection: "blog", path: "/" },
        { formats: ["rss"], collection: "media", path: "/" },
      ]),
      collections: { blog: blogItems, media: mediaItems },
      collectionNames: ["blog", "media"],
    };
    const rendered = await renderFeedFiles(input);
    const result = await writeFeedFiles({ outDir, ...input });

    expect(result.files).toEqual([]);
    expect(rendered.files).toEqual([]);
    expect(rendered.warning).toBe(result.warning);
    expect(result.warning).toContain('feeds output path "feed.xml"');
    expect(result.warning).toContain('"blog"');
    expect(result.warning).toContain('"media"');
    await expect(fs.access(path.join(outDir, "feed.xml"))).rejects.toThrow();
  });

  it("writes nothing when feeds is false", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-feeds-off-"));
    tempDirs.push(outDir);

    const result = await writeFeedFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/",
      options: resolveFeedsOptions(false),
      collections: { blog: blogItems },
      collectionNames: ["blog"],
    });

    expect(result.files).toEqual([]);
    await expect(fs.access(path.join(outDir, "feed.xml"))).rejects.toThrow();
  });
});
