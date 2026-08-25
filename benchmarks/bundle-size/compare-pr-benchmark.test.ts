import { spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";

const here = dirname(fileURLToPath(import.meta.url));
const script = join(here, "compare-pr-benchmark.mjs");

describe("compare-pr-benchmark", () => {
  it("reports build time and gates rendered HTML gzip growth", () => {
    const dir = mkdtempSync(join(tmpdir(), "ox-benchmark-"));
    const basePath = join(dir, "base.json");
    const headPath = join(dir, "head.json");
    writeJson(basePath, bundleReport({ gzipped: 1000, htmlGzipped: 100, buildMs: 1000 }));
    writeJson(headPath, bundleReport({ gzipped: 1010, htmlGzipped: 120, buildMs: 1150 }));

    const result = runCompare(
      "--base",
      basePath,
      "--head",
      headPath,
      "--base-bundle",
      basePath,
      "--head-bundle",
      headPath,
    );

    expect(result.status).toBe(1);
    expect(result.stdout).toContain("### Build and Output Size");
    expect(result.stdout).toContain(
      "| ox-content (bare) | 1000 B | 1010 B | +1.00% | 100 B | 120 B | +20.00% | 1.00 s | 1.15 s | +15.00% | 2 -> 2 | 3 -> 3 |",
    );
    expect(result.stdout).toContain("<!-- ox-content-benchmark-regression -->");
    expect(result.stdout).toContain(
      "| Rendered HTML gzip | ox-content (bare) | +20.00% | +5.00% |",
    );
  });

  it("keeps older bundle reports comparable when new fields are absent", () => {
    const dir = mkdtempSync(join(tmpdir(), "ox-benchmark-"));
    const basePath = join(dir, "base.json");
    const headPath = join(dir, "head.json");
    writeJson(basePath, bundleReport({ gzipped: 1000 }));
    writeJson(headPath, bundleReport({ gzipped: 1000 }));

    const result = runCompare(
      "--base",
      basePath,
      "--head",
      headPath,
      "--base-bundle",
      basePath,
      "--head-bundle",
      headPath,
    );

    expect(result.status).toBe(0);
    expect(result.stdout).toContain(
      "| ox-content (bare) | 1000 B | 1000 B | 0.00% | n/a | n/a | n/a | n/a | n/a | n/a | 2 -> 2 | 3 -> 3 |",
    );
  });
});

function writeJson(path: string, value: unknown) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

function bundleReport({
  gzipped,
  htmlGzipped,
  buildMs,
}: {
  gzipped: number;
  htmlGzipped?: number;
  buildMs?: number;
}) {
  return {
    name: "Bundle Size Benchmark",
    generatedAt: "2026-08-26T00:00:00.000Z",
    environment: { node: process.version },
    results: [
      {
        name: "ox-content (bare)",
        total: gzipped * 2,
        gzipped,
        ...(htmlGzipped === undefined ? {} : { htmlGzipped }),
        files: 3,
        requests: 2,
        ...(buildMs === undefined ? {} : { buildMs }),
      },
    ],
  };
}

function runCompare(...args: string[]) {
  return spawnSync(process.execPath, [script, ...args], {
    encoding: "utf8",
  });
}
