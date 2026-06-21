import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import { collectDocsTests, runDocsTests, writeDocsTestFiles } from "./docs-tests";

const srcDir = path.dirname(fileURLToPath(import.meta.url));
const packageRoot = path.resolve(srcDir, "..");
const repoRoot = path.resolve(packageRoot, "../..");

describe("docs test harness", () => {
  it("collects runnable docs fences with source metadata", async () => {
    const cwd = await mkdtemp(path.join(tmpdir(), "ox-content-docs-tests-"));
    try {
      await writeFile(
        path.join(cwd, "guide.md"),
        [
          "# Guide",
          "",
          "```ts docs-test",
          "import { expect, it } from 'vitest';",
          "it('works', () => expect(1).toBe(1));",
          "```",
          "",
          "```ts",
          "const ignored = true;",
          "```",
        ].join("\n"),
      );

      const blocks = await collectDocsTests({ cwd, include: ["**/*.md"] });

      expect(blocks).toHaveLength(1);
      expect(blocks[0]).toMatchObject({
        language: "ts",
        meta: "docs-test",
        relativePath: "guide.md",
        startLine: 4,
        endLine: 5,
      });
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("collects runnable JSDoc example fences through docs extraction", async () => {
    const cwd = await mkdtemp(path.join(tmpdir(), "ox-content-docs-tests-"));
    try {
      await mkdir(path.join(cwd, "src"), { recursive: true });
      await writeFile(
        path.join(cwd, "src/math.ts"),
        [
          "/**",
          " * Adds two numbers.",
          " *",
          " * @example",
          " * ```ts docs-test",
          ' * import { expect } from "vitest";',
          ' * import { add } from "../src/math";',
          " *",
          " * expect(add(1, 2)).toBe(3);",
          " * ```",
          " */",
          "export function add(left: number, right: number): number {",
          "  return left + right;",
          "}",
        ].join("\n"),
      );

      const blocks = await collectDocsTests({
        cwd,
        source: "jsdoc",
        src: ["src"],
        include: ["**/*.ts"],
      });

      expect(blocks).toHaveLength(1);
      expect(blocks[0]).toMatchObject({
        language: "ts",
        meta: "docs-test",
        relativePath: "src/math.ts",
        startLine: 12,
        endLine: 14,
      });
      expect(blocks[0]?.code).toMatchSnapshot();
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("writes generated Vitest files for collected docs blocks", async () => {
    const cwd = await mkdtemp(path.join(tmpdir(), "ox-content-docs-tests-"));
    try {
      await writeFile(
        path.join(cwd, "guide.md"),
        [
          "```tsx runnable",
          "import { expect } from 'vitest';",
          "const value = <span />;",
          "expect(value.type).toBe('span');",
          "```",
        ].join("\n"),
      );

      const result = await writeDocsTestFiles({
        cwd,
        include: ["**/*.md"],
        generatedDir: ".generated",
        setupCode: "import { expect, it } from 'vitest';",
      });

      expect(result.files).toHaveLength(1);
      expect(result.files[0].filePath.endsWith("guide.md-L2-1.test.tsx")).toBe(true);
      const generated = await readFile(result.files[0].filePath, "utf-8");
      expect(generated).toMatchSnapshot();
    } finally {
      await rm(cwd, { recursive: true, force: true });
    }
  });

  it("executes repository docs examples through the Vitest runner", async () => {
    const result = await runDocsTests({
      cwd: packageRoot,
      include: [
        normalizePath(path.relative(packageRoot, path.join(repoRoot, "docs/content/**/*.md"))),
      ],
      generatedDir: ".cache/ox-content-docs-tests",
      importRewrites: {
        vitest: "vite-plus/test",
      },
      vitestCommand: "vp",
      vitestArgs: ["test"],
      env: {
        VITEST: undefined,
        VITEST_POOL_ID: undefined,
        VITEST_WORKER_ID: undefined,
      },
    });

    expect(result.files.length).toBeGreaterThan(0);
    expect(result.exitCode).toBe(0);
  });

  it("executes implementation JSDoc examples through the Vitest runner", async () => {
    const result = await runDocsTests({
      cwd: packageRoot,
      source: "jsdoc",
      src: ["src"],
      include: ["**/docs-tests.ts"],
      docs: {
        internal: true,
      },
      generatedDir: ".cache/ox-content-source-docs-tests",
      importRewrites: {
        vitest: "vite-plus/test",
      },
      vitestCommand: "vp",
      vitestArgs: ["test"],
      env: {
        VITEST: undefined,
        VITEST_POOL_ID: undefined,
        VITEST_WORKER_ID: undefined,
      },
    });

    expect(result.files.length).toBeGreaterThan(0);
    expect(result.blocks[0]?.relativePath).toBe("src/docs-tests.ts");
    expect(result.exitCode).toBe(0);
  });
});

function normalizePath(value: string): string {
  return value.split(path.sep).join("/");
}
