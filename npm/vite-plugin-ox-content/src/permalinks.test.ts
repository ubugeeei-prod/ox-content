import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { buildCollectionManifest, resolveCollectionsOptions } from "./collections";
import { applyCollectionRoutes, applySsgPageRoutes, remapNavGroups } from "./apply-permalinks";
import {
  escapeAttribute,
  resolveCascadeOptions,
  resolvePageRoutes,
  resolvePermalinksOptions,
} from "./permalinks";
import { buildSsg } from "./ssg";
import type { ResolvedOptions } from "./types";

const tempDirs: string[] = [];
afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("resolvePermalinksOptions / resolveCascadeOptions", () => {
  it("treats omitted and false as off", () => {
    expect(resolvePermalinksOptions(undefined)).toEqual({ enabled: false });
    expect(resolvePermalinksOptions(false)).toEqual({ enabled: false });
    expect(resolveCascadeOptions(undefined)).toEqual({ enabled: false });
    expect(resolveCascadeOptions(false)).toEqual({ enabled: false });
  });

  it("enables defaults for true or an object", () => {
    expect(resolvePermalinksOptions(true)).toEqual({ enabled: true });
    expect(resolvePermalinksOptions({})).toEqual({ enabled: true });
    expect(resolveCascadeOptions(true)).toEqual({ enabled: true });
    expect(resolveCascadeOptions({ enabled: false })).toEqual({ enabled: false });
  });
});

describe("resolvePageRoutes", () => {
  it("keeps file-tree URLs when disabled", () => {
    const output = resolvePageRoutes({
      pages: [
        page("guide/intro.md", "guide/intro", { permalink: "/custom", slug: "other" }),
        page("guide/child.md", "guide/child", {}),
      ],
      permalinks: { enabled: false },
      cascade: { enabled: false },
    });
    expect(output.pages.map((item) => item.urlPath)).toEqual(["guide/intro", "guide/child"]);
    expect(output.errors).toEqual([]);
  });

  it("rewrites permalink and slug, then inherits cascade defaults", () => {
    const output = resolvePageRoutes({
      pages: [
        page("guide/_index.md", "guide/_index", { sidebar: "Guide", title: "Section" }),
        page("guide/intro.md", "guide/intro", { permalink: "/getting-started", title: "Intro" }),
        page("guide/child.md", "guide/child", { slug: "hello" }),
      ],
      permalinks: { enabled: true },
      cascade: { enabled: true },
    });
    expect(urlOf(output, "guide/intro.md")).toBe("getting-started");
    expect(urlOf(output, "guide/child.md")).toBe("guide/hello");
    expect(field(output, "guide/child.md", "sidebar")).toBe("Guide");
    expect(field(output, "guide/intro.md", "title")).toBe("Intro");
    expect(output.errors).toEqual([]);
  });

  it("rejects path escape and reports URL collisions", () => {
    const rejected = resolvePageRoutes({
      pages: [
        page("a.md", "a", { permalink: "../etc/passwd" }),
        page("b.md", "b", { permalink: "javascript:alert(1)" }),
        page("c.md", "c", { permalink: "//evil.example" }),
        page("d.md", "d", { slug: ".." }),
      ],
      permalinks: { enabled: true },
    });
    expect(rejected.pages.map((item) => item.urlPath)).toEqual(["a", "b", "c", "d"]);
    expect(rejected.errors).toHaveLength(4);

    const collision = resolvePageRoutes({
      pages: [
        page("first.md", "first", { permalink: "/guide" }),
        page("second.md", "second", { permalink: "/guide/" }),
      ],
      permalinks: { enabled: true },
    });
    expect(collision.pages.map((item) => item.source)).toEqual(["first.md"]);
    expect(collision.errors[0]).toContain("collision");
  });

  it("escapes permalinks for HTML attributes and ignores hostile values", () => {
    expect(escapeAttribute(`/foo" onclick="alert(1)`)).toContain("&quot;");
    expect(escapeAttribute(`/foo" onclick="alert(1)`)).not.toContain(`" onclick="`);
    const output = resolvePageRoutes({
      pages: [
        page("a.md", "a", { permalink: "" }),
        { source: "b.md", fileUrl: "b", frontmatter: { permalink: true, slug: ["x"] } },
        page("c.md", "c", { permalink: "foo\nbar" }),
      ],
      permalinks: { enabled: true },
      cascade: { enabled: true },
    });
    expect(output.pages.map((item) => item.urlPath)).toEqual(["a", "b", "c"]);
  });
});

