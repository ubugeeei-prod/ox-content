import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { generatePwa, injectPwaPageTags, resolvePwaOptions, writePwaFiles } from "./pwa";
import { buildSsg } from "./ssg";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

async function makeSite(files: Record<string, string>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-pwa-"));
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

describe("resolvePwaOptions", () => {
  it("disables the feature by default", () => {
    expect(resolvePwaOptions(undefined)).toEqual({
      enabled: false,
      offline: true,
    });
    expect(resolvePwaOptions(false)).toEqual({
      enabled: false,
      offline: true,
    });
  });

  it("enables defaults when true", () => {
    expect(resolvePwaOptions(true)).toEqual({
      enabled: true,
      offline: true,
    });
  });

  it("enables the feature from an object and overrides only set fields", () => {
    expect(
      resolvePwaOptions({
        name: "Docs",
        shortName: "Docs",
        themeColor: "#0f172a",
        backgroundColor: "#ffffff",
        startUrl: "/app/",
        offline: false,
      }),
    ).toEqual({
      enabled: true,
      offline: false,
      name: "Docs",
      shortName: "Docs",
      themeColor: "#0f172a",
      backgroundColor: "#ffffff",
      startUrl: "/app/",
    });
    expect(resolvePwaOptions({})).toEqual({
      enabled: true,
      offline: true,
    });
  });
});

describe("generatePwa", () => {
  it("writes nothing when the feature is omitted or disabled", () => {
    expect(generatePwa({})).toEqual({});
    expect(
      generatePwa({
        options: { enabled: false, offline: true },
        siteUrl: "https://example.com",
        siteName: "Docs",
      }),
    ).toEqual({});
  });

  it("warns and writes nothing when siteUrl is missing", () => {
    expect(
      generatePwa({
        options: { enabled: true, offline: true },
        siteName: "Docs",
      }),
    ).toEqual({
      warning:
        "[ox-content] pwa is enabled but ssg.siteUrl is not set; manifest.webmanifest and sw.js were not written",
    });
    expect(
      generatePwa({
        options: { enabled: true, offline: true },
        siteUrl: "   ",
        siteName: "Docs",
      }).warning,
    ).toBeDefined();
  });

  it("writes a manifest and network-first service worker when enabled", () => {
    const output = generatePwa({
      options: { enabled: true, offline: true },
      siteUrl: "https://example.com",
      siteName: "Docs",
      base: "/docs/",
    });

    expect(output.warning).toBeUndefined();
    expect(output.manifest).toContain('"name": "Docs"');
    expect(output.manifest).toContain('"short_name": "Docs"');
    expect(output.manifest).toContain('"start_url": "/docs/"');
    expect(output.manifest).toContain('"scope": "/docs/"');
    expect(output.manifest).toContain('"display": "standalone"');
    expect(output.serviceWorker).toContain('ASSET_PREFIX = "/docs/assets/"');
    expect(output.serviceWorker).toContain("networkFirst");
    expect(output.serviceWorker).toContain("cacheFirst");
    expect(output.serviceWorker).toContain("isHashedAsset");
  });

  it("writes the manifest only when offline is false", () => {
    const output = generatePwa({
      options: { enabled: true, offline: false, name: "Installable" },
      siteUrl: "https://example.com",
      siteName: "Docs",
    });

    expect(output.manifest).toContain('"name": "Installable"');
    expect(output.serviceWorker).toBeUndefined();
  });

  it("escapes a hostile name and rejects unsafe colors and start URLs", () => {
    const output = generatePwa({
      options: {
        enabled: true,
        offline: true,
        name: `</script><script>alert(1)</script>\n- [Injected](https://evil.example/)`,
        themeColor: `"><img src=x onerror=alert(1)>`,
        backgroundColor: "javascript:alert(1)",
        startUrl: "javascript:alert(1)",
      },
      siteUrl: "https://example.com",
      siteName: "Docs",
      base: "/",
    });

    expect(output.manifest).not.toContain("<script>");
    expect(output.manifest).toContain("\\u003c/script\\u003e");
    expect(output.manifest).not.toContain('theme_color": "">');
    expect(output.manifest).toContain('"theme_color": "#000000"');
    expect(output.manifest).toContain('"background_color": "#ffffff"');
    expect(output.manifest).toContain('"start_url": "/"');
    expect(output.manifest).not.toContain("javascript:");
  });
});

describe("writePwaFiles", () => {
  it("does not write files when the feature is disabled", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-pwa-write-"));
    tempDirs.push(outDir);

    const result = await writePwaFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/",
      siteName: "Docs",
    });

    expect(result.files).toEqual([]);
    expect(result.warning).toBeUndefined();
    await expect(fs.access(path.join(outDir, "manifest.webmanifest"))).rejects.toThrow();
    await expect(fs.access(path.join(outDir, "sw.js"))).rejects.toThrow();
  });

  it("does not write files when siteUrl is missing", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-pwa-write-"));
    tempDirs.push(outDir);

    const result = await writePwaFiles({
      outDir,
      base: "/",
      siteName: "Docs",
      options: { enabled: true, offline: true },
    });

    expect(result.files).toEqual([]);
    expect(result.warning).toContain("ssg.siteUrl is not set");
    await expect(fs.access(path.join(outDir, "manifest.webmanifest"))).rejects.toThrow();
  });

  it("writes the manifest and service worker next to generated HTML", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-pwa-write-"));
    tempDirs.push(outDir);

    const result = await writePwaFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/docs/",
      siteName: "Docs",
      options: { enabled: true, offline: true },
    });

    expect(result.files).toEqual([
      path.join(outDir, "manifest.webmanifest"),
      path.join(outDir, "sw.js"),
    ]);
    expect(await fs.readFile(path.join(outDir, "manifest.webmanifest"), "utf8")).toContain(
      '"start_url": "/docs/"',
    );
    expect(await fs.readFile(path.join(outDir, "sw.js"), "utf8")).toContain(
      'ASSET_PREFIX = "/docs/assets/"',
    );
  });

  it("writes the manifest only when offline is false", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-pwa-write-"));
    tempDirs.push(outDir);

    const result = await writePwaFiles({
      outDir,
      siteUrl: "https://example.com",
      base: "/",
      siteName: "Docs",
      options: { enabled: true, offline: false },
    });

    expect(result.files).toEqual([path.join(outDir, "manifest.webmanifest")]);
    await expect(fs.access(path.join(outDir, "sw.js"))).rejects.toThrow();
  });
});

