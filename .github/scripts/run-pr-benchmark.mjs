#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const options = parseOptions(process.argv.slice(2));
const checkoutRoot = process.cwd();
const sourceRoot = resolve(options.source ?? requiredEnv("GITHUB_WORKSPACE"));

for (const file of [
  "benchmarks/bundle-size/parse-benchmark.mjs",
  "benchmarks/bundle-size/parse-benchmark-bun.mjs",
  "benchmarks/bundle-size/measure.mjs",
]) {
  const from = join(sourceRoot, file);
  const to = join(checkoutRoot, file);
  mkdirSync(dirname(to), { recursive: true });
  copyFileSync(from, to);
}

run("vp", ["install"]);
run("vp", ["run", "build:npm"]);
run("node", [
  "benchmarks/bundle-size/parse-benchmark.mjs",
  "--runs",
  "3",
  "--json",
  options.runtimeJson,
]);
run("node", ["benchmarks/bundle-size/measure.mjs", "--skip-install", "--json", options.bundleJson]);

/**
 * @param {string[]} args
 * @returns {{ source: string | null; runtimeJson: string; bundleJson: string }}
 */
function parseOptions(args) {
  const parsed = {
    source: null,
    runtimeJson: null,
    bundleJson: null,
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

    throw new Error(`Unknown argument: ${arg}`);
  }

  if (!parsed.runtimeJson) {
    throw new Error("--runtime-json is required");
  }
  if (!parsed.bundleJson) {
    throw new Error("--bundle-json is required");
  }

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
