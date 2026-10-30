#!/usr/bin/env node

import { appendFileSync, readFileSync } from "node:fs";

const options = parseOptions(process.argv.slice(2));
const base = JSON.parse(readFileSync(options.base, "utf8"));
const head = JSON.parse(readFileSync(options.head, "utf8"));

if (base.skipped || head.skipped) {
  process.exit(0);
}

const rows = [];
let regression = false;
for (const size of Object.keys(head.sizes)) {
  const baseSize = base.sizes[size];
  const headSize = head.sizes[size];
  if (!baseSize || !headSize) {
    throw new Error(`Missing collection benchmark size ${size}`);
  }
  for (const [label, baseMs, headMs] of metrics(baseSize, headSize)) {
    const delta = (headMs / baseMs - 1) * 100;
    if (delta > options.maxRegression) {
      regression = true;
    }
    rows.push(
      `| ${Number(size).toLocaleString()} | ${label} | ${formatMs(baseMs)} | ${formatMs(headMs)} | ${formatDelta(delta)} |`,
    );
  }
}

appendFileSync(
  options.append,
  `\n## Collection scale\n\n| Entries | Operation | Base | Head | Delta |\n| ---: | --- | ---: | ---: | ---: |\n${rows.join("\n")}\n`,
);

if (regression && process.env.OX_CONTENT_BENCHMARK_ALLOW_REGRESSION !== "1") {
  console.error(`Collection scale regression exceeded ${options.maxRegression}%`);
  process.exit(1);
}

function metrics(baseSize, headSize) {
  const rows = [
    ["native manifest", baseSize.manifestMs, headSize.manifestMs],
    ["path first (indexed)", baseSize.queries.pathFirstMs, headSize.queries.pathFirstMs],
    ["LIKE count", baseSize.queries.likeCountMs, headSize.queries.likeCountMs],
    ["nested count", baseSize.queries.nestedCountMs, headSize.queries.nestedCountMs],
    ["filter + order + limit", baseSize.queries.orderedPageMs, headSize.queries.orderedPageMs],
  ];
  if (baseSize.themeRenderAllMs !== undefined && headSize.themeRenderAllMs !== undefined) {
    rows.push(["custom theme render all", baseSize.themeRenderAllMs, headSize.themeRenderAllMs]);
  }
  return rows;
}

function formatMs(value) {
  return `${value < 0.01 ? value.toFixed(4) : value.toFixed(2)} ms`;
}

function formatDelta(value) {
  return `${value >= 0 ? "+" : ""}${value.toFixed(1)}%`;
}

function parseOptions(args) {
  const parsed = { base: null, head: null, append: null, maxRegression: 20 };
  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--base" || arg === "--head" || arg === "--append") {
      parsed[arg.slice(2)] = readValue(args, ++index, arg);
      continue;
    }
    if (arg === "--max-regression") {
      parsed.maxRegression = Number.parseFloat(readValue(args, ++index, arg));
      continue;
    }
    throw new Error(`Unknown argument: ${arg}`);
  }
  for (const required of ["base", "head", "append"]) {
    if (!parsed[required]) {
      throw new Error(`--${required} is required`);
    }
  }
  return parsed;
}

function readValue(args, index, option) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${option} requires a value`);
  }
  return value;
}
