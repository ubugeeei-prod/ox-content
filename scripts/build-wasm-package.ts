#!/usr/bin/env -S node --experimental-strip-types

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const CRATE_DIR = path.join(ROOT, "crates", "ox_content_wasm");
const PKG_DIR = path.join(CRATE_DIR, "pkg");
const PKG_JSON_PATH = path.join(PKG_DIR, "package.json");
const README_SOURCE = path.join(CRATE_DIR, "README.md");
const LICENSE_SOURCE = path.join(ROOT, "LICENSE");

type PackageJson = {
  name: string;
  version: string;
  description?: string;
  files?: string[];
  homepage?: string;
  keywords?: string[];
  main?: string;
  type?: string;
  types?: string;
  repository?: {
    type: string;
    url: string;
    directory?: string;
  };
  publishConfig?: {
    access?: string;
    provenance?: boolean;
  };
  bugs?: {
    url: string;
  };
  author?: string;
  collaborators?: string[];
  [key: string]: unknown;
};

function run(command: string, args: string[], cwd = ROOT): void {
  console.log(`$ ${command} ${args.join(" ")}`.trim());
  execFileSync(command, args, { cwd, stdio: "inherit" });
}

function readJson(filePath: string): PackageJson {
  return JSON.parse(fs.readFileSync(filePath, "utf8")) as PackageJson;
}

function writeJson(filePath: string, value: PackageJson): void {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function copyFile(source: string, target: string): void {
  fs.copyFileSync(source, target);
  console.log(`  copied ${path.relative(ROOT, source)} -> ${path.relative(ROOT, target)}`);
}

function buildWasmPackage(): void {
  run("wasm-pack", ["build", "--target", "web", "--out-dir", "pkg"], CRATE_DIR);
}

function normalizePackageJson(): void {
  const pkg = readJson(PKG_JSON_PATH);

  pkg.name = "@ox-content/wasm";
  pkg.author = "ubugeeei";
  delete pkg.collaborators;
  pkg.homepage = "https://ubugeeei.github.io/ox-content/packages/wasm";
  pkg.repository = {
    type: "git",
    url: "https://github.com/ubugeeei/ox-content.git",
    directory: "crates/ox_content_wasm",
  };
  pkg.bugs = {
    url: "https://github.com/ubugeeei/ox-content/issues",
  };
  pkg.publishConfig = {
    access: "public",
  };
  pkg.files = [
    "ox_content_wasm.js",
    "ox_content_wasm.d.ts",
    "ox_content_wasm_bg.wasm",
    "ox_content_wasm_bg.wasm.d.ts",
  ];
  pkg.keywords = Array.from(
    new Set([...(pkg.keywords ?? []), "browser", "markdown", "wasm", "webassembly"]),
  );

  writeJson(PKG_JSON_PATH, pkg);
  console.log(`  normalized ${path.relative(ROOT, PKG_JSON_PATH)}`);
}

function copyPublishAssets(): void {
  copyFile(README_SOURCE, path.join(PKG_DIR, "README.md"));
  copyFile(LICENSE_SOURCE, path.join(PKG_DIR, "LICENSE"));
}

function main(): void {
  buildWasmPackage();

  if (!fs.existsSync(PKG_JSON_PATH)) {
    throw new Error(`Expected generated package at ${PKG_JSON_PATH}`);
  }

  normalizePackageJson();
  copyPublishAssets();

  console.log(`\nWASM package is ready in ${path.relative(ROOT, PKG_DIR)}`);
  console.log("Next steps:");
  console.log("  pnpm --filter @ox-content/wasm pack --dry-run");
  console.log("  pnpm --filter @ox-content/wasm publish --access public --no-git-checks");
}

main();
