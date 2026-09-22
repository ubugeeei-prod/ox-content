#!/usr/bin/env node
// Usage: node tools/scripts/release.ts [patch|minor|major|alpha|beta|x.y.z] [--prepare-only]

import { execSync } from "child_process";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";
import { categorizeCommits, generateChangelog, getCommitsSinceTag } from "./release-changelog.ts";
import { verifyPublishWorkflow } from "./verify-publish-targets.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..", "..");

import { NPM_PACKAGES, CARGO_PUBLISH_PACKAGES } from "./release-targets.ts";

const CARGO_TOML = "Cargo.toml";
const CARGO_LOCK = "Cargo.lock";
const RUST_DOC_FILES = ["docs/content/getting-started.md"];
const ZED_EXTENSION_TOML = "editors/zed/extension.toml";
const ZED_CARGO_TOML = "editors/zed/Cargo.toml";

function exec(cmd: string, options: { cwd?: string; stdio?: "inherit" | "pipe" } = {}): string {
  console.log(`$ ${cmd}`);
  try {
    return execSync(cmd, {
      cwd: options.cwd ?? ROOT,
      encoding: "utf-8",
      stdio: options.stdio ?? "pipe",
    });
  } catch (e) {
    if (options.stdio === "inherit") throw e;
    const err = e as { stderr?: string; stdout?: string; message?: string };
    throw new Error(`Command failed: ${cmd}\n${err.stderr || err.stdout || err.message}`);
  }
}

function getPackageJson(pkgPath: string): {
  name: string;
  version: string;
  [key: string]: unknown;
} {
  const fullPath = path.join(ROOT, pkgPath, "package.json");
  return JSON.parse(fs.readFileSync(fullPath, "utf-8"));
}

function setPackageVersion(pkgPath: string, version: string): void {
  const fullPath = path.join(ROOT, pkgPath, "package.json");
  const pkg = JSON.parse(fs.readFileSync(fullPath, "utf-8"));
  pkg.version = version;
  fs.writeFileSync(fullPath, JSON.stringify(pkg, null, 2) + "\n", "utf-8");
  console.log(`  Updated ${pkg.name} to ${version}`);
}

