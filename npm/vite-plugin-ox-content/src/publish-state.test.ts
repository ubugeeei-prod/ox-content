import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import {
  classifyPublishState,
  filterNavGroups,
  hiddenNavKeys,
  partitionPublishedPages,
  resolvePublishStateOptions,
} from "./publish-state";
import { buildSearchIndex } from "./search";
import { buildSsg } from "./ssg";
import { generateSiteMaps, resolveSiteMapsOptions } from "./site-maps";
import type { ResolvedOptions } from "./types";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("resolvePublishStateOptions", () => {
  it("stays off when omitted or false", () => {
    expect(resolvePublishStateOptions(undefined)).toEqual({ enabled: false, includeDrafts: false });
    expect(resolvePublishStateOptions(false)).toEqual({ enabled: false, includeDrafts: false });
  });

  it("enables defaults from true or an object", () => {
    expect(resolvePublishStateOptions(true)).toEqual({ enabled: true, includeDrafts: false });
    expect(resolvePublishStateOptions({ now: "2026-08-24T00:00:00Z" })).toEqual({
      enabled: true,
      now: "2026-08-24T00:00:00Z",
      includeDrafts: false,
    });
    expect(resolvePublishStateOptions({ enabled: false })).toEqual({
      enabled: false,
      includeDrafts: false,
    });
  });
});

describe("classifyPublishState", () => {
  const now = { enabled: true, now: "2026-08-24T00:00:00Z", includeDrafts: false };

  it("publishes every page while the feature is off", () => {
    expect(classifyPublishState({ draft: true }, { enabled: false, includeDrafts: false })).toEqual(
      {
        output: true,
        listed: true,
      },
    );
  });

  it("honors draft, unlisted, scheduled, date, and expiry", () => {
    expect(classifyPublishState({ draft: true }, now)).toEqual({ output: false, listed: false });
    expect(classifyPublishState({ unlisted: true }, now)).toEqual({ output: true, listed: false });
    expect(classifyPublishState({ scheduled: "2026-08-25T00:00:00Z" }, now)).toEqual({
      output: false,
      listed: false,
    });
    expect(classifyPublishState({ date: "2026-08-25" }, now)).toEqual({
      output: false,
      listed: false,
    });
    expect(classifyPublishState({ expiry: "2026-08-23T00:00:00Z" }, now)).toEqual({
      output: false,
      listed: false,
    });
  });

  it("does not panic on hostile frontmatter and treats invalid scheduled as unpublished", () => {
    expect(() =>
      classifyPublishState(
        { draft: "yes", scheduled: "<script>alert(1)</script>", title: "</loc>" },
        now,
      ),
    ).not.toThrow();
    expect(classifyPublishState({ scheduled: "<script>alert(1)</script>" }, now)).toEqual({
      output: false,
      listed: false,
    });
    expect(classifyPublishState({ date: "Q1 2024" }, now)).toEqual({ output: true, listed: true });
    expect(classifyPublishState({ draft: "true" }, now)).toEqual({ output: true, listed: true });
  });
});

describe("partitionPublishedPages and nav filtering", () => {
  it("keeps unpublished pages out of listings and drops them from nav", () => {
    const pages = [
      page("public.md", "Public", "/public/index.html", "public", {}),
      page("draft.md", "</nav><script>alert(1)</script>", "/draft/index.html", "draft", {
        draft: true,
      }),
      page("hidden.md", "Hidden", "/hidden/index.html", "hidden", { unlisted: true }),
    ];
    const { output, listed } = partitionPublishedPages(pages, {
      enabled: true,
      now: "2026-08-24T00:00:00Z",
      includeDrafts: false,
    });
    expect(output.map((item) => item.inputPath)).toEqual(["public.md", "hidden.md"]);
    expect(listed.map((item) => item.inputPath)).toEqual(["public.md"]);

    const nav = filterNavGroups(
      [
        {
          title: "Guide",
          items: [
            { title: "Public", path: "public", href: "/public/index.html" },
            { title: "Draft", path: "draft", href: "/draft/index.html" },
            { title: "Hidden", path: "hidden", href: "/hidden/index.html" },
          ],
        },
      ],
      hiddenNavKeys(pages, listed),
    );
    expect(nav).toEqual([
      { title: "Guide", items: [{ title: "Public", path: "public", href: "/public/index.html" }] },
    ]);
  });
});

