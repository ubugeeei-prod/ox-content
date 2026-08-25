import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, expect, it } from "vite-plus/test";

const script = resolve(".github/scripts/run-pr-benchmark.mjs");

describe("run-pr-benchmark", () => {
  it("writes skipped reports without requiring a workspace checkout", () => {
    const dir = mkdtempSync(join(tmpdir(), "ox-pr-benchmark-"));
    const runtimeJson = join(dir, "runtime.json");
    const bundleJson = join(dir, "bundle.json");

    const result = spawnSync(
      process.execPath,
      [
        script,
        "--skip-runtime",
        "--skip-bundle",
        "--runtime-json",
        runtimeJson,
        "--bundle-json",
        bundleJson,
      ],
      {
        encoding: "utf8",
        env: { ...process.env, GITHUB_WORKSPACE: "" },
      },
    );

    expect(result.status).toBe(0);
    expect(JSON.parse(readFileSync(runtimeJson, "utf8"))).toMatchObject({
      name: "Parse/Render Speed Benchmark",
      skipped: true,
      sizes: {},
    });
    expect(JSON.parse(readFileSync(bundleJson, "utf8"))).toMatchObject({
      name: "Bundle Size Benchmark",
      skipped: true,
      results: [],
    });
  });
});
