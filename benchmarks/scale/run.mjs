#!/usr/bin/env node

/**
 * Scale benchmark.
 *
 * `bundle-size/build-time-benchmark.mjs` measures four-page apps against other
 * frameworks. That answers "is a small site fast", not "does a large one stay
 * fast" — and the cost that matters to a big project is the one that grows
 * faster than the page count. This suite measures growth, not just totals:
 *
 *   build     the same project at 4x the pages, so a superlinear build shows
 *             up as a ratio rather than a number nobody can calibrate
 *   og        OG image generation, every image a cache miss and then every
 *             image a cache hit, which is the difference between a cold CI
 *             build and a warm one
 *   features  the marginal cost of each builtin over the same corpus, so a
 *             feature that costs more than the whole rest of the pipeline is
 *             visible instead of averaged away
 *
 * Usage:
 *   node benchmarks/scale/run.mjs [--suite build|og|features|all]
 *                                 [--pages N] [--iterations N] [--json PATH]
 */

import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, rmSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import { cpus, totalmem } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { generateSite } from "./site.mjs";

const HERE = dirname(fileURLToPath(import.meta.url));
const CONTENT_DIR = join(HERE, "content");

/** Page counts for the growth measurement: each step is 4x the last. */
const BUILD_STEPS = [200, 800, 3200];

/**
 * Builtins measured one at a time. Each name is the `OX_SCALE_*` suffix the
 * config reads; the corpus already contains the syntax each one looks for.
 */
const FEATURES = [
  "HIGHLIGHT",
  "CODE_ANNOTATIONS",
  "AUTOLINKS",
  "TASK_LISTS",
  "STRIKETHROUGH",
  "MATH",
  "ATTRS",
  "WIKI_LINKS",
  "EMOJI",
  "CONTAINERS",
  "IMAGES",
  "IMAGE_GALLERIES",
  "TIMELINES",
  "DEFINITION_LISTS",
  "KEYBOARD_KEYS",
  "ABBREVIATIONS",
  "MAGIC_LINKS",
  "BADGES",
  "CARDS",
  "STEPS",
  "CODE_GROUPS",
  "FILE_TREE",
  "DATA_TABLES",
  "CONDITIONAL_BLOCKS",
  "CJK_EMPHASIS",
  "SEMANTIC_FOOTNOTES",
  "MDX",
  "SEARCH",
  "SITE_MAPS",
  "FEEDS",
  "THEME",
];

function parseOptions(argv) {
  const parsed = { suite: "all", pages: null, iterations: 3, jsonPath: null };

  for (let index = 0; index < argv.length; index++) {
    const arg = argv[index];
    const take = (name) => {
      const value = argv[++index];
      if (!value || value.startsWith("--")) {
        throw new Error(`${name} requires a value`);
      }
      return value;
    };

    if (arg === "--suite") {
      parsed.suite = take("--suite");
    } else if (arg === "--pages") {
      parsed.pages = Number(take("--pages"));
    } else if (arg === "--iterations") {
      parsed.iterations = Number(take("--iterations"));
    } else if (arg === "--json") {
      parsed.jsonPath = take("--json");
    } else if (arg === "--help" || arg === "-h") {
      console.log(
        "Usage: node benchmarks/scale/run.mjs [--suite build|og|features|all]\n" +
          "                                    [--pages N] [--iterations N] [--json PATH]",
      );
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  if (!["build", "og", "features", "all"].includes(parsed.suite)) {
    throw new Error(`Unknown suite: ${parsed.suite}`);
  }
  if (parsed.pages !== null && !Number.isInteger(parsed.pages)) {
    throw new Error("--pages must be an integer");
  }
  if (!Number.isInteger(parsed.iterations) || parsed.iterations < 1) {
    throw new Error("--iterations must be a positive integer");
  }

  return parsed;
}

/**
 * Runs one production build and returns its wall time in milliseconds.
 *
 * The output directory is removed first: a build that reuses the previous
 * result measures the cache, and the cache is what the `og` suite measures on
 * purpose rather than by accident.
 */
function build(env, outDir) {
  rmSync(join(HERE, outDir), { recursive: true, force: true });

  const started = process.hrtime.bigint();
  execFileSync("vp", ["build"], {
    cwd: HERE,
    stdio: "pipe",
    env: { ...process.env, ...env, OX_SCALE_OUT_DIR: outDir },
  });
  return Number(process.hrtime.bigint() - started) / 1e6;
}

/** Best of `iterations`, which is the run least disturbed by the machine. */
function fastestBuild(env, outDir, iterations) {
  let best = Infinity;
  for (let index = 0; index < iterations; index++) {
    best = Math.min(best, build(env, outDir));
  }
  return best;
}

async function suiteBuild(options) {
  const rows = [];
  let previous = null;

  for (const pages of options.pages ? [options.pages] : BUILD_STEPS) {
    await regenerate(pages);
    const ms = fastestBuild({}, "dist", options.iterations);
    const growth = previous ? ms / previous.ms : null;
    rows.push({ pages, ms, msPerPage: ms / pages, growth });
    previous = { pages, ms };
  }

  return rows;
}

async function suiteFeatures(options) {
  const pages = options.pages ?? 400;
  await regenerate(pages);

  const baseline = fastestBuild({}, "dist", options.iterations);
  const rows = [{ feature: "(baseline)", ms: baseline, deltaMs: 0, sharePct: 0 }];

  for (const feature of FEATURES) {
    const ms = fastestBuild({ [`OX_SCALE_${feature}`]: "1" }, "dist", options.iterations);
    rows.push({
      feature,
      ms,
      deltaMs: ms - baseline,
      sharePct: ((ms - baseline) / baseline) * 100,
    });
  }

  rows.sort((left, right) => right.deltaMs - left.deltaMs);
  return { pages, baseline, rows };
}

/** Recursively counts the PNG files under `dir`. */
function countPngs(dir) {
  let total = 0;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      total += countPngs(join(dir, entry.name));
    } else if (entry.name.endsWith(".png")) {
      total += 1;
    }
  }
  return total;
}

