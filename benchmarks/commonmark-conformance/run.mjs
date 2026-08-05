#!/usr/bin/env node

/**
 * CommonMark conformance sweep for every engine in the speed benchmark.
 *
 * The speed tables put strict and deliberately-non-strict parsers in one
 * ranking, which is only a fair comparison if the reader can see which is
 * which. This measures that instead of asserting it: every engine is run over
 * the same vendored CommonMark 0.31.2 `spec.txt` the Rust conformance suite
 * uses, and its HTML is compared to the spec's expected output.
 *
 * Both sides of the comparison pass through the normalizer the in-repo
 * conformance suite uses, so engines are ranked by whether they implement
 * CommonMark rather than by HTML spelling. See `conformance.rs` for why a
 * byte-exact comparison does not answer the question this table asks.
 *
 * Usage:
 *   node benchmarks/commonmark-conformance/run.mjs
 *   node benchmarks/commonmark-conformance/run.mjs --json results.json
 *
 * The committed `results.json` feeds the "CommonMark" column in the README and
 * docs benchmark tables via `scripts/render-benchmark-tables.mjs`.
 */

import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { collectJsRenderers } from "./engines.mjs";
import { parseSpec } from "./spec-txt.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..", "..");

const SPEC_VERSION = "0.31.2";
const SPEC_PATH = join(
  ROOT,
  "crates",
  "ox_content_renderer",
  "tests",
  "spec_fixtures",
  `commonmark-${SPEC_VERSION}-spec.txt`,
);

const NATIVE_DIR = join(ROOT, "benchmarks", "native-competitors");
const NATIVE_MANIFEST = join(NATIVE_DIR, "Cargo.toml");
const NATIVE_TARGET_DIR = process.env.CARGO_TARGET_DIR
  ? resolve(__dirname, process.env.CARGO_TARGET_DIR)
  : join(NATIVE_DIR, "target");
const NATIVE_BINARY = join(
  NATIVE_TARGET_DIR,
  "release",
  process.platform === "win32"
    ? "ox-content-native-competitors.exe"
    : "ox-content-native-competitors",
);

const BUN_SCRIPT = join(__dirname, "run-bun.mjs");

function parseArgs(argv) {
  const opts = { jsonPath: null };
  for (let index = 0; index < argv.length; index++) {
    if (argv[index] === "--json") {
      opts.jsonPath = argv[++index] ?? join(__dirname, "results.json");
      continue;
    }
    throw new Error(`Unknown argument: ${argv[index]}`);
  }
  return opts;
}

/**
 * Renders every spec example with one engine.
 *
 * A throwing engine records an empty string rather than aborting the sweep:
 * refusing to parse an input the spec defines is a conformance result, not a
 * harness error, and an empty rendering never matches a non-empty expectation.
 */
function renderAll(render, examples) {
  return examples.map((example) => {
    try {
      return String(render(example.markdown));
    } catch {
      return "";
    }
  });
}

/**
 * Normalizes HTML through the native binary's `--normalize` filter, so JS
 * engines are judged by the same equivalence rule as the native ones and as the
 * in-repo conformance suite.
 *
 * Records are framed as `<byte-length>\n<bytes>` in both directions, which
 * keeps the protocol independent of HTML that contains newlines and quotes.
 */
function normalizeBatch(values) {
  const encoder = new TextEncoder();
  const payload = Buffer.concat(
    values.flatMap((value) => {
      const bytes = Buffer.from(encoder.encode(value));
      return [Buffer.from(`${bytes.length}\n`), bytes];
    }),
  );

  const run = spawnSync(NATIVE_BINARY, ["--normalize"], {
    input: payload,
    maxBuffer: 512 * 1024 * 1024,
  });
  if (run.status !== 0) {
    throw new Error(`normalize filter failed: ${String(run.stderr ?? "").trim()}`);
  }

  const out = run.stdout;
  const results = [];
  let cursor = 0;
  while (cursor < out.length) {
    const newline = out.indexOf(0x0a, cursor);
    if (newline === -1) break;
    const length = Number.parseInt(out.subarray(cursor, newline).toString("utf8"), 10);
    const bodyStart = newline + 1;
    results.push(out.subarray(bodyStart, bodyStart + length).toString("utf8"));
    cursor = bodyStart + length;
  }

  if (results.length !== values.length) {
    throw new Error(`normalize filter returned ${results.length} of ${values.length} records`);
  }
  return results;
}

