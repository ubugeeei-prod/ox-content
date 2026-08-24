import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { buildSearchIndex } from "./search";
import { buildSsg, resolveSsgOptions } from "./ssg";
import {
  notFoundSearchDocumentId,
  notFoundSearchExcludeIds,
  resolveNotFoundOptions,
} from "./not-found";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

async function makeSite(files: Record<string, string>): Promise<{ root: string; srcDir: string }> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-not-found-"));
  tempDirs.push(root);
  const srcDir = path.join(root, "content");
  await fs.mkdir(srcDir, { recursive: true });
  for (const [relative, body] of Object.entries(files)) {
    const filePath = path.join(srcDir, relative);
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, body, "utf8");
  }
  return { root, srcDir };
}

describe("resolveNotFoundOptions", () => {
  it("disables the feature by default", () => {
    expect(resolveNotFoundOptions(undefined)).toEqual({
      enabled: false,
      source: "404.md",
      output: "404.html",
    });
    expect(resolveNotFoundOptions(false)).toEqual({
      enabled: false,
      source: "404.md",
      output: "404.html",
    });
    expect(resolveSsgOptions(undefined).notFound?.enabled).toBe(false);
    expect(resolveSsgOptions(true).notFound?.enabled).toBe(false);
  });

  it("enables defaults when true", () => {
    expect(resolveNotFoundOptions(true)).toEqual({
      enabled: true,
      source: "404.md",
      output: "404.html",
    });
    expect(resolveSsgOptions({ notFound: true }).notFound).toEqual({
      enabled: true,
      source: "404.md",
      output: "404.html",
    });
  });

  it("enables the feature from an object and overrides only set fields", () => {
    expect(resolveNotFoundOptions({ source: "pages/missing.md" })).toEqual({
      enabled: true,
      source: "pages/missing.md",
      output: "404.html",
    });
    expect(resolveNotFoundOptions({ output: "not-found.html" })).toEqual({
      enabled: true,
      source: "404.md",
      output: "not-found.html",
    });
    expect(resolveNotFoundOptions({})).toEqual({
      enabled: true,
      source: "404.md",
      output: "404.html",
    });
  });
});

describe("buildSsg notFound", () => {
  it("does not write 404.html when the feature is omitted or disabled", async () => {
    const { root } = await makeSite({
      "index.md": "# Home\n\nWelcome.\n",
      "404.md": "# Missing\n\nThis should stay a regular page.\n",
    });
    const base = createDocsResolvedOptions();
    const result = await buildSsg(
      createDocsResolvedOptions({
        ssg: { ...base.ssg, notFound: resolveNotFoundOptions(false) },
      }),
      root,
    );

    expect(result.files.some((file) => file.endsWith(`${path.sep}404.html`))).toBe(false);
    await expect(fs.access(path.join(root, "dist", "404.html"))).rejects.toThrow();
    await expect(fs.access(path.join(root, "dist", "404", "index.html"))).resolves.toBeUndefined();
  });

  it("writes a themed 404.html from 404.md when enabled", async () => {
    const { root } = await makeSite({
      "index.md": "# Home\n\nWelcome home.\n",
      "404.md": "---\ntitle: Lost page\n---\n\n# Lost page\n\nNothing lives here.\n",
    });
    const base = createDocsResolvedOptions();
    const result = await buildSsg(
      createDocsResolvedOptions({
        ssg: {
          ...base.ssg,
          siteName: "Docs",
          notFound: resolveNotFoundOptions(true),
        },
      }),
      root,
    );

    const outputPath = path.join(root, "dist", "404.html");
    expect(result.files).toContain(outputPath);
    const html = await fs.readFile(outputPath, "utf8");
    expect(html).toContain("Lost page");
    expect(html).toContain("Nothing lives here.");
    expect(html).toContain('class="search-button"');
    expect(html).toContain('class="search-input"');
    expect(html).toContain('class="sidebar');
    expect(html).toContain('href="/index.html"');
    expect(html).toContain("Overview");
    await expect(fs.access(path.join(root, "dist", "404", "index.html"))).rejects.toThrow();
  });

  it("writes a themed fallback when the source file is missing", async () => {
    const { root } = await makeSite({
      "index.md": "# Home\n\nWelcome home.\n",
    });
    const base = createDocsResolvedOptions();
    await buildSsg(
      createDocsResolvedOptions({
        ssg: {
          ...base.ssg,
          siteName: "Docs",
          notFound: resolveNotFoundOptions(true),
        },
      }),
      root,
    );

    const html = await fs.readFile(path.join(root, "dist", "404.html"), "utf8");
    expect(html).toContain("Page not found");
    expect(html).toContain("<title>Page not found - Docs</title>");
    expect(html).toContain('class="search-button"');
    expect(html).toContain('class="sidebar');
  });

  it("excludes the 404 page from the search index and sitemap", async () => {
    const { root, srcDir } = await makeSite({
      "index.md": "# Home\n\nWelcome home.\n",
      "404.md": "# Lost page\n\nUnique 404 phrase ox-not-found-exclude.\n",
    });
    const notFound = resolveNotFoundOptions(true);
    const base = createDocsResolvedOptions();
    await buildSsg(
      createDocsResolvedOptions({
        ssg: {
          ...base.ssg,
          siteUrl: "https://example.com",
          notFound,
        },
        siteMaps: { enabled: true, robots: true, llms: true },
      }),
      root,
    );

    const sitemap = await fs.readFile(path.join(root, "dist", "sitemap.xml"), "utf8");
    expect(sitemap).toContain("https://example.com/");
    expect(sitemap).not.toContain("404");

    const included = JSON.parse(await buildSearchIndex(srcDir, "/")) as {
      documents: Array<{ id: string; body: string }>;
    };
    expect(included.documents.some((doc) => doc.id === "404")).toBe(true);

    const excluded = JSON.parse(
      await buildSearchIndex(
        srcDir,
        "/",
        [".md", ".markdown", ".mdx"],
        undefined,
        notFoundSearchExcludeIds(notFound),
      ),
    ) as { documents: Array<{ id: string; body: string }> };
    expect(notFoundSearchDocumentId(notFound.source)).toBe("404");
    expect(excluded.documents.some((doc) => doc.id === "404")).toBe(false);
    expect(excluded.documents.some((doc) => doc.body.includes("ox-not-found-exclude"))).toBe(false);
    expect(excluded.documents.some((doc) => doc.id === "index")).toBe(true);
  });

  it("escapes a hostile title from 404.md", async () => {
    const { root } = await makeSite({
      "index.md": "# Home\n\nWelcome.\n",
      "404.md": `---
title: "</title><script>alert(1)</script>"
---

# Safe heading

Body.
`,
    });
    const base = createDocsResolvedOptions();
    await buildSsg(
      createDocsResolvedOptions({
        ssg: {
          ...base.ssg,
          siteName: "Docs",
          notFound: resolveNotFoundOptions(true),
        },
      }),
      root,
    );

    const html = await fs.readFile(path.join(root, "dist", "404.html"), "utf8");
    expect(html).not.toContain("<script>alert(1)</script>");
    expect(html).toMatch(/&#60;script&#62;|&lt;script&gt;/);
    expect(html).toMatch(/&#60;\/title&#62;|&lt;\/title&gt;/);
  });
});
