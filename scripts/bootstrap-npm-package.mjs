#!/usr/bin/env node
/**
 * First-time publish for a workspace package that does not exist on npm yet.
 * Trusted publishing cannot create the package; run this once from a laptop
 * with npm credentials, then add the trusted publisher on npmjs.com.
 *
 * Usage: node scripts/bootstrap-npm-package.mjs npm/ox-content-code-play
 */
import { spawnSync } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const packageDir = process.argv[2];
if (!packageDir) {
  console.error("Usage: node scripts/bootstrap-npm-package.mjs <package-dir>");
  process.exit(1);
}

const root = resolve(import.meta.dirname, "..");
const cwd = resolve(root, packageDir);
const dest = mkdtempSync(join(tmpdir(), "ox-content-bootstrap-"));
const pkg = JSON.parse(
  spawnSync("node", ["-p", "JSON.stringify(require('./package.json'))"], {
    cwd,
    encoding: "utf8",
  }).stdout,
);

console.log(`Building ${pkg.name}@${pkg.version} in ${packageDir}`);
run("vp", ["exec", "--filter", pkg.name, "--", "vp", "run", "build"], root);

console.log(`Packing to ${dest}`);
const pack = spawnSync("vp", ["exec", "--", "pnpm", "pack", "--pack-destination", dest], {
  cwd,
  encoding: "utf8",
});
if (pack.status !== 0) {
  console.error(pack.stderr || pack.stdout);
  process.exit(pack.status ?? 1);
}
const tarball = pack.stdout.trim().split("\n").at(-1);
if (!tarball) {
  console.error("pnpm pack did not print a tarball path.");
  process.exit(1);
}

console.log(`Publishing ${tarball} (no provenance; CI will attest later versions)`);
run("npm", ["publish", tarball, "--access", "public", "--provenance=false"], cwd);

console.log(`
${pkg.name}@${pkg.version} is on npm. Add the trusted publisher on npmjs.com:

  Package settings → Trusted Publisher → GitHub Actions
  Organization or user:  ubugeeei-prod
  Repository:            ox-content
  Workflow filename:     publish.yml
  Environment name:      npm
`);

function run(command, args, commandCwd) {
  const result = spawnSync(command, args, { cwd: commandCwd, stdio: "inherit" });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