/**
 * Renders the Bun-only engine through a helper subprocess, mirroring how the
 * speed benchmark reaches `Bun.markdown`. Returns its raw HTML per example;
 * normalization and scoring happen in `main` alongside every other engine.
 */
function loadBunRenderings() {
  const version = spawnSync("bun", ["--version"], { cwd: __dirname, encoding: "utf8" });
  if (version.status !== 0) {
    console.warn("bun not available, skipping Bun.markdown");
    return null;
  }

  const run = spawnSync("bun", [BUN_SCRIPT], {
    cwd: __dirname,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (run.status !== 0) {
    console.warn(`Bun conformance helper failed: ${(run.stderr ?? "").trim()}`);
    return null;
  }

  try {
    return JSON.parse(run.stdout);
  } catch (error) {
    console.warn(`Failed to parse Bun conformance output: ${String(error)}`);
    return null;
  }
}

/**
 * Builds the native competitor crate. Its binary supplies both the native
 * engine scores and the `--normalize` filter every engine is judged through, so
 * the sweep cannot run without it.
 */
function buildNativeBinary() {
  const version = spawnSync("cargo", ["--version"], { cwd: __dirname, encoding: "utf8" });
  if (version.status !== 0) {
    throw new Error(
      "cargo is required: the sweep scores every engine through the native binary's --normalize filter",
    );
  }

  const build = spawnSync(
    "cargo",
    ["build", "--release", "--quiet", "--manifest-path", NATIVE_MANIFEST],
    { cwd: __dirname, encoding: "utf8" },
  );
  if (build.status !== 0) {
    throw new Error(`Failed to build native competitors: ${(build.stderr ?? "").trim()}`);
  }
}

/**
 * Runs the native Rust engines through the standalone competitor crate, the
 * same binary the speed benchmark shells out to.
 */
function loadNativeResults() {
  const run = spawnSync(NATIVE_BINARY, ["--conformance", SPEC_PATH], {
    cwd: __dirname,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (run.status !== 0) {
    console.warn(`Failed to run native conformance: ${(run.stderr ?? "").trim()}`);
    return [];
  }

  try {
    return JSON.parse(run.stdout).results ?? [];
  } catch (error) {
    console.warn(`Failed to parse native conformance output: ${String(error)}`);
    return [];
  }
}

async function main() {
  const opts = parseArgs(process.argv.slice(2));
  const examples = parseSpec(readFileSync(SPEC_PATH, "utf8"));

  console.log(`CommonMark ${SPEC_VERSION} conformance — ${examples.length} examples\n`);

  buildNativeBinary();
  const expected = normalizeBatch(examples.map((example) => example.html));

  const rendered = (await collectJsRenderers()).map(([name, render]) => [
    name,
    renderAll(render, examples),
  ]);

  const bun = loadBunRenderings();
  if (bun) {
    // Scoring compares by index and divides by `examples.length`, so a short or
    // shifted helper payload would publish a wrong percentage instead of failing.
    if (!Array.isArray(bun.rendered) || bun.rendered.length !== examples.length) {
      throw new Error(
        `Bun helper returned ${Array.isArray(bun.rendered) ? bun.rendered.length : "no"} renderings for ${examples.length} examples`,
      );
    }
    rendered.push([bun.name, bun.rendered]);
  }

  const results = rendered.map(([name, htmls]) => {
    const actual = normalizeBatch(htmls);
    return {
      name,
      passed: actual.filter((html, index) => html === expected[index]).length,
      total: examples.length,
    };
  });

  results.push(...loadNativeResults());

  results.sort((a, b) => b.passed - a.passed || a.name.localeCompare(b.name));

  const width = Math.max(...results.map((r) => r.name.length));
  for (const result of results) {
    const percent = ((result.passed / result.total) * 100).toFixed(1);
    console.log(
      `${result.name.padEnd(width)}  ${String(result.passed).padStart(3)}/${result.total}  ${percent.padStart(5)}%`,
    );
  }

  if (opts.jsonPath) {
    const report = {
      spec: { name: "CommonMark", version: SPEC_VERSION, examples: examples.length },
      comparison: "normalized",
      engines: Object.fromEntries(
        results.map(({ name, passed, total }) => [name, { passed, total }]),
      ),
    };
    writeFileSync(opts.jsonPath, JSON.stringify(report, null, 2) + "\n");
    console.log(`\nWrote ${opts.jsonPath}`);
  }
}

await main();
