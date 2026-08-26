import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import {
  PageResourceError,
  isInsideRoot,
  parseResourceSrc,
  processPageResources,
  resolveResourcesOptions,
  resourceCacheKey,
} from "./resources";
import { createRgba, encodeJpeg, encodePng, pngSize } from "./resources-image";
import { buildSsg } from "./ssg";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("resolveResourcesOptions", () => {
  it("disables the feature when omitted or false", () => {
    expect(resolveResourcesOptions(undefined)).toEqual({
      enabled: false,
      formats: ["png", "jpeg", "webp"],
      widths: [],
      missing: "error",
      dedupe: false,
    });
    expect(resolveResourcesOptions(false)).toEqual({
      enabled: false,
      formats: ["png", "jpeg", "webp"],
      widths: [],
      missing: "error",
      dedupe: false,
    });
    expect(createDocsResolvedOptions().resources).toBeUndefined();
  });

  it("enables defaults when true, and overrides only set fields", () => {
    expect(resolveResourcesOptions(true)).toEqual({
      enabled: true,
      formats: ["png", "jpeg", "webp"],
      widths: [],
      missing: "error",
      dedupe: false,
    });
    expect(resolveResourcesOptions({})).toEqual({
      enabled: true,
      formats: ["png", "jpeg", "webp"],
      widths: [],
      missing: "error",
      dedupe: false,
    });
    expect(
      resolveResourcesOptions({
        formats: ["PNG", "jpg"],
        widths: [16, 32],
        missing: "warn",
        dedupe: true,
      }),
    ).toEqual({
      enabled: true,
      formats: ["png", "jpeg"],
      widths: [16, 32],
      missing: "warn",
      dedupe: true,
    });
  });
});

describe("resource path and cache helpers", () => {
  it("rejects paths that leave the bundle or content root", () => {
    const bundle = path.join(os.tmpdir(), "ox-bundle");
    const inside = path.join(bundle, "hero.png");
    const outside = path.join(bundle, "..", "secret.png");
    expect(isInsideRoot(bundle, inside)).toBe(true);
    expect(isInsideRoot(bundle, outside)).toBe(false);
    expect(parseResourceSrc("../secret.png")?.pathname).toBe("../secret.png");
    expect(parseResourceSrc("javascript:alert(1)")).toBeUndefined();
    expect(parseResourceSrc("//evil.example/x.png")).toBeUndefined();
    expect(parseResourceSrc("/absolute.png")).toBeUndefined();
  });

  it("changes the cache key when mtime or transform params change", () => {
    const first = resourceCacheKey("/page/hero.png", 10, { width: 16, format: "png" });
    const same = resourceCacheKey("/page/hero.png", 10, { width: 16, format: "png" });
    const later = resourceCacheKey("/page/hero.png", 11, { width: 16, format: "png" });
    const other = resourceCacheKey("/page/hero.png", 10, { width: 32, format: "png" });
    expect(first).toBe(same);
    expect(first).not.toBe(later);
    expect(first).not.toBe(other);
  });
});

describe("encodeJpeg", () => {
  it("writes a small JPEG for a tiny source", () => {
    const jpeg = encodeJpeg(createRgba(8, 8, (x, y) => [x * 20, y * 20, 80, 255]));
    expect(jpeg[0]).toBe(0xff);
    expect(jpeg[1]).toBe(0xd8);
    expect(jpeg.length).toBeLessThan(8_000);
  });
});

