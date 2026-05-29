#!/usr/bin/env -S node --experimental-strip-types

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

const DIST = path.join(ROOT, "dist");

// Build outputs produced by `vp build` (build:docs / doc:cargo / build:playground),
// assembled into a single directory for `void deploy --dir dist`.
const parts = [
  { src: path.join(ROOT, "docs", "dist", "docs"), dest: DIST, label: "docs site -> /" },
  { src: path.join(ROOT, "target", "doc"), dest: path.join(DIST, "api"), label: "rust docs -> /api" },
  {
    src: path.join(ROOT, "examples", "playground", "dist"),
    dest: path.join(DIST, "playground"),
    label: "playground -> /playground",
  },
];

const missing = parts.filter((p) => !fs.existsSync(p.src)).map((p) => path.relative(ROOT, p.src));
if (missing.length > 0) {
  console.error(`Missing build output(s): ${missing.join(", ")}\nRun \`vp build\` first.`);
  process.exit(1);
}

fs.rmSync(DIST, { recursive: true, force: true });
fs.mkdirSync(DIST, { recursive: true });

for (const { src, dest, label } of parts) {
  fs.cpSync(src, dest, { recursive: true });
  console.log(`assembled ${label}`);
}

console.log(`site assembled at ${path.relative(ROOT, DIST)}/`);
