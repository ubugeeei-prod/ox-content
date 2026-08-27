import { describe, expect, it } from "vite-plus/test";
import { generateSiteMaps } from "./site-maps";
import { importNapiModuleSync } from "./napi";
import type { SiteMapPageInput } from "./site-maps";

/**
 * The crawl-manifest corpus.
 *
 * `site-maps.ts` used to hold a second implementation of
 * `ox_content_ssg::site_maps`, and nothing checked that the two agreed — the
 * audit in `.plans/060-typescript-rust-migration-audit.md` listed the pair as
 * unverified. They were proven byte-identical on this corpus, then the
 * TypeScript copy was deleted.
 *
 * The corpus stays. It leans on what the renderer escapes and how it orders,
 * which is where a silent change would live.
 */
const PAGES: SiteMapPageInput[] = [
  // Out of order on purpose: both sides sort by `loc`.
  { loc: "https://example.com/z", title: "Zed", lastUpdated: 1_700_000_000_000 },
  { loc: "https://example.com/a", title: "Ay", description: "first" },
  // Every character each side escapes, in the title and the URL.
  {
    loc: "https://example.com/q?a=1&b=2<3>4'5\"6",
    title: "Title & <tag> \"quoted\" 'single'",
    description: "desc & <more>",
    lastUpdated: 0,
  },
  // Dropped by both: draft, unlisted, and an empty loc.
  { loc: "https://example.com/draft", title: "Draft", draft: true },
  { loc: "https://example.com/unlisted", title: "Unlisted", unlisted: true },
  { loc: "", title: "No location" },
  // A negative timestamp has no `<lastmod>` on either side.
  { loc: "https://example.com/ancient", title: "Ancient", lastUpdated: -1 },
];

const OPTIONS = {
  enabled: true,
  robots: true,
  llms: true,
} as const;

const INPUT = {
  options: OPTIONS,
  siteUrl: "https://example.com",
  sitemapLoc: "https://example.com/sitemap.xml",
  siteName: "Docs & Co",
  siteDescription: "Example <docs>",
  pages: PAGES,
};

interface NativeOutput {
  sitemapXml?: string;
  robotsTxt?: string;
  llmsTxt?: string;
  warning?: string;
}

function native(pages: SiteMapPageInput[] = PAGES): NativeOutput {
  const napi = importNapiModuleSync() as unknown as {
    generateSiteMapBodies(
      options: Record<string, unknown>,
      pages: readonly SiteMapPageInput[],
    ): NativeOutput;
  };
  return napi.generateSiteMapBodies(
    {
      enabled: true,
      siteUrl: INPUT.siteUrl,
      sitemapLoc: INPUT.sitemapLoc,
      siteName: INPUT.siteName,
      siteDescription: INPUT.siteDescription,
      robots: true,
      llms: true,
    },
    pages,
  );
}

describe("crawl manifests", () => {
  it("renders sitemap.xml", () => {
    expect(native().sitemapXml).toMatchSnapshot();
  });

  it("renders robots.txt", () => {
    expect(native().robotsTxt).toMatchSnapshot();
  });

  it("renders llms.txt", () => {
    expect(native().llmsTxt).toMatchSnapshot();
  });

  it("drops draft, unlisted, and locationless pages", () => {
    const xml = native().sitemapXml ?? "";
    expect(xml).not.toContain("/draft");
    expect(xml).not.toContain("/unlisted");
    expect(xml).not.toContain("<loc></loc>");
  });

  // The plugin still owns this: the binding renders, it does not validate.
  it("still refuses to render without a site URL", () => {
    expect(generateSiteMaps({ ...INPUT, siteUrl: undefined }).warning).toContain("ssg.siteUrl");
  });
});
