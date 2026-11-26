import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { processPageResources, resolveResourcesOptions } from "./resources";
import {
  MAX_RESOURCE_PIXELS,
  createRgba,
  decodePng,
  encodePng,
  resizeNearest,
} from "./resources-image";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

function tinyPng(): Buffer {
  return encodePng(createRgba(2, 2, () => [10, 20, 30, 255]));
}

describe("page resource image size limits", () => {
  it("refuses to resize beyond the pixel budget", () => {
    const side = Math.ceil(Math.sqrt(MAX_RESOURCE_PIXELS)) + 1;
    expect(() => resizeNearest(decodePng(tinyPng()), side, side)).toThrow(/exceeds/);
    expect(resizeNearest(decodePng(tinyPng()), 4, 4).data).toHaveLength(64);
  });

  it("refuses a PNG whose header declares an oversized image", () => {
    const png = Buffer.from(tinyPng());
    // IHDR width and height follow the 8-byte signature and 8-byte chunk header.
    png.writeUInt32BE(100_000, 16);
    png.writeUInt32BE(100_000, 20);
    expect(() => decodePng(png)).toThrow(/exceeds/);
  });

  it("fails the page resource instead of allocating a huge resize", async () => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-resources-limit-"));
    tempDirs.push(root);
    await fs.mkdir(path.join(root, "content", "post"), { recursive: true });
    await fs.writeFile(path.join(root, "content", "post", "hero.png"), tinyPng());

    const result = await processPageResources({
      html: '<p><img src="hero.png?width=50000&height=50000"></p>',
      inputPath: path.join(root, "content", "post", "index.md"),
      outputPath: path.join(root, "dist", "post", "index.html"),
      srcDir: path.join(root, "content"),
      options: resolveResourcesOptions(true),
      cacheDir: path.join(root, ".cache"),
    });

    expect(result.files).toEqual([]);
    expect(result.fatal.some((message) => message.includes("exceeds"))).toBe(true);
  });
});
