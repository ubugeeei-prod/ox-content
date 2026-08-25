import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolveBlogOptions } from "./blog";
import { buildSsg } from "./ssg";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-blog-"));
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

function enabledOptions(blog: ReturnType<typeof resolveBlogOptions>) {
  return createDocsResolvedOptions({ blog, ssg: { ...createDocsResolvedOptions().ssg } });
}

describe("buildSsg blog", () => {
  it("writes no blog files or ox-blog-meta when omitted or false", async () => {
    const files = {
      "hello.md": "---\ntitle: Hello\ndate: 2024-01-15\ntags: rust\n---\n\n# Hello\nPost.\n",
    };
    const omittedRoot = await makeSite(files);
    const disabledRoot = await makeSite(files);
    await buildSsg(createDocsResolvedOptions(), omittedRoot);
    await buildSsg(enabledOptions(resolveBlogOptions(false)), disabledRoot);

    for (const root of [omittedRoot, disabledRoot]) {
      const hello = await fs.readFile(path.join(root, "dist", "hello", "index.html"), "utf8");
      expect(hello).not.toContain("ox-blog-meta");
      await expect(fs.access(path.join(root, "dist", "blog", "index.html"))).rejects.toThrow();
      await expect(fs.access(path.join(root, "dist", "blog", "tags"))).rejects.toThrow();
      await expect(fs.access(path.join(root, "dist", "blog", "archive"))).rejects.toThrow();
    }
  });

  it("writes a paginated index, tag page, archive, and reading time", async () => {
    const root = await makeSite({
      "alpha.md": `---
title: Alpha Post
date: 2024-01-15
author: ada
tags:
  - rust
---

# Alpha Post

hello world
`,
      "beta.md": `---
title: Beta Post
date: 2024-03-01
authors:
  - ada
tags: rust
---

# Beta Post

more words here
`,
      "wip.md": `---
title: Draft Post
draft: true
date: 2024-06-01
tags: rust
---

# Draft Post

secret
`,
      "hidden.md": `---
title: Hidden Post
unlisted: true
date: 2024-07-01
tags: rust
---

# Hidden Post

quiet
`,
    });
    const result = await buildSsg(
      enabledOptions(
        resolveBlogOptions({
          pageSize: 1,
          authors: {
            ada: {
              name: "Ada Lovelace",
              bio: "Mathematician",
              url: "https://example.com/ada",
            },
          },
        }),
      ),
      root,
    );

    expect(result.files).toContain(path.join(root, "dist", "blog", "index.html"));
    expect(result.files).toContain(path.join(root, "dist", "blog", "page", "2", "index.html"));
    expect(result.files).toContain(path.join(root, "dist", "blog", "tags", "rust", "index.html"));
    expect(result.files).toContain(path.join(root, "dist", "blog", "archive", "index.html"));
    expect(result.files).toContain(
      path.join(root, "dist", "blog", "archive", "2024", "index.html"),
    );
    expect(result.files).toContain(
      path.join(root, "dist", "blog", "archive", "2024", "01", "index.html"),
    );
    expect(result.files).toContain(
      path.join(root, "dist", "blog", "archive", "2024", "03", "index.html"),
    );

    const index = await fs.readFile(path.join(root, "dist", "blog", "index.html"), "utf8");
    const page2 = await fs.readFile(
      path.join(root, "dist", "blog", "page", "2", "index.html"),
      "utf8",
    );
    const rust = await fs.readFile(
      path.join(root, "dist", "blog", "tags", "rust", "index.html"),
      "utf8",
    );
    const archive = await fs.readFile(
      path.join(root, "dist", "blog", "archive", "index.html"),
      "utf8",
    );
    const year = await fs.readFile(
      path.join(root, "dist", "blog", "archive", "2024", "index.html"),
      "utf8",
    );
    const january = await fs.readFile(
      path.join(root, "dist", "blog", "archive", "2024", "01", "index.html"),
      "utf8",
    );
    const march = await fs.readFile(
      path.join(root, "dist", "blog", "archive", "2024", "03", "index.html"),
      "utf8",
    );
    const alpha = await fs.readFile(path.join(root, "dist", "alpha", "index.html"), "utf8");

    expect(index).toContain("Beta Post");
    expect(index).not.toContain("Alpha Post");
    expect(index).not.toContain("Draft Post");
    expect(index).not.toContain("Hidden Post");
    expect(index).toContain("/blog/page/2/");
    expect(page2).toContain("Alpha Post");
    expect(page2).not.toContain("Beta Post");
    expect(rust).toContain("Alpha Post");
    expect(rust).toContain("Beta Post");
    expect(rust).not.toContain("Draft Post");
    expect(rust).not.toContain("Hidden Post");
    expect(archive).toContain("/blog/archive/2024/");
    expect(year).toContain("/blog/archive/2024/01/");
    expect(year).toContain("/blog/archive/2024/03/");
    expect(january).toContain("Alpha Post");
    expect(january).not.toContain("Beta Post");
    expect(march).toContain("Beta Post");
    expect(alpha).toContain("ox-blog-meta");
    expect(alpha).toContain("Ada Lovelace");
    expect(alpha).toContain("Mathematician");
    expect(alpha).toContain('href="https://example.com/ada"');
    expect(alpha).toMatch(/1 min read/);
  });

  it("escapes hostile author, bio, and tag values and drops unsafe hrefs", async () => {
    const root = await makeSite({
      "safe.md": "---\ntitle: Safe\ndate: 2024-01-01\ntags:\n  - ok\n---\n\n# Safe\n",
      "hostile.md": `---
title: "</title><script>alert(1)</script>"
date: 2024-02-01
author: evil
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
    await buildSsg(
      enabledOptions(
        resolveBlogOptions({
          authors: {
            evil: {
              name: "<img src=x onerror=alert(1)>",
              bio: "<script>alert(1)</script>",
              url: "javascript:alert(1)",
            },
          },
        }),
      ),
      root,
    );

    const hostile = await fs.readFile(path.join(root, "dist", "hostile", "index.html"), "utf8");
    const rust = await fs.readFile(
      path.join(root, "dist", "blog", "tags", "script-xss-script", "index.html"),
      "utf8",
    );
    const index = await fs.readFile(path.join(root, "dist", "blog", "index.html"), "utf8");

    for (const html of [hostile, rust, index]) {
      expect(html).not.toContain("<script>alert(1)</script>");
      expect(html).not.toContain("<img src=x");
      expect(html).not.toContain("javascript:");
      expect(html).not.toContain('href="../');
      expect(html).not.toContain('href="//evil.com');
      expect(html).not.toContain("</title><script>");
    }
    expect(hostile).toContain("&lt;img src=x onerror=alert(1)&gt;");
    expect(hostile).toContain("&lt;script&gt;alert(1)&lt;/script&gt;");
    expect(rust).toContain("&lt;script&gt;");
    await expect(
      fs.access(path.join(root, "dist", "blog", "tags", "secret", "index.html")),
    ).rejects.toThrow();
  });

  it("reads frontmatter tags only and ignores fences or inline code", async () => {
    const root = await makeSite({
      "real.md": "---\ntitle: Real\ndate: 2024-01-01\ntags:\n  - napi\n---\n\n# Real\n",
      "fence.md": `---
title: Fence
date: 2024-02-01
---

# Fence

\`\`\`yaml
tags: rust
\`\`\`

Use \`tags: rust\` in frontmatter.
`,
    });
    await buildSsg(enabledOptions(resolveBlogOptions(true)), root);

    await expect(
      fs.access(path.join(root, "dist", "blog", "tags", "napi", "index.html")),
    ).resolves.toBeUndefined();
    await expect(
      fs.access(path.join(root, "dist", "blog", "tags", "rust", "index.html")),
    ).rejects.toThrow();
  });

  it("emits nothing extra when several collections exist and none is blog", async () => {
    const root = await makeSite({
      "docs/guide.md": "---\ntitle: Guide\ndate: 2024-01-01\ntags: rust\n---\n\n# Guide\n",
      "notes/memo.md": "---\ntitle: Memo\ndate: 2024-02-01\ntags: rust\n---\n\n# Memo\n",
    });
    await buildSsg(
      createDocsResolvedOptions({
        blog: resolveBlogOptions(true),
        collections: {
          enabled: true,
          collections: {
            docs: { name: "docs", source: ["docs/**/*.md"], include: [] },
            notes: { name: "notes", source: ["notes/**/*.md"], include: [] },
          },
        },
      }),
      root,
    );

    await expect(fs.access(path.join(root, "dist", "blog", "index.html"))).rejects.toThrow();
    const guide = await fs.readFile(path.join(root, "dist", "docs", "guide", "index.html"), "utf8");
    expect(guide).not.toContain("ox-blog-meta");
  });

  it("honors ssg.blog when the top-level option is omitted", async () => {
    const root = await makeSite({
      "hello.md": "---\ntitle: Hello\ndate: 2024-01-15\ntags: rust\n---\n\n# Hello\nPost.\n",
    });
    const base = createDocsResolvedOptions();
    await buildSsg(
      createDocsResolvedOptions({
        ssg: { ...base.ssg, blog: resolveBlogOptions(true) },
      }),
      root,
    );

    await expect(fs.access(path.join(root, "dist", "blog", "index.html"))).resolves.toBeUndefined();
    const hello = await fs.readFile(path.join(root, "dist", "hello", "index.html"), "utf8");
    expect(hello).toContain("ox-blog-meta");
  });
});
