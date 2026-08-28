import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolveMathOptions } from "./index";
import { buildSsg } from "./ssg";
import type { KatexFontFormats, ResolvedOptions } from "./types";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

async function build(
  pages: Record<string, string>,
  math?: { enabled: boolean; fontFormats?: KatexFontFormats },
): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-katex-"));
  tempDirs.push(root);
  const srcDir = path.join(root, "content");
  await fs.mkdir(srcDir, { recursive: true });
  for (const [name, body] of Object.entries(pages)) {
    await fs.writeFile(path.join(srcDir, name), body, "utf8");
  }

  const defaults = createDocsResolvedOptions();
  const result = await buildSsg(
    createDocsResolvedOptions({
      math: math
        ? { ...resolveMathOptions(true), ...math }
        : (defaults.math as ResolvedOptions["math"]),
      ssg: { ...defaults.ssg, siteName: "Docs" },
    } as Partial<ResolvedOptions>),
    root,
  );
  expect(result.errors).toEqual([]);
  return path.join(root, "dist");
}

async function katexFiles(dist: string): Promise<string[]> {
  try {
    return await fs.readdir(path.join(dist, "__ox_katex__", "fonts"));
  } catch {
    return [];
  }
}

async function hasKatexDir(dist: string): Promise<boolean> {
  try {
    await fs.access(path.join(dist, "__ox_katex__"));
    return true;
  } catch {
    return false;
  }
}

describe("KaTeX assets", () => {
  it("emits nothing when math is on but no page renders any", async () => {
    const dist = await build(
      {
        "index.md": "# Home\n\nCosts $5.00, or $10 for two.\n",
        "guide.md": "# Guide\n\nUse `$PATH` and ${score} in prose.\n",
      },
      { enabled: true },
    );

    expect(await hasKatexDir(dist)).toBe(false);
    const html = await fs.readFile(path.join(dist, "index.html"), "utf8");
    expect(html).not.toContain("__ox_katex__");
  });

  it("emits them once a page renders math", async () => {
    const dist = await build(
      {
        "index.md": "# Home\n\nNo math here.\n",
        "formula.md": "# Formula\n\n$E = mc^2$\n",
      },
      { enabled: true },
    );

    expect(await hasKatexDir(dist)).toBe(true);
    const html = await fs.readFile(path.join(dist, "formula", "index.html"), "utf8");
    expect(html).toContain("__ox_katex__/katex.min.css");
  });

  it("ships only woff2 by default", async () => {
    const dist = await build({ "index.md": "# Home\n\n$E = mc^2$\n" }, { enabled: true });
    const fonts = await katexFiles(dist);

    expect(fonts.length).toBeGreaterThan(0);
    expect(fonts.every((file) => file.endsWith(".woff2"))).toBe(true);
    expect(fonts.some((file) => file.endsWith(".ttf"))).toBe(false);
    expect(fonts.some((file) => file.endsWith(".woff"))).toBe(false);
  });

  it("ships every format when asked", async () => {
    const dist = await build(
      { "index.md": "# Home\n\n$E = mc^2$\n" },
      { enabled: true, fontFormats: "all" },
    );
    const fonts = await katexFiles(dist);

    expect(fonts.some((file) => file.endsWith(".woff2"))).toBe(true);
    expect(fonts.some((file) => file.endsWith(".woff"))).toBe(true);
    expect(fonts.some((file) => file.endsWith(".ttf"))).toBe(true);
  });

  it("emits nothing when math is off", async () => {
    const dist = await build({ "index.md": "# Home\n\n$E = mc^2$\n" });

    expect(await hasKatexDir(dist)).toBe(false);
  });

  it("defaults fontFormats to woff2", () => {
    expect(resolveMathOptions(true).fontFormats).toBe("woff2");
    expect(resolveMathOptions({}).fontFormats).toBe("woff2");
    expect(resolveMathOptions({ fontFormats: "all" }).fontFormats).toBe("all");
  });
});