describe("production SSG and search", () => {
  it("omits drafts from HTML, nav, sitemap, and search when enabled", async () => {
    const root = await makeSite({
      "index.md": "---\ntitle: Home\n---\n# Home\n",
      "guide.md": "---\ntitle: Guide\n---\n# Guide\nVisible body.\n",
      "draft.md":
        '---\ntitle: "</title><script>alert(1)</script>"\ndraft: true\n---\n# Draft\nSecret.\n',
      "hidden.md": "---\ntitle: Hidden\nunlisted: true\n---\n# Hidden\nDirect only.\n",
      "later.md": "---\ntitle: Later\nscheduled: 2099-01-01T00:00:00Z\n---\n# Later\n",
    });

    const off = await buildSsg(options(root, false), root);
    expect(off.files.some((file) => file.endsWith(`${path.sep}draft${path.sep}index.html`))).toBe(
      true,
    );

    const built = await buildSsg(options(root, true), root);
    const htmlFiles = built.files.filter((file) => file.endsWith(".html"));
    expect(htmlFiles.some((file) => file.includes(`${path.sep}draft${path.sep}`))).toBe(false);
    expect(htmlFiles.some((file) => file.includes(`${path.sep}later${path.sep}`))).toBe(false);
    expect(htmlFiles.some((file) => file.includes(`${path.sep}hidden${path.sep}`))).toBe(true);
    expect(htmlFiles.some((file) => file.includes(`${path.sep}guide${path.sep}`))).toBe(true);

    const guide = await fs.readFile(
      htmlFiles.find((file) => file.includes(`${path.sep}guide${path.sep}`)) ?? "",
      "utf8",
    );
    expect(guide).toContain("Guide");
    expect(guide).not.toContain("Draft");
    expect(guide).not.toContain("<script>alert(1)</script>");
    expect(guide).not.toContain("Hidden");

    const hidden = await fs.readFile(
      htmlFiles.find((file) => file.includes(`${path.sep}hidden${path.sep}`)) ?? "",
      "utf8",
    );
    expect(hidden).toContain("Direct only");

    const writtenSitemap = await fs.readFile(path.join(root, "dist", "sitemap.xml"), "utf8");
    expect(writtenSitemap).toContain("guide");
    expect(writtenSitemap).not.toContain("hidden");
    expect(writtenSitemap).not.toContain("draft");
    expect(writtenSitemap).not.toContain("later");

    const sitemap = generateSiteMaps({
      options: resolveSiteMapsOptions(true),
      siteUrl: "https://example.com",
      siteName: "Docs",
      pages: [
        { loc: "https://example.com/guide/", title: "Guide" },
        { loc: "https://example.com/hidden/", title: "Hidden", unlisted: true },
        {
          loc: `https://example.com/x?a=1&b=2<>"'`,
          title: "</loc></urlset><script>alert(1)</script>",
        },
      ],
    });
    expect(sitemap.sitemapXml).toContain("https://example.com/guide/");
    expect(sitemap.sitemapXml).not.toContain("hidden");
    expect(sitemap.sitemapXml).toContain("&amp;");
    expect(sitemap.sitemapXml).toContain("&lt;");
    expect(sitemap.sitemapXml).not.toContain("<script>");
    expect(sitemap.llmsTxt).not.toContain("<script>");
    expect(sitemap.llmsTxt).toContain("&lt;/loc&gt;");

    const index = JSON.parse(
      await buildSearchIndex(path.join(root, "content"), "/", [".md"], {
        enabled: true,
        now: "2026-08-24T00:00:00Z",
        includeDrafts: false,
      }),
    ) as { documents: Array<{ title: string; id: string }> };
    const ids = index.documents.map((doc) => doc.id);
    expect(ids).toContain("guide");
    expect(ids).not.toContain("draft");
    expect(ids).not.toContain("hidden");
    expect(ids).not.toContain("later");
    expect(index.documents.some((doc) => doc.title.includes("<script>"))).toBe(false);
  });

  it("keeps drafts in the index when includeDrafts is set", async () => {
    const root = await makeSite({
      "draft.md": "---\ntitle: Draft\ndraft: true\n---\n# Draft\n",
      "hidden.md": "---\ntitle: Hidden\nunlisted: true\n---\n# Hidden\n",
    });
    const index = JSON.parse(
      await buildSearchIndex(path.join(root, "content"), "/", [".md"], {
        enabled: true,
        includeDrafts: true,
      }),
    ) as { documents: Array<{ id: string }> };
    expect(index.documents.map((doc) => doc.id)).toEqual(["draft"]);
  });
});

