import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import {
  planRedirectOutputs,
  resolveRedirectsOptions,
  writeRedirectOutputs,
  type RedirectsOptions,
} from ".";
import { planRedirectFiles } from "./redirects";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("custom-host redirect outputs", () => {
  it("treats disabled redirects as an explicit no-op", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-no-redirects-"));
    tempDirs.push(outDir);

    expect(planRedirectOutputs({ redirects: false }).outputs).toEqual([]);
    await expect(writeRedirectOutputs({ outDir, redirects: false })).resolves.toEqual({
      files: [],
      outputs: [],
    });
    await expect(fs.readdir(outDir)).resolves.toEqual([]);
  });

  it("plans Cloudflare redirects without built-in page generation", () => {
    const redirects = {
      provider: "cloudflare",
      html: false,
      headers: true,
      json: true,
      map: { "/old-guide": "/guide" },
    } satisfies RedirectsOptions;

    const plan = planRedirectOutputs({
      redirects,
      routes: [{ path: "/guide", aliases: ["/old"], redirect: "/retired" }],
      occupiedPaths: ["/occupied"],
      base: "/docs/",
      env: {},
    });

    expect(plan.outputs.map((output) => output.kind)).toEqual(["provider", "headers", "json"]);
    expect(plan.outputs[0]).toEqual({
      kind: "provider",
      provider: "cloudflare",
      path: "_redirects",
      contents: "/old /guide 301\n/retired /guide 301\n/old-guide /guide 301\n",
    });
    expect(plan.outputs[1]?.contents).toContain("Location: /guide");
    expect(plan.outputs[2]?.contents).toContain('"/old-guide"');
  });

  it("emits explicit outputs and protects host-rendered HTML pages", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-public-redirects-"));
    tempDirs.push(outDir);
    const existing = path.join(outDir, "old", "index.html");
    await fs.mkdir(path.dirname(existing), { recursive: true });
    await fs.writeFile(existing, "host page", "utf8");

    const result = await writeRedirectOutputs({
      outDir,
      redirects: { provider: "netlify", map: { "/legacy": "/guide" } },
      routes: [{ path: "/guide", aliases: ["/old"] }],
      env: {},
    });

    expect(result.outputs.map((output) => output.kind)).toEqual(["html", "html", "provider"]);
    expect(result.files).toEqual([
      path.join(outDir, "legacy", "index.html"),
      path.join(outDir, "_redirects"),
    ]);
    expect(await fs.readFile(existing, "utf8")).toBe("host page");
    expect(await fs.readFile(path.join(outDir, "_redirects"), "utf8")).toBe(
      "/old /guide 301\n/legacy /guide 301\n",
    );
  });

  it("preserves destination policy and built-in SSG redirect bytes", () => {
    const blocked = planRedirectOutputs({
      redirects: { provider: "cloudflare", map: { "/external": "https://example.com" } },
      env: {},
    });
    expect(blocked.outputs).toEqual([]);

    const publicPlan = planRedirectOutputs({
      redirects: {
        provider: "cloudflare",
        allowExternal: true,
        map: { "/external": "https://example.com" },
      },
      env: {},
    });
    const builtInPlan = planRedirectFiles({
      options: resolveRedirectsOptions({
        provider: "cloudflare",
        allowExternal: true,
        map: { "/external": "https://example.com" },
      }),
      pages: [],
    });

    expect(publicPlan.outputs[0]).toMatchObject({
      kind: "html",
      from: "/external",
      to: "https://example.com",
    });
    expect(publicPlan.outputs.find((output) => output.kind === "provider")?.contents).toBe(
      builtInPlan.netlify,
    );
  });

  it("omits unsafe redirect source paths before writing", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-unsafe-redirects-"));
    tempDirs.push(outDir);

    const result = await writeRedirectOutputs({
      outDir,
      redirects: { map: { "/../escape": "/guide" } },
      env: {},
    });

    expect(result).toEqual({ files: [], outputs: [] });
    await expect(fs.readdir(outDir)).resolves.toEqual([]);
  });
});
