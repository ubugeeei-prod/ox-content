#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, symlinkSync, writeFileSync } from "node:fs";
import { cpus, totalmem } from "node:os";
import { dirname, join, resolve } from "node:path";

const options = parseOptions(process.argv.slice(2));
const checkoutRoot = process.cwd();

if (options.skipRuntime && options.skipBundle) {
  writeSkippedRuntimeReport(options.runtimeJson);
  writeSkippedBundleReport(options.bundleJson);
  process.exit(0);
}

const sourceRoot = resolve(options.source ?? requiredEnv("GITHUB_WORKSPACE"));

for (const file of [
  "tools/benchmarks/mizchi-markdown-native.mjs",
  "tools/benchmarks/mizchi-markdown-native-template.mjs",
  "tools/benchmarks/bundle-size/parse-benchmark.mjs",
  "tools/benchmarks/bundle-size/parse-benchmark-bun.mjs",
  "tools/benchmarks/bundle-size/measure.mjs",
  "tools/benchmarks/bundle-size/measure-artifacts.mjs",
  "tools/benchmarks/native-competitors/Cargo.toml",
  "tools/benchmarks/native-competitors/Cargo.lock",
  "tools/benchmarks/native-competitors/src/bench.rs",
  "tools/benchmarks/native-competitors/src/cli.rs",
  "tools/benchmarks/native-competitors/src/conformance.rs",
  "tools/benchmarks/native-competitors/src/json.rs",
  "tools/benchmarks/native-competitors/src/main.rs",
]) {
  const from = join(sourceRoot, file);
  const to = join(checkoutRoot, file);
  mkdirSync(dirname(to), { recursive: true });
  copyFileSync(from, to);
}

run("vp", ["install"]);
linkLegacyBenchmarkDependencies();
run("vp", ["run", "build:npm"]);
if (options.skipRuntime) {
  writeSkippedRuntimeReport(options.runtimeJson);
} else {
  run("node", [
    "tools/benchmarks/bundle-size/parse-benchmark.mjs",
    "--runs",
    options.runs,
    "--json",
    options.runtimeJson,
  ]);
}
if (options.skipBundle) {
  writeSkippedBundleReport(options.bundleJson);
} else {
  run("node", [
    "tools/benchmarks/bundle-size/measure.mjs",
    "--skip-install",
    "--json",
    options.bundleJson,
  ]);
}

// Artifact sizes come after `build:npm`, so the native binding and the
// published JavaScript both exist to measure. Optional: a caller that does not
// ask for it simply gets no artifact section in the report.
if (options.artifactsJson) {
  run("node", [
    "tools/benchmarks/bundle-size/measure-artifacts.mjs",
    "--json",
    options.artifactsJson,
  ]);
}

/**
 * @param {string[]} args
 * @returns {{
 *   source: string | null;
 *   runtimeJson: string;
 *   bundleJson: string;
 *   runs: string;
 *   skipRuntime: boolean;
 *   skipBundle: boolean;
 * }}
 */
function parseOptions(args) {
  const parsed = {
    source: null,
    runtimeJson: null,
    bundleJson: null,
    artifactsJson: null,
    runs: process.env.OX_CONTENT_BENCHMARK_RUNS || "5",
    skipRuntime: false,
    skipBundle: false,
  };

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--source") {
      parsed.source = readOptionValue(args, ++index, "--source");
      continue;
    }
    if (arg === "--runtime-json") {
      parsed.runtimeJson = readOptionValue(args, ++index, "--runtime-json");
      continue;
    }
    if (arg === "--bundle-json") {
      parsed.bundleJson = readOptionValue(args, ++index, "--bundle-json");
      continue;
    }
    if (arg === "--artifacts-json") {
      parsed.artifactsJson = readOptionValue(args, ++index, "--artifacts-json");
      continue;
    }
    if (arg === "--runs") {
      parsed.runs = String(readPositiveIntegerOption(args, ++index, "--runs"));
      continue;
    }
    if (arg === "--skip-runtime") {
      parsed.skipRuntime = true;
      continue;
    }
    if (arg === "--skip-bundle") {
      parsed.skipBundle = true;
      continue;
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  if (!parsed.runtimeJson) {
    throw new Error("--runtime-json is required");
  }
  if (!parsed.bundleJson) {
    throw new Error("--bundle-json is required");
  }
  parsePositiveInteger(parsed.runs, "--runs");

  return parsed;
}

/**
 * @param {string[]} args
 * @param {number} index
 * @param {string} optionName
 * @returns {string}
 */
function readOptionValue(args, index, optionName) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${optionName} requires a value`);
  }

  return value;
}

/**
 * @param {string[]} args
 * @param {number} index
 * @param {string} optionName
 * @returns {number}
 */
function readPositiveIntegerOption(args, index, optionName) {
  return parsePositiveInteger(readOptionValue(args, index, optionName), optionName);
}

/**
 * @param {string} value
 * @param {string} optionName
 * @returns {number}
 */
function parsePositiveInteger(value, optionName) {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isSafeInteger(parsed) || parsed < 1 || String(parsed) !== value) {
    throw new Error(`${optionName} requires a positive integer`);
  }

  return parsed;
}

/**
 * @param {string} name
 * @returns {string}
 */
function requiredEnv(name) {
  const value = process.env[name];
  if (!value) {
    throw new Error(`${name} is required`);
  }

  return value;
}

function linkLegacyBenchmarkDependencies() {
  const benchmarkNodeModules = join(checkoutRoot, "tools/benchmarks/bundle-size/node_modules");
  const legacyNodeModules = join(checkoutRoot, "benchmarks/bundle-size/node_modules");

  if (!existsSync(benchmarkNodeModules) && existsSync(legacyNodeModules)) {
    mkdirSync(dirname(benchmarkNodeModules), { recursive: true });
    symlinkSync(legacyNodeModules, benchmarkNodeModules, "dir");
  }
}

/**
 * @param {string} command
 * @param {string[]} args
 */
function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: checkoutRoot,
    stdio: "inherit",
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

/**
 * @param {string} path
 */
function writeSkippedRuntimeReport(path) {
  writeFileSync(
    path,
    `${JSON.stringify(
      {
        name: "Parse/Render Speed Benchmark",
        generatedAt: new Date().toISOString(),
        skipped: true,
        runs: Number.parseInt(options.runs, 10),
        environment: collectEnvironment(),
        sizes: {},
      },
      null,
      2,
    )}\n`,
  );
}

/**
 * @param {string} path
 */
function writeSkippedBundleReport(path) {
  writeFileSync(
    path,
    `${JSON.stringify(
      {
        name: "Bundle Size Benchmark",
        generatedAt: new Date().toISOString(),
        skipped: true,
        environment: collectEnvironment(),
        results: [],
      },
      null,
      2,
    )}\n`,
  );
}

function collectEnvironment() {
  const cpuList = cpus();
  const firstCpu = cpuList[0];

  return {
    node: process.version,
    v8: process.versions.v8,
    platform: process.platform,
    arch: process.arch,
    ci: process.env.CI === "true",
    runnerName: process.env.RUNNER_NAME ?? null,
    runnerOs: process.env.RUNNER_OS ?? null,
    runnerArch: process.env.RUNNER_ARCH ?? null,
    runnerLabel: process.env.OX_CONTENT_BENCHMARK_RUNNER ?? null,
    cpuModel: firstCpu?.model ?? null,
    cpuCount: cpuList.length,
    totalMemoryGB: Number((totalmem() / 1024 ** 3).toFixed(2)),
  };
}
