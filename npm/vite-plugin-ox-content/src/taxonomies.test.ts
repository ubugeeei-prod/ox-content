import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolvePublishStateOptions } from "./publish-state";
import { buildSsg } from "./ssg";
import { resolveTaxonomiesOptions, termSlug } from "./taxonomies";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-taxonomies-"));
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

function enabledOptions(taxonomies: ReturnType<typeof resolveTaxonomiesOptions>) {
  return createDocsResolvedOptions({ taxonomies, ssg: { ...createDocsResolvedOptions().ssg } });
}

const taggedSite = {
  "guide.md":
    "---\ntitle: Guide\ntags:\n  - rust\ncategories: docs\n---\n\n# Guide\nShared rust.\n",
  "install.md": "---\ntitle: Install\ntags: rust\n---\n\n# Install\nAlso rust.\n",
  "plain.md": "---\ntitle: Plain\n---\n\n# Plain\nNo terms.\n",
};

describe("resolveTaxonomiesOptions", () => {
  it("disables the feature when omitted or false", () => {
    expect(resolveTaxonomiesOptions(undefined)).toEqual({
      enabled: false,
      taxonomies: ["tags", "categories"],
      relatedLimit: 5,
    });
    expect(resolveTaxonomiesOptions(false)).toEqual({
      enabled: false,
      taxonomies: ["tags", "categories"],
      relatedLimit: 5,
    });
    expect(createDocsResolvedOptions().taxonomies).toBeUndefined();
  });

  it("enables defaults when true, and overrides only set fields", () => {
    expect(resolveTaxonomiesOptions(true)).toEqual({
      enabled: true,
      taxonomies: ["tags", "categories"],
      relatedLimit: 5,
    });
    expect(resolveTaxonomiesOptions({ relatedLimit: 2 })).toEqual({
      enabled: true,
      taxonomies: ["tags", "categories"],
      relatedLimit: 2,
    });
    expect(resolveTaxonomiesOptions({ taxonomies: ["topics"] })).toEqual({
      enabled: true,
      taxonomies: ["topics"],
      relatedLimit: 5,
    });
    expect(resolveTaxonomiesOptions({})).toEqual({
      enabled: true,
      taxonomies: ["tags", "categories"],
      relatedLimit: 5,
    });
  });
});

describe("termSlug", () => {
  it("keeps slugs stable, lowercase, and [a-z0-9-]", () => {
    expect(termSlug("Rust")).toBe("rust");
    expect(termSlug("Hello World")).toBe("hello-world");
    expect(termSlug("Rust")).toBe(termSlug("rust"));
    expect(termSlug("Hello World")).toMatch(/^[a-z0-9-]+$/);
  });

  it("drops path and javascript hrefs and never emits a hostile slug", () => {
    expect(termSlug("javascript:alert(1)")).toBeUndefined();
    expect(termSlug("../secret")).toBeUndefined();
    expect(termSlug("//evil.com")).toBeUndefined();
    expect(termSlug("<script>xss</script>")).toBe("script-xss-script");
    expect(termSlug("<script>xss</script>")).toMatch(/^[a-z0-9-]+$/);
  });
});

