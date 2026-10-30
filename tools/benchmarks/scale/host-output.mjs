#!/usr/bin/env node

/**
 * Measures a custom host that returns already bundled HTML. This matches Vue
 * and other framework integrations: their pages contain external Vite assets,
 * not the inline markers emitted by Ox Content's built-in theme.
 *
 * Usage:
 *   node host-output.mjs [--pages N] [--bytes-per-page N] [--iterations N]
 *                        [--externalize-assets true|false] [--json PATH]
 */

import { execFileSync } from "node:child_process";
import { readdirSync, rmSync, statSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import { cpus, totalmem } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { generateSite } from "./site.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const contentDir = join(here, "content");

function options(argv) {
  const parsed = {
    pages: 2000,
    bytesPerPage: 65_536,
    iterations: 5,
    externalizeAssets: true,
    jsonPath: null,
  };
  for (let index = 0; index < argv.length; index++) {
    const name = argv[index];
    const value = argv[++index];
    if (!value) throw new Error(`${name} requires a value`);
    if (name === "--pages") parsed.pages = Number(value);
    else if (name === "--bytes-per-page") parsed.bytesPerPage = Number(value);
    else if (name === "--iterations") parsed.iterations = Number(value);
    else if (name === "--externalize-assets") parsed.externalizeAssets = value !== "false";
    else if (name === "--json") parsed.jsonPath = value;
    else throw new Error(`unknown option: ${name}`);
  }
  for (const [name, value] of Object.entries(parsed)) {
    if (
      name !== "jsonPath" &&
      name !== "externalizeAssets" &&
      (!Number.isInteger(value) || value < 1)
    ) {
      throw new Error(`${name} must be a positive integer`);
    }
  }
  return parsed;
}

function build(config) {
  rmSync(join(here, "dist-host"), { recursive: true, force: true });
  const started = process.hrtime.bigint();
  execFileSync("vp", ["build"], {
    cwd: here,
    stdio: "pipe",
    env: {
      ...process.env,
      OX_SCALE_HOST_HTML: "1",
      OX_SCALE_HOST_PADDING_BYTES: String(config.bytesPerPage),
      OX_SCALE_EXTERNALIZE_ASSETS: config.externalizeAssets ? "1" : "0",
      OX_SCALE_OUT_DIR: "dist-host",
    },
  });
  const elapsedMs = Number(process.hrtime.bigint() - started) / 1e6;
  validateOutput(config);
  return elapsedMs;
}

function validateOutput(config) {
  let htmlPages = 0;
  let htmlBytes = 0;
  const visit = (dir) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const file = join(dir, entry.name);
      if (entry.isDirectory()) visit(file);
      else if (entry.name.endsWith(".html")) {
        htmlPages++;
        htmlBytes += statSync(file).size;
      }
    }
  };
  visit(join(here, "dist-host"));
  if (htmlPages !== config.pages || htmlBytes < config.pages * config.bytesPerPage) {
    throw new Error(`invalid output: ${htmlPages} pages and ${htmlBytes} HTML bytes`);
  }
}

async function main() {
  const config = options(process.argv.slice(2));
  rmSync(contentDir, { recursive: true, force: true });
  await mkdir(contentDir, { recursive: true });
  await generateSite(here, config.pages, {
    containers: true,
    images: true,
    math: true,
    mermaid: false,
  });

  const samples = [];
  for (let index = 0; index < config.iterations; index++) samples.push(build(config));
  const sorted = [...samples].sort((left, right) => left - right);
  const result = {
    machine: { cpus: cpus().length, memoryGb: Math.round(totalmem() / 1024 ** 3) },
    pages: config.pages,
    bytesPerPage: config.bytesPerPage,
    externalizeAssets: config.externalizeAssets,
    outputMb: (config.pages * config.bytesPerPage) / 1024 ** 2,
    samplesMs: samples,
    fastestMs: sorted[0],
    medianMs: sorted[Math.floor(sorted.length / 2)],
  };

  console.log(JSON.stringify(result, null, 2));
  if (config.jsonPath) await writeFile(config.jsonPath, `${JSON.stringify(result, null, 2)}\n`);
}

await main();