describe("processPageResources", () => {
  it("applies width, crop, and format from HTML src query strings", async () => {
    const root = await makeSite({ "hero.png": samplePng(8, 8) });
    const inputPath = path.join(root, "content", "guide.md");
    await fs.writeFile(inputPath, "# Guide\n");
    const result = await processPageResources({
      html: [
        '<img src="./hero.png?width=4" alt="Wide">',
        '<img src="./hero.png?width=4&amp;height=2&amp;crop=center" alt="Crop">',
        '<img src="./hero.png?width=4&amp;format=png" alt="Png">',
      ].join(""),
      inputPath,
      outputPath: path.join(root, "dist", "guide", "index.html"),
      srcDir: path.join(root, "content"),
      options: resolveResourcesOptions(true),
      cacheDir: path.join(root, ".cache"),
    });
    expect(result.fatal).toEqual([]);
    expect(result.files).toHaveLength(3);
    expect(new Set(result.files).size).toBe(3);
    expect(pngSize(await fs.readFile(result.files[0]!))).toEqual({ width: 4, height: 4 });
    expect(pngSize(await fs.readFile(result.files[1]!))).toEqual({ width: 4, height: 2 });
  });

  it("leaves fenced and inline code image syntax untouched", async () => {
    const root = await makeSite({});
    const inputPath = path.join(root, "content", "guide.md");
    const html = `<pre><code>![Hero](./hero.png)</code></pre><p>Use <code>![Hero](./hero.png)</code></p>`;
    const result = await processPageResources({
      html,
      inputPath,
      outputPath: path.join(root, "dist", "guide", "index.html"),
      srcDir: path.join(root, "content"),
      options: resolveResourcesOptions(true),
      cacheDir: path.join(root, ".cache"),
    });
    expect(result.html).toBe(html);
    expect(result.files).toEqual([]);
  });

  it("ignores hostile destinations and does not emit them as output names", async () => {
    const root = await makeSite({});
    const inputPath = path.join(root, "content", "guide.md");
    const result = await processPageResources({
      html: `<p><img src="javascript:alert(1)" alt="x"><img src="./missing.png" alt="y"></p>`,
      inputPath,
      outputPath: path.join(root, "dist", "guide", "index.html"),
      srcDir: path.join(root, "content"),
      options: resolveResourcesOptions(true),
      cacheDir: path.join(root, ".cache"),
    });
    expect(result.files).toEqual([]);
    expect(result.html).toContain("javascript:alert(1)");
    expect(result.fatal.some((issue) => issue.includes("missing page resource"))).toBe(true);
  });
});

