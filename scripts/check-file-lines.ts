#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import type { SpawnSyncReturns } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { extname } from "node:path";

type Options = {
  base?: string;
  limit?: string;
};

type RunGitOptions = {
  allowFailure?: boolean;
};

type FileLineViolation = {
  file: string;
  lines: number;
  baseLines?: number;
};

const defaultLimit = 250;
const sourceExtensions = new Set([
  ".cjs",
  ".css",
  ".cts",
  ".js",
  ".jsx",
  ".mjs",
  ".mts",
  ".rs",
  ".scss",
  ".svelte",
  ".ts",
  ".tsx",
  ".vue",
]);

const options = parseArgs(process.argv.slice(2));
const limit = parsePositiveInteger(options.limit ?? process.env.FILE_LINE_LIMIT, defaultLimit);
const baseRef = options.base ?? findDefaultBaseRef();
const files = collectChangedFiles(baseRef);
const checkedFiles = files.filter(shouldCheckFile).sort();
const violations: FileLineViolation[] = [];

for (const file of checkedFiles) {
  const lines = countLines(file);
  if (lines <= limit) {
    continue;
  }

  const baseLines = countBaseLines(baseRef, file);
  if (baseLines !== undefined && baseLines > limit) {
    continue;
  }

  violations.push({ file, lines, baseLines });
}

if (violations.length > 0) {
  console.error(`New or newly oversized source files should stay within ${limit} lines.`);
  console.error("Split these files before opening or merging the PR:");
  for (const { file, lines, baseLines } of violations) {
    const baseDetail = baseLines === undefined ? "new file" : `${baseLines} lines on base`;
    console.error(`  - ${file}: ${lines} lines (${baseDetail})`);
  }
  process.exit(1);
}

console.log(
  `Checked ${checkedFiles.length} changed source file(s); no new files exceed ${limit} lines.`,
);

function parseArgs(args: string[]): Options {
  const parsed: Options = {};

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--base" || arg === "--limit") {
      const value = args[index + 1];
      if (!value) {
        throw new Error(`${arg} requires a value.`);
      }
      parsed[arg.slice(2) as keyof Options] = value;
      index += 1;
      continue;
    }

    if (arg.startsWith("--base=")) {
      parsed.base = arg.slice("--base=".length);
      continue;
    }
    if (arg.startsWith("--limit=")) {
      parsed.limit = arg.slice("--limit=".length);
      continue;
    }

    throw new Error(`Unknown option: ${arg}`);
  }

  return parsed;
}

function parsePositiveInteger(value: string | undefined, fallback: number): number {
  if (value === undefined || value === "") {
    return fallback;
  }

  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`Expected a positive integer, got ${value}.`);
  }

  return parsed;
}

function findDefaultBaseRef(): string {
  const candidates: string[] = [];
  if (process.env.GITHUB_BASE_REF) {
    candidates.push(`origin/${process.env.GITHUB_BASE_REF}`);
  }
  candidates.push("origin/main", "main");

  for (const candidate of candidates) {
    if (gitRefExists(candidate)) {
      return candidate;
    }
  }

  return "HEAD";
}

function gitRefExists(ref: string): boolean {
  return runGit(["rev-parse", "--verify", "--quiet", ref], { allowFailure: true }).status === 0;
}

function collectChangedFiles(base: string): string[] {
  const files = new Set<string>();
  for (const file of changedFilesFromBase(base)) {
    files.add(file);
  }

  if (!process.env.CI) {
    for (const file of runGitLines(["diff", "--name-only", "--diff-filter=ACMR", "HEAD"])) {
      files.add(file);
    }
    for (const file of runGitLines(["ls-files", "--others", "--exclude-standard"])) {
      files.add(file);
    }
  }

  return [...files];
}

function changedFilesFromBase(base: string): string[] {
  if (base === "HEAD") {
    return [];
  }

  const mergeBaseDiff = runGit(["diff", "--name-only", "--diff-filter=ACMR", `${base}...HEAD`], {
    allowFailure: true,
  });
  if (mergeBaseDiff.status === 0) {
    return splitLines(mergeBaseDiff.stdout);
  }

  const directDiff = runGit(["diff", "--name-only", "--diff-filter=ACMR", `${base}..HEAD`], {
    allowFailure: true,
  });
  if (directDiff.status === 0) {
    return splitLines(directDiff.stdout);
  }

  throw new Error(mergeBaseDiff.stderr || directDiff.stderr || `Could not diff against ${base}.`);
}

function shouldCheckFile(file: string): boolean {
  if (!existsSync(file) || file.endsWith(".d.ts")) {
    return false;
  }

  const normalized = file.replaceAll("\\", "/");
  if (
    normalized.startsWith(".github/actions/stickydisk/dist/") ||
    normalized.includes("/__snapshots__/") ||
    normalized.includes("/snapshots/")
  ) {
    return false;
  }

  const segments = normalized.split("/");
  if (segments.some((segment) => segment === "dist" || segment === "node_modules")) {
    return false;
  }

  return sourceExtensions.has(extname(normalized));
}

function countLines(file: string): number {
  const content = readFileSync(file, "utf8");
  return countTextLines(content);
}

function countBaseLines(base: string, file: string): number | undefined {
  if (base === "HEAD") {
    return undefined;
  }

  const result = runGit(["show", `${base}:${file}`], { allowFailure: true });
  if (result.status !== 0) {
    return undefined;
  }

  return countTextLines(result.stdout);
}

function countTextLines(content: string): number {
  if (content.length === 0) {
    return 0;
  }

  const newlineCount = content.match(/\n/g)?.length ?? 0;
  return content.endsWith("\n") ? newlineCount : newlineCount + 1;
}

function runGitLines(args: string[]): string[] {
  return splitLines(runGit(args).stdout);
}

function runGit(args: string[], options: RunGitOptions = {}): SpawnSyncReturns<string> {
  const result = spawnSync("git", args, { encoding: "utf8" });
  if (result.error) {
    throw result.error;
  }
  if (!options.allowFailure && result.status !== 0) {
    throw new Error(result.stderr || `git ${args.join(" ")} failed.`);
  }

  return result;
}

function splitLines(output: string): string[] {
  return output
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}
