#!/usr/bin/env node

import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { performance } from "node:perf_hooks";

const options = parseOptions(process.argv.slice(2));
const workspace = resolve(options.workspace);
const napi = await import(pathToFileURL(join(workspace, "crates/ox_content_napi/index.js")));
const { generateCollectionsModule } = await import(
  pathToFileURL(join(workspace, "npm/vite-plugin-ox-content/src/collections-runtime.ts"))
);
const { renderAllPages } = await import(
  pathToFileURL(join(workspace, "npm/vite-plugin-ox-content/dist/index.mjs"))
);
const sizes = {};

for (const size of [2_000, 10_000]) {
  const root = createCollectionFixture(size);
  try {
    const manifestOptions = {
      srcDir: root,
      extensions: [".md"],
      frontmatter: true,
      collections: [{ name: "docs", source: ["**/*.md"], include: [] }],
    };
    const firstManifest = napi.buildCollectionManifest(manifestOptions);
    assertCollectionSize(firstManifest, size);
    const manifestMs = median(
      Array.from({ length: options.runs }, () => {
        const started = performance.now();
        const manifest = napi.buildCollectionManifest(manifestOptions);
        const elapsed = performance.now() - started;
        assertCollectionSize(manifest, size);
        return elapsed;
      }),
    );

    const rows = createQueryRows(size);
    const code = generateCollectionsModule({ collections: { docs: rows } });
    const collections = await import(
      `data:text/javascript;base64,${Buffer.from(code).toString("base64")}`
    );
    const path = `/docs/${size - 1}`;

    // Prime the lazy path index before measuring steady-state page resolution.
    await collections.queryCollection("docs").path(path).first();
    const queries = {
      pathFirstMs: await measureMedian(
        () => collections.queryCollection("docs").path(path).first(),
        options.runs,
        1_000,
      ),
      likeCountMs: await measureMedian(
        () => collections.queryCollection("docs").where("path", "LIKE", "/docs/%").count(),
        options.runs,
        100,
      ),
      nestedCountMs: await measureMedian(
        () => collections.queryCollection("docs").where("frontmatter.draft", true).count(),
        options.runs,
        100,
      ),
      orderedPageMs: await measureMedian(
        () =>
          collections
            .queryCollection("docs")
            .where("category", "=", "group-3")
            .order("rank", "ASC")
            .limit(20)
            .all(),
        options.runs,
        100,
      ),
    };

    const themeRenderAllMs =
      size === 2_000
        ? await measureMedian(
            () =>
              renderAllPages(
                rows.map((row) => ({ ...row, html: "<p>body</p>", toc: [], url: row.path })),
                {
                  theme: ({ children }) => children,
                  siteName: "Docs",
                  base: "/",
                  nav: [],
                  pages: [],
                },
              ),
            options.runs,
            1,
          )
        : undefined;

    sizes[String(size)] = { manifestMs, queries, themeRenderAllMs };
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

writeFileSync(
  resolve(options.json),
  `${JSON.stringify(
    {
      name: "Collection Scale Benchmark",
      generatedAt: new Date().toISOString(),
      runs: options.runs,
      sizes,
    },
    null,
    2,
  )}\n`,
);

function createCollectionFixture(size) {
  const root = mkdtempSync(join(tmpdir(), "ox-collection-benchmark-"));
  for (let start = 0; start < size; start += 100) {
    const directory = join(root, `group-${Math.floor(start / 100)}`);
    mkdirSync(directory, { recursive: true });
    for (let index = start; index < Math.min(start + 100, size); index++) {
      writeFileSync(
        join(directory, `${index}.md`),
        `---\ntitle: Page ${index}\nrank: ${index}\n---\n# Page ${index}\n`,
      );
    }
  }
  return root;
}

function createQueryRows(size) {
  return Array.from({ length: size }, (_, index) => ({
    id: `docs/${index}`,
    collection: "docs",
    path: `/docs/${index}`,
    stem: `docs/${index}`,
    source: `docs/${index}.md`,
    extension: ".md",
    title: `Page ${index}`,
    category: `group-${index % 20}`,
    rank: size - index,
    frontmatter: { draft: index % 7 === 0 },
  }));
}

function assertCollectionSize(manifest, expected) {
  const actual = JSON.parse(manifest).collections.docs.length;
  if (actual !== expected) {
    throw new Error(`Expected ${expected} collection entries, received ${actual}`);
  }
}

async function measureMedian(execute, runs, iterations) {
  for (let index = 0; index < 10; index++) {
    await execute();
  }
  const samples = [];
  for (let run = 0; run < runs; run++) {
    const started = performance.now();
    for (let iteration = 0; iteration < iterations; iteration++) {
      await execute();
    }
    samples.push((performance.now() - started) / iterations);
  }
  return median(samples);
}

function median(values) {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.floor(sorted.length / 2)];
}

function parseOptions(args) {
  const parsed = { json: null, runs: 5, workspace: process.cwd() };
  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--json") {
      parsed.json = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--runs") {
      parsed.runs = Number.parseInt(readValue(args, ++index, arg), 10);
      continue;
    }
    if (arg === "--workspace") {
      parsed.workspace = readValue(args, ++index, arg);
      continue;
    }
    throw new Error(`Unknown argument: ${arg}`);
  }
  if (!parsed.json) {
    throw new Error("--json is required");
  }
  if (!Number.isSafeInteger(parsed.runs) || parsed.runs < 1) {
    throw new Error("--runs requires a positive integer");
  }
  return parsed;
}

function readValue(args, index, option) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${option} requires a value`);
  }
  return value;
}
