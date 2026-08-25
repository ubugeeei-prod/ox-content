import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { buildSsg, resolveSsgOptions } from "./ssg";
import {
  escapeSectionIndexHtml,
  isSafeSectionHref,
  renderSectionIndexHtml,
  resolveSectionIndexOptions,
} from "./section-index";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-section-index-"));
  tempDirs.push(root);
  const srcDir = path.join(root, "content");
  await fs.mkdir(srcDir, { recursive: true });
  for (const [relative, body] of Object.entries(files)) {
    const filePath = path.join(srcDir, relative);
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, body, "utf8");
  }
  return root;
}

describe("resolveSectionIndexOptions", () => {
  it("disables the feature by default", () => {
    expect(resolveSectionIndexOptions(undefined)).toEqual({ enabled: false, style: "cards" });
    expect(resolveSectionIndexOptions(false)).toEqual({ enabled: false, style: "cards" });
    expect(resolveSsgOptions(undefined).sectionIndex?.enabled).toBe(false);
    expect(resolveSsgOptions(true).sectionIndex?.enabled).toBe(false);
    expect(resolveSsgOptions({}).sectionIndex?.enabled).toBe(false);
  });

  it("enables cards when true, and honors style on an object", () => {
    expect(resolveSectionIndexOptions(true)).toEqual({ enabled: true, style: "cards" });
    expect(resolveSsgOptions({ sectionIndex: true }).sectionIndex).toEqual({
      enabled: true,
      style: "cards",
    });
    expect(resolveSectionIndexOptions({})).toEqual({ enabled: true, style: "cards" });
    expect(resolveSectionIndexOptions({ style: "list" })).toEqual({
      enabled: true,
      style: "list",
    });
    expect(resolveSsgOptions({ sectionIndex: { style: "list" } }).sectionIndex).toEqual({
      enabled: true,
      style: "list",
    });
  });
});

describe("section index listing safety", () => {
  it("escapes hostile titles and drops javascript: hrefs", () => {
    const html = renderSectionIndexHtml(
      `</title><script>alert(1)</script>`,
      [
        {
          title: `<img src=x onerror=alert(1)>`,
          href: "/guide/a/index.html",
        },
        {
          title: "XSS",
          href: "javascript:alert(1)",
        },
        {
          title: `'" onclick="alert(1)"`,
          href: `/guide/b/index.html" onclick="alert(1)`,
        },
      ],
      "cards",
    );

    expect(html).toContain("ox-section-index");
    expect(html).not.toContain("<script>alert(1)</script>");
    expect(html).toContain("&lt;script&gt;alert(1)&lt;/script&gt;");
    expect(html).toContain("&lt;img src=x onerror=alert(1)&gt;");
    expect(html).not.toContain("<img src=x");
    expect(html).not.toContain("javascript:");
    expect(html).not.toContain(">XSS<");
    expect(html).not.toContain(`onclick="alert(1)"`);
    expect(html).toContain("/guide/b/index.html&quot; onclick=&quot;alert(1)");
    expect(isSafeSectionHref("javascript:alert(1)")).toBe(false);
    expect(isSafeSectionHref("data:text/html,hi")).toBe(false);
    expect(isSafeSectionHref("//evil.example/x")).toBe(false);
    expect(isSafeSectionHref("/guide/a/index.html")).toBe(true);
    expect(escapeSectionIndexHtml(`<>&"'`)).toBe("&lt;&gt;&amp;&quot;&#39;");
  });
});

