#!/usr/bin/env bun

/**
 * Bun-only half of the CommonMark conformance sweep.
 *
 * `Bun.markdown` has no Node build, so it renders in a Bun subprocess and the
 * output is merged back by `run.mjs` — the same split the speed benchmark uses
 * for its Bun row. Only rendering happens here; normalization and scoring stay
 * in `run.mjs` so this engine is judged by exactly the same rule as the others.
 *
 * Emits `{"name": ..., "rendered": [html, ...]}` on stdout.
 */

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { parseSpec } from "./spec-txt.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..", "..");

const SPEC_PATH = join(
  ROOT,
  "crates",
  "ox_content_renderer",
  "tests",
  "spec_fixtures",
  "commonmark-0.31.2-spec.txt",
);

if (typeof Bun === "undefined" || typeof Bun.markdown?.html !== "function") {
  throw new Error("Bun.markdown is not available");
}

const rendered = parseSpec(readFileSync(SPEC_PATH, "utf8")).map((example) => {
  try {
    return String(Bun.markdown.html(example.markdown));
  } catch {
    return "";
  }
});

console.log(JSON.stringify({ name: "Bun.markdown.html", rendered }));
