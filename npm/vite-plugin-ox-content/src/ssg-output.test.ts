import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";
import {
  planSsgOutputs,
  renderFeedFiles,
  resolveGitLastmod,
  writeFeedFiles,
  writeMarkdownCompanions,
  writeResourceFiles,
  writeSiteMapFiles,
} from "./index";
import { createRgba, encodePng, pngSize } from "./resources-image";

const tempDirs: string[] = [];
const repoRoot = path.resolve(fileURLToPath(new URL("../../..", import.meta.url)));

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("planSsgOutputs", () => {
  it("plans nothing when output features are off", () => {
    const plan = planSsgOutputs({
      outDir: "/site/dist",
      options: { ssg: false },
      pages: [
        {
          inputPath: "/site/content/guide.md",
          urlPath: "guide",
          outputPath: "/site/dist/guide/index.html",
          html: '<img src="./hero.png" alt="Hero">',
          source: "# Guide\n",
          title: "Guide",
        },
      ],
    });

    expect(plan.resources.options?.enabled).toBe(false);
    expect(plan.resources.pages).toEqual([]);
    expect(plan.markdownCompanions.options.enabled).toBe(false);
    expect(plan.markdownCompanions.pages).toEqual([]);
    expect(plan.feeds.options.enabled).toBe(false);
    expect(plan.siteMaps.options.enabled).toBe(false);
  });

  it("keeps markdownSource and lastUpdated when ssg.enabled is false", () => {
    const plan = planSsgOutputs({
      outDir: "/site/dist",
      options: {
        ssg: {
          enabled: false,
          markdownSource: true,
          lastUpdated: true,
          siteUrl: "https://example.com",
          siteName: "Docs",
        },
        siteMaps: true,
        feeds: true,
      },
      pages: [
        {
          inputPath: "/site/content/guide.md",
          urlPath: "guide",
          source: "---\ntitle: Guide\n---\n# Guide\n",
          title: "Guide",
          lastUpdated: 1_704_067_200_000,
        },
        {
          inputPath: "/site/content/secret.md",
          urlPath: "secret",
          source: "---\ndraft: true\n---\n# Secret\n",
          title: "Secret",
          draft: true,
        },
      ],
    });

    expect(plan.markdownCompanions.options.enabled).toBe(true);
    expect(plan.markdownCompanions.pages.map((page) => page.urlPath)).toEqual(["guide"]);
    expect(plan.feeds.options.enabled).toBe(true);
    expect(plan.feeds.items?.[0]?.title).toBe("Guide");
    expect(plan.siteMaps.pages[0]).toMatchObject({
      loc: "https://example.com/guide/",
      lastUpdated: 1_704_067_200_000,
    });
    expect(plan.siteMaps.pages[1]?.draft).toBe(true);
  });

  it("keeps self-hosted assets available for host-rendered pages", () => {
    const plan = planSsgOutputs({
      outDir: "/site/dist",
      root: "/site",
      srcDir: "content",
      options: {
        base: "/docs/",
        icons: { safelist: ["mdi:github"] },
        ssg: {
          enabled: false,
          theme: {
            fonts: {
              sans: {
                family: "Ox Test",
                provider: "local",
                path: "./fonts/ox-test.woff2",
                selfHost: true,
                preload: true,
              },
            },
            socialLinks: [{ icon: "mdi:discord", link: "https://discord.example" }],
          },
        },
      },
      pages: [],
    });

    expect(plan.selfHostedAssets).toMatchObject({
      outDir: "/site/dist",
      root: "/site",
      options: {
        base: "/docs/",
        srcDir: "content",
        icons: {
          enabled: true,
          mode: "css-mask",
          syntax: "unocss",
          include: [],
          safelist: ["mdi:github"],
        },
        ssg: {
          enabled: false,
          theme: {
            fonts: {
              sans: {
                family: "Ox Test",
                path: "./fonts/ox-test.woff2",
                selfHost: true,
                preload: true,
              },
            },
            socialLinks: [{ icon: "mdi:discord", link: "https://discord.example" }],
          },
        },
      },
    });
  });
});

