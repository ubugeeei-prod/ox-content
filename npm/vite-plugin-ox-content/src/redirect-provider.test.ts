import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { planRedirectFiles, resolveRedirectsOptions, writeRedirectFiles } from "./redirects";
import type { ResolvedRedirectsOptions } from "./types";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

const on: ResolvedRedirectsOptions = {
  enabled: true,
  map: {},
  headers: false,
  json: false,
  allowExternal: false,
};

const NETLIFY_REDIRECTS_BYTES = "/old /other 301\n";

describe("redirect provider detection", () => {
  it("detects Cloudflare Pages, Workers CI, and Netlify from injected env", () => {
    expect(resolveRedirectsOptions({ map: { "/old": "/new" } }, { CF_PAGES: "1" })).toEqual({
      enabled: true,
      map: { "/old": "/new" },
      provider: "cloudflare",
      headers: false,
      json: false,
      allowExternal: false,
    });
    expect(resolveRedirectsOptions(true, { WORKERS_CI: "1" }).provider).toBe("cloudflare");
    expect(resolveRedirectsOptions({ "/old": "/new" }, { NETLIFY: "true" }).provider).toBe(
      "netlify",
    );
    expect(resolveRedirectsOptions(true, { CF_PAGES: "1", WORKERS_CI: "1" }).provider).toBe(
      "cloudflare",
    );
  });

  it("does not treat nearby env values as a provider match", () => {
    expect(resolveRedirectsOptions(true, { CF_PAGES: "true" }).provider).toBeUndefined();
    expect(resolveRedirectsOptions(true, { WORKERS_CI: "true" }).provider).toBeUndefined();
    expect(resolveRedirectsOptions(true, { NETLIFY: "1" }).provider).toBeUndefined();
  });

  it("lets an explicit provider win over detected env", () => {
    expect(
      resolveRedirectsOptions({ provider: "netlify" }, { CF_PAGES: "1", WORKERS_CI: "1" }).provider,
    ).toBe("netlify");
    expect(resolveRedirectsOptions({ provider: "cloudflare" }, { NETLIFY: "true" }).provider).toBe(
      "cloudflare",
    );
  });

  it("warns and omits a provider when Cloudflare and Netlify env vars conflict", () => {
    const warnings: string[] = [];
    const original = console.warn;
    console.warn = (...args: unknown[]) => {
      warnings.push(args.map(String).join(" "));
    };
    try {
      expect(
        resolveRedirectsOptions({ map: { "/old": "/new" } }, { CF_PAGES: "1", NETLIFY: "true" }),
      ).toEqual({
        enabled: true,
        map: { "/old": "/new" },
        headers: false,
        json: false,
        allowExternal: false,
      });
      expect(resolveRedirectsOptions(true, { WORKERS_CI: "1", NETLIFY: "true" }).provider).toBe(
        undefined,
      );
    } finally {
      console.warn = original;
    }
    expect(warnings.join("\n")).toMatch(/conflict/i);
    expect(warnings.join("\n")).toMatch(/provider/i);
  });

  it("does not detect a provider when the feature is disabled", () => {
    expect(resolveRedirectsOptions(false, { NETLIFY: "true" }).provider).toBeUndefined();
    expect(resolveRedirectsOptions(undefined, { CF_PAGES: "1" }).provider).toBeUndefined();
  });
});

describe("redirect provider host files", () => {
  it("writes the same _redirects body for explicit netlify and cloudflare", () => {
    const map = { "/old": "/other" };
    const netlify = planRedirectFiles({
      options: { ...on, map, provider: "netlify" },
      pages: [],
    });
    const cloudflare = planRedirectFiles({
      options: { ...on, map, provider: "cloudflare" },
      pages: [],
    });

    expect(netlify.netlify).toBe(NETLIFY_REDIRECTS_BYTES);
    expect(cloudflare.netlify).toBe(netlify.netlify);
  });

  it("omits the host _redirects body when no provider is resolved", () => {
    const plan = planRedirectFiles({
      options: { ...on, map: { "/old": "/guide" } },
      pages: [],
    });

    expect(plan.files).toHaveLength(1);
    expect(plan.netlify).toBeUndefined();
  });

  it("skips wildcard HTML pages for an explicit cloudflare provider", () => {
    const plan = planRedirectFiles({
      options: {
        ...on,
        map: { "/talks*": "/works/talks", "/old": "/guide" },
        provider: "cloudflare",
      },
      pages: [],
    });

    expect(plan.files).toHaveLength(1);
    expect(plan.files[0]?.from).toBe("/old");
    expect(plan.netlify).toBe("/talks* /works/talks 301\n/old /guide 301\n");
  });

  it("writes the same _redirects bytes for an explicit cloudflare provider", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-redirects-cf-"));
    tempDirs.push(outDir);

    await writeRedirectFiles({
      outDir,
      options: {
        ...on,
        map: { "/old-guide": "/guide" },
        provider: "cloudflare",
      },
      pages: [{ dest: "/guide", aliases: ["/old"] }],
    });

    expect(await fs.readFile(path.join(outDir, "_redirects"), "utf8")).toBe(
      "/old /guide 301\n/old-guide /guide 301\n",
    );
  });

  it("does not write _redirects without a provider", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-redirects-nop-"));
    tempDirs.push(outDir);

    await writeRedirectFiles({
      outDir,
      options: { ...on, map: { "/old": "/guide" } },
      pages: [],
    });

    expect(await fs.readFile(path.join(outDir, "old", "index.html"), "utf8")).toContain(
      "url=/guide",
    );
    await expect(fs.access(path.join(outDir, "_redirects"))).rejects.toThrow();
  });
});
