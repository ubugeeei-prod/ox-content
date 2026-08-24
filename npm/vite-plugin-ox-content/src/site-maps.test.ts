import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { generateSiteMaps, resolveSiteMapsOptions, writeSiteMapFiles } from "./site-maps";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

const samplePages = [
  {
    loc: "https://example.com/guide/",
    title: "Getting Started",
    description: "How to install",
  },
  { loc: "https://example.com/api/", title: "API" },
];

describe("resolveSiteMapsOptions", () => {
  it("disables the feature by default", () => {
    expect(resolveSiteMapsOptions(undefined)).toEqual({
      enabled: false,
      robots: true,
      llms: true,
    });
    expect(resolveSiteMapsOptions(false)).toEqual({
      enabled: false,
      robots: true,
      llms: true,
    });
  });

  it("enables defaults when true", () => {
    expect(resolveSiteMapsOptions(true)).toEqual({
      enabled: true,
      robots: true,
      llms: true,
    });
  });

  it("enables the feature from an object and overrides only set fields", () => {
    expect(resolveSiteMapsOptions({ robots: false, llms: false })).toEqual({
      enabled: true,
      robots: false,
      llms: false,
    });
    expect(resolveSiteMapsOptions({})).toEqual({
      enabled: true,
      robots: true,
      llms: true,
    });
  });
});

describe("generateSiteMaps", () => {
  it("writes nothing when the feature is omitted or disabled", () => {
    expect(generateSiteMaps({ pages: samplePages })).toEqual({});
    expect(
      generateSiteMaps({
        options: { enabled: false, robots: true, llms: true },
        siteUrl: "https://example.com",
        pages: samplePages,
      }),
    ).toEqual({});
  });

  it("warns and writes nothing when siteUrl is missing", () => {
    expect(
      generateSiteMaps({
        options: { enabled: true, robots: true, llms: true },
        pages: samplePages,
      }),
    ).toEqual({
      warning:
        "[ox-content] siteMaps is enabled but ssg.siteUrl is not set; sitemap.xml, robots.txt, and llms.txt were not written",
    });
    expect(
      generateSiteMaps({
        options: { enabled: true, robots: true, llms: true },
        siteUrl: "   ",
        pages: samplePages,
      }).warning,
    ).toBeDefined();
  });

  it("writes a sorted sitemap, robots.txt, and llms.txt", () => {
    const output = generateSiteMaps({
      options: { enabled: true, robots: true, llms: true },
      siteUrl: "https://example.com",
      sitemapLoc: "https://example.com/sitemap.xml",
      siteName: "Docs",
      siteDescription: "Example docs",
      pages: samplePages,
    });

    expect(output.sitemapXml).toBe(`\
<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url>
    <loc>https://example.com/api/</loc>
  </url>
  <url>
    <loc>https://example.com/guide/</loc>
  </url>
</urlset>
`);
    expect(output.robotsTxt).toBe(
      "User-agent: *\nAllow: /\n\nSitemap: https://example.com/sitemap.xml\n",
    );
    expect(output.llmsTxt).toBe(`\
# Docs

> Example docs

## Pages

- [API](https://example.com/api/)
- [Getting Started](https://example.com/guide/): How to install
`);
    expect(output.warning).toBeUndefined();
  });

  it("honors object overrides for robots and llms", () => {
    const output = generateSiteMaps({
      options: { enabled: true, robots: false, llms: false },
      siteUrl: "https://example.com",
      sitemapLoc: "https://example.com/sitemap.xml",
      pages: samplePages,
    });

    expect(output.sitemapXml).toContain("<loc>https://example.com/api/</loc>");
    expect(output.robotsTxt).toBeUndefined();
    expect(output.llmsTxt).toBeUndefined();
  });

  it("omits draft pages", () => {
    const output = generateSiteMaps({
      options: { enabled: true, robots: true, llms: true },
      siteUrl: "https://example.com",
      siteName: "Docs",
      pages: [
        { loc: "https://example.com/secret/", title: "Secret", draft: true },
        { loc: "https://example.com/public/", title: "Public" },
      ],
    });

    expect(output.sitemapXml).toContain("https://example.com/public/");
    expect(output.sitemapXml).not.toContain("secret");
    expect(output.llmsTxt).toContain("Public");
    expect(output.llmsTxt).not.toContain("Secret");
  });

  it("escapes hostile titles and descriptions", () => {
    const output = generateSiteMaps({
      options: { enabled: true, robots: true, llms: true },
      siteUrl: "https://example.com",
      siteName: "Docs",
      pages: [
        {
          loc: `https://example.com/x?a=1&b=2<>"'`,
          title: "</loc></urlset><script>alert(1)</script>\n- [Injected](https://evil.example/)",
          description: '">\n<img src=x onerror=alert(1)>',
        },
      ],
    });

    expect(output.sitemapXml).not.toContain("<script>");
    expect(output.sitemapXml).toContain("&amp;");
    expect(output.sitemapXml).toContain("&lt;");
    expect(output.llmsTxt).not.toContain("<script>");
    expect(output.llmsTxt).not.toContain("\n- [Injected](https://evil.example/)");
    expect(output.llmsTxt).not.toContain("<img");
    expect(output.llmsTxt).toContain("\\[Injected\\]");
  });
});

describe("writeSiteMapFiles", () => {
  it("does not write files when the feature is disabled", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-sitemaps-"));
    tempDirs.push(outDir);

    const result = await writeSiteMapFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/",
      pages: samplePages,
    });

    expect(result.files).toEqual([]);
    expect(result.warning).toBeUndefined();
    await expect(fs.access(path.join(outDir, "sitemap.xml"))).rejects.toThrow();
  });

  it("does not write files when siteUrl is missing", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-sitemaps-"));
    tempDirs.push(outDir);

    const result = await writeSiteMapFiles({
      outDir,
      base: "/",
      options: { enabled: true, robots: true, llms: true },
      pages: samplePages,
    });

    expect(result.files).toEqual([]);
    expect(result.warning).toContain("ssg.siteUrl is not set");
    await expect(fs.access(path.join(outDir, "sitemap.xml"))).rejects.toThrow();
  });

  it("writes the enabled files next to generated HTML", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-sitemaps-"));
    tempDirs.push(outDir);

    const result = await writeSiteMapFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/docs/",
      siteName: "Docs",
      siteDescription: "Example docs",
      options: { enabled: true, robots: true, llms: true },
      pages: samplePages,
    });

    expect(result.files).toEqual([
      path.join(outDir, "sitemap.xml"),
      path.join(outDir, "robots.txt"),
      path.join(outDir, "llms.txt"),
    ]);
    expect(await fs.readFile(path.join(outDir, "sitemap.xml"), "utf8")).toContain(
      "<loc>https://example.com/api/</loc>",
    );
    expect(await fs.readFile(path.join(outDir, "robots.txt"), "utf8")).toContain(
      "Sitemap: https://example.com/docs/sitemap.xml",
    );
    expect(await fs.readFile(path.join(outDir, "llms.txt"), "utf8")).toContain("# Docs");
  });
});
