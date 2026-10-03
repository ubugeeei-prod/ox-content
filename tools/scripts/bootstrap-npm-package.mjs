#!/usr/bin/env node
/**
 * First-time publish for a workspace package that does not exist on npm yet.
 * Trusted publishing cannot create the package; run this once from a laptop
 * with npm credentials, then this script registers the GitHub Actions trusted
 * publisher via `npm trust`.
 *
 * Usage: vp node tools/scripts/bootstrap-npm-package.mjs npm/oxct [--pack-only|--register-only]
 */
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const TRUST_REPO = "ubugeeei-prod/ox-content";
const TRUST_WORKFLOW = "publish.yml";
const TRUST_ENV = "npm";

const packageDir = process.argv[2];
const flags = process.argv.slice(3);
const packOnly = flags.includes("--pack-only");
const registerOnly = flags.includes("--register-only");
if (
  flags.some((flag) => !["--pack-only", "--register-only"].includes(flag)) ||
  (packOnly && registerOnly)
) {
  throw new Error("Use --pack-only or --register-only, without other flags.");
}
if (!packageDir) {
  console.error(
    "Usage: vp node tools/scripts/bootstrap-npm-package.mjs <package-dir> [--pack-only|--register-only]",
  );
  process.exit(1);
}

const root = resolve(import.meta.dirname, "../..");
const cwd = resolve(root, packageDir);
const dest = mkdtempSync(join(tmpdir(), "ox-content-bootstrap-"));
const pkg = JSON.parse(readFileSync(join(cwd, "package.json"), "utf8"));
if (pkg.private) throw new Error(`Cannot publish private package ${pkg.name}`);

const tag = distTag(pkg.version);
// Do not start publishing without a usable maintainer identity.
if (!packOnly) npm(["whoami"]);

if (!registerOnly) {
  console.log(`Building ${pkg.name}@${pkg.version} in ${packageDir}`);
  run("vp", ["exec", "--filter", pkg.name, "--", "vp", "run", "build"], root);

  console.log(`Packing to ${dest}`);
  const pack = spawnSync("vp", ["exec", "--", "pnpm", "pack", "--pack-destination", dest], {
    cwd,
    encoding: "utf8",
  });
  if (pack.error) throw pack.error;
  if (pack.status !== 0) {
    console.error(pack.stderr || pack.stdout);
    process.exit(pack.status ?? 1);
  }
  const tarball = pack.stdout.trim().split("\n").at(-1);
  if (!tarball) {
    console.error("pnpm pack did not print a tarball path.");
    process.exit(1);
  }

  if (packOnly) {
    console.log(
      `Prepared ${pkg.name}@${pkg.version}: ${tarball}\nNo publication or trusted-publisher changes made.`,
    );
    process.exit(0);
  }

  console.log(`Publishing ${tarball} as ${tag} (no provenance; CI will attest later versions)`);
  npm(["publish", tarball, "--access", "public", "--provenance=false", "--tag", tag]);
}

console.log(`Configuring trusted publisher for ${pkg.name}`);
npm([
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
]);

console.log(`
Trusted publishing configured for ${pkg.name}:

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
  if (result.error) throw result.error;
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

function npm(args) {
  run("vp", ["exec", "--", "pnpm", "dlx", "npm@11.19.0", ...args], cwd);
}
