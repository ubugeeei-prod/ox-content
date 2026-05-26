#!/usr/bin/env node

import { readFileSync, writeFileSync } from "node:fs";

const COMMENT_MARKER = "<!-- ox-content-benchmark-report -->";
const TARGET_NAMES = new Set(["@ox-content/napi", "@ox-content/napi (async)"]);
const COMMENT_SIZE_NAMES = new Set(["large"]);
const NOISE_THRESHOLD_PERCENT = 5;
const RUNTIME_REGRESSION_THRESHOLD_PERCENT = -10;
const BUNDLE_REGRESSION_THRESHOLD_PERCENT = 5;
const BENCHMARK_OVERRIDE_ENV = "OX_CONTENT_BENCHMARK_ALLOW_REGRESSION";
const BUNDLE_APP_ORDER = [
  "ox-content (bare)",
  "ox-content (default)",
  "ox-content + Vue",
  "VitePress (bare)",
  "VitePress (default)",
  "Astro",
  "Astro + Vue",
];

const SUITE_LABELS = {
  parseOnly: "Parse only",
  parseAndRender: "Parse + render",
  parseAndRenderAsync: "Parse + render async",
};

const SIZE_ORDER = ["small", "medium", "large"];
const SUITE_ORDER = ["parseOnly", "parseAndRender", "parseAndRenderAsync"];

const options = parseOptions(process.argv.slice(2));
const body = buildComment(options);

if (options.outputPath) {
  writeFileSync(options.outputPath, body);
}

console.log(body);

if (body.includes("<!-- ox-content-benchmark-regression -->")) {
  process.exitCode = 1;
}

function parseOptions(args) {
  const parsed = {
    basePath: null,
    headPath: null,
    baseBundlePath: null,
    headBundlePath: null,
    outputPath: null,
    baseSha: null,
    headSha: null,
  };

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--base") {
      parsed.basePath = readOptionValue(args, ++index, "--base");
      continue;
    }
    if (arg === "--head") {
      parsed.headPath = readOptionValue(args, ++index, "--head");
      continue;
    }
    if (arg === "--base-bundle") {
      parsed.baseBundlePath = readOptionValue(args, ++index, "--base-bundle");
      continue;
    }
    if (arg === "--head-bundle") {
      parsed.headBundlePath = readOptionValue(args, ++index, "--head-bundle");
      continue;
    }
    if (arg === "--output") {
      parsed.outputPath = readOptionValue(args, ++index, "--output");
      continue;
    }
    if (arg === "--base-sha") {
      parsed.baseSha = readOptionValue(args, ++index, "--base-sha");
      continue;
    }
    if (arg === "--head-sha") {
      parsed.headSha = readOptionValue(args, ++index, "--head-sha");
      continue;
    }
    if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  if (!parsed.basePath || !parsed.headPath) {
    throw new Error("--base and --head are required");
  }
  if (Boolean(parsed.baseBundlePath) !== Boolean(parsed.headBundlePath)) {
    throw new Error("--base-bundle and --head-bundle must be used together");
  }

  return parsed;
}

function readOptionValue(args, index, optionName) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${optionName} requires a value`);
  }

  return value;
}

function printUsage() {
  console.log(`Usage: node compare-pr-benchmark.mjs --base <path> --head <path> [--output <path>]

