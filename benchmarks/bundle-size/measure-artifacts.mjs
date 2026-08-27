#!/usr/bin/env node
// Usage: node benchmarks/bundle-size/measure-artifacts.mjs [--json <path>]
//
// Records the size of the artifacts a Rust migration moves bytes between: the
// native binding, the wasm package, and the JavaScript that ships beside them.
// Moving a rule into Rust grows the first and shrinks the last, and #1075 asks
// that a migration show both halves of that trade rather than only its wins.

import { createRequire } from "node:module";
import { readFileSync, existsSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join, dirname, extname } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const { gzipSizeSync } = require("gzip-size");

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, "..", "..");

/** Artifacts a migration moves bytes between. */
const TARGETS = [
  { name: "napi binding", path: "crates/ox_content_napi", match: (f) => extname(f) === ".node" },
  {
    name: "wasm package",
    path: "crates/ox_content_wasm/pkg",
    match: (f) => extname(f) === ".wasm",
  },
  {
    name: "vite-plugin js",
    path: "npm/vite-plugin-ox-content/dist",
    match: (f) => [".js", ".cjs", ".mjs"].includes(extname(f)),
  },
];

/** Every matching file below `dir`, recursively. */
function walk(dir, match, out = []) {
  if (!existsSync(dir)) return out;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) walk(full, match, out);
    else if (match(entry.name)) out.push(full);
  }
  return out;
}

/**
 * Raw and gzipped bytes for one target.
 *
 * A missing directory reports `present: false` rather than zero: a target that
 * was never built and a target that shrank to nothing are different facts, and
 * a zero would read as a spectacular win in the delta table.
 */
function measure(target) {
  const dir = join(ROOT, target.path);
  const files = walk(dir, target.match);
  if (files.length === 0) {
    return { name: target.name, present: false, files: 0, raw: 0, gzipped: 0 };
  }
  let raw = 0;
  let gzipped = 0;
  for (const file of files) {
    const content = readFileSync(file);
    raw += statSync(file).size;
    gzipped += gzipSizeSync(content);
  }
  return { name: target.name, present: true, files: files.length, raw, gzipped };
}

const report = { targets: TARGETS.map(measure) };

const jsonIndex = process.argv.indexOf("--json");
const output = JSON.stringify(report, null, 2);
if (jsonIndex !== -1 && process.argv[jsonIndex + 1]) {
  writeFileSync(process.argv[jsonIndex + 1], `${output}\n`);
  process.stdout.write(`Wrote artifact sizes to ${process.argv[jsonIndex + 1]}\n`);
} else {
  process.stdout.write(`${output}\n`);
}
