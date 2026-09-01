import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";

const here = dirname(fileURLToPath(import.meta.url));
const script = join(here, "check-budgets.mjs");
const committedBudgets = join(here, "..", "perf-budgets.json");

describe("check-budgets", () => {
  it("loads the committed budget file for the ox-content fixtures", () => {
    const parsed = JSON.parse(readFileSync(committedBudgets, "utf8"));
    expect(parsed.apps["ox-content (bare)"].gzippedBytes).toBeGreaterThan(0);
    expect(parsed.apps["ox-content (default)"].gzippedBytes).toBeGreaterThan(
      parsed.apps["ox-content (bare)"].gzippedBytes,
    );
    expect(parsed.runtime.floors).toHaveLength(2);
  });

  it("passes when fixture measurements stay inside ceilings", () => {
    const paths = writeReports({
      gzipped: 6000,
      htmlGzipped: 3000,
      buildMs: 400,
      requests: 1,
      opsPerSec: 5000,
    });

    const result = runCheck(paths);
    expect(result.status).toBe(0);
    expect(result.stdout).toContain("No budget violations found.");
    expect(result.stdout).toContain("pass");
    expect(result.stdout).toContain(
      "| large / parseOnly / @ox-content/napi | Runtime ops/sec | 5,000 | >= 2,500 | pass |",
    );
  });

  it("fails when bundle gzip exceeds the ceiling", () => {
    const paths = writeReports({
      gzipped: 20000,
      htmlGzipped: 3000,
      buildMs: 400,
      requests: 1,
      opsPerSec: 5000,
    });

    const result = runCheck(paths);
    expect(result.status).toBe(1);
    expect(result.stdout).toContain("<!-- ox-content-budget-regression -->");
    expect(result.stdout).toContain("| ox-content (bare) | Bundle gzip |");
    expect(result.stdout).toContain("FAIL");
  });

  it("fails when runtime throughput misses the floor", () => {
    const paths = writeReports({
      gzipped: 6000,
      htmlGzipped: 3000,
      buildMs: 400,
      requests: 1,
      opsPerSec: 100,
    });

    const result = runCheck(paths);
    expect(result.status).toBe(1);
    expect(result.stdout).toContain("Runtime ops/sec");
    expect(result.stdout).toContain("FAIL");
  });

  it("accepts the same override env as the relative regression gate", () => {
    const paths = writeReports({
      gzipped: 20000,
      htmlGzipped: 3000,
      buildMs: 400,
      requests: 1,
      opsPerSec: 5000,
    });

    const result = runCheck(paths, { OX_CONTENT_BENCHMARK_ALLOW_REGRESSION: "1" });
    expect(result.status).toBe(0);
    expect(result.stdout).toContain("maintainer override is active");
  });

  it("skips ceilings when the benchmark report was skipped", () => {
    const dir = mkdtempSync(join(tmpdir(), "ox-budgets-"));
    const budgetsPath = join(dir, "budgets.json");
    writeFileSync(budgetsPath, JSON.stringify(sampleBudgets(), null, 2));
    const bundlePath = join(dir, "bundle.json");
    writeFileSync(
      bundlePath,
      JSON.stringify({ name: "Bundle Size Benchmark", skipped: true, results: [] }, null, 2),
    );

    const result = spawnSync(
      process.execPath,
      [script, "--budgets", budgetsPath, "--bundle", bundlePath],
      { encoding: "utf8" },
    );
    expect(result.status).toBe(0);
    expect(result.stdout).toContain("No budgeted measurements were present");
  });
});

function sampleBudgets() {
  return {
    version: 1,
    apps: {
      "ox-content (bare)": {
        gzippedBytes: 10240,
        htmlGzippedBytes: 5120,
        buildMs: 5000,
        requests: 2,
      },
    },
    runtime: {
      floors: [
        {
          size: "large",
          suite: "parseOnly",
          target: "@ox-content/napi",
          minOpsPerSec: 2500,
        },
      ],
    },
  };
}

function writeReports({
  gzipped,
  htmlGzipped,
  buildMs,
  requests,
  opsPerSec,
}: {
  gzipped: number;
  htmlGzipped: number;
  buildMs: number;
  requests: number;
  opsPerSec: number;
}) {
  const dir = mkdtempSync(join(tmpdir(), "ox-budgets-"));
  const budgetsPath = join(dir, "budgets.json");
  const bundlePath = join(dir, "bundle.json");
  const runtimePath = join(dir, "runtime.json");
  writeFileSync(budgetsPath, `${JSON.stringify(sampleBudgets(), null, 2)}\n`);
  writeFileSync(
    bundlePath,
    `${JSON.stringify(
      {
        name: "Bundle Size Benchmark",
        results: [
          {
            name: "ox-content (bare)",
            gzipped,
            htmlGzipped,
            buildMs,
            requests,
          },
        ],
      },
      null,
      2,
    )}\n`,
  );
  writeFileSync(
    runtimePath,
    `${JSON.stringify(
      {
        name: "Parse/Render Speed Benchmark",
        sizes: {
          large: {
            suites: {
              parseOnly: [{ name: "@ox-content/napi", opsPerSec }],
            },
          },
        },
      },
      null,
      2,
    )}\n`,
  );
  return { budgetsPath, bundlePath, runtimePath };
}

function runCheck(
  paths: { budgetsPath: string; bundlePath: string; runtimePath: string },
  env: NodeJS.ProcessEnv = {},
) {
  return spawnSync(
    process.execPath,
    [
      script,
      "--budgets",
      paths.budgetsPath,
      "--bundle",
      paths.bundlePath,
      "--runtime",
      paths.runtimePath,
    ],
    { encoding: "utf8", env: { ...process.env, ...env } },
  );
}
