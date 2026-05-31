#!/usr/bin/env node

// Render the parse / parse+render comparison bar charts under `docs/public/`
// from a benchmark JSON report produced by
// `benchmarks/bundle-size/parse-benchmark.mjs --json <path>`.
//
// Keeping the charts generated (rather than hand-edited) means refreshing the
// numbers — or adding a competitor / input size — is a single command:
//
//   node benchmarks/bundle-size/parse-benchmark.mjs --runs 7 --json /tmp/b.json
//   node scripts/render-benchmark-charts.mjs /tmp/b.json --size large
//
// For the `large` size the output keeps the historical file names
// (`benchmark-parse.svg`, `benchmark-render.svg`); other sizes are suffixed
// (e.g. `benchmark-parse-huge.svg`).

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const PUBLIC_DIR = join(__dirname, "..", "docs", "public");

const WIDTH = 650;
const LABEL_X = 120; // right edge of the library label column
const BAR_X = 130; // left edge of the bars
const BAR_MAX = 300; // width of the longest (fastest) bar
const BAR_H = 20;
const ROW_H = 30;
const FIRST_TOP = 42; // top y of the first bar
const VALUE_X = 540; // right-aligned ops/sec value
const UNIT_X = 545; // left-aligned "ops/s" unit
const RATIO_X = 636; // right-aligned ratio

function parseArgs(argv) {
  const opts = { jsonPath: "/tmp/ox-bench-result.json", size: "large" };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--size") {
      opts.size = argv[++i];
    } else if (arg.startsWith("--size=")) {
      opts.size = arg.slice("--size=".length);
    } else if (!arg.startsWith("--")) {
      opts.jsonPath = arg;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return opts;
}

function sizeLabel(sizeName, bytes) {
  const kb = bytes / 1024;
  const human = kb >= 1024 ? `${(kb / 1024).toFixed(2)} MB` : `${kb.toFixed(1)} KB`;
  const titled = sizeName.charAt(0).toUpperCase() + sizeName.slice(1);
  return `${titled} ${human}`;
}

function formatOps(ops) {
  return Math.round(ops).toLocaleString("en-US");
}

function formatRatio(fastest, ops) {
  if (ops >= fastest) return "1.00x";
  const ratio = fastest / ops;
  return ratio >= 100 ? `${Math.round(ratio)}x` : `${ratio.toFixed(2)}x`;
}

function escapeXml(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function renderChart(title, caption, rows) {
  const valid = rows.filter((r) => !r.error && r.opsPerSec > 0);
  const fastest = Math.max(...valid.map((r) => r.opsPerSec));
  const height = FIRST_TOP + rows.length * ROW_H + 26;

  const bars = rows
    .map((row, index) => {
      const top = FIRST_TOP + index * ROW_H;
      const baseline = top + 14;
      const label = escapeXml(row.name);
      if (row.error || !row.opsPerSec) {
        return [
          `  <text x="${LABEL_X}" y="${baseline}" text-anchor="end" class="label">${label}</text>`,
          `  <rect x="${BAR_X}" y="${top}" width="4" height="${BAR_H}" rx="4" fill="url(#grayGrad)"/>`,
          `  <text x="${VALUE_X}" y="${baseline}" text-anchor="end" class="value">Error</text>`,
        ].join("\n");
      }
      const isOx = row.name.includes("ox-content");
      const width = Math.max(4, Math.round((BAR_MAX * row.opsPerSec) / fastest));
      const fill = isOx ? "url(#oxGrad)" : "url(#grayGrad)";
      return [
        `  <text x="${LABEL_X}" y="${baseline}" text-anchor="end" class="label">${label}</text>`,
        `  <rect x="${BAR_X}" y="${top}" width="${width}" height="${BAR_H}" rx="4" fill="${fill}"/>`,
        `  <text x="${VALUE_X}" y="${baseline}" text-anchor="end" class="value">${formatOps(row.opsPerSec)}</text>`,
        `  <text x="${UNIT_X}" y="${baseline}" class="unit">ops/s</text>`,
        `  <text x="${RATIO_X}" y="${baseline}" text-anchor="end" class="ratio">${formatRatio(fastest, row.opsPerSec)}</text>`,
      ].join("\n");
    })
    .join("\n\n");

  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${WIDTH} ${height}">
  <defs>
    <linearGradient id="oxGrad" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" style="stop-color:#7a5cff"/>
      <stop offset="100%" style="stop-color:#4acdff"/>
    </linearGradient>
    <linearGradient id="grayGrad" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" style="stop-color:#4f607b"/>
      <stop offset="100%" style="stop-color:#8190b0"/>
    </linearGradient>
  </defs>

  <style>
    .title { font: bold 14px IBM Plex Sans, system-ui, sans-serif; fill: #131a30; }
    .label { font: 12px IBM Plex Sans, system-ui, sans-serif; fill: #4f607b; }
    .value { font: bold 11px IBM Plex Sans, system-ui, sans-serif; fill: #131a30; }
    .unit { font: 10px IBM Plex Sans, system-ui, sans-serif; fill: #8190b0; }
    .ratio { font: bold 10px IBM Plex Sans, system-ui, sans-serif; fill: #364462; }
  </style>

  <rect width="${WIDTH}" height="${height}" fill="#f5f7fb" rx="8"/>

  <text x="${WIDTH / 2}" y="24" text-anchor="middle" class="title">${escapeXml(title)}</text>

${bars}

  <text x="${WIDTH / 2}" y="${height - 12}" text-anchor="middle" class="unit">${escapeXml(caption)}</text>
</svg>
`;
}

function main() {
  const opts = parseArgs(process.argv.slice(2));
  const report = JSON.parse(readFileSync(opts.jsonPath, "utf8"));
  const size = report.sizes?.[opts.size];
  if (!size) {
    throw new Error(`Size "${opts.size}" not found in ${opts.jsonPath}`);
  }

  const label = sizeLabel(opts.size, size.bytes);
  const suffix = opts.size === "large" ? "" : `-${opts.size}`;

  const charts = [
    {
      suite: "parseOnly",
      file: `benchmark-parse${suffix}.svg`,
      title: `Parse Speed (${label})`,
      caption: "Higher is better. Ratio shows how much slower vs ox-content.",
    },
    {
      suite: "parseAndRender",
      file: `benchmark-render${suffix}.svg`,
      title: `Parse + Render Speed (${label})`,
      caption: "Higher is better. Ratio shows how much slower vs ox-content.",
    },
  ];

  for (const chart of charts) {
    const rows = size.suites?.[chart.suite];
    if (!rows) {
      console.warn(`No "${chart.suite}" suite for size "${opts.size}", skipping ${chart.file}`);
      continue;
    }
    const svg = renderChart(chart.title, chart.caption, rows);
    const outPath = join(PUBLIC_DIR, chart.file);
    writeFileSync(outPath, svg);
    console.log(`Wrote ${outPath}`);
  }
}

main();
