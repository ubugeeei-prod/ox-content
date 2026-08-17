#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import { join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

type Registry = {
  label: string;
  tokenEnv: string;
  args: (vsix: string, token: string) => string[];
};

// Marketplace outages surface as a hard failure on the very first VSIX (Azure
// DevOps returns TF10216), which would otherwise leave a release with only
// some platforms published. Both publishers run with --skip-duplicate, so a
// retry after an attempt that actually landed is skipped rather than failing.
const maxAttempts = 6;
const initialDelayMs = 30_000;
const maxDelayMs = 300_000;
const vsixDir = "dist/vscode";

const registries: Record<string, Registry> = {
  "vscode-marketplace": {
    label: "VS Code Marketplace",
    tokenEnv: "VSCE_PAT",
    args: (vsix) => [
      "exec",
      "--",
      "pnpm",
      "dlx",
      "@vscode/vsce",
      "publish",
      "--skip-duplicate",
      "--packagePath",
      vsix,
    ],
  },
  "open-vsx": {
    label: "Open VSX",
    tokenEnv: "OVSX_PAT",
    args: (vsix, token) => [
      "exec",
      "--",
      "pnpm",
      "dlx",
      "ovsx",
      "publish",
      vsix,
      "--skip-duplicate",
      "-p",
      token,
    ],
  },
};

const registry = registries[process.argv[2] ?? ""];
if (!registry) {
  console.error(`Usage: publish-editor-extensions.ts <${Object.keys(registries).join("|")}>`);
  process.exit(2);
}

// A packaging-only workflow_dispatch builds the VSIXs without shipping them.
if (
  process.env.GITHUB_EVENT_NAME === "workflow_dispatch" &&
  process.env.PUBLISH_FROM_DISPATCH !== "true"
) {
  console.log(`Packaging-only dispatch; skipping ${registry.label} publish.`);
  process.exit(0);
}

const token = process.env[registry.tokenEnv];
if (!token) {
  console.log(
    `::warning::${registry.tokenEnv} is not configured; skipping ${registry.label} publish.`,
  );
  process.exit(0);
}

const packages = listVsixPackages();
console.log(
  `Publishing ${packages.length} package(s) to ${registry.label}: ${packages
    .map((vsix) => vsix.slice(vsixDir.length + 1))
    .join(", ")}`,
);

for (const vsix of packages) {
  await publishWithRetry(registry, vsix, token);
}

function listVsixPackages(): string[] {
  let entries: string[] = [];
  try {
    entries = readdirSync(vsixDir);
  } catch (error) {
    console.error(`::error::Cannot read ${vsixDir}: ${(error as Error).message}`);
    process.exit(1);
  }

  const found = entries
    .filter((name) => name.endsWith(".vsix"))
    .sort()
    .map((name) => join(vsixDir, name));

  if (found.length === 0) {
    console.error(`::error::No .vsix packages found in ${vsixDir}`);
    process.exit(1);
  }

  return found;
}

async function publishWithRetry(target: Registry, vsix: string, pat: string): Promise<void> {
  let delay = initialDelayMs;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    const result = spawnSync("vp", target.args(vsix, pat), { stdio: "inherit" });

    if (result.status === 0) {
      return;
    }

    const reason = result.error
      ? result.error.message
      : `exit code ${result.status ?? `signal ${result.signal}`}`;

    if (attempt === maxAttempts) {
      console.error(
        `::error::${target.label} publish of ${vsix} failed after ${maxAttempts} attempts (${reason})`,
      );
      process.exit(1);
    }

    console.log(
      `::warning::${target.label} publish of ${vsix} failed (${reason}); attempt ${attempt}/${maxAttempts}, retrying in ${delay / 1000}s`,
    );
    await sleep(delay);
    delay = Math.min(delay * 2, maxDelayMs);
  }
}