describe("buildSsg taxonomies", () => {
  it("writes no term files or ox-related markup when omitted or false", async () => {
    const omittedRoot = await makeSite(taggedSite);
    const disabledRoot = await makeSite(taggedSite);
    await buildSsg(createDocsResolvedOptions(), omittedRoot);
    await buildSsg(enabledOptions(resolveTaxonomiesOptions(false)), disabledRoot);

    for (const root of [omittedRoot, disabledRoot]) {
      const guide = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
      expect(guide).not.toContain("ox-related");
      await expect(fs.access(path.join(root, "dist", "tags", "index.html"))).rejects.toThrow();
      await expect(
        fs.access(path.join(root, "dist", "categories", "index.html")),
      ).rejects.toThrow();
    }
  });

  it("writes term lists, per-term pages, and related markup from string and string[]", async () => {
    const root = await makeSite(taggedSite);
    const result = await buildSsg(enabledOptions(resolveTaxonomiesOptions(true)), root);

    expect(result.files).toContain(path.join(root, "dist", "tags", "index.html"));
    expect(result.files).toContain(path.join(root, "dist", "tags", "rust", "index.html"));
    expect(result.files).toContain(path.join(root, "dist", "categories", "index.html"));
    expect(result.files).toContain(path.join(root, "dist", "categories", "docs", "index.html"));

    const tags = await fs.readFile(path.join(root, "dist", "tags", "index.html"), "utf8");
    const rust = await fs.readFile(path.join(root, "dist", "tags", "rust", "index.html"), "utf8");
    const guide = await fs.readFile(path.join(root, "dist", "guide", "index.html"), "utf8");
    const plain = await fs.readFile(path.join(root, "dist", "plain", "index.html"), "utf8");

    expect(tags).toContain("rust");
    expect(rust).toContain("Guide");
    expect(rust).toContain("Install");
    expect(guide).toContain("ox-related");
    expect(guide).toContain("Install");
    expect(plain).not.toContain("ox-related");
  });

  it("honors object overrides for taxonomy keys and relatedLimit", async () => {
    const root = await makeSite({
      "one.md": "---\ntitle: One\ntopics:\n  - shared\n---\n\n# One\n",
      "two.md": "---\ntitle: Two\ntopics:\n  - shared\n---\n\n# Two\n",
      "three.md": "---\ntitle: Three\ntopics:\n  - shared\n---\n\n# Three\n",
    });
    await buildSsg(
      enabledOptions(resolveTaxonomiesOptions({ taxonomies: ["topics"], relatedLimit: 1 })),
      root,
    );

    await expect(
      fs.access(path.join(root, "dist", "topics", "index.html")),
    ).resolves.toBeUndefined();
    await expect(
      fs.access(path.join(root, "dist", "topics", "shared", "index.html")),
    ).resolves.toBeUndefined();
    await expect(fs.access(path.join(root, "dist", "tags", "index.html"))).rejects.toThrow();

    const one = await fs.readFile(path.join(root, "dist", "one", "index.html"), "utf8");
    const related = one.slice(one.indexOf("ox-related"));
    const relatedTitles = ["Two", "Three"].filter((title) => related.includes(title));
    expect(relatedTitles).toHaveLength(1);
  });

  it("reads frontmatter only and ignores tags mentioned in fences or inline code", async () => {
    const root = await makeSite({
      "real.md": "---\ntitle: Real\ntags:\n  - napi\n---\n\n# Real\n",
      "fence.md": `---
title: Fence
---

# Fence

\`\`\`yaml
tags: rust
\`\`\`

Use \`categories: guide\` in frontmatter.
`,
    });
    await buildSsg(enabledOptions(resolveTaxonomiesOptions(true)), root);

    await expect(
      fs.access(path.join(root, "dist", "tags", "napi", "index.html")),
    ).resolves.toBeUndefined();
    await expect(
      fs.access(path.join(root, "dist", "tags", "rust", "index.html")),
    ).rejects.toThrow();
    await expect(
      fs.access(path.join(root, "dist", "categories", "guide", "index.html")),
    ).rejects.toThrow();
  });

  it("escapes hostile titles and terms and drops unsafe hrefs", async () => {
    const root = await makeSite({
      "safe.md": "---\ntitle: Safe\ntags:\n  - ok\n---\n\n# Safe\n",
      "hostile.md": `---
title: "</title><script>alert(1)</script>"
tags:
  - "<script>xss</script>"
  - "javascript:alert(1)"
  - "../secret"
  - "//evil.com"
  - ok
---

# Hostile
`,
    });
    await buildSsg(enabledOptions(resolveTaxonomiesOptions(true)), root);

    const tags = await fs.readFile(path.join(root, "dist", "tags", "index.html"), "utf8");
    const term = await fs.readFile(
      path.join(root, "dist", "tags", "script-xss-script", "index.html"),
      "utf8",
    );
    const safe = await fs.readFile(path.join(root, "dist", "safe", "index.html"), "utf8");

    for (const html of [tags, term, safe]) {
      expect(html).not.toContain("<script>alert(1)</script>");
      expect(html).not.toContain('javascript:alert(1)"');
      expect(html).not.toContain('href="../');
      expect(html).not.toContain('href="//evil.com');
      expect(html).not.toContain("</title><script>");
    }
    expect(term).toContain("&lt;script&gt;");
    await expect(
      fs.access(path.join(root, "dist", "tags", "secret", "index.html")),
    ).rejects.toThrow();
  });

  it("omits draft and unlisted pages from term pages and related lists when publishState is on", async () => {
    const files = {
      "public.md": "---\ntitle: Public\ntags:\n  - rust\n---\n\n# Public\n",
      "wip.md": "---\ntitle: Draft\ndraft: true\ntags:\n  - rust\n---\n\n# Draft\n",
      "hidden.md": "---\ntitle: Hidden\nunlisted: true\ntags:\n  - rust\n---\n\n# Hidden\n",
      "later.md":
        "---\ntitle: Later\nscheduled: 2099-01-01T00:00:00Z\ntags:\n  - rust\n---\n\n# Later\n",
    };
    const root = await makeSite(files);
    await buildSsg(
      createDocsResolvedOptions({
        taxonomies: resolveTaxonomiesOptions(true),
        publishState: resolvePublishStateOptions({ now: "2026-08-24T00:00:00Z" }),
      }),
      root,
    );

    const rust = await fs.readFile(path.join(root, "dist", "tags", "rust", "index.html"), "utf8");
    expect(rust).toContain("Public");
    expect(rust).not.toContain("Draft");
    expect(rust).not.toContain("Hidden");
    expect(rust).not.toContain("Later");

    const published = await fs.readFile(path.join(root, "dist", "public", "index.html"), "utf8");
    expect(published).not.toContain("Draft");
    expect(published).not.toContain("Hidden");
    expect(published).not.toContain("Later");
    await expect(fs.access(path.join(root, "dist", "wip", "index.html"))).rejects.toThrow();
  });

  it("maps the same term to the same path", async () => {
    const root = await makeSite({
      "a.md": "---\ntitle: Alpha\ntags:\n  - Rust\n---\n\n# Alpha\n",
      "b.md": "---\ntitle: Beta\ntags: rust\n---\n\n# Beta\n",
    });
    const result = await buildSsg(enabledOptions(resolveTaxonomiesOptions(true)), root);

    const rustPath = path.join(root, "dist", "tags", "rust", "index.html");
    const rust = await fs.readFile(rustPath, "utf8");
    expect(result.files).toContain(rustPath);
    expect(
      result.files.some((file) => file.includes(`${path.sep}tags${path.sep}Rust${path.sep}`)),
    ).toBe(false);
    expect(rust).toContain("Alpha");
    expect(rust).toContain("Beta");
    const tags = await fs.readFile(path.join(root, "dist", "tags", "index.html"), "utf8");
    expect(tags).toContain("/tags/rust/");
    expect(tags).not.toContain("/tags/Rust/");
  });
});
