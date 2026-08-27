#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { appendFileSync, existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const runnerTemp = requiredEnv("RUNNER_TEMP");
const commentPath = join(runnerTemp, "benchmark-comment.md");

// The artifact rows are optional: a run that skipped the native build has
// nothing to compare, and a half-present pair would be worse than no section.
const baseArtifacts = join(runnerTemp, "artifacts-base.json");
const headArtifacts = join(runnerTemp, "artifacts-head.json");
const artifactArgs =
  existsSync(baseArtifacts) && existsSync(headArtifacts)
    ? ["--base-artifacts", baseArtifacts, "--head-artifacts", headArtifacts]
    : [];

const status = run("node", [
  "benchmarks/bundle-size/compare-pr-benchmark.mjs",
  "--base",
  join(runnerTemp, "benchmark-base.json"),
  "--head",
  join(runnerTemp, "benchmark-head.json"),
  "--base-bundle",
  join(runnerTemp, "bundle-base.json"),
  "--head-bundle",
  join(runnerTemp, "bundle-head.json"),
  ...artifactArgs,
  "--output",
  commentPath,
  "--base-sha",
  requiredEnv("BASE_SHA"),
  "--head-sha",
  requiredEnv("HEAD_SHA"),
]);

const budgetStatus = run("node", [
  "benchmarks/bundle-size/check-budgets.mjs",
  "--budgets",
  "benchmarks/perf-budgets.json",
  "--bundle",
  join(runnerTemp, "bundle-head.json"),
  "--runtime",
  join(runnerTemp, "benchmark-head.json"),
  "--append",
  commentPath,
]);

if (process.env.GITHUB_STEP_SUMMARY) {
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${readFileSync(commentPath, "utf8")}\n`);
}

if (status !== 0 || budgetStatus !== 0) {
  process.exit(status !== 0 ? status : budgetStatus);
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
  return result.status ?? 1;
}
