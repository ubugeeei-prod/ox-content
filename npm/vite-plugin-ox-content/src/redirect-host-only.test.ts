import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { planRedirectFiles, writeRedirectFiles } from "./redirects";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

const on = {
  enabled: true,
  map: {},
  headers: false,
  json: false,
  html: true,
  allowExternal: false,
};

describe("host-only redirects", () => {
  it("preserves cloudflare host output without ordinary HTML fallbacks", () => {
    const plan = planRedirectFiles({
      options: {
        ...on,
        allowExternal: true,
        html: false,
        map: {
          "/external": "https://example.com/docs",
          "/guide": "/elsewhere",
        },
        provider: "cloudflare",
      },
      pages: [{ dest: "/guide", aliases: ["/old"], redirect: "/retired" }],
    });

    expect(plan.files).toEqual([]);
    expect(plan.netlify).toBe(
      "/old /guide 301\n/retired /guide 301\n/external https://example.com/docs 301\n",
    );
  });

  it("writes netlify host output without HTML fallback files", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-redirects-host-only-"));
    tempDirs.push(outDir);

    const result = await writeRedirectFiles({
      outDir,
      options: {
        ...on,
        html: false,
        map: { "/old-guide": "/guide" },
        provider: "netlify",
      },
      pages: [{ dest: "/guide", aliases: ["/old"] }],
    });

    expect(await fs.readFile(path.join(outDir, "_redirects"), "utf8")).toBe(
      "/old /guide 301\n/old-guide /guide 301\n",
    );
    await expect(fs.access(path.join(outDir, "old", "index.html"))).rejects.toThrow();
    await expect(fs.access(path.join(outDir, "old-guide", "index.html"))).rejects.toThrow();
    expect(result.files).toEqual([path.join(outDir, "_redirects")]);
  });
});
