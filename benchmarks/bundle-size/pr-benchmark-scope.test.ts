import { spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, expect, it } from "vite-plus/test";

const script = resolve(".github/scripts/detect-pr-benchmark-scope.mjs");

describe("detect-pr-benchmark-scope", () => {
  it("skips neutral documentation edits", () => {
    const result = runScope(["docs/content/built-in/embeds.md", "README.md"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ runtime: "false", bundle: "false" });
  });

  it("runs both gates for parser and renderer changes", () => {
    const result = runScope(["crates/ox_content_parser/src/lib.rs"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ runtime: "true", bundle: "true" });
  });

  it("runs only the bundle gate for theme and UI package edits", () => {
    const result = runScope(["npm/theme/swiss/src/style.css"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ runtime: "false", bundle: "true" });
  });

  it("skips test-only package edits", () => {
    const result = runScope([
      "npm/vite-plugin-ox-content-react/src/transform.test.ts",
      "npm/vite-plugin-ox-content-react/src/__snapshots__/transform.test.ts.snap",
    ]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ runtime: "false", bundle: "false" });
  });

  it("keeps package implementation edits on the bundle gate", () => {
    const result = runScope(["npm/vite-plugin-ox-content-react/src/transform.ts"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ runtime: "false", bundle: "true" });
  });

  it("skips package preview helper edits", () => {
    const result = runScope([
      ".github/scripts/detect-pr-preview-scope.mjs",
      "scripts/pr-preview-scope.test.ts",
    ]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ runtime: "false", bundle: "false" });
  });

  it("runs both gates for benchmark harness edits", () => {
    const result = runScope(["benchmarks/bundle-size/parse-benchmark.mjs"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ runtime: "true", bundle: "true" });
  });

  it("runs only the bundle gate for committed performance budgets", () => {
    const result = runScope(["benchmarks/perf-budgets.json"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ runtime: "false", bundle: "true" });
  });

  it("falls back to both gates for unclassified paths", () => {
    const result = runScope(["tools/new-source-tree/file.rs"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ runtime: "true", bundle: "true" });
  });
});

function runScope(files: string[]) {
  const dir = mkdtempSync(join(tmpdir(), "ox-benchmark-scope-"));
  const changedFilesPath = join(dir, "changed-files.txt");
  writeFileSync(changedFilesPath, `${files.join("\n")}\n`);

  return spawnSync(process.execPath, [script, changedFilesPath], {
    encoding: "utf8",
  });
}

function parseOutputs(stdout: string) {
  return Object.fromEntries(
    stdout
      .trim()
      .split(/\r?\n/)
      .filter(Boolean)
      .map((line) => {
        const index = line.indexOf("=");
        return [line.slice(0, index), line.slice(index + 1)];
      }),
  );
}
