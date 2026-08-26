import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { planRedirectFiles, resolveRedirectsOptions, writeRedirectFiles } from "./redirects";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

const off = {
  enabled: false,
  map: {},
  netlify: false,
  headers: false,
  json: false,
  allowExternal: false,
};

const on = {
  enabled: true,
  map: {},
  netlify: false,
  headers: false,
  json: false,
  allowExternal: false,
};

describe("resolveRedirectsOptions", () => {
  it("disables the feature when omitted or false", () => {
    expect(resolveRedirectsOptions(undefined)).toEqual(off);
    expect(resolveRedirectsOptions(false)).toEqual(off);
  });

  it("enables defaults when true", () => {
    expect(resolveRedirectsOptions(true)).toEqual(on);
  });

  it("enables from an object and overrides only set fields", () => {
    expect(resolveRedirectsOptions({})).toEqual(on);
    expect(
      resolveRedirectsOptions({
        map: { "/old": "/guide" },
        netlify: true,
        headers: true,
        json: true,
        allowExternal: true,
      }),
    ).toEqual({
      enabled: true,
      map: { "/old": "/guide" },
      netlify: true,
      headers: true,
      json: true,
      allowExternal: true,
    });
  });

  it("treats a path map as enabled defaults plus that map", () => {
    expect(resolveRedirectsOptions({ "/old-guide": "/guide" })).toEqual({
      ...on,
      map: { "/old-guide": "/guide" },
    });
  });
});

describe("planRedirectFiles", () => {
  it("plans nothing when the feature is omitted or disabled", () => {
    expect(planRedirectFiles({ pages: [{ dest: "/guide", aliases: ["/old"] }] })).toEqual({
      files: [],
    });
    expect(
      planRedirectFiles({
        options: { ...off, map: { "/old": "/guide" } },
        pages: [{ dest: "/guide", aliases: ["/old"] }],
      }),
    ).toEqual({ files: [] });
  });

  it("plans frontmatter alias HTML at the old pretty path", () => {
    const plan = planRedirectFiles({
      options: on,
      pages: [{ dest: "/guide", aliases: ["/old", "/legacy"], redirect: "/retired" }],
    });

    expect(plan.files.map((file) => file.relativePath)).toEqual([
      "old/index.html",
      "legacy/index.html",
      "retired/index.html",
    ]);
    expect(plan.files[0]).toMatchObject({ from: "/old", to: "/guide" });
    expect(plan.files[0]?.html).toContain("url=/guide");
    expect(plan.files[0]?.html).toContain('<link rel="canonical" href="/guide">');
  });

  it("plans config-map redirects and last-wins after slash folding", () => {
    const plan = planRedirectFiles({
      options: { ...on, map: { "/old/": "/guide/", "/old": "/other" }, netlify: true },
      pages: [],
    });

    expect(plan.files).toHaveLength(1);
    expect(plan.files[0]).toMatchObject({
      from: "/old",
      to: "/other",
      relativePath: "old/index.html",
    });
    expect(plan.netlify).toBe("/old /other 301\n");
  });

  it("does not treat fenced or non-list aliases as input", () => {
    const plan = planRedirectFiles({
      options: on,
      pages: [
        { dest: "/guide", aliases: { fence: "```\naliases:\n  - /evil\n```" } },
        { dest: "/other", aliases: 12 },
      ],
    });

    expect(plan.files).toEqual([]);
  });

  it("rejects control characters and path traversal in sources and dests", () => {
    const plan = planRedirectFiles({
      options: {
        ...on,
        json: true,
        map: {
          "/../outside": "/guide",
          "/..\\outside": "/guide",
          "/old": "/guide\tv2",
        },
      },
      pages: [],
    });

    expect(plan.files).toEqual([]);
    expect(plan.json).toBeUndefined();
  });

  it("ignores open-redirect destinations unless allowExternal is set", () => {
    const blocked = planRedirectFiles({
      options: {
        ...on,
        map: {
          "/js": "javascript:alert(1)",
          "/proto": "//evil.example",
          "/abs": "https://evil.example",
        },
      },
      pages: [{ dest: "javascript:alert(1)", aliases: ["/legacy"] }],
    });
    expect(blocked.files).toEqual([]);

    const allowed = planRedirectFiles({
      options: {
        ...on,
        allowExternal: true,
        map: { "/docs": "https://example.com/docs", "/js": "javascript:alert(1)" },
      },
      pages: [],
    });
    expect(allowed.files).toHaveLength(1);
    expect(allowed.files[0]).toMatchObject({ from: "/docs", to: "https://example.com/docs" });
  });

  it("escapes a weird but allowed dest in the file plan HTML", () => {
    const plan = planRedirectFiles({
      options: { ...on, map: { "/old": `/foo<bar>&"'` } },
      pages: [],
    });

    expect(plan.files[0]?.html).toContain("url=/foo&lt;bar&gt;&amp;&quot;&#39;");
    expect(plan.files[0]?.html).not.toContain("url=/foo<bar>");
    expect(plan.files[0]?.html).not.toContain("<script>");
  });

  it("prefixes same-origin dests with the site base", () => {
    const plan = planRedirectFiles({
      options: { ...on, map: { "/old": "/guide" } },
      base: "/ox-content/",
      pages: [],
    });

    expect(plan.files[0]?.html).toContain("url=/ox-content/guide");
    expect(plan.files[0]?.to).toBe("/guide");
  });

  it("skips sources that would overwrite a real page", () => {
    const plan = planRedirectFiles({
      options: { ...on, map: { "/guide": "/elsewhere" } },
      pages: [{ dest: "/guide", aliases: ["/guide"] }],
    });

    expect(plan.files).toEqual([]);
  });

  it("keeps wildcard sources in host files but omits HTML pages", () => {
    const plan = planRedirectFiles({
      options: {
        ...on,
        map: {
          "/talks*": "/works/talks",
          "/projects*": "/works",
          "/old": "/guide",
        },
        netlify: true,
        headers: true,
        json: true,
      },
      pages: [],
    });

    expect(plan.files).toHaveLength(1);
    expect(plan.files[0]).toMatchObject({
      from: "/old",
      to: "/guide",
      relativePath: "old/index.html",
    });
    expect(
      plan.files.some((file) => file.from.includes("*") || file.relativePath.includes("*")),
    ).toBe(false);
    expect(plan.netlify).toBe("/talks* /works/talks 301\n/projects* /works 301\n/old /guide 301\n");
    expect(plan.headers).toBe(
      "/talks*\n  Location: /works/talks\n/projects*\n  Location: /works\n/old\n  Location: /guide\n",
    );
    expect(plan.json).toBe(
      '[{"from":"/talks*","to":"/works/talks"},{"from":"/projects*","to":"/works"},{"from":"/old","to":"/guide"}]',
    );
  });
});

