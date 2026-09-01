#!/usr/bin/env node
// Regenerates every npm/theme-* skin package through the typed Rust generator.
//
// Skins are authored as real stylesheets so editors can lint and format them;
// the Rust generator inlines each one into a plain data module, which is why
// the published package has no runtime dependency.
//
// Usage: node tools/scripts/theme-skins/generate.mjs

import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, "..", "..", "..");

run("cargo", ["run", "-p", "ox_content_theme_generator", "--", "skins"]);
formatGeneratedSources();

function run(command, args) {
  const result = spawnSync(command, args, { cwd: ROOT, stdio: "inherit" });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function formatGeneratedSources() {
  const result = spawnSync("npx", ["vp", "fmt"], { cwd: ROOT, stdio: "ignore" });
  if (result.error) {
    console.warn("  (could not run `vp fmt` — format the output before committing)");
    return;
  }
  if (result.status !== 0) {
    console.warn("  (could not run `vp fmt` — format the output before committing)");
  }
}
