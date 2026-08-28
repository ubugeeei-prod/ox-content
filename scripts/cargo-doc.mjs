#!/usr/bin/env node

/**
 * Builds the Rust API documentation.
 *
 * `target/doc` is discarded first. CI keeps `target/` on a Blacksmith sticky
 * disk, so the previous run's documentation is still there, and rustdoc then
 * re-merges its shared cross-crate index against everything already on disk.
 * That merge is single-threaded and memory-hungry: measured on the real CI
 * disk, `cargo doc --workspace --no-deps` took 9m09s at 102% CPU with a 10.7 GB
 * peak RSS against an accumulated 252 MB / 34-crate `target/doc`, and 15s on the
 * same disk with it removed. Compilation artifacts elsewhere in `target/` still
 * come from the cache, so the warm-build win is kept.
 */

import { spawnSync } from "node:child_process";
import { rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = dirname(dirname(fileURLToPath(import.meta.url)));

for (const stale of ["doc", "doc.parts"]) {
  rmSync(join(repoRoot, "target", stale), { recursive: true, force: true });
}

const result = spawnSync("cargo", ["doc", "--workspace", "--no-deps", ...process.argv.slice(2)], {
  cwd: repoRoot,
  stdio: "inherit",
});

if (result.error) {
  throw result.error;
}
process.exit(result.status ?? 1);
