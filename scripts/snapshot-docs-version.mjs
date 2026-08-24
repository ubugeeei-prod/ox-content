#!/usr/bin/env node
// Usage: node scripts/snapshot-docs-version.mjs --tag v2.90.0 --prefix 2.90
//
// Copies docs/content from a git tag into docs/versions/<prefix>.
// Historical snapshots are frozen: later builds must not rewrite them.

import { spawnSync } from "node:child_process";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const args = parseArgs(process.argv.slice(2));

if (!args.tag || !args.prefix || !/^[a-z0-9](?:[a-z0-9.-]{0,62})$/.test(args.prefix)) {
  console.error("Usage: node scripts/snapshot-docs-version.mjs --tag v2.90.0 --prefix 2.90");
  process.exit(1);
}

const dest = path.join(ROOT, "docs/versions", args.prefix);
const staging = path.join(ROOT, ".tmp-docs-snapshot");
rmSync(staging, { recursive: true, force: true });
rmSync(dest, { recursive: true, force: true });

const archive = spawnSync("git", ["archive", args.tag, "docs/content"], {
  cwd: ROOT,
  encoding: "buffer",
  maxBuffer: 32 * 1024 * 1024,
});
if (archive.status !== 0) {
  console.error(archive.stderr?.toString() || `git archive ${args.tag} failed`);
  process.exit(archive.status ?? 1);
}

mkdirSync(staging, { recursive: true });
const extract = spawnSync("tar", ["-x", "-C", staging], {
  cwd: ROOT,
  input: archive.stdout,
});
if (extract.status !== 0) {
  console.error(extract.stderr?.toString() || "tar extract failed");
  process.exit(extract.status ?? 1);
}

mkdirSync(path.dirname(dest), { recursive: true });
spawnSync("mv", [path.join(staging, "docs/content"), dest], { cwd: ROOT, stdio: "inherit" });
rmSync(staging, { recursive: true, force: true });
writeFileSync(
  path.join(dest, ".ox-content-version.json"),
  `${JSON.stringify({ label: args.tag.replace(/^v/, ""), tag: args.tag, frozen: true }, null, 2)}\n`,
  "utf8",
);
console.log(`Wrote ${dest} from ${args.tag}`);

function parseArgs(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i += 1) {
    const key = argv[i];
    if (key === "--tag" || key === "--prefix") {
      out[key.slice(2)] = argv[i + 1];
      i += 1;
    }
  }
  return out;
}
