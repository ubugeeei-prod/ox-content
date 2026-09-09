// Run from npm/vite-plugin-ox-content; see pipeline-extensions.md.
import { createHash } from "node:crypto";
import { performance } from "node:perf_hooks";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
const { cases } = await import(pathToFileURL(resolve(process.argv[2])).href);
for (const workload of cases) {
  if (process.env.OX_EXTENSION_FILTER && !workload.name.includes(process.env.OX_EXTENSION_FILTER))
    continue;
  const output = await workload.run(workload.input);
  if (workload.validate && !workload.validate(output))
    throw new Error(`Inactive fixture: ${workload.name}`);
  const expected = JSON.stringify(output);
  const samples = [];
  for (let sample = -2; sample < 9; sample++) {
    const start = performance.now();
    for (let i = 0; i < 5; i++) await workload.run(workload.input);
    if (sample >= 0) samples.push((performance.now() - start) / 5);
  }
  if (JSON.stringify(await workload.run(workload.input)) !== expected)
    throw new Error(`Unstable output: ${workload.name}`);
  console.log(
    JSON.stringify({
      name: workload.name,
      bytes: Buffer.byteLength(workload.input),
      samples_ms: samples,
      median_ms: [...samples].sort((a, b) => a - b)[4],
      sha256: createHash("sha256").update(expected).digest("hex"),
    }),
  );
}
