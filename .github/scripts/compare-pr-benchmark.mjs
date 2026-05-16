#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { join } from "node:path";

const runnerTemp = requiredEnv("RUNNER_TEMP");
const commentPath = join(runnerTemp, "benchmark-comment.md");

run("node", [
  "benchmarks/bundle-size/compare-pr-benchmark.mjs",
  "--base",
  join(runnerTemp, "benchmark-base.json"),
  "--head",
  join(runnerTemp, "benchmark-head.json"),
  "--base-bundle",
  join(runnerTemp, "bundle-base.json"),
  "--head-bundle",
  join(runnerTemp, "bundle-head.json"),
  "--output",
  commentPath,
  "--base-sha",
  requiredEnv("BASE_SHA"),
  "--head-sha",
  requiredEnv("HEAD_SHA"),
]);

if (process.env.GITHUB_STEP_SUMMARY) {
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${readFileSync(commentPath, "utf8")}\n`);
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
  const result = spawnSync(command, args, { stdio: "inherit" });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