/**
 * Builds with OG images on and fails unless every image was written.
 *
 * A build whose renderer never starts still succeeds — failures are reported
 * per page, not thrown — and it looks spectacularly fast. Timing a run without
 * checking what it produced is how a broken renderer reads as a speedup.
 */
function buildOgImages(pages, env) {
  rmSync(join(HERE, ".cache", "og-images"), { recursive: true, force: true });
  const ms = build({ OX_SCALE_OG_IMAGE: "1", ...env }, "dist");
  const produced = countPngs(join(HERE, "dist"));
  if (produced !== pages) {
    throw new Error(
      `expected ${pages} OG images, found ${produced}. ` +
        "Install Chromium with `vp exec --filter @ox-content/vite-plugin -- playwright install chromium`.",
    );
  }
  return ms;
}

async function suiteOg(options) {
  const pages = options.pages ?? 100;
  await regenerate(pages);

  const without = fastestBuild({}, "dist", options.iterations);

  // The first build writes every image; the second finds all of them cached.
  const cold = buildOgImages(pages, {});
  const warm = build({ OX_SCALE_OG_IMAGE: "1" }, "dist");

  const concurrency = [];
  for (const workers of [1, 2, 4, 8]) {
    const ms = buildOgImages(pages, { OX_SCALE_OG_CONCURRENCY: String(workers) });
    concurrency.push({ workers, ms, perImage: (ms - without) / pages });
  }

  return {
    pages,
    without,
    cold,
    warm,
    coldPerImage: (cold - without) / pages,
    warmPerImage: (warm - without) / pages,
    cacheSpeedup: cold / warm,
    concurrency,
  };
}

async function regenerate(pages) {
  rmSync(CONTENT_DIR, { recursive: true, force: true });
  await mkdir(CONTENT_DIR, { recursive: true });
  await generateSite(HERE, pages, {
    containers: true,
    images: true,
    math: true,
    mermaid: false,
  });
}

function table(header, rows) {
  const widths = header.map((name, column) =>
    Math.max(name.length, ...rows.map((row) => String(row[column]).length)),
  );
  const line = (cells) =>
    cells.map((cell, column) => String(cell).padEnd(widths[column])).join("  ");
  console.log(line(header));
  console.log(widths.map((width) => "-".repeat(width)).join("  "));
  for (const row of rows) {
    console.log(line(row));
  }
}

async function main() {
  const options = parseOptions(process.argv.slice(2));

  if (!existsSync(join(HERE, "node_modules"))) {
    throw new Error(
      "benchmarks/scale has no node_modules; run `vp install` from the repository root first",
    );
  }

  const results = {
    machine: { cpus: cpus().length, memoryGb: Math.round(totalmem() / 1024 ** 3) },
    iterations: options.iterations,
  };

  if (options.suite === "build" || options.suite === "all") {
    results.build = await suiteBuild(options);
    console.log("\n## Build scaling\n");
    table(
      ["pages", "ms", "ms/page", "growth"],
      results.build.map((row) => [
        row.pages,
        row.ms.toFixed(0),
        row.msPerPage.toFixed(2),
        row.growth ? `x${row.growth.toFixed(2)}` : "-",
      ]),
    );
    console.log("\nEach step is 4x the pages: linear is about x4, quadratic about x16.");
  }

  if (options.suite === "og" || options.suite === "all") {
    results.og = await suiteOg(options);
    const og = results.og;
    console.log(`\n## OG images (${og.pages} pages)\n`);
    table(
      ["case", "build ms", "ms/image"],
      [
        ["no OG images", og.without.toFixed(0), "-"],
        ["cold cache", og.cold.toFixed(0), og.coldPerImage.toFixed(1)],
        ["warm cache", og.warm.toFixed(0), og.warmPerImage.toFixed(1)],
      ],
    );
    console.log(`\nWhole build, cold against warm: x${og.cacheSpeedup.toFixed(1)}`);
    console.log("\n### Renderer concurrency (cold cache)\n");
    table(
      ["workers", "build ms", "ms/image"],
      og.concurrency.map((row) => [row.workers, row.ms.toFixed(0), row.perImage.toFixed(1)]),
    );
  }

  if (options.suite === "features" || options.suite === "all") {
    results.features = await suiteFeatures(options);
    console.log(`\n## Builtin features (${results.features.pages} pages)\n`);
    table(
      ["feature", "ms", "delta ms", "delta %"],
      results.features.rows.map((row) => [
        row.feature,
        row.ms.toFixed(0),
        row.deltaMs.toFixed(0),
        `${row.sharePct.toFixed(1)}%`,
      ]),
    );
  }

  if (options.jsonPath) {
    await writeFile(options.jsonPath, `${JSON.stringify(results, null, 2)}\n`);
    console.log(`\nWrote ${options.jsonPath}`);
  }
}

await main();