describe("injectPwaPageTags", () => {
  const themed = `<!DOCTYPE html>\n<html><head><title>Docs</title>\n</head>\n<body><p>Hi</p>\n</body></html>\n`;

  it("leaves HTML unchanged when the feature is off", () => {
    expect(injectPwaPageTags(themed, { options: { enabled: false, offline: true } })).toBe(themed);
    expect(injectPwaPageTags(themed, {})).toBe(themed);
  });

  it("injects the manifest link and register script on themed pages", () => {
    const html = injectPwaPageTags(themed, {
      options: { enabled: true, offline: true, themeColor: "#0f172a" },
      base: "/docs/",
    });

    expect(html).toContain('<link rel="manifest" href="/docs/manifest.webmanifest">');
    expect(html).toContain('<meta name="theme-color" content="#0f172a">');
    expect(html).toContain('navigator.serviceWorker.register("/docs/sw.js")');
    expect(html).not.toContain("<script>alert");
  });

  it("keeps the manifest link without a register script when offline is false", () => {
    const html = injectPwaPageTags(themed, {
      options: { enabled: true, offline: false },
      base: "/",
    });

    expect(html).toContain('<link rel="manifest" href="/manifest.webmanifest">');
    expect(html).not.toContain("serviceWorker");
  });

  it("does not inject into bare fragments", () => {
    const fragment = "<article>Hello</article>";
    expect(
      injectPwaPageTags(fragment, { options: { enabled: true, offline: true }, base: "/" }),
    ).toBe(fragment);
  });
});

describe("buildSsg pwa", () => {
  it("writes PWA files and injects tags on themed pages", async () => {
    const root = await makeSite({
      "index.md": "# Home\n\nWelcome home.\n",
    });
    const base = createDocsResolvedOptions();
    const result = await buildSsg(
      createDocsResolvedOptions({
        ssg: { ...base.ssg, siteUrl: "https://example.com", siteName: "Docs" },
        pwa: resolvePwaOptions(true),
      }),
      root,
    );

    const manifestPath = path.join(root, "dist", "manifest.webmanifest");
    const swPath = path.join(root, "dist", "sw.js");
    expect(result.files).toContain(manifestPath);
    expect(result.files).toContain(swPath);

    const html = await fs.readFile(path.join(root, "dist", "index.html"), "utf8");
    expect(html).toContain('<link rel="manifest" href="/manifest.webmanifest">');
    expect(html).toContain('navigator.serviceWorker.register("/sw.js")');
    expect(await fs.readFile(manifestPath, "utf8")).toContain('"name": "Docs"');
  });
});