describe("writeRedirectFiles", () => {
  it("writes planned HTML and optional host files", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-redirects-"));
    tempDirs.push(outDir);

    const result = await writeRedirectFiles({
      outDir,
      options: {
        ...on,
        map: { "/old-guide": "/guide" },
        netlify: true,
        headers: true,
        json: true,
      },
      pages: [{ dest: "/guide", aliases: ["/old"] }],
    });

    const oldHtml = await fs.readFile(path.join(outDir, "old", "index.html"), "utf8");
    const mapHtml = await fs.readFile(path.join(outDir, "old-guide", "index.html"), "utf8");

    expect(result.files).toHaveLength(5);
    expect(oldHtml).toContain("url=/guide");
    expect(mapHtml).toContain("url=/guide");
    expect(await fs.readFile(path.join(outDir, "_redirects"), "utf8")).toContain("/old /guide 301");
    expect(await fs.readFile(path.join(outDir, "_headers"), "utf8")).toContain("Location: /guide");
    expect(await fs.readFile(path.join(outDir, "redirects.json"), "utf8")).toContain('"/old"');
  });

  it("does not overwrite an existing page file", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-redirects-keep-"));
    tempDirs.push(outDir);
    const existing = path.join(outDir, "old", "index.html");
    await fs.mkdir(path.dirname(existing), { recursive: true });
    await fs.writeFile(existing, "real page", "utf8");

    await writeRedirectFiles({
      outDir,
      options: on,
      pages: [{ dest: "/guide", aliases: ["/old"] }],
    });

    expect(await fs.readFile(existing, "utf8")).toBe("real page");
  });

  it("does not write outside outDir for traversal sources", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-redirects-trav-"));
    tempDirs.push(outDir);

    const result = await writeRedirectFiles({
      outDir,
      options: {
        ...on,
        map: { "/../outside": "/guide", "/..\\outside": "/guide" },
      },
      pages: [],
    });

    expect(result.files).toEqual([]);
    await expect(fs.access(path.join(outDir, "..", "outside", "index.html"))).rejects.toThrow();
  });

  it("writes nothing when disabled", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-redirects-off-"));
    tempDirs.push(outDir);

    const result = await writeRedirectFiles({
      outDir,
      pages: [{ dest: "/guide", aliases: ["/old"] }],
    });

    expect(result.files).toEqual([]);
    await expect(fs.access(path.join(outDir, "old", "index.html"))).rejects.toThrow();
  });

  it("does not write literal HTML directories for wildcard sources", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-redirects-wild-"));
    tempDirs.push(outDir);

    const result = await writeRedirectFiles({
      outDir,
      options: {
        ...on,
        map: { "/talks*": "/works/talks", "/old": "/guide" },
        netlify: true,
      },
      pages: [],
    });

    expect(await fs.readFile(path.join(outDir, "_redirects"), "utf8")).toBe(
      "/talks* /works/talks 301\n/old /guide 301\n",
    );
    expect(await fs.readFile(path.join(outDir, "old", "index.html"), "utf8")).toContain(
      "url=/guide",
    );
    await expect(fs.access(path.join(outDir, "talks*", "index.html"))).rejects.toThrow();
    expect(result.files.some((file) => file.includes("talks*"))).toBe(false);
  });
});