function page(
  inputPath: string,
  title: string,
  href: string,
  urlPath: string,
  frontmatter: Record<string, unknown>,
) {
  return { inputPath, title, frontmatter, routePaths: { href, urlPath } };
}

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-publish-"));
  tempDirs.push(root);
  const content = path.join(root, "content");
  await fs.mkdir(content, { recursive: true });
  await Promise.all(
    Object.entries(files).map(([name, source]) => fs.writeFile(path.join(content, name), source)),
  );
  return root;
}

function options(root: string, enabled: boolean): ResolvedOptions {
  return {
    srcDir: "content",
    outDir: "dist",
    base: "/",
    extensions: [".md"],
    ssg: {
      enabled: true,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
      pagination: false,
      breadcrumbs: false,
      jsonLd: false,
      readerChrome: false,
      localeSwitcher: false,
      a11y: false,
      pageChrome: false,
      siteName: "Docs",
      siteUrl: "https://example.com",
    },
    siteMaps: { enabled: true, robots: true, llms: true },
    publishState: {
      enabled,
      now: "2026-08-24T00:00:00Z",
      includeDrafts: false,
    },
    gfm: true,
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    autolinks: true,
    highlight: false,
    codeAnnotations: {
      enabled: false,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    },
    wikiLinks: { enabled: false, baseUrl: "/" },
    emojiShortcodes: { enabled: false, custom: {} },
    attrs: { enabled: false },
    badges: { enabled: false },
    containers: { enabled: false, types: {} },
    images: { enabled: false, lazy: true },
    codeImports: { enabled: false },
    includes: { enabled: false },
    sanitize: { enabled: false },
    editThisPage: { enabled: false, branch: "main", label: "Edit this page" },
    cjkEmphasis: false,
    codeBlockLint: { enabled: false, requireLanguage: false, trailingSpaces: true, mode: "warn" },
    codeBlockTypecheck: {
      enabled: false,
      languages: ["ts", "tsx"],
      requireMeta: true,
      tsgoCommand: "tsgo",
      mode: "warn",
    },
    docsTests: { enabled: false, languages: ["js", "jsx", "ts", "tsx"], requireMeta: true },
    mermaid: false,
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    ogImage: false,
    ogImageOptions: {
      renderer: "chromium",
      width: 1200,
      height: 630,
      cache: true,
      concurrency: 1,
      vuePlugin: "vitejs",
      satori: { fonts: [], systemFontFallback: true },
    },
    transformers: [],
    docs: false,
    search: {
      enabled: true,
      limit: 10,
      prefix: true,
      placeholder: "Search documentation...",
      hotkey: "/",
    },
    collections: { enabled: false, collections: {} },
    ogViewer: false,
    embeds: {
      github: {},
      openGraph: {},
      pm: false,
      spotify: false,
      stackBlitz: false,
      twitter: false,
      bluesky: false,
      webContainer: false,
    },
    i18n: false,
  };
}
