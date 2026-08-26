import { spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, expect, it } from "vite-plus/test";

const script = resolve(".github/scripts/detect-pr-preview-scope.mjs");

describe("detect-pr-preview-scope", () => {
  it("skips documentation-only preview builds", () => {
    const result = runScope(["docs/content/built-in/embeds.md", "CHANGELOG.md"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ preview: "false" });
  });

  it("skips package test and snapshot-only edits", () => {
    const result = runScope([
      "npm/vite-plugin-ox-content-react/src/transform.test.ts",
      "npm/vite-plugin-ox-content-react/src/__snapshots__/transform.test.ts.snap",
    ]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ preview: "false" });
  });

  it("keeps package implementation edits on the preview release", () => {
    const result = runScope(["npm/vite-plugin-ox-content-react/src/transform.ts"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ preview: "true" });
  });

  it("keeps native and dependency edits on the preview release", () => {
    const result = runScope(["crates/ox_content_renderer/src/html.rs", "pnpm-lock.yaml"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ preview: "true" });
  });

  it("keeps publish workflow edits on the preview release", () => {
    const result = runScope([".github/workflows/nightly.yml"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ preview: "true" });
  });

  it("falls back to preview builds for unclassified paths", () => {
    const result = runScope(["tools/new-publish-surface/file.ts"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ preview: "true" });
  });
});

function runScope(files: string[]) {
  const dir = mkdtempSync(join(tmpdir(), "ox-preview-scope-"));
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
