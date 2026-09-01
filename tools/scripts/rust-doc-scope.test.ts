import { spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, expect, it } from "vite-plus/test";

const script = resolve(".github/scripts/detect-rust-doc-scope.mjs");

describe("detect-rust-doc-scope", () => {
  it("skips documentation and preview-only edits", () => {
    const result = runScope([
      "docs/content/built-in/embeds.md",
      "npm/theme/swiss/src/style.css",
      "examples/playground/src/App.tsx",
      ".github/scripts/detect-pr-preview-scope.mjs",
    ]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ rust_docs: "false" });
  });

  it("runs rust docs for workspace crate edits", () => {
    const result = runScope(["crates/ox_content_renderer/src/html.rs"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ rust_docs: "true" });
  });

  it("runs rust docs for cargo graph and task definition edits", () => {
    const result = runScope(["Cargo.lock", "vite.config.ts"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ rust_docs: "true" });
  });

  it("runs rust docs for the CI hook and scope detector", () => {
    const result = runScope([
      ".github/workflows/ci.yml",
      ".github/scripts/detect-rust-doc-scope.mjs",
    ]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ rust_docs: "true" });
  });

  it("skips non-workspace editor cargo edits", () => {
    const result = runScope(["editors/zed/Cargo.toml"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ rust_docs: "false" });
  });

  it("falls back to rust docs for unclassified paths", () => {
    const result = runScope(["tools/new-rust-surface/file.rs"]);

    expect(result.status).toBe(0);
    expect(parseOutputs(result.stdout)).toEqual({ rust_docs: "true" });
  });
});

function runScope(files: string[]) {
  const dir = mkdtempSync(join(tmpdir(), "ox-rust-doc-scope-"));
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
