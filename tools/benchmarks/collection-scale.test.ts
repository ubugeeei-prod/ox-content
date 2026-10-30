import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vite-plus/test";

const script = resolve("tools/benchmarks/compare-collection-scale.mjs");

describe("compare collection scale", () => {
  it("appends improvements to the benchmark report", () => {
    const result = compareReports(10, 5);

    expect(result.status).toBe(0);
    expect(result.report).toContain("## Collection scale");
    expect(result.report).toContain("-50.0%");
  });

  it("rejects collection regressions beyond the tolerance", () => {
    const result = compareReports(10, 13);

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("regression exceeded 20% and 0.05 ms");
  });

  it("ignores relative noise below the absolute tolerance", () => {
    const result = compareReports(0.0004, 0.0008);

    expect(result.status).toBe(0);
    expect(result.report).toContain("+100.0%");
  });
});

function compareReports(baseMs: number, headMs: number) {
  const directory = mkdtempSync(join(tmpdir(), "ox-collection-compare-"));
  const base = join(directory, "base.json");
  const head = join(directory, "head.json");
  const report = join(directory, "report.md");
  writeFileSync(base, JSON.stringify(createReport(baseMs)));
  writeFileSync(head, JSON.stringify(createReport(headMs)));
  writeFileSync(report, "# Benchmark\n");

  const result = spawnSync(
    process.execPath,
    [script, "--base", base, "--head", head, "--append", report],
    {
      encoding: "utf8",
      env: { ...process.env, OX_CONTENT_BENCHMARK_ALLOW_REGRESSION: "0" },
    },
  );
  return { ...result, report: readFileSync(report, "utf8") };
}

function createReport(milliseconds: number) {
  return {
    name: "Collection Scale Benchmark",
    sizes: {
      2000: {
        manifestMs: milliseconds,
        queries: {
          pathFirstMs: milliseconds,
          likeCountMs: milliseconds,
          nestedCountMs: milliseconds,
          orderedPageMs: milliseconds,
        },
        themeRenderAllMs: milliseconds,
      },
    },
  };
}
