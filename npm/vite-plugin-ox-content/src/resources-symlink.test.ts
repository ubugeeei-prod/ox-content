import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { processPageResources, resolveResourcesOptions } from "./resources";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("page resources behind symlinks", () => {
  for (const dedupe of [false, true]) {
    it(`does not copy a symlink that leaves the content root (dedupe: ${dedupe})`, async () => {
      const root = await makeRoot();
      await fs.writeFile(path.join(root, "outside.txt"), "outside\n");
      await fs.symlink("../../outside.txt", path.join(root, "content", "post", "linked.png"));

      const result = await runResources(root, '<p><img src="linked.png"></p>', dedupe);

      expect(result.files).toEqual([]);
      expect(result.fatal.some((message) => message.includes("outside the page bundle"))).toBe(
        true,
      );
      await expect(fs.access(path.join(root, "dist", "post", "linked.png"))).rejects.toThrow();
    });
  }

  it("still copies a symlink that resolves inside the page bundle", async () => {
    const root = await makeRoot();
    await fs.writeFile(path.join(root, "content", "post", "hero.png"), "inside\n");
    await fs.symlink("hero.png", path.join(root, "content", "post", "alias.png"));

    const result = await runResources(root, '<p><img src="alias.png"></p>', false);

    expect(result.fatal).toEqual([]);
    expect(await fs.readFile(path.join(root, "dist", "post", "alias.png"), "utf8")).toBe(
      "inside\n",
    );
  });

  it("ignores an href symlink that resolves outside the content root", async () => {
    const root = await makeRoot();
    await fs.writeFile(path.join(root, "outside.txt"), "outside\n");
    await fs.symlink("../../outside.txt", path.join(root, "content", "post", "notes.txt"));

    const result = await runResources(root, '<p><a href="notes.txt">notes</a></p>', false);

    expect(result).toMatchObject({ files: [], errors: [], fatal: [] });
    await expect(fs.access(path.join(root, "dist", "post", "notes.txt"))).rejects.toThrow();
  });
});

async function runResources(root: string, html: string, dedupe: boolean) {
  return processPageResources({
    html,
    inputPath: path.join(root, "content", "post", "index.md"),
    outputPath: path.join(root, "dist", "post", "index.html"),
    srcDir: path.join(root, "content"),
    outDir: path.join(root, "dist"),
    options: resolveResourcesOptions({ dedupe }),
    cacheDir: path.join(root, ".cache"),
  });
}

async function makeRoot(): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-resources-link-"));
  tempDirs.push(root);
  await fs.mkdir(path.join(root, "content", "post"), { recursive: true });
  return root;
}