describe("SSG and collections", () => {
  it("writes permalink and slug output when enabled", async () => {
    const root = await makeSite({
      "guide/intro.md": "---\ntitle: Intro\npermalink: /getting-started\n---\n# Intro\n",
      "guide/child.md": "---\ntitle: Child\nslug: hello\n---\n# Child\n",
    });
    const off = await buildSsg(ssgOptions({ permalinks: { enabled: false } }), root);
    expect(off.files.some((file) => file.includes(`${path.sep}guide${path.sep}intro`))).toBe(true);

    const on = await buildSsg(ssgOptions({ permalinks: { enabled: true } }), root);
    expect(on.files.some((file) => file.includes(`${path.sep}getting-started${path.sep}`))).toBe(
      true,
    );
    expect(on.files.some((file) => file.includes(`${path.sep}guide${path.sep}hello`))).toBe(true);
    expect(on.errors).toEqual([]);
  });

  it("escapes a hostile permalink in generated HTML", async () => {
    const root = await makeSite({
      "page.md": "---\ntitle: Page\npermalink: '/foo\" onclick=\"alert(1)'\n---\n# Page\n",
    });
    const built = await buildSsg(
      ssgOptions({
        permalinks: { enabled: true },
        ssg: {
          enabled: true,
          extension: ".html",
          clean: false,
          bare: true,
          generateOgImage: false,
          lastUpdated: false,
          pagination: false,
          breadcrumbs: false,
          jsonLd: false,
          readerChrome: false,
          localeSwitcher: false,
          a11y: false,
          pageChrome: false,
          siteUrl: "https://example.com",
        },
      }),
      root,
    );
    const html = await fs.readFile(
      built.files.find((file) => file.endsWith(".html")) ?? "",
      "utf8",
    );
    expect(html).not.toContain(`onclick="alert(1)`);
    expect(html).toMatch(/&quot;|&#34;/u);
  });

  it("applies permalinks to collection paths", async () => {
    const root = await makeSite({
      "docs/guide.md": "---\ntitle: Guide\npermalink: /install\n---\n# Guide\n",
    });
    const manifest = await buildCollectionManifest(root, collectionOptions());
    expect(manifest.collections.docs[0]?.path).toBe("/install");
  });
});

describe("helpers", () => {
  it("remaps nav items onto resolved URLs", () => {
    const nav = remapNavGroups(
      [{ title: "Guide", items: [{ title: "Intro", path: "guide/intro", href: "/old" }] }],
      [{ fileUrl: "guide/intro", urlPath: "getting-started", href: "/getting-started/index.html" }],
      [],
    );
    expect(nav[0]?.items[0]).toEqual({
      title: "Intro",
      path: "getting-started",
      href: "/getting-started/index.html",
    });
  });

  it("rebuilds SSG route paths from a resolved URL", () => {
    const applied = applySsgPageRoutes({
      pages: [
        {
          inputPath: "/repo/content/guide/intro.md",
          routePaths: {
            outputPath: "/repo/dist/guide/intro/index.html",
            urlPath: "guide/intro",
            href: "/guide/intro/index.html",
            ogImagePath: "/repo/dist/guide/intro/og-image.png",
            ogImageUrl: "/guide/intro/og-image.png",
          },
          frontmatter: { permalink: "/getting-started" },
        },
      ],
      permalinks: { enabled: true },
      srcDir: "/repo/content",
      outDir: "/repo/dist",
      base: "/",
      extension: ".html",
    });
    expect(applied.pages[0]?.routePaths.urlPath).toBe("getting-started");
    expect(applied.errors).toEqual([]);
  });

  it("rewrites collection entries in place", () => {
    const { manifest, errors } = applyCollectionRoutes(
      {
        collections: {
          docs: [
            {
              id: "docs/guide",
              collection: "docs",
              path: "/docs/guide",
              stem: "docs/guide",
              source: "docs/guide.md",
              extension: ".md",
              title: "Guide",
              frontmatter: { permalink: "/install", title: "Guide" },
            },
          ],
        },
      },
      { enabled: true },
    );
    expect(manifest.collections.docs[0]?.path).toBe("/install");
    expect(errors).toEqual([]);
  });
});

function page(source: string, fileUrl: string, frontmatter: Record<string, unknown>) {
  return { source, fileUrl, frontmatter };
}

function urlOf(output: ReturnType<typeof resolvePageRoutes>, source: string): string | undefined {
  return output.pages.find((item) => item.source === source)?.urlPath;
}

function field(output: ReturnType<typeof resolvePageRoutes>, source: string, key: string): unknown {
  return output.pages.find((item) => item.source === source)?.frontmatter[key];
}

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-permalinks-"));
  tempDirs.push(root);
  for (const [name, source] of Object.entries(files)) {
    const full = path.join(root, "content", name);
    await fs.mkdir(path.dirname(full), { recursive: true });
    await fs.writeFile(full, source);
  }
  return root;
}

