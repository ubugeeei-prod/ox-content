#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFileSync, readdirSync } from "node:fs";
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

// pnpm 12 fails any install whose dependencies carry unapproved build scripts,
// and `pnpm dlx` installs outside the workspace, so the `allowBuilds` list in
// pnpm-workspace.yaml never reaches these trees. The approval is accepted only
// as a CLI flag; ovsx pulls vsce, so both registries need the same pair.
const allowBuildFlags = ["@vscode/vsce-sign", "keytar"].flatMap((name) => ["--allow-build", name]);

const registries: Record<string, Registry> = {
  "vscode-marketplace": {
    label: "VS Code Marketplace",
    tokenEnv: "VSCE_PAT",
    args: (vsix) => [
      "exec",
      "--",
      "pnpm",
      "dlx",
      ...allowBuildFlags,
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
      ...allowBuildFlags,
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

const releaseVersion = getReleaseVersion();
if (registry === registries["vscode-marketplace"] && isPrereleaseVersion(releaseVersion)) {
  console.log(
    `::notice::Skipping ${registry.label} publish for prerelease extension version ${releaseVersion}; VS Code Marketplace rejects prerelease semver strings.`,
  );
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

function getReleaseVersion(): string | null {
  const input = process.env.VERSION_INPUT?.trim();
  if (input) {
    return input;
  }

  const refName = process.env.GITHUB_REF_NAME?.trim();
  if (refName?.startsWith("v")) {
    return refName.slice(1);
  }

  try {
    const pkg = JSON.parse(readFileSync("npm/vscode-ox-content/package.json", "utf8")) as {
      version?: unknown;
    };
    return typeof pkg.version === "string" ? pkg.version : null;
  } catch {
    return null;
  }
}

function isPrereleaseVersion(version: string | null): boolean {
  return version !== null && /^\d+\.\d+\.\d+-/.test(version);
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
