// Run with node --expose-gc tools/benchmarks/static-diagrams.mjs [source-module.ts].
// Use the same source module path and Node version for before/after comparisons.
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";
const moduleUrl = process.argv[2]
  ? pathToFileURL(process.argv[2])
  : new URL("../../npm/vite-plugin-ox-content/src/plugins/mermaid-protect.ts", import.meta.url);
const { protectStaticDiagramSvgs } = await import(moduleUrl);
const results = [];
for (const count of [0, 10, 100, 300]) {
  const source = (
    "<p>日本語の通常の段落: API documentation and configuration.</p>".repeat(10) +
    '<div class="ox-mermaid"><svg><foreignObject><div>diagram<br /></div></foreignObject></svg></div><figure class="ox-graphviz"><svg><g>graph</g></svg></figure>'
  ).repeat(count || 100);
  const html = count
    ? source
    : source.replaceAll("ox-mermaid", "unrelated").replaceAll("ox-graphviz", "unrelated");
  const runs = [];
  const iterations = count >= 100 ? 20 : 100;
  for (let sample = -2; sample < 9; sample++) {
    global.gc?.();
    const start = performance.now();
    for (let i = 0; i < iterations; i++) {
      const result = protectStaticDiagramSvgs(html);
      if (result.svgs.size !== count * 2) throw new Error("wrong diagram count");
    }
    if (sample >= 0) runs.push((performance.now() - start) / iterations);
  }
  results.push({
    diagrams: count * 2,
    bytes: Buffer.byteLength(html),
    iterations,
    samples_ms: runs,
    median_ms: [...runs].sort((a, b) => a - b)[4],
  });
}
console.log(JSON.stringify(results, null, 2));
