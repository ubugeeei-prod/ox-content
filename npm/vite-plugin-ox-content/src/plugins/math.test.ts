import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { copyKatexAssets } from "./math-assets";
import { renderKatexMath, resetKatexWarningForTests, resolveKatexDist } from "./math";

afterEach(() => {
  resetKatexWarningForTests();
});

describe("renderKatexMath", () => {
  it("is a no-op when the page has no math placeholders", async () => {
    const html = "<p>no dollars</p>";
    expect(await renderKatexMath(html)).toBe(html);
  });

  it("renders inline TeX with KaTeX when the package is installed", async () => {
    expect(resolveKatexDist()).toBeTruthy();

    const html = await renderKatexMath(
      '<p><span class="ox-math ox-math-inline" data-ox-tex="E=mc^2"><math><mtext>E=mc^2</mtext></math></span></p>',
    );
    expect(html).toContain('class="ox-math ox-math-inline"');
    expect(html).toContain("katex");
    expect(html).not.toContain("data-ox-tex");
    expect(html).toContain("mc");
  });

  it("renders display TeX in display mode", async () => {
    const html = await renderKatexMath(
      '<div class="ox-math ox-math-block" data-ox-tex="\\int_0^1 x\\,dx"><math display="block"><mtext>\\int_0^1 x\\,dx</mtext></math></div>',
    );
    expect(html).toContain("ox-math-block");
    expect(html).toContain("katex-display");
  });
});

describe("copyKatexAssets", () => {
  it("copies CSS and fonts from the installed katex package", async () => {
    const dir = await mkdtemp(join(tmpdir(), "ox-katex-"));
    try {
      const written = await copyKatexAssets(dir);
      expect(written.length).toBeGreaterThan(0);
      const css = await readFile(join(dir, "__ox_katex__", "katex.min.css"), "utf8");
      expect(css).toContain("katex");
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
