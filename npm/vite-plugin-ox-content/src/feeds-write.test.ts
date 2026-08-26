import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { writeFeedFiles } from "./feeds";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

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

describe("writeFeedFiles", () => {
  it("does not write files when the feature is disabled", async () => {
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
      options: { enabled: true, formats: ["rss", "atom", "json"], limit: 20, path: "/" },
      items: sampleItems,
    });

    expect(result.files).toEqual([]);
    expect(result.warning).toContain("ssg.siteUrl is not set");
    await expect(fs.access(path.join(outDir, "feed.xml"))).rejects.toThrow();
  });

  it("does not write files when siteUrl is unsafe", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-feeds-"));
    tempDirs.push(outDir);

    const result = await writeFeedFiles({
      outDir,
      siteUrl: "javascript:alert(1)",
      base: "/",
      options: { enabled: true, formats: ["rss"], limit: 20, path: "/" },
      items: sampleItems,
    });

    expect(result.files).toEqual([]);
    expect(result.warning).toContain("safe absolute http(s) URL");
    await expect(fs.access(path.join(outDir, "feed.xml"))).rejects.toThrow();
  });

  it("rejects unsafe feed output paths before writing files", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-feeds-"));
    tempDirs.push(outDir);

    const result = await writeFeedFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/",
      options: { enabled: true, formats: ["rss"], limit: 20, path: "../feeds" },
      items: sampleItems,
    });

    expect(result.files).toEqual([]);
    expect(result.warning).toContain("uses an unsafe output path");
    await expect(fs.readdir(outDir)).resolves.toEqual([]);
  });

  it("writes the enabled files next to generated HTML", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-feeds-"));
    tempDirs.push(outDir);

    const result = await writeFeedFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/docs/",
      siteName: "Docs",
      siteDescription: "Example docs",
      options: { enabled: true, formats: ["rss", "atom", "json"], limit: 20, path: "/feeds" },
      collections: { content: sampleItems },
      collectionNames: ["content"],
    });

    expect(result.files).toEqual([
      path.join(outDir, "feeds/feed.xml"),
      path.join(outDir, "feeds/atom.xml"),
      path.join(outDir, "feeds/feed.json"),
    ]);
    expect(await fs.readFile(path.join(outDir, "feeds/feed.xml"), "utf8")).toContain(
      "<title>Newer</title>",
    );
    expect(await fs.readFile(path.join(outDir, "feeds/atom.xml"), "utf8")).toContain(
      'href="https://example.com/docs/feeds/atom.xml"',
    );
    expect(await fs.readFile(path.join(outDir, "feeds/feed.json"), "utf8")).toContain(
      '"feed_url": "https://example.com/docs/feeds/feed.json"',
    );
  });
});
