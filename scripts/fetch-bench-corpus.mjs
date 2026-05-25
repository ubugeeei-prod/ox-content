#!/usr/bin/env node
/**
 * Fetches large, license-compatible Markdown corpora from upstream OSS docs
 * and stages them under `benchmarks/corpus/<project>/` for use by
 * `cargo bench -p ox_content_parser --bench corpus` and the JS comparison
 * benchmarks in `benchmarks/bundle-size/`.
 *
 * Each corpus entry pins:
 *   - the upstream repo (HTTPS, no auth)
 *   - the commit-ish to check out (sparse-checkout of the docs subtree only)
 *   - the SPDX-allowed license the corpus ships under
 *
 * The fetched trees live entirely under `benchmarks/corpus/`, which is
 * `.gitignore`'d. Run this script before opening a benchmark PR if you want
 * the real-world numbers to include the OSS docs corpus; the in-tree
 * synthetic fixtures continue to work without it.
 */

import { execSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, "..");
const CORPUS_ROOT = join(REPO_ROOT, "benchmarks", "corpus");

// All entries are MIT or Apache-2.0 documentation that explicitly permits
// redistribution and modification. Each entry's `license` field documents
// the SPDX identifier verified at the time the entry was added.
const SOURCES = [
  {
    name: "vue-docs",
    repo: "https://github.com/vuejs/docs.git",
    rev: "main",
    sparse: ["src/guide", "src/api", "src/tutorial"],
    license: "MIT",
  },
  {
    name: "vite-docs",
    repo: "https://github.com/vitejs/vite.git",
    rev: "main",
    sparse: ["docs/guide", "docs/config"],
    license: "MIT",
  },
  {
    name: "rust-book",
    repo: "https://github.com/rust-lang/book.git",
    rev: "main",
    sparse: ["src"],
    license: "MIT OR Apache-2.0",
  },
  {
    name: "typescript-handbook",
    repo: "https://github.com/microsoft/TypeScript-Website.git",
    rev: "v2",
    sparse: ["packages/documentation/copy/en"],
    license: "MIT",
  },
];

/**
 * @param {string} cmd
 * @param {{cwd?: string}} [options]
 */
function run(cmd, options = {}) {
  execSync(cmd, { stdio: "inherit", cwd: options.cwd ?? process.cwd() });
}

/**
 * @param {string} cmd
 * @param {{cwd?: string}} [options]
 * @returns {string}
 */
function capture(cmd, options = {}) {
  return execSync(cmd, { stdio: ["ignore", "pipe", "pipe"], cwd: options.cwd ?? process.cwd() })
    .toString()
    .trim();
}

mkdirSync(CORPUS_ROOT, { recursive: true });

const summary = [];
for (const source of SOURCES) {
  const dest = join(CORPUS_ROOT, source.name);
  console.log(`\n[corpus] ${source.name} <- ${source.repo}@${source.rev}`);

  if (existsSync(dest)) {
    rmSync(dest, { recursive: true, force: true });
  }
  mkdirSync(dest, { recursive: true });

  try {
    run(`git -C "${dest}" init -q`);
    run(`git -C "${dest}" remote add origin "${source.repo}"`);
    run(`git -C "${dest}" config core.sparseCheckout true`);
    writeFileSync(
      join(dest, ".git", "info", "sparse-checkout"),
      `${source.sparse.join("\n")}\nLICENSE\nLICENSE.md\nLICENSE.txt\n`,
    );
    run(`git -C "${dest}" fetch --depth=1 origin "${source.rev}"`);
    run(`git -C "${dest}" checkout -q FETCH_HEAD`);

    const sha = capture(`git -C "${dest}" rev-parse HEAD`);
    const fileCount = capture(`find "${dest}" -name "*.md" -not -path "*/.git/*" | wc -l`);
    const bytes = capture(
      `find "${dest}" -name "*.md" -not -path "*/.git/*" -exec wc -c {} + | tail -1 | awk '{print $1}'`,
    );

    summary.push({
      name: source.name,
      repo: source.repo,
      rev: source.rev,
      sha,
      license: source.license,
      markdownFiles: Number(fileCount.trim()),
      markdownBytes: Number(bytes.trim()) || 0,
    });
  } catch (err) {
    console.error(`[corpus] FAILED ${source.name}: ${err instanceof Error ? err.message : err}`);
    rmSync(dest, { recursive: true, force: true });
  }
}

writeFileSync(
  join(CORPUS_ROOT, "manifest.json"),
  `${JSON.stringify({ generatedAt: new Date().toISOString(), entries: summary }, null, 2)}\n`,
);

console.log("\n[corpus] done. Summary:");
for (const entry of summary) {
  console.log(
    `  - ${entry.name.padEnd(22)} ${entry.markdownFiles} files, ` +
      `${(entry.markdownBytes / 1024).toFixed(1)} KiB (${entry.license})`,
  );
}
console.log(`\nManifest written to ${join(CORPUS_ROOT, "manifest.json")}`);
