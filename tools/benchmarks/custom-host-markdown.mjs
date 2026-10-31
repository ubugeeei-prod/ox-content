// node tools/benchmarks/custom-host-markdown.mjs corpus.json /path/to/plugin [samples]
// Corpus: { root, options, documents: [{ documentPath, source, convertMdLinks? }] }.
// Prepare the corpus before timing. Keep it local when it contains consumer text.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [corpusFile, packageDirectory, sampleArgument = "9"] = process.argv.slice(2);
assert.ok(corpusFile && packageDirectory, "Pass a corpus JSON and a built plugin directory");
const samples = Number(sampleArgument);
assert.ok(Number.isSafeInteger(samples) && samples > 0, "Samples must be a positive integer");
const corpusBytes = await readFile(corpusFile);
const corpus = JSON.parse(corpusBytes);
assert.ok(corpus.root && Array.isArray(corpus.documents) && corpus.documents.length > 0);
for (const input of corpus.documents) {
  assert.equal(typeof input.source, "string");
  assert.equal(typeof input.documentPath, "string");
}
const packageRoot = resolve(packageDirectory);
const require = createRequire(join(packageRoot, "package.json"));
const { build } = await import(pathToFileURL(require.resolve("vite")).href);
const { createOxContentCustomHostPlugin } = await import(
  pathToFileURL(join(packageRoot, "dist/index.mjs")).href
);
const outDir = await mkdtemp(join(tmpdir(), "ox-custom-host-markdown-"));
const rows = [];
let expected;

try {
  await build({
    root: corpus.root,
    configFile: false,
    logLevel: "silent",
    publicDir: false,
    plugins: [
      {
        name: "benchmark-client",
        resolveId: (id) => (id === "virtual:benchmark-client" ? "\0benchmark-client" : null),
        load: (id) => (id === "\0benchmark-client" ? "export {};" : null),
      },
      createOxContentCustomHostPlugin({
        oxContent: { ...corpus.options, outDir },
        host: {
          async routes(host) {
            // One complete warmup; hashing and output checks stay outside timers.
            for (let index = 0; index <= samples; index++) {
              const rendered = [];
              const start = performance.now();
              for (const input of corpus.documents) {
                rendered.push(await host.markdown.render(input));
              }
              const milliseconds = performance.now() - start;
              const hash = createHash("sha256");
              let htmlBytes = 0;
              for (const result of rendered) {
                htmlBytes += Buffer.byteLength(result.html);
                // Include all structured fields, excluding unused JS module code.
                hash.update(
                  JSON.stringify({
                    ...result,
                    transform: { ...result.transform, code: undefined },
                  }),
                );
              }
              const sha256 = hash.digest("hex");
              expected ??= sha256;
              assert.equal(sha256, expected, "Rendered output changed between samples");
              if (index > 0) rows.push({ milliseconds, htmlBytes, sha256 });
            }
            return [];
          },
        },
      }),
    ],
    build: { outDir, rollupOptions: { input: "virtual:benchmark-client" } },
  });
} finally {
  await rm(outDir, { recursive: true, force: true });
}

const sorted = rows.map((row) => row.milliseconds).sort((a, b) => a - b);
console.log(
  JSON.stringify(
    {
      node: process.version,
      corpusSha256: createHash("sha256").update(corpusBytes).digest("hex"),
      documents: corpus.documents.length,
      inputBytes: corpus.documents.reduce((sum, input) => sum + Buffer.byteLength(input.source), 0),
      medianMilliseconds:
        (sorted[Math.floor((samples - 1) / 2)] + sorted[Math.floor(samples / 2)]) / 2,
      rows,
    },
    null,
    2,
  ),
);
