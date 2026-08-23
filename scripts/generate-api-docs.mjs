#!/usr/bin/env node
// Usage: node scripts/generate-api-docs.mjs --write | --check
//
// Regenerates `docs/content/api` the same way the docs plugin does on
// `buildStart`. Keep the options here in lockstep with `docs` in
// `docs/vite.config.ts`. `--check` writes in place and fails if git sees a
// diff — the same shape as `cargo fmt --check` / the NAPI `index.d.ts` job.

import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DOCS_ROOT = path.join(ROOT, "docs");
const API_DIR = path.join(DOCS_ROOT, "content/api");
const API_GIT_PATH = "docs/content/api";
const SRC_DIR = path.join(ROOT, "npm/vite-plugin-ox-content/src");

const mode = parseMode(process.argv.slice(2));
const napi = createRequire(import.meta.url)("../crates/ox_content_napi");

const extracted = napi.extractDocsFromDirectories(
  [SRC_DIR],
  ["**/*.ts"],
  ["**/*.test.*"],
  false,
  false,
  false,
);
const modules = extracted.map((doc) => ({
  file: doc.file,
  entries: doc.entries.map((entry) => ({
    ...entry,
    tags: entry.tags
      ? Object.entries(entry.tags).map(([tag, value]) => ({ tag, value }))
      : undefined,
  })),
}));
const generated = napi.generateDocsMarkdown(modules, {
  groupBy: "file",
  githubUrl: "https://github.com/ubugeeei-prod/ox-content",
  linkStyle: "markdown",
  pathStrategy: "flat",
  renderStyle: "html",
  indexFormat: "none",
  parametersFormat: "none",
  interfacePropertiesFormat: "none",
  classPropertiesFormat: "none",
  typeAliasPropertiesFormat: "none",
  enumMembersFormat: "none",
  propertyMembersFormat: "none",
  typeDeclarationFormat: "none",
  renderStats: true,
  renderGeneratedBy: true,
  sortEntryPoints: true,
  singleEntryRoot: "preserve",
});

napi.writeGeneratedDocs(generated, API_DIR, modules, {
  generateNav: true,
  groupBy: "file",
  generatedAt: existingGeneratedAt(API_DIR) ?? new Date().toISOString(),
  pathStrategy: "flat",
  sortEntryPoints: true,
  singleEntryRoot: "preserve",
});

console.log(`Generated ${Object.keys(generated).length} API markdown files into ${API_GIT_PATH}`);

if (mode === "write") {
  process.exit(0);
}

const diff = git(["diff", "--exit-code", "--", API_GIT_PATH]);
const untracked = git(["ls-files", "--others", "--exclude-standard", "--", API_GIT_PATH]);
const extra = untracked.stdout.trim();

if (diff.status === 0 && extra.length === 0) {
  process.exit(0);
}

console.error("Checked-in API docs are stale. Run `vp run docs:api` and commit the result.");
if (diff.status !== 0) {
  process.stderr.write(diff.stdout);
  process.stderr.write(diff.stderr);
}
if (extra.length > 0) {
  console.error("Untracked generated files:");
  console.error(extra);
}
process.exit(1);

function parseMode(args) {
  if (args.includes("--check") && !args.includes("--write")) return "check";
  if (args.includes("--write") && !args.includes("--check")) return "write";
  throw new Error("Usage: scripts/generate-api-docs.mjs --write | --check");
}

function existingGeneratedAt(outDir) {
  try {
    const parsed = JSON.parse(readFileSync(path.join(outDir, "docs.json"), "utf8"));
    return typeof parsed.generatedAt === "string" && parsed.generatedAt.length > 0
      ? parsed.generatedAt
      : undefined;
  } catch {
    return undefined;
  }
}

function git(args) {
  return spawnSync("git", args, { cwd: ROOT, encoding: "utf-8" });
}
