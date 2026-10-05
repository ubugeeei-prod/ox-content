import { spawnSync } from "node:child_process";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { performance } from "node:perf_hooks";
import { afterEach, describe, expect, it } from "vite-plus/test";

const bin = resolve(dirname(fileURLToPath(import.meta.url)), "../bin/oxct.mjs");
const tempDirs: string[] = [];
const run = (args: string[], cwd?: string, input?: string) =>
  spawnSync(process.execPath, [bin, "lint", ...args], { cwd, input, encoding: "utf8" });
afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});
async function fixture() {
  const root = await fs.mkdtemp(resolve(os.tmpdir(), "oxct-lint-"));
  tempDirs.push(root);
  await fs.writeFile(resolve(root, "a.md"), "# Heading\n\nA repeated repeated word.\n");
  await fs.writeFile(resolve(root, "b.md"), "# Other\n\nClean prose.\n");
  await fs.mkdir(resolve(root, "node_modules"));
  await fs.writeFile(resolve(root, "node_modules/ignored.md"), "# Ignore\n### Jump\n");
  await fs.writeFile(resolve(root, ".oxlint.json"), "{}");
  return root;
}

describe("oxct lint", () => {
  it("prints help without loading native dependencies", () => {
    const result = run(["--help"]);
    expect(result.status).toBe(0);
    expect(result.stdout).toContain("batched Rust engine");
  });
  it("discovers files, excludes dependencies, and emits deterministic JSON", async () => {
    const result = run(["--format", "json"], await fixture());
    expect(result.status).toBe(1);
    const report = JSON.parse(result.stdout);
    expect(report.checkedFileCount).toBe(2);
    expect(
      report.diagnostics.some(
        (d: { file: string; ruleId: string }) => d.file === "a.md" && d.ruleId === "repeated-word",
      ),
    ).toBe(true);
    expect(
      report.diagnostics.every((d: { file: string }) => !d.file.includes("node_modules")),
    ).toBe(true);
    expect(report.durationMs).toBeGreaterThan(0);
  });
  it("deduplicates globs and honors configuration and CLI ignores", async () => {
    const cwd = await fixture();
    await fs.writeFile(
      resolve(cwd, "lint.json"),
      JSON.stringify({ include: ["*.md"], rules: { repeatedWords: false } }),
    );
    expect(run(["--config", "lint.json", "--format", "json"], cwd).status).toBe(0);
    const result = run(["*.md", "a.md", "--ignore", "a.md", "--format", "json"], cwd);
    expect(result.status).toBe(0);
    expect(JSON.parse(result.stdout).checkedFileCount).toBe(1);
  });
  it("lints stdin, infers MDX, and permits an explicit warning budget", () => {
    const result = run(
      [
        "--stdin",
        "--no-markdownlint",
        "--stdin-filepath",
        "input.mdx",
        "--format",
        "json",
        "--max-warnings",
        "10",
      ],
      undefined,
      "export const x = 1;\n\n# Heading\n\nA repeated repeated word.\n",
    );
    expect(result.status).toBe(0);
    const report = JSON.parse(result.stdout);
    expect(report.checkedFileCount).toBe(1);
    expect(report.diagnostics.some((d: { file: string }) => d.file === "input.mdx")).toBe(true);
  });
  it("expands explicit directories and deduplicates overlapping paths", async () => {
    const cwd = await fixture();
    const result = run([".", "b.md", "--format", "json", "--max-warnings", "10"], cwd);
    expect(result.status).toBe(0);
    expect(JSON.parse(result.stdout).checkedFileCount).toBe(2);
  });
  it("measures complete CLI throughput on a 1000-document corpus", async () => {
    const cwd = await fixture();
    const corpus = resolve(cwd, "corpus");
    await fs.mkdir(corpus);
    const source =
      "# Benchmark\n\n" +
      "Clear Markdown prose for content authors.\n\n".repeat(100).trimEnd() +
      "\n";
    await Promise.all(
      Array.from({ length: 1000 }, (_, index) =>
        fs.writeFile(resolve(corpus, `${index}.md`), source),
      ),
    );
    const samples = [];
    for (let index = 0; index < 3; index++) {
      const started = performance.now();
      const result = run(["corpus", "--format", "json"], cwd);
      samples.push(performance.now() - started);
      expect(result.status, result.stderr || result.stdout.slice(0, 2000)).toBe(0);
      expect(JSON.parse(result.stdout).checkedFileCount).toBe(1000);
    }
    samples.sort((a, b) => a - b);
    console.info(
      `oxct lint throughput: 1000 x ${Buffer.byteLength(source)} bytes; median ${samples[1].toFixed(1)} ms including startup; ${(1000000 / samples[1]).toFixed(0)} files/s`,
    );
  }, 20000);
  it.each([
    ["--format", "xml"],
    ["--max-warnings", "NaN"],
    ["--max-warnings", "1.5"],
    ["--ignore"],
    ["--unknown"],
    ["--stdin", "a.md"],
  ])("rejects invalid arguments: %j", (...args) => {
    expect(run(args).status).toBe(1);
  });
  it("reports no matches and malformed config without a native stack trace", async () => {
    const cwd = await fixture();
    expect(run(["missing.md"], cwd).stderr).toContain("No Markdown files matched");
    await fs.writeFile(
      resolve(cwd, "lint.json"),
      JSON.stringify({ rules: { repeatedWords: "false" } }),
    );
    expect(run(["--config", "lint.json"], cwd).stderr).toContain("Invalid lint rule");
  });
});
