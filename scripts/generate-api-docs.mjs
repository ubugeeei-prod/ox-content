#!/usr/bin/env node
// Usage: node scripts/generate-api-docs.mjs [--write]
//
// Writes `docs/content/api` the same way the docs plugin does on `buildStart`.
// Keep the options here in lockstep with `docs` in `docs/vite.config.ts`.
// The output is gitignored; `vp run dev:docs` / `build:docs` regenerate it.

import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DOCS_ROOT = path.join(ROOT, "docs");
const API_DIR = path.join(DOCS_ROOT, "content/api");
const SRC_DIR = path.join(ROOT, "npm/vite-plugin-ox-content/src");

if (process.argv.slice(2).some((arg) => arg !== "--write")) {
  throw new Error("Usage: scripts/generate-api-docs.mjs [--write]");
}

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

console.log(`Generated ${Object.keys(generated).length} API markdown files into docs/content/api`);

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
