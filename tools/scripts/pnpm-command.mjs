#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const stdioOptions = {
  encoding: "utf8",
  stdio: ["ignore", "pipe", "pipe"],
};

export function runPnpm(args) {
  const corepack = runCommand("corepack", ["pnpm", ...args]);
  if (!shouldUseFallback(corepack)) {
    return corepack;
  }

  const direct = runCommand("pnpm", args);
  if (!shouldUseFallback(direct)) {
    return direct;
  }

  return runCommand("npm", [
    "exec",
    "--yes",
    `--package=pnpm@${readPinnedPnpmVersion()}`,
    "--",
    "pnpm",
    ...args,
  ]);
}

function runCommand(command, args) {
  return spawnSync(command, args, stdioOptions);
}

function shouldUseFallback(result) {
  return result.error?.code === "ENOENT" || hasCorepackPnpmEntrypointMismatch(result);
}

function hasCorepackPnpmEntrypointMismatch(result) {
  if (result.status === 0) {
    return false;
  }

  return (
    result.stderr.includes("Cannot find module") &&
    result.stderr.includes("corepack") &&
    result.stderr.includes("pnpm") &&
    result.stderr.includes("bin/pnpm.cjs")
  );
}

function readPinnedPnpmVersion() {
  const packageJson = JSON.parse(readFileSync(resolve("package.json"), "utf8"));
  const match = /^pnpm@([^+]+)(?:\+.+)?$/.exec(packageJson.packageManager ?? "");
  if (!match) {
    throw new Error(
      `packageManager must pin pnpm, found: ${packageJson.packageManager ?? "unset"}`,
    );
  }
  return match[1];
}
