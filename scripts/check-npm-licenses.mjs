#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const policy = JSON.parse(readFileSync(resolve("config/dependency-policy.json"), "utf8"));
const allowedLicenses = new Set(policy.licenses?.allowed ?? []);
const exceptions = new Set(
  (policy.licenses?.exceptions ?? []).map((item) => `${item.name}\0${item.license}`),
);
const report = runLicenseList();
const violations = [];
const licenseGroups = collectLicenseGroups(report);

for (const { license, packages } of licenseGroups) {
  for (const pkg of packages) {
    const key = `${pkg.name}\0${license}`;
    if (allowedLicenses.has(license) || exceptions.has(key)) {
      continue;
    }

    violations.push(`${pkg.name}@${(pkg.versions ?? []).join(",")} uses ${license}`);
  }
}

if (licenseGroups.length === 0) {
  if (report.error) {
    throw new Error(`pnpm licenses failed: ${JSON.stringify(report.error)}`);
  }
  throw new Error(
    `pnpm licenses output did not contain any license package groups. Top-level keys: ${Object.keys(
      report,
    )
      .slice(0, 10)
      .join(", ")}`,
  );
}

if (violations.length > 0) {
  console.error("Blocked npm packages with unapproved licenses:");
  for (const violation of violations.sort()) {
    console.error(`  - ${violation}`);
  }
  process.exit(1);
}

console.log("All npm dependency licenses match config/dependency-policy.json.");

function runLicenseList() {
  const report = readLicenseReport();
  if (report.error?.code === "ERR_PNPM_MISSING_PACKAGE_INDEX_FILE") {
    runInstall();
    return readLicenseReport();
  }

  return report;
}

function readLicenseReport() {
  const result = runPnpm(["licenses", "list", "--recursive", "--json"]);

  if (result.error) {
    throw result.error;
  }
  const output = result.stdout.trim();
  if (!output) {
    if (result.status === 0) {
      return {};
    }
    throw new Error(result.stderr || "pnpm licenses produced no JSON output.");
  }

  return parseJsonOutput(output);
}

function runInstall() {
  const result = runPnpm(["install", "--frozen-lockfile", "--force"]);
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(result.stderr || result.stdout || "pnpm install failed.");
  }
}

function runPnpm(args) {
  const result = spawnSync("corepack", ["pnpm", ...args], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error?.code !== "ENOENT") {
    return result;
  }

  return spawnSync("pnpm", args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function parseJsonOutput(output) {
  const jsonStart = findJsonStart(output);
  if (jsonStart === -1) {
    throw new Error(`pnpm licenses did not emit JSON output:\n${output}`);
  }

  return JSON.parse(output.slice(jsonStart));
}

function findJsonStart(output) {
  let offset = 0;
  for (const line of output.split(/(?<=\n)/)) {
    const firstContent = line.search(/\S/);
    if (firstContent !== -1) {
      const firstChar = line[firstContent];
      if (firstChar === "{" || firstChar === "[") {
        return offset + firstContent;
      }
    }
    offset += line.length;
  }

  return -1;
}

function collectPackages(group) {
  if (Array.isArray(group)) {
    return group;
  }
  if (!group || typeof group !== "object") {
    return null;
  }
  if (Array.isArray(group.packages)) {
    return group.packages;
  }

  const values = Object.values(group);
  if (values.length > 0 && values.every(isPackageEntry)) {
    return values;
  }

  return null;
}

function collectLicenseGroups(node) {
  if (!node || typeof node !== "object" || Array.isArray(node)) {
    return [];
  }

  const groups = [];
  for (const [license, group] of Object.entries(node)) {
    const packages = collectPackages(group);
    if (packages) {
      groups.push({ license, packages });
      continue;
    }
    groups.push(...collectLicenseGroups(group));
  }

  return groups;
}

function isPackageEntry(value) {
  return Boolean(value && typeof value === "object" && "name" in value);
}
