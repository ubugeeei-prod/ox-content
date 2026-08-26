#!/usr/bin/env node

import { readFileSync } from "node:fs";

const RUNTIME_AND_BUNDLE_PATTERNS = [
  /^\.github\/workflows\/benchmark\.yml$/,
  /^\.github\/scripts\/(?:detect-pr-benchmark-scope|run-pr-benchmark|compare-pr-benchmark)\.mjs$/,
  /^benchmarks\/bundle-size\/(?:parse-benchmark|parse-benchmark-bun|compare-pr-benchmark)\.(?:mjs|test\.ts)$/,
  /^benchmarks\/native-competitors\//,
  /^crates\/(?:ox_content_allocator|ox_content_napi|ox_content_parser|ox_content_renderer|ox_content_wasm)\//,
  /^Cargo\.(?:toml|lock)$/,
  /^package\.json$/,
  /^pnpm-lock\.yaml$/,
  /^vite\.config\.ts$/,
  /^\.node-version$/,
];

const RUNTIME_PATTERNS = [
  /^benchmarks\/commonmark-conformance\//,
  /^benchmarks\/polonius-borrowck\//,
];

const TEST_ONLY_PATTERNS = [
  /^npm\/[^/]+\/src\/.*\.test\.[cm]?[jt]sx?$/,
  /^npm\/[^/]+\/src\/(?:.*\/)?__snapshots__\/.*\.snap$/,
];

const BUNDLE_PATTERNS = [
  /^benchmarks\/perf-budgets\.json$/,
  /^benchmarks\/bundle-size\//,
  /^crates\//,
  /^npm\//,
  /^scripts\/(?:theme-colors|theme-skins)\//,
];

const BENCHMARK_NEUTRAL_PATTERNS = [
  /^\.github\/(?:ISSUE_TEMPLATE|PULL_REQUEST_TEMPLATE)\//,
  /^\.github\/scripts\/(?!(?:detect-pr-benchmark-scope|run-pr-benchmark|compare-pr-benchmark)\.mjs$)[^/]+\.mjs$/,
  /^\.github\/workflows\/(?!benchmark\.yml$)/,
  /^docs\//,
  /^examples\//,
  /^scripts\/.*\.test\.ts$/,
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

const scope = files.reduce(
  (current, file) => {
    const fileScope = classifyFile(file);
    return {
      runtime: current.runtime || fileScope.runtime,
      bundle: current.bundle || fileScope.bundle,
    };
  },
  { runtime: false, bundle: false },
);

console.error(
  `Benchmark scope: runtime=${formatBoolean(scope.runtime)} bundle=${formatBoolean(
    scope.bundle,
  )} (${files.length} changed file${files.length === 1 ? "" : "s"})`,
);
console.log(`runtime=${formatBoolean(scope.runtime)}`);
console.log(`bundle=${formatBoolean(scope.bundle)}`);

/**
 * @param {string} file
 * @returns {{ runtime: boolean; bundle: boolean }}
 */
function classifyFile(file) {
  if (matchesAny(file, RUNTIME_AND_BUNDLE_PATTERNS)) {
    return { runtime: true, bundle: true };
  }
  if (matchesAny(file, RUNTIME_PATTERNS)) {
    return { runtime: true, bundle: false };
  }
  if (matchesAny(file, TEST_ONLY_PATTERNS)) {
    return { runtime: false, bundle: false };
  }
  if (matchesAny(file, BUNDLE_PATTERNS)) {
    return { runtime: false, bundle: true };
  }
  if (matchesAny(file, BENCHMARK_NEUTRAL_PATTERNS)) {
    return { runtime: false, bundle: false };
  }

  // Unknown paths stay conservative. This keeps new source trees protected
  // until they are deliberately classified.
  return { runtime: true, bundle: true };
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
  console.log(`Usage: node .github/scripts/detect-pr-benchmark-scope.mjs <changed-files.txt>

Writes GitHub Actions outputs:
  runtime=true|false
  bundle=true|false`);
}
