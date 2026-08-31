#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";
import { checkReaderChromeDeclarations } from "./package-dry-run-reader-chrome.mjs";
import { checkTwitterClientDeclarations } from "./package-dry-run-twitter-client.mjs";
import { checkVitePluginDeclarations } from "./package-dry-run-vite-plugin.mjs";

const packages = [
  "crates/ox_content_napi",
  "npm/ox-content-islands",
  "npm/ox-content-code-play",
  "npm/vite-plugin-ox-content",
  "npm/unplugin-ox-content",
  "npm/vite-plugin-ox-content-vue",
  "npm/vite-plugin-ox-content-react",
  "npm/vite-plugin-ox-content-solid",
  "npm/vite-plugin-ox-content-svelte",
  "npm/theme-color/ayu",
  "npm/theme-color/catppuccin",
  "npm/theme-color/dracula",
  "npm/theme-color/github",
  "npm/theme-color/gruvbox",
  "npm/theme-color/horizon",
  "npm/theme-color/kanagawa",
  "npm/theme-color/material",
  "npm/theme-color/night-owl",
  "npm/theme-color/one-dark",
  "npm/theme-color/rose-pine",
  "npm/theme-color/solarized",
  "npm/theme-color/vitesse",
];
const packDir = mkdtempSync(join(tmpdir(), "ox-content-pack-"));
const failures = [];
const publicVitePeerRange = "^0.2.8 || ^8.0.0";

try {
  for (const packageDir of packages) {
    checkPackage(packageDir);
  }
} finally {
  rmSync(packDir, { recursive: true, force: true });
}

if (failures.length > 0) {
  console.error("Package dry-run checks failed:");
  for (const failure of failures) {
    console.error(`  - ${failure}`);
  }
  process.exit(1);
}

console.log("Package dry-run checks passed.");

function checkPackage(packageDir) {
  const pkg = JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8"));
  const packed = pack(packageDir);
  const packedPkg = readPackedPackageJson(packed.filename);
  /** @type {Set<string>} */
  const files = new Set(packed.files.map((file) => String(file.path)));

  console.log(`\n${pkg.name}@${pkg.version}`);
  for (const file of [...files].sort((a, b) => a.localeCompare(b))) {
    console.log(`  ${file}`);
  }

  requirePackedFile(files, "package.json", pkg.name);
  checkPackageJsonReferences(packedPkg, files);
  checkVitePeerDependency(packedPkg);

  if (pkg.name === "@ox-content/napi") {
    checkNapiPackage(packageDir, pkg, files);
  }
  if (pkg.name === "@ox-content/vite-plugin") {
    checkVitePluginDeclarations({
      pkg,
      tarball: packed.filename,
      failures,
      readPackedFile,
    });
    checkReaderChromeDeclarations({
      pkg,
      tarball: packed.filename,
      packDir,
      failures,
      readPackedFile,
    });
    checkTwitterClientDeclarations({
      pkg,
      tarball: packed.filename,
      packDir,
      failures,
      readPackedFile,
    });
  }
}

function readPackedPackageJson(filename) {
  return JSON.parse(readPackedFile(filename, "package.json"));
}

function readPackedFile(filename, path) {
  const result = spawnSync("tar", ["-xOf", filename, `package/${path}`], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(result.stderr || `Failed to read ${path} from ${filename}`);
  }

  return result.stdout;
}

function checkVitePeerDependency(pkg) {
  if (!("vite" in (pkg.peerDependencies ?? {}))) {
    return;
  }

  if (pkg.peerDependencies.vite !== publicVitePeerRange) {
    failures.push(
      `${pkg.name} publishes vite peer ${pkg.peerDependencies.vite}; expected ${publicVitePeerRange}`,
    );
  }
}

function pack(packageDir) {
  const result = spawnSync(
    "vp",
    ["exec", "--", "pnpm", "--dir", packageDir, "pack", "--json", "--pack-destination", packDir],
    { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
  );

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(result.stderr || `pnpm pack failed for ${packageDir}`);
  }

  return parsePackOutput(result.stdout);
}

function parsePackOutput(output) {
  const jsonStart = output.search(/[[{]/);
  if (jsonStart === -1) {
    throw new Error(`pnpm pack did not emit JSON output:\n${output}`);
  }

  return JSON.parse(output.slice(jsonStart));
}

function checkPackageJsonReferences(pkg, files) {
  if (pkg.main) {
    requirePackedFile(files, pkg.main, `${pkg.name} main`);
  }
  if (pkg.types) {
    requirePackedFile(files, pkg.types, `${pkg.name} types`);
  }
  for (const [name, target] of Object.entries(pkg.bin ?? {})) {
    requirePackedFile(files, target, `${pkg.name} bin ${name}`);
  }
  collectExportTargets(pkg.exports).forEach(({ condition, target }) => {
    requirePackedFile(files, target, `${pkg.name} export ${condition}`);
  });
}

function collectExportTargets(exportsField, prefix = "exports") {
  if (!exportsField) {
    return [];
  }
  if (typeof exportsField === "string") {
    return [{ condition: prefix, target: exportsField }];
  }
  if (typeof exportsField !== "object") {
    return [];
  }

  const targets = [];
  for (const [key, value] of Object.entries(exportsField)) {
    const nextPrefix = `${prefix}.${key}`;
    if (typeof value === "string") {
      targets.push({ condition: nextPrefix, target: value });
    } else {
      targets.push(...collectExportTargets(value, nextPrefix));
    }
  }
  return targets;
}

function checkNapiPackage(packageDir, pkg, files) {
  requirePackedFile(files, "index.js", "@ox-content/napi wrapper");
  requirePackedFile(files, "index.d.ts", "@ox-content/napi declarations");

  const targets = new Set(pkg.napi?.targets ?? []);
  for (const target of [
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-gnu",
    "aarch64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
  ]) {
    if (!targets.has(target)) {
      failures.push(`@ox-content/napi is missing NAPI target ${target}`);
    }
  }

  const wrapper = readFileSync(join(packageDir, "index.js"), "utf8");
  for (const bindingPackage of [
    "@ox-content/binding-darwin-arm64",
    "@ox-content/binding-darwin-x64",
    "@ox-content/binding-linux-x64-gnu",
    "@ox-content/binding-linux-x64-musl",
    "@ox-content/binding-linux-arm64-gnu",
    "@ox-content/binding-linux-arm64-musl",
    "@ox-content/binding-win32-x64-msvc",
  ]) {
    if (!wrapper.includes(bindingPackage)) {
      failures.push(`@ox-content/napi loader is missing ${bindingPackage}`);
    }
  }
}

function requirePackedFile(files, target, label) {
  const normalized = normalizePackagePath(target);
  if (!files.has(normalized)) {
    failures.push(`${label} points to ${target}, but ${normalized} is not in the packed files`);
  }
}

function normalizePackagePath(target) {
  return relative(".", target)
    .replace(/\\/g, "/")
    .replace(/^(\.\.\/)+/, "");
}