function setCargoVersion(version: string): void {
  const fullPath = path.join(ROOT, CARGO_TOML);
  let content = fs.readFileSync(fullPath, "utf-8");
  // Update workspace.package version
  content = content.replace(
    /(\[workspace\.package\]\s*\n(?:[^[]*\n)*?version\s*=\s*)"[^"]+"/,
    `$1"${version}"`,
  );
  // Update workspace.dependencies versions for internal crates
  content = content.replace(/(ox_content_\w+\s*=\s*\{\s*version\s*=\s*)"[^"]+"/g, `$1"${version}"`);
  fs.writeFileSync(fullPath, content, "utf-8");
  console.log(`  Updated Cargo.toml workspace version to ${version}`);
}

function setCargoLockVersion(version: string): void {
  const fullPath = path.join(ROOT, CARGO_LOCK);
  let content = fs.readFileSync(fullPath, "utf-8");
  content = content.replace(
    /(\[\[package\]\]\nname = "ox_content_[^"]+"\nversion = )"[^"]+"/g,
    `$1"${version}"`,
  );
  fs.writeFileSync(fullPath, content, "utf-8");
  console.log(`  Updated Cargo.lock workspace package versions to ${version}`);
}

function setZedVersion(version: string): void {
  const extensionTomlPath = path.join(ROOT, ZED_EXTENSION_TOML);
  let extensionToml = fs.readFileSync(extensionTomlPath, "utf-8");
  extensionToml = extensionToml.replace(/^version = ".*"$/m, `version = "${version}"`);
  fs.writeFileSync(extensionTomlPath, extensionToml, "utf-8");

  const cargoTomlPath = path.join(ROOT, ZED_CARGO_TOML);
  let cargoToml = fs.readFileSync(cargoTomlPath, "utf-8");
  cargoToml = cargoToml.replace(/^version = ".*"$/m, `version = "${version}"`);
  fs.writeFileSync(cargoTomlPath, cargoToml, "utf-8");
  console.log(`  Updated Zed extension version to ${version}`);
}

function updateRustDocsVersion(version: string): void {
  for (const relativePath of RUST_DOC_FILES) {
    const fullPath = path.join(ROOT, relativePath);
    let content = fs.readFileSync(fullPath, "utf-8");

    const updated = content.replace(/(ox_content_[a-z_]+\s*=\s*)"[^"]+"/g, `$1"${version}"`);

    if (updated !== content) {
      fs.writeFileSync(fullPath, updated, "utf-8");
      console.log(`  Updated Rust crate versions in ${relativePath}`);
    } else {
      console.log(`  No Rust crate versions to update in ${relativePath}`);
    }
  }
}

export function bumpVersion(
  current: string,
  type: "patch" | "minor" | "major" | "alpha" | "beta",
): string {
  // Handle prerelease versions (alpha/beta)
  if (type === "alpha" || type === "beta") {
    const prereleaseMatch = current.match(/^(\d+\.\d+\.\d+)-(alpha|beta)\.(\d+)$/);
    if (prereleaseMatch && prereleaseMatch[2] === type) {
      const [, base, , num] = prereleaseMatch;
      return `${base}-${type}.${Number(num) + 1}`;
    }
    // Start new prerelease or switch from alpha to beta
    const baseVersion = current.replace(/-.*$/, "");
    return `${baseVersion}-${type}.0`;
  }

  // Remove any prerelease suffix for standard bumps
  const baseVersion = current.replace(/-.*$/, "");
  const [major, minor, patch] = baseVersion.split(".").map(Number);
  switch (type) {
    case "major":
      return `${major + 1}.0.0`;
    case "minor":
      return `${major}.${minor + 1}.0`;
    case "patch":
      return `${major}.${minor}.${patch + 1}`;
  }
}

function isValidVersion(v: string): boolean {
  return /^\d+\.\d+\.\d+(-[\w.]+)?$/.test(v);
}

function getLatestTag(): string | undefined {
  try {
    return exec("git describe --tags --abbrev=0").trim();
  } catch {
    return undefined;
  }
}

function updateChangelogFile(content: string): void {
  const changelogPath = path.join(ROOT, "CHANGELOG.md");
  let existing = "";

  if (fs.existsSync(changelogPath)) {
    existing = fs.readFileSync(changelogPath, "utf-8");
    // Remove header if exists
    existing = existing.replace(/^# Changelog\n+/, "");
  }

  const full = `# Changelog\n\n${content}${existing}`;
  fs.writeFileSync(changelogPath, full, "utf-8");
  console.log("  Updated CHANGELOG.md");
}

export function prepareRelease(input: string): void {
  const status = exec("git status --porcelain");
  if (status.trim()) {
    console.error("Error: Working directory is not clean. Commit or stash changes first.");
    process.exit(1);
  }

  const currentPkg = getPackageJson(NPM_PACKAGES[0]);
  const currentVersion = currentPkg.version || "0.0.0";
  let newVersion: string;

  if (["patch", "minor", "major", "alpha", "beta"].includes(input)) {
    newVersion = bumpVersion(
      currentVersion,
      input as "patch" | "minor" | "major" | "alpha" | "beta",
    );
  } else if (isValidVersion(input)) {
    newVersion = input;
  } else {
    console.error(`Invalid version: ${input}`);
    process.exit(1);
  }

  console.log(`\nReleasing v${newVersion} (from ${currentVersion})\n`);

  console.log("Updating Cargo.toml version...");
  setCargoVersion(newVersion);
  setCargoLockVersion(newVersion);
  setZedVersion(newVersion);

  console.log("Updating package versions...");
  for (const pkg of NPM_PACKAGES) {
    setPackageVersion(pkg, newVersion);
  }

  console.log("Updating Rust docs versions...");
  updateRustDocsVersion(newVersion);

  console.log("Verifying publish workflow targets...");
  verifyPublishWorkflow({
    root: ROOT,
    workflowRel: ".github/workflows/publish.yml",
    cargoPackages: CARGO_PUBLISH_PACKAGES,
    npmPackages: NPM_PACKAGES,
  });

  console.log("\nGenerating changelog...");
  const latestTag = getLatestTag();
  const commits = getCommitsSinceTag(exec, latestTag, {
    root: ROOT,
    npmPackages: NPM_PACKAGES,
  });
  const categorized = categorizeCommits(commits);
  const changelogContent = generateChangelog(newVersion, categorized);
  updateChangelogFile(changelogContent);

  console.log(`\nPrepared v${newVersion} without commit, tag, or push.`);
}
