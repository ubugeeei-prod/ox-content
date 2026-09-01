#!/usr/bin/env node
/**
 * First-time publish for a workspace package that does not exist on npm yet.
 * Trusted publishing cannot create the package; run this once from a laptop
 * with npm credentials, then this script registers the GitHub Actions trusted
 * publisher via `npm trust`.
 *
 * Usage: node tools/scripts/bootstrap-npm-package.mjs npm/ox-content-code-play
 */
import { spawnSync } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const TRUST_REPO = "ubugeeei-prod/ox-content";
const TRUST_WORKFLOW = "publish.yml";
const TRUST_ENV = "npm";

const packageDir = process.argv[2];
if (!packageDir) {
  console.error("Usage: node tools/scripts/bootstrap-npm-package.mjs <package-dir>");
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

const tag = distTag(pkg.version);

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

console.log(`Publishing ${tarball} as ${tag} (no provenance; CI will attest later versions)`);
run("npm", ["publish", tarball, "--access", "public", "--provenance=false", "--tag", tag], cwd);

console.log(`Configuring trusted publisher for ${pkg.name}`);
run(
  "npm",
  [
    "trust",
    "github",
    pkg.name,
    "--file",
    TRUST_WORKFLOW,
    "--repo",
    TRUST_REPO,
    "--env",
    TRUST_ENV,
    "--allow-publish",
    "-y",
  ],
  cwd,
);

console.log(`
${pkg.name}@${pkg.version} is on npm (${tag}) with trusted publishing:

  Organization or user:  ubugeeei-prod
  Repository:            ox-content
  Workflow filename:     ${TRUST_WORKFLOW}
  Environment name:      ${TRUST_ENV}
`);

function distTag(version) {
  const dash = version.indexOf("-");
  if (dash === -1) return "latest";
  return version.slice(dash + 1).split(".")[0] || "latest";
}

function run(command, args, commandCwd) {
  const result = spawnSync(command, args, { cwd: commandCwd, stdio: "inherit" });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