describe("buildSsg resources", () => {
  it("does not copy sibling images when omitted or false", async () => {
    const png = samplePng(8, 8);
    const omitted = await makeSite({
      "guide.md": "# Guide\n\n![Hero](./hero.png)\n",
      "hero.png": png,
    });
    const disabled = await makeSite({
      "guide.md": "# Guide\n\n![Hero](./hero.png)\n",
      "hero.png": png,
    });
    await buildSsg(createDocsResolvedOptions(), omitted);
    await buildSsg(enabledOptions(resolveResourcesOptions(false)), disabled);

    for (const root of [omitted, disabled]) {
      const html = await fs.readFile(pageHtml(root, "guide"), "utf8");
      expect(html).toContain("./hero.png");
      await expect(fs.access(path.join(root, "dist", "guide", "hero.png"))).rejects.toThrow();
    }
  });

  it("copies a colocated sibling image and keeps it addressable", async () => {
    const root = await makeSite({
      "guide.md": "# Guide\n\n![Hero](./hero.png)\n",
      "hero.png": samplePng(8, 8),
    });
    const result = await buildSsg(enabledOptions(resolveResourcesOptions(true)), root);
    expect(result.errors).toEqual([]);
    expect(result.files.some((file) => file.endsWith(`${path.sep}hero.png`))).toBe(true);

    const html = await fs.readFile(pageHtml(root, "guide"), "utf8");
    expect(html).toContain('src="hero.png"');
    const copied = await fs.readFile(path.join(root, "dist", "guide", "hero.png"));
    expect(pngSize(copied)).toEqual({ width: 8, height: 8 });
  });

  it("copies images referenced from opt-in galleries", async () => {
    const root = await makeSite({
      "guide.md": [
        "# Guide",
        "",
        '::: gallery title="Screenshots"',
        '![Hero](./hero.png?width=4 "Hero view")',
        ":::",
        "",
      ].join("\n"),
      "hero.png": samplePng(8, 8),
    });
    const result = await buildSsg(
      {
        ...enabledOptions(resolveResourcesOptions(true)),
        imageGalleries: { enabled: true, missingAlt: "error", empty: "error" },
      },
      root,
    );
    expect(result.errors).toEqual([]);

    const html = await fs.readFile(pageHtml(root, "guide"), "utf8");
    expect(html).toContain('class="ox-image-gallery"');
    expect(html).toMatch(/src="hero\.[a-f0-9]{12}\.png"/);
    expect(result.files.some((file) => /hero\.[a-f0-9]{12}\.png$/.test(file))).toBe(true);
  });

  it("resizes, crops, and converts format at build time", async () => {
    const root = await makeSite({
      "guide.md": [
        "# Guide",
        "",
        "![Wide](./hero.png?width=4)",
        "![Crop](./hero.png?crop=2,2,4,2)",
        "![Png](./hero.png?format=png)",
        "",
      ].join("\n"),
      "hero.png": samplePng(8, 8),
    });
    const result = await buildSsg(enabledOptions(resolveResourcesOptions(true)), root);
    expect(result.errors).toEqual([]);

    const html = await fs.readFile(pageHtml(root, "guide"), "utf8");
    const dist = path.join(root, "dist", "guide");
    const outputs = (await fs.readdir(dist)).filter((name) => name !== "index.html");
    expect(outputs.filter((name) => name.endsWith(".png")).length).toBeGreaterThanOrEqual(2);

    const sizes = await Promise.all(
      outputs
        .filter((name) => name.endsWith(".png"))
        .map(async (name) => pngSize(await fs.readFile(path.join(dist, name)))),
    );
    expect(sizes.some((size) => size.width === 4 && size.height === 4)).toBe(true);
    expect(sizes.some((size) => size.width === 4 && size.height === 2)).toBe(true);
    expect(html).toMatch(/hero\.[a-f0-9]{12}\.png/);
  });

  it("reuses the cache when source mtime and params are unchanged", async () => {
    const root = await makeSite({
      "guide.md": "# Guide\n\n![Hero](./hero.png?width=4&format=png)\n",
      "hero.png": samplePng(8, 8),
    });
    const options = enabledOptions(resolveResourcesOptions(true));
    await buildSsg(options, root);
    const first = await fs.readdir(path.join(root, ".cache", "ox-content-resources"));
    expect(first).toHaveLength(1);

    await buildSsg(options, root);
    const second = await fs.readdir(path.join(root, ".cache", "ox-content-resources"));
    expect(second).toEqual(first);
  });

  it("rejects ../ that leaves the page bundle", async () => {
    const root = await makeSite({
      "posts/guide.md": "# Guide\n\n![Leak](../secret.png)\n",
    });
    await fs.writeFile(path.join(root, "secret.png"), samplePng(2, 2));
    await expect(
      buildSsg(enabledOptions(resolveResourcesOptions(true)), root),
    ).rejects.toBeInstanceOf(PageResourceError);
    await expect(
      fs.access(path.join(root, "dist", "posts", "guide", "secret.png")),
    ).rejects.toThrow();
  });

  it("fails the build when a source is missing and missing is error", async () => {
    const root = await makeSite({
      "guide.md": "# Guide\n\n![Hero](./missing.png)\n",
    });
    await expect(buildSsg(enabledOptions(resolveResourcesOptions(true)), root)).rejects.toThrow(
      /missing page resource/,
    );
  });

  it("does not throw when missing is warn", async () => {
    const root = await makeSite({
      "guide.md": "# Guide\n\n![Hero](./missing.png)\n",
    });
    const result = await buildSsg(
      enabledOptions(resolveResourcesOptions({ missing: "warn" })),
      root,
    );
    expect(result.errors.some((error) => error.includes("missing page resource"))).toBe(true);
    await fs.readFile(pageHtml(root, "guide"), "utf8");
  });

  it("does not process image syntax inside fences during SSG", async () => {
    const root = await makeSite({
      "guide.md": ["# Guide", "", "```md", "![Hero](./hero.png)", "```", ""].join("\n"),
      "hero.png": samplePng(4, 4),
    });
    await buildSsg(enabledOptions(resolveResourcesOptions(true)), root);
    await expect(fs.access(path.join(root, "dist", "guide", "hero.png"))).rejects.toThrow();
  });
});

function enabledOptions(resources: ReturnType<typeof resolveResourcesOptions>) {
  return createDocsResolvedOptions({
    resources,
    ssg: { ...createDocsResolvedOptions().ssg, bare: true },
  });
}

function pageHtml(root: string, name: string): string {
  return path.join(root, "dist", name, "index.html");
}

function samplePng(width: number, height: number): Buffer {
  return encodePng(createRgba(width, height, (x, y) => [x * 30, y * 30, 120, 255]));
}

async function makeSite(files: Record<string, string | Buffer>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-resources-"));
  tempDirs.push(root);
  for (const [relative, body] of Object.entries(files)) {
    const filePath = path.join(root, "content", relative);
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, body);
  }
  return root;
}