describe("buildSsg sectionIndex", () => {
  it("does not write a generated index when the feature is omitted or disabled", async () => {
    const root = await makeSite({
      "index.md": "# Home\n\nWelcome.\n",
      "guide/a.md": "---\ntitle: Page A\n---\n\n# Page A\n",
      "guide/b.md": "---\ntitle: Page B\n---\n\n# Page B\n",
    });
    const base = createDocsResolvedOptions();
    const result = await buildSsg(
      createDocsResolvedOptions({
        ssg: { ...base.ssg, sectionIndex: resolveSectionIndexOptions(false) },
      }),
      root,
    );

    expect(result.files.some((file) => file.endsWith(path.join("guide", "index.html")))).toBe(
      false,
    );
    await expect(fs.access(path.join(root, "dist", "guide", "index.html"))).rejects.toThrow();
    await expect(
      fs.access(path.join(root, "dist", "guide", "a", "index.html")),
    ).resolves.toBeUndefined();
  });

  it("generates /guide/index.html from child pages when no index.md exists", async () => {
    const root = await makeSite({
      "index.md": "# Home\n\nWelcome home.\n",
      "guide/a.md": "---\ntitle: Page A\n---\n\n# Page A\n",
      "guide/b.md": "---\ntitle: Page B\n---\n\n# Page B\n",
    });
    const base = createDocsResolvedOptions();
    const result = await buildSsg(
      createDocsResolvedOptions({
        ssg: {
          ...base.ssg,
          siteName: "Docs",
          sectionIndex: resolveSectionIndexOptions(true),
        },
      }),
      root,
    );

    const outputPath = path.join(root, "dist", "guide", "index.html");
    expect(result.files).toContain(outputPath);
    const html = await fs.readFile(outputPath, "utf8");
    expect(html).toContain("ox-section-index");
    expect(html).toContain("Page A");
    expect(html).toContain("Page B");
    expect(html).toContain('href="/guide/a/index.html"');
    expect(html).toContain('href="/guide/b/index.html"');
    expect(html).toContain("<title>");
    expect(html).toContain("Guide");
    expect(html).toContain('class="sidebar');
    expect(html).not.toContain("javascript:");
  });

  it("keeps an existing index.md and does not overwrite it", async () => {
    const root = await makeSite({
      "index.md": "# Home\n\nWelcome.\n",
      "guide/index.md":
        "---\ntitle: Real guide index\n---\n\n# Real guide index\n\nAuthored body.\n",
      "guide/a.md": "---\ntitle: Page A\n---\n\n# Page A\n",
      "guide/b.md": "---\ntitle: Page B\n---\n\n# Page B\n",
    });
    const base = createDocsResolvedOptions();
    await buildSsg(
      createDocsResolvedOptions({
        ssg: {
          ...base.ssg,
          siteName: "Docs",
          sectionIndex: resolveSectionIndexOptions(true),
        },
      }),
      root,
    );

    const html = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
    expect(html).toContain("Real guide index");
    expect(html).toContain("Authored body.");
    expect(html).not.toContain("ox-section-index");
  });

  it("escapes a hostile frontmatter title on a generated index", async () => {
    const root = await makeSite({
      "index.md": "# Home\n\nWelcome.\n",
      "guide/a.md": `---
title: "</title><script>alert(1)</script>"
---

# Safe heading

Body.
`,
      "guide/b.md": "---\ntitle: Page B\n---\n\n# Page B\n",
    });
    const base = createDocsResolvedOptions();
    await buildSsg(
      createDocsResolvedOptions({
        ssg: {
          ...base.ssg,
          siteName: "Docs",
          sectionIndex: resolveSectionIndexOptions(true),
        },
      }),
      root,
    );

    const html = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
    expect(html).not.toContain("<script>alert(1)</script>");
    expect(html).toMatch(/&#60;script&#62;|&lt;script&gt;/);
    expect(html).toMatch(/&#60;\/title&#62;|&lt;\/title&gt;/);
    expect(html).toContain("ox-section-index");
    expect(html).toContain("Page B");
  });

  it("omits draft and unlisted children from the generated listing", async () => {
    const root = await makeSite({
      "index.md": "# Home\n\nWelcome.\n",
      "guide/a.md": "---\ntitle: Visible A\n---\n\n# Visible A\n",
      "guide/secret.md": "---\ntitle: Draft secret\ndraft: true\n---\n\n# Draft secret\n",
      "guide/hidden.md": "---\ntitle: Unlisted hidden\nunlisted: true\n---\n\n# Unlisted hidden\n",
    });
    const base = createDocsResolvedOptions();
    await buildSsg(
      createDocsResolvedOptions({
        ssg: {
          ...base.ssg,
          siteName: "Docs",
          sectionIndex: resolveSectionIndexOptions({ style: "list" }),
        },
      }),
      root,
    );

    const html = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
    expect(html).toContain("ox-section-index--list");
    expect(html).toContain("Visible A");
    expect(html).not.toContain("Draft secret");
    expect(html).not.toContain("Unlisted hidden");
  });
});
