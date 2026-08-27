#!/usr/bin/env node

import { readFileSync } from "node:fs";

const RUST_DOC_PATTERNS = [
  /^\.github\/workflows\/ci\.yml$/,
  /^\.github\/scripts\/detect-rust-doc-scope\.mjs$/,
  /^\.cargo\//,
  /^crates\//,
  /^Cargo\.(?:toml|lock)$/,
  /^rust-toolchain(?:\.toml)?$/,
  /^vite\.config\.ts$/,
];

const RUST_DOC_NEUTRAL_PATTERNS = [
  /^\.github\/(?:ISSUE_TEMPLATE|PULL_REQUEST_TEMPLATE)\//,
  /^\.github\/scripts\/(?!(?:detect-rust-doc-scope)\.mjs$)[^/]+\.mjs$/,
  /^\.github\/workflows\/(?!ci\.yml$)/,
  /^benchmarks\//,
  /^binding-packages\//,
  /^docs\//,
  /^editors\//,
  /^examples\//,
  /^npm\//,
  /^scripts\//,
  /^package\.json$/,
  /^pnpm-lock\.yaml$/,
  /^README\.md$/,
  /^CHANGELOG\.md$/,
  /^CHENGELOG\.md$/,
  /^LICENSE$/,
  /^AGENTS\.md$/,
  /^\.node-version$/,
  /\.(?:md|mdx|png|jpe?g|gif|webp|svg|avif)$/i,
];

const inputPath = process.argv[2];

if (!inputPath || process.argv.includes("--help") || process.argv.includes("-h")) {
  printUsage();
  process.exit(inputPath ? 0 : 2);
}

const files = readFileSync(inputPath, "utf8")
  .split(/\r?\n/)
  .map((file) => file.trim())
  .filter(Boolean);

const rustDocs = files.some((file) => classifyFile(file));

console.error(
  `Rust documentation scope: rust_docs=${formatBoolean(rustDocs)} (${files.length} changed file${
    files.length === 1 ? "" : "s"
  })`,
);
console.log(`rust_docs=${formatBoolean(rustDocs)}`);

/**
 * @param {string} file
 * @returns {boolean}
 */
function classifyFile(file) {
  if (matchesAny(file, RUST_DOC_PATTERNS)) {
    return true;
  }
  if (matchesAny(file, RUST_DOC_NEUTRAL_PATTERNS)) {
    return false;
  }

  // Unknown paths stay conservative. New Rust-facing surfaces can opt out
  // after they are deliberately classified.
  return true;
}

/**
 * @param {string} file
 * @param {readonly RegExp[]} patterns
 * @returns {boolean}
 */
function matchesAny(file, patterns) {
  return patterns.some((pattern) => pattern.test(file));
}

/**
 * @param {boolean} value
 * @returns {"true" | "false"}
 */
function formatBoolean(value) {
  return value ? "true" : "false";
}

function printUsage() {
  console.log(`Usage: node .github/scripts/detect-rust-doc-scope.mjs <changed-files.txt>

Writes GitHub Actions outputs:
  rust_docs=true|false`);
}
