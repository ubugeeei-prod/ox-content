import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { planRedirectFiles, resolveRedirectsOptions, writeRedirectFiles } from "./redirects";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("resolveRedirectsOptions", () => {
  it("omitted => false", () => {
    expect(resolveRedirectsOptions(undefined)).toEqual({
      enabled: false,
      map: {},
      netlify: false,
    });
    expect(resolveRedirectsOptions(false).enabled).toBe(false);
  });

  it("true => true", () => {
    expect(resolveRedirectsOptions(true)).toEqual({
      enabled: true,
      map: {},
      netlify: false,
    });
  });

  it("{} => true", () => {
    expect(resolveRedirectsOptions({})).toEqual({
      enabled: true,
      map: {},
      netlify: false,
    });
  });

  it("treats a path map as enabled defaults plus that map", () => {
    expect(resolveRedirectsOptions({ "/old-guide": "/guide" })).toEqual({
      enabled: true,
      map: { "/old-guide": "/guide" },
      netlify: false,
    });
  });

  it("enables from an options object and honors netlify aliases", () => {
    expect(
      resolveRedirectsOptions({
        enabled: true,
        map: { "/old": "/guide" },
        writeNetlify: true,
      }),
    ).toEqual({
      enabled: true,
      map: { "/old": "/guide" },
      netlify: true,
    });
    expect(resolveRedirectsOptions({ netlify: true }).netlify).toBe(true);
  });
});

describe("planRedirectFiles", () => {
  it("plans nothing when the feature is omitted or disabled", () => {
    expect(planRedirectFiles({ pages: [{ dest: "/guide", aliases: ["/old"] }] })).toEqual({
      files: [],
    });
    expect(
      planRedirectFiles({
        options: { enabled: false, map: { "/old": "/guide" }, netlify: false },
        pages: [{ dest: "/guide", aliases: ["/old"] }],
      }),
    ).toEqual({ files: [] });
  });

  it("plans frontmatter alias HTML at the old pretty path", () => {
    const plan = planRedirectFiles({
      options: { enabled: true, map: {}, netlify: false },
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
      options: {
        enabled: true,
        map: { "/old/": "/guide/", "/old": "/other" },
        netlify: true,
      },
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

  it("ignores open-redirect destinations", () => {
    const plan = planRedirectFiles({
      options: {
        enabled: true,
        map: {
          "/js": "javascript:alert(1)",
          "/proto": "//evil.example",
          "/abs": "https://evil.example",
        },
        netlify: false,
      },
      pages: [{ dest: "javascript:alert(1)", aliases: ["/legacy"] }],
    });

    expect(plan.files).toEqual([]);
  });

  it("escapes a weird but allowed dest in the file plan HTML", () => {
    const plan = planRedirectFiles({
      options: { enabled: true, map: { "/old": "/foo<bar>" }, netlify: false },
      pages: [],
    });

    expect(plan.files[0]?.html).toContain("url=/foo&lt;bar&gt;");
    expect(plan.files[0]?.html).not.toContain("url=/foo<bar>");
  });
});

describe("writeRedirectFiles", () => {
  it("writes planned HTML and an optional host _redirects file", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-redirects-"));
    tempDirs.push(outDir);

    const result = await writeRedirectFiles({
      outDir,
      options: {
        enabled: true,
        map: { "/old-guide": "/guide" },
        netlify: true,
      },
      pages: [{ dest: "/guide", aliases: ["/old"] }],
    });

    const oldHtml = await fs.readFile(path.join(outDir, "old", "index.html"), "utf8");
    const mapHtml = await fs.readFile(path.join(outDir, "old-guide", "index.html"), "utf8");
    const netlify = await fs.readFile(path.join(outDir, "_redirects"), "utf8");

    expect(result.files).toHaveLength(3);
    expect(oldHtml).toContain("url=/guide");
    expect(mapHtml).toContain("url=/guide");
    expect(netlify).toContain("/old /guide 301");
    expect(netlify).toContain("/old-guide /guide 301");
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
});
