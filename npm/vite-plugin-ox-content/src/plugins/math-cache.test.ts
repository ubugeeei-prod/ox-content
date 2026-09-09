import { createRequire } from "node:module";
import { afterEach, describe, expect, it, vi } from "vite-plus/test";
import { renderKatexMath, type MathRenderFailure } from "./math";
const katex = createRequire(import.meta.url)("katex");
function placeholder(tex: string, block = false, span = "1:2") {
  const tag = block ? "div" : "span";
  return `<${tag} class="ox-math ox-math-${block ? "block" : "inline"}" data-ox-tex="${tex}" data-source-span="${span}">${tex}</${tag}>`;
}
afterEach(() => vi.restoreAllMocks());

describe("per-document formula reuse", () => {
  it("reuses repeated formulas by display mode while preserving every source span", async () => {
    const render = vi.spyOn(katex, "renderToString");
    const source =
      placeholder("x^2", false, "1:2") +
      placeholder("x^2", true, "3:4") +
      placeholder("x^2", false, "5:6") +
      placeholder("x^2", false, "7:8");
    const html = await renderKatexMath(source);
    expect(render).toHaveBeenCalledTimes(2);
    for (const span of ["1:2", "3:4", "5:6", "7:8"])
      expect(html).toContain(`data-source-span="${span}"`);
    expect(html.match(/class="ox-math ox-math-inline"/g)).toHaveLength(3);
    expect(html).toContain('class="ox-math ox-math-block"');
  });

  it.each(["literal", "render"] as const)(
    "reports each failed occurrence in %s mode",
    async (policy) => {
      const failures: MathRenderFailure[] = [];
      const render = vi.spyOn(katex, "renderToString");
      const html = await renderKatexMath(
        placeholder("\\notARealCommand").repeat(3),
        policy,
        failures,
      );
      expect(failures).toHaveLength(3);
      expect(failures[0]).toEqual(failures[1]);
      expect(render).toHaveBeenCalledTimes(policy === "literal" ? 1 : 2);
      expect(html).toContain(policy === "literal" ? "$\\notARealCommand$" : 'class="ox-math');
    },
  );

  it("records the first failure before throwing in error mode", async () => {
    const failures: MathRenderFailure[] = [];
    await expect(
      renderKatexMath(placeholder("\\notARealCommand").repeat(2), "error", failures),
    ).rejects.toThrow("notARealCommand");
    expect(failures).toHaveLength(1);
  });

  it("does not retain formula results between documents", async () => {
    const render = vi.spyOn(katex, "renderToString");
    await renderKatexMath(placeholder("x").repeat(5));
    await renderKatexMath(placeholder("x").repeat(5));
    expect(render).toHaveBeenCalledTimes(2);
  });

  it("keeps distinct formulas correct after the cache limit", async () => {
    const source =
      placeholder("x").repeat(2) +
      Array.from({ length: 160 }, (_, index) => placeholder(`x_{${index}}`)).join("");
    const html = await renderKatexMath(source);
    expect(html.match(/class="ox-math ox-math-inline"/g)).toHaveLength(162);
    expect(html).toContain("x_{159}");
  });
  it("does not retain renderings larger than the string budget", async () => {
    const render = vi.spyOn(katex, "renderToString").mockReturnValue("x".repeat(128 * 1024 + 1));
    await renderKatexMath(placeholder("x").repeat(3));
    expect(render).toHaveBeenCalledTimes(3);
  });
});
