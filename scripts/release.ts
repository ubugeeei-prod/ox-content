#!/usr/bin/env -S node --experimental-strip-types
/**
 * Release script for ox-content
 *
 * Usage:
 *   bun scripts/release.ts [patch|minor|major|x.y.z]
 *
 * This script:
 *   1. Updates all package.json versions
 *   2. Updates Rust dependency versions in docs
 *   3. Generates CHANGELOG.md
 *   4. Creates a git commit and tag
 *   5. Pushes to remote
 */

import { execSync } from "child_process";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

// Packages to publish (relative to root)
const NPM_PACKAGES = [
  "crates/ox_content_napi",
  "npm/ox-content-islands",
  "npm/unplugin-ox-content",
  "npm/vite-plugin-ox-content",
  "npm/vite-plugin-ox-content-react",
  "npm/vite-plugin-ox-content-svelte",
  "npm/vite-plugin-ox-content-vue",
];

const CARGO_TOML = "Cargo.toml";
const RUST_DOC_FILES = ["docs/content/getting-started.md"];

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

function bumpVersion(
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

function getCommitsSinceTag(tag?: string): string[] {
  try {
    const range = tag ? `${tag}..HEAD` : "HEAD";
    const log = exec(`git log ${range} --pretty=format:"%s" --no-merges`);
    return log.trim().split("\n").filter(Boolean);
  } catch {
    return [];
  }
}

function getLatestTag(): string | undefined {
  try {
    return exec("git describe --tags --abbrev=0").trim();
  } catch {
    return undefined;
  }
}

function categorizeCommits(commits: string[]): Record<string, string[]> {
  const categories: Record<string, string[]> = {
    feat: [],
    fix: [],
    perf: [],
    refactor: [],
    docs: [],
    chore: [],
    other: [],
  };

  for (const commit of commits) {
    // Skip generated commits
    if (commit.includes("Generated with [Claude Code]")) continue;

    const match = commit.match(/^(\w+)(?:\([^)]+\))?:\s*(.+)/);
    if (match) {
      const [, type, message] = match;
      const category = categories[type] ? type : "other";
      categories[category].push(message);
    } else {
      categories.other.push(commit);
    }
  }

  return categories;
}

function generateChangelog(version: string, commits: Record<string, string[]>): string {
  const date = new Date().toISOString().split("T")[0];
  let changelog = `## [${version}] - ${date}\n\n`;

  const sections: [string, string][] = [
    ["feat", "Features"],
    ["fix", "Bug Fixes"],
    ["perf", "Performance"],
    ["refactor", "Refactoring"],
    ["docs", "Documentation"],
  ];

  for (const [key, title] of sections) {
    if (commits[key]?.length) {
      changelog += `### ${title}\n\n`;
      for (const msg of commits[key]) {
        changelog += `- ${msg}\n`;
      }
      changelog += "\n";
    }
  }

  return changelog;
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

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const input = args[0];

  if (!input) {
    console.error("Usage: bun scripts/release.ts [patch|minor|major|x.y.z]");
    process.exit(1);
  }

  // Check for uncommitted changes
  const status = exec("git status --porcelain");
  if (status.trim()) {
    console.error("Error: Working directory is not clean. Commit or stash changes first.");
    process.exit(1);
  }

  // Determine new version
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

  // Update Cargo.toml workspace version
  console.log("Updating Cargo.toml version...");
  setCargoVersion(newVersion);

  // Update all package.json versions
  console.log("Updating package versions...");
  for (const pkg of NPM_PACKAGES) {
    setPackageVersion(pkg, newVersion);
  }

  // Update Rust dependency versions in docs
  console.log("Updating Rust docs versions...");
  updateRustDocsVersion(newVersion);

  // Generate changelog
  console.log("\nGenerating changelog...");
  const latestTag = getLatestTag();
  const commits = getCommitsSinceTag(latestTag);
  const categorized = categorizeCommits(commits);
  const changelogContent = generateChangelog(newVersion, categorized);
  updateChangelogFile(changelogContent);

  // Git operations
  console.log("\nCreating git commit and tag...");
  exec("git add -A");
  exec(`git commit -m "chore(release): v${newVersion}"`);
  exec(`git tag -a v${newVersion} -m "Release v${newVersion}"`);

  console.log("\nPushing to remote...");
  exec("git push");
  exec("git push --tags");

  console.log(`\n✅ Released v${newVersion} successfully!`);
  console.log("\nNext steps:");
  console.log("  1. GitHub Actions will automatically publish to npm");
  console.log("  2. Create a GitHub release at:");
  console.log(`     https://github.com/ubugeeei/ox-content/releases/new?tag=v${newVersion}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