function ssgOptions(overrides: Partial<ResolvedOptions> = {}): ResolvedOptions {
  return {
    srcDir: "content",
    outDir: "dist",
    base: "/",
    extensions: [".md"],
    ssg: {
      enabled: true,
      extension: ".html",
      clean: false,
      bare: true,
      generateOgImage: false,
      lastUpdated: false,
      pagination: false,
      breadcrumbs: false,
      jsonLd: false,
      readerChrome: false,
      localeSwitcher: false,
      a11y: false,
      pageChrome: false,
    },
    permalinks: { enabled: false },
    cascade: { enabled: false },
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
    steps: { enabled: false },
    codeGroups: { enabled: false },
    sanitize: { enabled: false },
    editThisPage: { enabled: false, branch: "main", label: "Edit this page" },
    cjkEmphasis: false,
    codeBlockLint: { enabled: false, requireLanguage: false, trailingSpaces: true, mode: "warn" },
    codeBlockTypecheck: {
      enabled: false,
      languages: ["ts"],
      requireMeta: true,
      tsgoCommand: "tsgo",
      mode: "warn",
    },
    docsTests: { enabled: false, languages: ["js"], requireMeta: true },
    mermaid: false,
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    ogImage: false,
    ogImageOptions: { width: 1200, height: 630, cache: true, concurrency: 1, vuePlugin: "vitejs" },
    transformers: [],
    docs: false,
    search: { enabled: false, limit: 10, prefix: true, placeholder: "Search", hotkey: "/" },
    collections: { enabled: false, collections: {} },
    ogViewer: false,
    embeds: {
      github: {},
      openGraph: {},
      pm: false,
      spotify: false,
      appleMusic: false,
      speakerDeck: false,
      stackBlitz: false,
      twitter: false,
      bluesky: false,
      webContainer: false,
    },
    i18n: false,
    ...overrides,
  };
}

function collectionOptions(): ResolvedOptions {
  return {
    srcDir: "content",
    extensions: [".md"],
    frontmatter: true,
    permalinks: { enabled: true },
    cascade: { enabled: false },
    collections: resolveCollectionsOptions({ docs: "docs/**/*.md" }),
  } as ResolvedOptions;
}
