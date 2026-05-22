#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const policy = readPolicy();
const minimumSeverity = policy.npmAudit?.minimumSeverity ?? "high";
const allowlist = new Map(
  (policy.npmAudit?.advisoryAllowlist ?? []).map((item) => [item.id, item]),
);
const audit = runAudit();
const advisories = Object.values(audit.advisories ?? {});
const failures = [];
const allowed = [];

for (const advisory of advisories) {
  const severity = String(advisory.severity ?? "unknown");
  if (severityRank(severity) < severityRank(minimumSeverity)) {
    continue;
  }

  const id = advisory.github_advisory_id ?? String(advisory.id);
  const allowedAdvisory = allowlist.get(id);
  if (allowedAdvisory) {
    if (isExpired(allowedAdvisory.expires)) {
      failures.push(
        `${id} ${severity} ${advisory.module_name} allowlist expired on ${allowedAdvisory.expires}`,
      );
      continue;
    }
    allowed.push(`${id} ${severity} ${advisory.module_name}`);
    continue;
  }

  failures.push(`${id} ${severity} ${advisory.module_name} ${advisory.vulnerable_versions}`);
}

if (allowed.length > 0) {
  console.log(`Allowed ${allowed.length} npm advisories from policy:`);
  for (const line of allowed.sort()) {
    console.log(`  - ${line}`);
  }
}

if (failures.length > 0) {
  console.error(`Blocked ${failures.length} npm advisories at ${minimumSeverity}+ severity:`);
  for (const line of failures.sort()) {
    console.error(`  - ${line}`);
  }
  process.exit(1);
}

console.log(`No unapproved npm advisories at ${minimumSeverity}+ severity.`);

function runAudit() {
  const result = runPnpm(["audit", "--json"]);

  if (result.error) {
    throw result.error;
  }

  const output = result.stdout.trim();
  if (!output) {
    if (result.status === 0) {
      return { advisories: {} };
    }
    throw new Error(result.stderr || "pnpm audit produced no JSON output.");
  }

  return parseJsonOutput(output);
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

function readPolicy() {
  return JSON.parse(readFileSync(resolve("config/dependency-policy.json"), "utf8"));
}

function isExpired(expires) {
  if (!expires) {
    return false;
  }
  return Date.parse(`${expires}T23:59:59.999Z`) < Date.now();
}

function severityRank(severity) {
  return (
    {
      info: 0,
      low: 1,
      moderate: 2,
      high: 3,
      critical: 4,
    }[severity] ?? 0
  );
}

function parseJsonOutput(output) {
  const jsonStart = output.indexOf("{");
  if (jsonStart === -1) {
    throw new Error(`pnpm audit did not emit JSON output:\n${output}`);
  }

  return JSON.parse(output.slice(jsonStart));
}