Options:
  --base <path>      Base benchmark JSON
  --head <path>      Head benchmark JSON
  --base-bundle <path> Base bundle size JSON
  --head-bundle <path> Head bundle size JSON
  --output <path>    Write the Markdown comment to a file
  --base-sha <sha>   Base commit SHA for the heading
  --head-sha <sha>   Head commit SHA for the heading
  -h, --help         Show this help message`);
}

function buildComment({ basePath, headPath, baseBundlePath, headBundlePath, baseSha, headSha }) {
  const base = readJson(basePath);
  const head = readJson(headPath);
  const runtimeRows = compareRuntimeReports(base, head);
  const bundleRows =
    baseBundlePath && headBundlePath
      ? compareBundleReports(readJson(baseBundlePath), readJson(headBundlePath))
      : [];
  const summary = summarizeRows(runtimeRows, bundleRows);
  const regressions = collectRegressions(runtimeRows, bundleRows);
  const overrideEnabled = process.env[BENCHMARK_OVERRIDE_ENV] === "1";
  const compareText =
    baseSha && headSha
      ? `Comparing base \`${shortSha(baseSha)}\` to head \`${shortSha(headSha)}\`.`
      : "Comparing base to head.";

  const lines = [
    COMMENT_MARKER,
    "## Benchmark report",
    "",
    `${summary} ${compareText}`,
    "",
    "### Runtime",
    "",
    "| Suite | Size | Target | Base time | Head time | Head speed (base=100%) |",
    "| --- | --- | --- | ---: | ---: | ---: |",
  ];

  if (runtimeRows.length === 0) {
    lines.push("| n/a | n/a | n/a | n/a | n/a | n/a |");
  } else {
    for (const row of runtimeRows) {
      lines.push(
        `| ${row.suiteLabel} | ${row.sizeName} | ${row.targetName} | ${formatTimeMs(row.baseOps)} | ${formatTimeMs(row.headOps)} | ${formatSpeedPercent(row.speedPercent)} |`,
      );
    }
  }

  lines.push(
    "",
    `Values are from the large runtime benchmarks. Time is shown in milliseconds to microsecond precision. Head speed is normalized with base = 1.00 (100%); changes within +/-${NOISE_THRESHOLD_PERCENT}% are treated as noise.`,
    "",
  );

  if (baseBundlePath && headBundlePath) {
    lines.push(
      "### Bundle Size",
      "",
      "| App | Base gzip | Head gzip | Delta | Base requests | Head requests | Base files | Head files |",
      "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    );

    if (bundleRows.length === 0) {
      lines.push("| n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |");
    } else {
      for (const row of bundleRows) {
        lines.push(
          `| ${String(row.name)} | ${formatBytes(row.baseGzipped)} | ${formatBytes(row.headGzipped)} | ${formatDelta(row.deltaPercent)} | ${formatNumber(row.baseRequests)} | ${formatNumber(row.headRequests)} | ${formatNumber(row.baseFiles)} | ${formatNumber(row.headFiles)} |`,
        );
      }
    }

    lines.push(
      "",
      "Bundle size uses gzipped JS/CSS/HTML/JSON assets. Requests estimate index.html plus referenced local initial assets. Lower is better.",
      "",
    );
  }

  lines.push(
    "### Regression Gate",
    "",
    `Runtime regressions fail when head throughput is more than ${Math.abs(RUNTIME_REGRESSION_THRESHOLD_PERCENT)}% slower than base. Bundle regressions fail when gzipped size grows by more than ${BUNDLE_REGRESSION_THRESHOLD_PERCENT}%. Maintainers can intentionally override by applying the \`benchmark-regression-accepted\` PR label, which sets \`${BENCHMARK_OVERRIDE_ENV}=1\`.`,
    "",
  );

  if (regressions.length === 0) {
    lines.push("No threshold regressions found.", "");
  } else {
    if (!overrideEnabled) {
      lines.push("<!-- ox-content-benchmark-regression -->");
    }
    lines.push(
      overrideEnabled
        ? "Threshold regressions were found, but the maintainer override is active."
        : "Threshold regressions were found and this check should fail.",
      "",
      "| Metric | Target | Delta | Threshold |",
      "| --- | --- | ---: | ---: |",
    );
    for (const regression of regressions) {
      lines.push(
        `| ${regression.metric} | ${regression.target} | ${formatDelta(regression.deltaPercent)} | ${formatDelta(regression.thresholdPercent)} |`,
      );
    }
    lines.push("");
  }

  return lines.join("\n");
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function compareRuntimeReports(base, head) {
  const baseRows = new Map(collectRows(base).map((row) => [row.key, row]));
  const headRows = new Map(collectRows(head).map((row) => [row.key, row]));
  const keys = [...new Set([...baseRows.keys(), ...headRows.keys()])];

  return keys.sort(compareKeys).map((key) => {
    const baseRow = baseRows.get(key);
    const headRow = headRows.get(key);
    const baseOps = baseRow?.opsPerSec ?? null;
    const headOps = headRow?.opsPerSec ?? null;
    return {
      key,
      suiteKey: headRow?.suiteKey ?? baseRow?.suiteKey ?? "unknown",
      suiteLabel: SUITE_LABELS[headRow?.suiteKey ?? baseRow?.suiteKey] ?? "Unknown",
      sizeName: headRow?.sizeName ?? baseRow?.sizeName ?? "unknown",
      targetName: headRow?.targetName ?? baseRow?.targetName ?? "unknown",
      baseOps,
      headOps,
      deltaPercent: percentChange(baseOps, headOps),
      speedPercent: speedPercent(baseOps, headOps),
    };
  });
}

function compareBundleReports(base, head) {
  const baseRows = new Map(collectBundleRows(base).map((row) => [row.name, row]));
  const headRows = new Map(collectBundleRows(head).map((row) => [row.name, row]));
  const names = [...new Set([...baseRows.keys(), ...headRows.keys()])];

  return names.sort(compareBundleNames).map((name) => {
    const baseRow = baseRows.get(name);
    const headRow = headRows.get(name);
    const baseGzipped = baseRow?.gzipped ?? null;
    const headGzipped = headRow?.gzipped ?? null;

    return {
      name,
      baseGzipped,
      headGzipped,
      baseRequests: baseRow?.requests ?? null,
      headRequests: headRow?.requests ?? null,
      baseFiles: baseRow?.files ?? null,
      headFiles: headRow?.files ?? null,
      deltaPercent: percentChange(baseGzipped, headGzipped),
    };
  });
}

function collectBundleRows(report) {
  return (report.results ?? [])
    .filter((result) => !result.error)
    .map((result) => ({
      name: String(result.name),
      gzipped: Number(result.gzipped),
      requests: Number(result.requests),
      files: Number(result.files),
    }));
}

function collectRows(report) {
  const rows = [];

  for (const [sizeName, sizeReport] of Object.entries(report.sizes ?? {})) {
    for (const [suiteKey, results] of Object.entries(sizeReport.suites ?? {})) {
      for (const result of results) {
        if (!TARGET_NAMES.has(result.name) || result.error) {
          continue;
        }
        if (!COMMENT_SIZE_NAMES.has(sizeName)) {
          continue;
        }

        rows.push({
          key: `${sizeName}:${suiteKey}:${result.name}`,
          sizeName,
          suiteKey,
          targetName: result.name,
          opsPerSec: result.opsPerSec,
        });
      }
    }
  }

  return rows;
}

function compareKeys(left, right) {
  const [leftSize, leftSuite, leftName] = left.split(":");
  const [rightSize, rightSuite, rightName] = right.split(":");
  const sizeDiff = orderIndex(SIZE_ORDER, leftSize) - orderIndex(SIZE_ORDER, rightSize);
  if (sizeDiff !== 0) {
    return sizeDiff;
  }

  const suiteDiff = orderIndex(SUITE_ORDER, leftSuite) - orderIndex(SUITE_ORDER, rightSuite);
  if (suiteDiff !== 0) {
    return suiteDiff;
  }

  return leftName.localeCompare(rightName);
}

function compareBundleNames(left, right) {
  const appDiff = orderIndex(BUNDLE_APP_ORDER, left) - orderIndex(BUNDLE_APP_ORDER, right);
  if (appDiff !== 0) {
    return appDiff;
  }

  return left.localeCompare(right);
}

function orderIndex(order, value) {
  const index = order.indexOf(value);
  return index === -1 ? order.length : index;
}

function percentChange(baseValue, headValue) {
  if (!Number.isFinite(baseValue) || !Number.isFinite(headValue) || baseValue <= 0) {
    return null;
  }

  return ((headValue - baseValue) / baseValue) * 100;
}

function speedPercent(baseValue, headValue) {
  if (!Number.isFinite(baseValue) || !Number.isFinite(headValue) || baseValue <= 0) {
    return null;
  }

  return (headValue / baseValue) * 100;
}

function summarizeRows(runtimeRows, bundleRows) {
  const runtimeCount = runtimeRows.filter((row) => row.deltaPercent !== null).length;
  const bundleCount = bundleRows.filter((row) => row.deltaPercent !== null).length;

  if (runtimeCount === 0 && bundleCount === 0) {
    return "No comparable benchmark rows were found.";
  }

  const parts = [];
  if (runtimeCount > 0) {
    parts.push(`${runtimeCount} runtime row${runtimeCount === 1 ? "" : "s"}`);
  }
  if (bundleCount > 0) {
    parts.push(`${bundleCount} bundle row${bundleCount === 1 ? "" : "s"}`);
  }

  return `${parts.join(" and ")} compared.`;
}

function collectRegressions(runtimeRows, bundleRows) {
  const regressions = [];

  for (const row of runtimeRows) {
    if (
      Number.isFinite(row.deltaPercent) &&
      row.deltaPercent < RUNTIME_REGRESSION_THRESHOLD_PERCENT
    ) {
      regressions.push({
        metric: `${row.suiteLabel} runtime`,
        target: `${row.sizeName} / ${row.targetName}`,
        deltaPercent: row.deltaPercent,
        thresholdPercent: RUNTIME_REGRESSION_THRESHOLD_PERCENT,
      });
    }
  }

  for (const row of bundleRows) {
    if (
      Number.isFinite(row.deltaPercent) &&
      row.deltaPercent > BUNDLE_REGRESSION_THRESHOLD_PERCENT
    ) {
      regressions.push({
        metric: "Bundle gzip",
        target: row.name,
        deltaPercent: row.deltaPercent,
        thresholdPercent: BUNDLE_REGRESSION_THRESHOLD_PERCENT,
      });
    }
  }

  return regressions;
}

function formatNumber(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }

  return Math.round(value).toLocaleString("en-US");
}

function formatTimeMs(opsPerSec) {
  if (!Number.isFinite(opsPerSec) || opsPerSec <= 0) {
    return "n/a";
  }

  return `${(1000 / opsPerSec).toFixed(3)} ms`;
}

function formatSpeedPercent(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }

  return `${value.toFixed(2)}%`;
}

function formatBytes(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }

  return `${(value / (1024 * 1024)).toFixed(2)} MB`;
}

function formatDelta(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }

  const sign = value > 0 ? "+" : "";
  return `${sign}${value.toFixed(2)}%`;
}

function shortSha(value) {
  return value.slice(0, 7);
}
