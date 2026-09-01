import { spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";

const here = dirname(fileURLToPath(import.meta.url));
const script = join(here, "compare-pr-benchmark.mjs");

function artifacts(napiRaw: number, napiGzip: number, jsRaw: number, jsGzip: number) {
  return {
    targets: [
      { name: "napi binding", present: true, files: 1, raw: napiRaw, gzipped: napiGzip },
      { name: "wasm package", present: false, files: 0, raw: 0, gzipped: 0 },
      { name: "vite-plugin js", present: true, files: 10, raw: jsRaw, gzipped: jsGzip },
    ],
  };
}

function render(base: unknown, head: unknown): string {
  const dir = mkdtempSync(join(tmpdir(), "ox-artifact-report-"));
  const runtime = join(dir, "runtime.json");
  const basePath = join(dir, "base.json");
  const headPath = join(dir, "head.json");
  writeFileSync(runtime, JSON.stringify({ results: [] }));
  writeFileSync(basePath, JSON.stringify(base));
  writeFileSync(headPath, JSON.stringify(head));

  const result = spawnSync(
    process.execPath,
    [
      script,
      "--base",
      runtime,
      "--head",
      runtime,
      "--base-artifacts",
      basePath,
      "--head-artifacts",
      headPath,
    ],
    { encoding: "utf8" },
  );
  expect(result.status, result.stderr).toBe(0);
  return result.stdout;
}

describe("native artifact size report", () => {
  it("shows the native binding growing and the JavaScript shrinking", () => {
    const out = render(artifacts(1000, 500, 2000, 1000), artifacts(1100, 550, 1800, 900));

    expect(out).toContain("### Native Artifacts");
    expect(out).toMatch(/\| napi binding \|.*\+10\.00%.*\+10\.00% \|/);
    expect(out).toMatch(/\| vite-plugin js \|.*-10\.00%.*-10\.00% \|/);
  });

  // A target that was never built and one that shrank to nothing are different
  // facts. Reporting zero would read as a total win.
  it("says an unbuilt artifact is unbuilt rather than zero bytes", () => {
    const out = render(artifacts(1000, 500, 2000, 1000), artifacts(1000, 500, 2000, 1000));
    expect(out).toMatch(/\| wasm package \| not built \| not built \| n\/a \|/);
  });

  it("requires the two artifact paths together", () => {
    const dir = mkdtempSync(join(tmpdir(), "ox-artifact-report-"));
    const runtime = join(dir, "runtime.json");
    const only = join(dir, "only.json");
    writeFileSync(runtime, JSON.stringify({ results: [] }));
    writeFileSync(only, JSON.stringify(artifacts(1, 1, 1, 1)));

    const result = spawnSync(
      process.execPath,
      [script, "--base", runtime, "--head", runtime, "--base-artifacts", only],
      { encoding: "utf8" },
    );
    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain("--base-artifacts and --head-artifacts must be used together");
  });

  it("omits the section entirely when no artifact data is given", () => {
    const dir = mkdtempSync(join(tmpdir(), "ox-artifact-report-"));
    const runtime = join(dir, "runtime.json");
    writeFileSync(runtime, JSON.stringify({ results: [] }));
    const result = spawnSync(process.execPath, [script, "--base", runtime, "--head", runtime], {
      encoding: "utf8",
    });
    expect(result.status, result.stderr).toBe(0);
    expect(result.stdout).not.toContain("### Native Artifacts");
  });
});