describe("writeResourceFiles", () => {
  it("fingerprints and rewrites host HTML without buildSsg", async () => {
    const root = await makeSite({ "hero.png": samplePng(8, 8) });
    const inputPath = path.join(root, "content", "guide.md");
    await fs.writeFile(inputPath, "# Guide\n");
    const outputPath = path.join(root, "dist", "guide", "index.html");
    const plan = planSsgOutputs({
      outDir: path.join(root, "dist"),
      srcDir: path.join(root, "content"),
      root,
      options: { ssg: false, resources: { dedupe: true } },
      pages: [
        {
          inputPath,
          urlPath: "guide",
          outputPath,
          html: '<p><img src="./hero.png" alt="Hero"></p>',
          title: "Guide",
        },
      ],
    });

    const written = await writeResourceFiles(plan.resources);
    expect(written.errors).toEqual([]);
    expect(written.pages[0]?.html).toMatch(/\/assets\/content\/[a-f0-9]{64}\.png/);
    expect(written.files.some((file) => file.includes(`${path.sep}assets${path.sep}content`))).toBe(
      true,
    );
    const copied = written.files.find((file) => file.endsWith(`${path.sep}hero.png`));
    expect(copied).toBeDefined();
    expect(pngSize(await fs.readFile(copied!))).toEqual({ width: 8, height: 8 });
  });
});

describe("writeMarkdownCompanions", () => {
  it("writes companion Markdown for a host-rendered page", async () => {
    const root = await makeSite({});
    const source = "---\ntitle: Guide\n---\n# Guide\nBody.\n";
    const plan = planSsgOutputs({
      outDir: path.join(root, "dist"),
      options: { ssg: { enabled: false, markdownSource: true } },
      pages: [
        {
          inputPath: path.join(root, "content", "guide.md"),
          urlPath: "guide",
          source,
          title: "Guide",
          frontmatter: { title: "Guide" },
        },
      ],
    });

    const written = await writeMarkdownCompanions(plan.markdownCompanions);
    const companion = path.join(root, "dist", "guide.md");
    expect(written.files).toEqual([companion]);
    expect(written.errors).toEqual([]);
    expect(await fs.readFile(companion, "utf8")).toBe(source);
  });
});

describe("writeFeedFiles and writeSiteMapFiles", () => {
  it("emit named feeds and lastmod from the public writers", async () => {
    const root = await makeSite({});
    const outDir = path.join(root, "dist");
    const plan = planSsgOutputs({
      outDir,
      siteDescription: "Example docs",
      options: {
        ssg: { enabled: false, siteUrl: "https://example.com", siteName: "Docs" },
        feeds: {
          blog: { formats: ["rss"], collection: "blog", path: "/blog" },
        },
        siteMaps: { robots: false, llms: false },
      },
      collections: {
        blog: [
          {
            title: "Newer",
            loc: "https://example.com/blog/newer/",
            date: "2024-02-01",
          },
        ],
      },
      pages: [
        {
          inputPath: path.join(root, "content", "guide.md"),
          urlPath: "guide",
          title: "Guide",
          lastUpdated: 1_704_067_200_000,
        },
      ],
    });

    const renderedFeeds = await renderFeedFiles(plan.feeds);
    expect(renderedFeeds.files.map((file) => [file.path, file.contentType])).toEqual([
      ["blog/feed.xml", "application/rss+xml; charset=utf-8"],
    ]);
    expect(renderedFeeds.files[0]?.content).toContain("<title>Newer</title>");

    const feeds = await writeFeedFiles(plan.feeds);
    const siteMaps = await writeSiteMapFiles(plan.siteMaps);
    expect(feeds.files).toEqual([path.join(outDir, "blog", "feed.xml")]);
    expect(await fs.readFile(feeds.files[0]!, "utf8")).toContain("<title>Newer</title>");
    expect(siteMaps.files).toEqual([path.join(outDir, "sitemap.xml")]);
    expect(await fs.readFile(siteMaps.files[0]!, "utf8")).toContain(
      "<loc>https://example.com/guide/</loc>\n    <lastmod>2024-01-01</lastmod>",
    );
  });
});

describe("resolveGitLastmod", () => {
  it("resolves a git timestamp or returns undefined without a root", () => {
    expect(resolveGitLastmod(path.join(repoRoot, "package.json"))).toBeUndefined();
    const updated = resolveGitLastmod(path.join(repoRoot, "package.json"), repoRoot);
    expect(updated === undefined || (typeof updated === "number" && updated > 0)).toBe(true);
  });
});

function samplePng(width: number, height: number): Buffer {
  return encodePng(createRgba(width, height, (x, y) => [x * 30, y * 30, 120, 255]));
}

async function makeSite(files: Record<string, string | Buffer>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-ssg-output-"));
  tempDirs.push(root);
  for (const [relative, body] of Object.entries(files)) {
    const filePath = path.join(root, "content", relative);
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, body);
  }
  return root;
}
