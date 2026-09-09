// Run with node tools/benchmarks/budoux-protected.mjs [source-module.ts].
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";
const moduleUrl = process.argv[2]
  ? pathToFileURL(process.argv[2])
  : new URL("../../npm/vite-plugin-ox-content/src/budoux.ts", import.meta.url);
const { transformBudouxHtml } = await import(moduleUrl);
const parser = { parse: (text) => text.split(" ") };
const options = { enabled: true, language: "ja", separator: "|", parser };
for (const count of [0, 10, 100, 1000]) {
  const html = count
    ? "<p>visible prose</p><code>protected text</code><svg><text>diagram text</text></svg>".repeat(
        count,
      )
    : "<p>visible prose</p>".repeat(1000);
  const expected = html.replaceAll("visible prose", "visible|prose");
  const runs = [];
  for (let sample = -2; sample < 9; sample++) {
    const start = performance.now();
    for (let i = 0; i < 10; i++) {
      if ((await transformBudouxHtml(html, options)) !== expected)
        throw new Error("Output changed");
    }
    if (sample >= 0) runs.push((performance.now() - start) / 10);
  }
  console.log(
    JSON.stringify({
      protected_blocks: count * 2,
      bytes: Buffer.byteLength(html),
      samples_ms: runs,
    }),
  );
}
