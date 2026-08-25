#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const result = spawnSync(
  process.execPath,
  [fileURLToPath(new URL("../src/bundle-browser.ts", import.meta.url))],
  { stdio: "inherit" },
);

process.exit(result.status ?? 1);
