#!/usr/bin/env node

import { appendFileSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const OVERRIDE_ENV = "OX_CONTENT_BENCHMARK_ALLOW_REGRESSION";
const REGRESSION_MARKER = "<!-- ox-content-budget-regression -->";
const DEFAULT_BUDGETS = join(dirname(fileURLToPath(import.meta.url)), "..", "perf-budgets.json");

const APP_METRICS = [
  { limitKey: "gzippedBytes", field: "gzipped", label: "Bundle gzip" },
  { limitKey: "htmlGzippedBytes", field: "htmlGzipped", label: "Rendered HTML gzip" },
  { limitKey: "buildMs", field: "buildMs", label: "Build time" },
  { limitKey: "requests", field: "requests", label: "Initial requests" },
];

const options = parseOptions(process.argv.slice(2));
const budgets = readJson(options.budgetsPath);
const bundleReport = options.bundlePath ? readJson(options.bundlePath) : null;
const runtimeReport = options.runtimePath ? readJson(options.runtimePath) : null;
const buildReport = options.buildPath ? readJson(options.buildPath) : null;
const rows = [
  ...collectBundleRows(bundleReport, budgets.apps ?? {}),
  ...collectBuildRows(buildReport, budgets.apps ?? {}),
  ...collectRuntimeRows(runtimeReport, budgets.runtime?.floors ?? []),
];
const violations = rows.filter((row) => !row.ok);
const overrideEnabled = process.env[OVERRIDE_ENV] === "1";
const body = renderReport(rows, violations, overrideEnabled, options.budgetsPath);

if (options.outputPath) {
  writeFileSync(options.outputPath, body);
}
if (options.appendPath) {
  appendFileSync(options.appendPath, `\n${body}`);
}

console.log(body);

if (violations.length > 0 && !overrideEnabled) {
  process.exitCode = 1;
}

function parseOptions(args) {
  const parsed = {
    budgetsPath: DEFAULT_BUDGETS,
    bundlePath: null,
    runtimePath: null,
    buildPath: null,
    outputPath: null,
    appendPath: null,
  };

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--budgets") {
      parsed.budgetsPath = readOptionValue(args, ++index, "--budgets");
      continue;
    }
    if (arg === "--bundle") {
      parsed.bundlePath = readOptionValue(args, ++index, "--bundle");
      continue;
    }
    if (arg === "--runtime") {
      parsed.runtimePath = readOptionValue(args, ++index, "--runtime");
      continue;
    }
    if (arg === "--build") {
      parsed.buildPath = readOptionValue(args, ++index, "--build");
      continue;
    }
    if (arg === "--output") {
      parsed.outputPath = readOptionValue(args, ++index, "--output");
      continue;
    }
    if (arg === "--append") {
      parsed.appendPath = readOptionValue(args, ++index, "--append");
      continue;
    }
    if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  if (!parsed.bundlePath && !parsed.runtimePath && !parsed.buildPath) {
    throw new Error("At least one of --bundle, --runtime, or --build is required");
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
  console.log(`Usage: node check-budgets.mjs [--budgets <path>] [--bundle <path>] [--runtime <path>] [--build <path>]

Compares fixture measurements against benchmarks/perf-budgets.json.
Exits 1 when a ceiling or runtime floor is missed, unless ${OVERRIDE_ENV}=1.`);
}

function collectBundleRows(report, apps) {
  if (!report || report.skipped) {
    return [];
  }

  const results = new Map((report.results ?? []).map((result) => [String(result.name), result]));
  const rows = [];

  for (const [name, limits] of Object.entries(apps)) {
    const result = results.get(name);
    if (!result || result.error) {
      rows.push(failureRow(name, "measurement", "missing fixture measurement"));
      continue;
    }

    for (const metric of APP_METRICS) {
      if (!Number.isFinite(limits[metric.limitKey])) {
        continue;
      }
      rows.push(limitRow(name, metric.label, result[metric.field], limits[metric.limitKey], "lte"));
    }
  }

  return rows;
}

function collectBuildRows(report, apps) {
  if (!report || report.skipped) {
    return [];
  }

  const results = new Map((report.results ?? []).map((result) => [String(result.name), result]));
  const rows = [];

  for (const [name, limits] of Object.entries(apps)) {
    if (!Number.isFinite(limits.buildMs)) {
      continue;
    }
    const result = results.get(name);
    const head = result?.avg ?? result?.buildMs;
    if (!result || result.error) {
      rows.push(failureRow(name, "Build time", "missing build-time measurement"));
      continue;
    }
    rows.push(limitRow(name, "Build time (dedicated)", head, limits.buildMs, "lte"));
  }

  return rows;
}

function collectRuntimeRows(report, floors) {
  if (!report || report.skipped) {
    return [];
  }

  return floors.flatMap((floor) => {
    const target = `${floor.size} / ${floor.suite} / ${floor.target}`;
    const results = report.sizes?.[floor.size]?.suites?.[floor.suite] ?? [];
    const result = results.find((entry) => entry.name === floor.target && !entry.error);
    if (!result) {
      return [failureRow(target, "Runtime ops/sec", "missing runtime measurement")];
    }
    return [limitRow(target, "Runtime ops/sec", result.opsPerSec, floor.minOpsPerSec, "gte")];
  });
}

function limitRow(target, metric, head, budget, comparator) {
  const numericHead = finiteNumber(head);
  const ok =
    numericHead !== null && (comparator === "lte" ? numericHead <= budget : numericHead >= budget);
  return { target, metric, head: numericHead, budget, comparator, ok };
}

function failureRow(target, metric, reason) {
  return { target, metric, head: null, budget: null, comparator: "lte", ok: false, reason };
}

function renderReport(rows, violations, overrideEnabled, budgetsPath) {
  const lines = [
    "### Performance Budgets",
    "",
    `Absolute ceilings from \`${budgetsPath}\`. Head measurements fail when a ceiling is exceeded or a runtime floor is missed. The \`${OVERRIDE_ENV}\` override is the same path as the relative regression gate.`,
    "",
  ];

  if (rows.length === 0) {
    lines.push("No budgeted measurements were present in the supplied reports.", "");
    return `${lines.join("\n")}\n`;
  }

  lines.push("| Target | Metric | Head | Budget | Status |", "| --- | --- | ---: | ---: | --- |");
  for (const row of rows) {
    lines.push(
      `| ${row.target} | ${row.metric} | ${formatHead(row)} | ${formatBudget(row)} | ${row.ok ? "pass" : "FAIL"} |`,
    );
  }
  lines.push("");

  if (violations.length === 0) {
    lines.push("No budget violations found.", "");
    return `${lines.join("\n")}\n`;
  }

  if (!overrideEnabled) {
    lines.push(REGRESSION_MARKER);
  }
  lines.push(
    overrideEnabled
      ? "Budget violations were found, but the maintainer override is active."
      : "Budget violations were found and this check should fail.",
    "",
  );
  return `${lines.join("\n")}\n`;
}

function formatHead(row) {
  if (row.reason) {
    return row.reason;
  }
  return formatValue(row.head, row.metric);
}

function formatBudget(row) {
  if (!Number.isFinite(row.budget)) {
    return "n/a";
  }
  const prefix = row.comparator === "gte" ? ">=" : "<=";
  return `${prefix} ${formatValue(row.budget, row.metric)}`;
}

function formatValue(value, metric) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  if (/gzip/i.test(metric) || /bytes/i.test(metric)) {
    return formatBytes(value);
  }
  if (/build time/i.test(metric)) {
    return formatDurationMs(value);
  }
  return Math.round(value).toLocaleString("en-US");
}

function formatBytes(value) {
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }
  return `${(value / (1024 * 1024)).toFixed(2)} MB`;
}

function formatDurationMs(value) {
  if (value < 1000) {
    return `${value.toFixed(0)} ms`;
  }
  return `${(value / 1000).toFixed(2)} s`;
}

function finiteNumber(value) {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}
