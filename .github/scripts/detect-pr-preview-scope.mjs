#!/usr/bin/env node

import { readFileSync } from "node:fs";

const PACKAGE_PREVIEW_PATTERNS = [
  /^\.github\/workflows\/(?:nightly|publish)\.yml$/,
  /^\.github\/scripts\/detect-pr-preview-scope\.mjs$/,
  /^binding-packages\//,
  /^crates\//,
  /^npm\/[^/]+\/package\.json$/,
  /^npm\/[^/]+\/src\//,
  /^scripts\/(?:package-dry-run|verify-publish-targets)\.(?:mjs|ts|test\.ts)$/,
  /^Cargo\.(?:toml|lock)$/,
  /^package\.json$/,
  /^pnpm-lock\.yaml$/,
  /^vite\.config\.ts$/,
  /^\.node-version$/,
];

const TEST_ONLY_PATTERNS = [
  /^npm\/[^/]+\/src\/.*\.test\.[cm]?[jt]sx?$/,
  /^npm\/[^/]+\/src\/(?:.*\/)?__snapshots__\/.*\.snap$/,
];

const PREVIEW_NEUTRAL_PATTERNS = [
  /^\.github\/(?:ISSUE_TEMPLATE|PULL_REQUEST_TEMPLATE)\//,
  /^\.github\/scripts\/(?:detect-pr-benchmark-scope|detect-rust-doc-scope|run-pr-benchmark|compare-pr-benchmark)\.mjs$/,
  /^\.github\/workflows\/(?!nightly\.yml$|publish\.yml$)/,
  /^benchmarks\//,
  /^docs\//,
  /^examples\//,
  /^scripts\/(?:pr-preview-scope|rust-doc-scope)\.test\.ts$/,
  /^README\.md$/,
  /^CHANGELOG\.md$/,
  /^CHENGELOG\.md$/,
  /^LICENSE$/,
  /^AGENTS\.md$/,
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

const preview = files.some((file) => classifyFile(file));

console.error(
  `Package preview scope: preview=${formatBoolean(preview)} (${files.length} changed file${
    files.length === 1 ? "" : "s"
  })`,
);
console.log(`preview=${formatBoolean(preview)}`);

/**
 * @param {string} file
 * @returns {boolean}
 */
function classifyFile(file) {
  if (matchesAny(file, TEST_ONLY_PATTERNS)) {
    return false;
  }
  if (matchesAny(file, PACKAGE_PREVIEW_PATTERNS)) {
    return true;
  }
  if (matchesAny(file, PREVIEW_NEUTRAL_PATTERNS)) {
    return false;
  }

  // Unknown paths stay conservative. A new publishable surface should opt out
  // only after it is deliberately classified.
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
  console.log(`Usage: node .github/scripts/detect-pr-preview-scope.mjs <changed-files.txt>

Writes GitHub Actions outputs:
  preview=true|false`);
}
