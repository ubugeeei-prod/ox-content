import { createHash } from "node:crypto";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import {
  createResourceDedupeStore,
  hashResourceBuffer,
  linkOrCopyAlias,
  rewriteToCanonicalUrl,
} from "./resources-dedupe";
import { createRgba, encodePng, pngSize } from "./resources-image";
import { processPageResources, resolveResourcesOptions } from "./resources";
import { buildSsg } from "./ssg";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("resource content dedupe", () => {
  it("is off for true and empty options", () => {
    expect(resolveResourcesOptions(true).dedupe).toBe(false);
    expect(resolveResourcesOptions({}).dedupe).toBe(false);
  });

  it("emits identical bytes once and rewrites src, poster, and href", async () => {
    const png = samplePng(8, 8);
    const root = await makeSite({
      "one/page.md": "# One\n",
      "one/hero.png": png,
      "two/page.md": "# Two\n",
      "two/hero.png": png,
    });
    const store = createResourceDedupeStore();
    const one = await processPage({
      root,
      page: "one/page.md",
      html: [
        '<img src="./hero.png" alt="a">',
        '<video poster="./hero.png"></video>',
        '<a href="./hero.png">dl</a>',
      ].join(""),
      store,
    });
    const two = await processPage({
      root,
      page: "two/page.md",
      html: '<img src="./hero.png" alt="b">',
      store,
    });

    expect(one.fatal).toEqual([]);
    expect(two.fatal).toEqual([]);
    const name = digestName(png, "png");
    const canonical = path.join(root, "dist", "assets", "content", name);
    expect(await fs.readFile(canonical)).toEqual(png);
    expect(one.html).toContain(`src="/assets/content/${name}"`);
    expect(one.html).toContain(`poster="/assets/content/${name}"`);
    expect(one.html).toContain(`href="/assets/content/${name}"`);
    expect(two.html).toContain(`src="/assets/content/${name}"`);
    const assetDir = `${path.sep}assets${path.sep}content`;
    const canonicalWrites = one.files.filter((file) => file.includes(assetDir));
    expect(canonicalWrites).toHaveLength(1);
    expect(two.files.some((file) => file.includes(`${path.sep}assets${path.sep}content`))).toBe(
      false,
    );
    await expect(fs.readFile(path.join(root, "dist", "one", "page", "hero.png"))).resolves.toEqual(
      png,
    );
    await expect(fs.readFile(path.join(root, "dist", "two", "page", "hero.png"))).resolves.toEqual(
      png,
    );
  });

  it("keeps different bytes and incompatible extensions separate", async () => {
    const first = samplePng(8, 8);
    const second = samplePng(4, 4);
    const root = await makeSite({
      "one/page.md": "# One\n",
      "one/hero.png": first,
      "one/hero.jpg": first,
      "two/page.md": "# Two\n",
      "two/hero.png": second,
    });
    const store = createResourceDedupeStore();
    await processPage({
      root,
      page: "one/page.md",
      html: '<img src="./hero.png"><img src="./hero.jpg">',
      store,
    });
    await processPage({
      root,
      page: "two/page.md",
      html: '<img src="./hero.png">',
      store,
    });
    const names = (await fs.readdir(path.join(root, "dist", "assets", "content"))).sort();
    expect(names).toEqual(
      [digestName(first, "png"), digestName(first, "jpg"), digestName(second, "png")].sort(),
    );
  });

  it("preserves leftover query and hash, including a site base", async () => {
    const png = samplePng(6, 6);
    const root = await makeSite({ "guide.md": "# Guide\n", "hero.png": png });
    const result = await processPage({
      root,
      page: "guide.md",
      html: '<img src="./hero.png?utm=keep#frag">',
      base: "/docs/",
    });
    const name = digestName(png, "png");
    expect(result.html).toContain(`src="/docs/assets/content/${name}?utm=keep#frag"`);
    expect(rewriteToCanonicalUrl("./hero.png?width=4&v=1#x", "/assets/content/ab.png")).toBe(
      "/assets/content/ab.png?v=1#x",
    );
  });

  it("leaves remote and data URLs untouched", async () => {
    const root = await makeSite({ "guide.md": "# Guide\n" });
    const html =
      '<img src="https://example.com/x.png"><img src="data:image/png;base64,xx"><img src="//cdn.example/x.png">';
    const result = await processPage({ root, page: "guide.md", html });
    expect(result.html).toBe(html);
    expect(result.files).toEqual([]);
  });

  it("copies when hard linking is not possible", async () => {
    const root = await makeSite({});
    const canonical = path.join(root, "canonical.png");
    const alias = path.join(root, "alias.png");
    const png = samplePng(2, 2);
    await fs.writeFile(canonical, png);
    const mode = await linkOrCopyAlias(canonical, alias, async () => {
      throw new Error("link unsupported");
    });
    expect(mode).toBe("copy");
    expect(await fs.readFile(alias)).toEqual(png);
  });

  it("does not change output when dedupe is off", async () => {
    const png = samplePng(8, 8);
    const root = await makeSite({
      "one/page.md": "# One\n\n![Hero](./hero.png)\n",
      "one/hero.png": png,
      "two/page.md": "# Two\n\n![Hero](./hero.png)\n",
      "two/hero.png": png,
    });
    await buildSsg(enabledOptions(resolveResourcesOptions(true)), root);
    const one = await fs.readFile(pageHtml(root, "one/page"), "utf8");
    const two = await fs.readFile(pageHtml(root, "two/page"), "utf8");
    expect(one).toContain('src="hero.png"');
    expect(two).toContain('src="hero.png"');
    expect(one).not.toContain("/assets/content/");
    await expect(fs.access(path.join(root, "dist", "assets", "content"))).rejects.toThrow();
    expect(pngSize(await fs.readFile(path.join(root, "dist", "one", "page", "hero.png")))).toEqual({
      width: 8,
      height: 8,
    });
  });

  it("dedupes across pages in an SSG build with deterministic names", async () => {
    const png = samplePng(8, 8);
    const root = await makeSite({
      "one/page.md": "# One\n\n![Hero](./hero.png)\n",
      "one/hero.png": png,
      "two/page.md": "# Two\n\n![Hero](./hero.png)\n",
      "two/hero.png": png,
    });
    const options = enabledOptions(resolveResourcesOptions({ dedupe: true }));
    await buildSsg(options, root);
    const name = digestName(png, "png");
    const listed = await fs.readdir(path.join(root, "dist", "assets", "content"));
    expect(listed).toEqual([name]);
    const html = await fs.readFile(pageHtml(root, "one/page"), "utf8");
    expect(html).toContain(`src="/assets/content/${name}"`);
    await buildSsg(options, root);
    expect(await fs.readdir(path.join(root, "dist", "assets", "content"))).toEqual([name]);
  });

  it("reuses a digest for the same source key", () => {
    const store = createResourceDedupeStore();
    const bytes = Buffer.from("same");
    const first = hashResourceBuffer(bytes, "png", store, "key");
    const second = hashResourceBuffer(Buffer.from("other"), "png", store, "key");
    expect(first).toBe(second);
    expect(first).toHaveLength(64);
  });
});

