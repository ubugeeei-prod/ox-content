// Run with node tools/benchmarks/katex-formulas.mjs [source-module.ts].
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";
const moduleUrl = process.argv[2]
  ? pathToFileURL(process.argv[2])
  : new URL("../../npm/vite-plugin-ox-content/src/plugins/math.ts", import.meta.url);
const { renderKatexMath } = await import(moduleUrl);
for (const kind of ["absent", "repeated", "distinct", "mixed", "invalid"]) {
  const html =
    kind === "absent"
      ? "<p>Ordinary prose and code.</p>".repeat(128)
      : Array.from({ length: 128 }, (_, i) => {
          const tex =
            kind === "invalid"
              ? "\\notARealCommand"
              : `x_{${kind === "repeated" ? 0 : kind === "mixed" ? i % 8 : i}}^2 + \\frac{1}{2}`;
          return `<span class="ox-math ox-math-inline" data-ox-tex="${tex}" data-source-span="${i}:1">${tex}</span>`;
        }).join("");
  const expected = await renderKatexMath(html);
  if (["repeated", "distinct", "mixed"].includes(kind) && !expected.includes('class="katex"')) {
    throw new Error("KaTeX did not render the fixture");
  }
  const runs = [];
  for (let sample = -2; sample < 9; sample++) {
    const start = performance.now();
    for (let i = 0; i < 5; i++) {
      if ((await renderKatexMath(html)) !== expected) throw new Error("Output changed");
    }
    if (sample >= 0) runs.push((performance.now() - start) / 5);
  }
  console.log(
    JSON.stringify({ kind, bytes: Buffer.byteLength(html), samples_ms: runs, html: expected }),
  );
}
