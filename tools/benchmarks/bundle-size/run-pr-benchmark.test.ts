import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, expect, it } from "vite-plus/test";
import {
  packageBuildConcurrencyEnvName,
  packageBuildConcurrencyFlag,
} from "../../scripts/package-build-concurrency";

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

  it("keeps package benchmark builds at the default fanout locally", () => {
    expect(packageBuildConcurrencyFlag({})).toBe("");
  });

  it("lets CI raise package benchmark build fanout", () => {
    expect(packageBuildConcurrencyFlag({ [packageBuildConcurrencyEnvName]: "12" })).toBe(
      " --concurrency-limit 12",
    );
  });

  it("rejects invalid package benchmark build fanout", () => {
    expect(() => packageBuildConcurrencyFlag({ [packageBuildConcurrencyEnvName]: "0" })).toThrow(
      `${packageBuildConcurrencyEnvName} must be a positive integer`,
    );
  });
});