function digestName(bytes: Buffer, ext: string): string {
  return `${createHash("sha256").update(bytes).update("\0").update(ext).digest("hex")}.${ext}`;
}

function enabledOptions(resources: ReturnType<typeof resolveResourcesOptions>) {
  return createDocsResolvedOptions({
    resources,
    ssg: { ...createDocsResolvedOptions().ssg, bare: true },
  });
}

function pageHtml(root: string, name: string): string {
  return path.join(root, "dist", ...name.split("/"), "index.html");
}

function samplePng(width: number, height: number): Buffer {
  return encodePng(createRgba(width, height, (x, y) => [x * 30, y * 30, 120, 255]));
}

async function processPage(input: {
  root: string;
  page: string;
  html: string;
  store?: ReturnType<typeof createResourceDedupeStore>;
  base?: string;
}) {
  const inputPath = path.join(input.root, "content", input.page);
  const route = input.page.replace(/\.md$/, "");
  return processPageResources({
    html: input.html,
    inputPath,
    outputPath: path.join(input.root, "dist", ...route.split("/"), "index.html"),
    srcDir: path.join(input.root, "content"),
    options: resolveResourcesOptions({ dedupe: true }),
    cacheDir: path.join(input.root, ".cache"),
    outDir: path.join(input.root, "dist"),
    base: input.base ?? "/",
    dedupeStore: input.store,
  });
}

async function makeSite(files: Record<string, string | Buffer>): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-dedupe-"));
  tempDirs.push(root);
  for (const [relative, body] of Object.entries(files)) {
    const filePath = path.join(root, "content", relative);
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, body);
  }
  return root;
}
