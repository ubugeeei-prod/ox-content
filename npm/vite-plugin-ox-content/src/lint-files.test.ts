import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { lintMarkdownFile, lintMarkdownFiles, shouldLintMarkdownFile } from "./lint-files";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("lintMarkdownFile", () => {
  it("supports end-user file targeting, ignores, and custom words", async () => {
    const cwd = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-lint-files-"));
    tempDirs.push(cwd);

    const filePath = path.join(cwd, "docs", "guide.md");
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, "Hello wrld\nProjectName\n", "utf-8");

    const result = await lintMarkdownFile(filePath, {
      cwd,
      dictionary: {
        ignoredWords: ["wrld"],
        words: ["ProjectName"],
      },
      include: ["docs/**/*.md"],
    });

    expect(result.skipped).toBe(false);
    expect(result.relativePath).toBe("docs/guide.md");
    expect(result.diagnostics).toHaveLength(0);
  });

  it("returns a skipped result for ignored files", async () => {
    const cwd = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-lint-files-"));
    tempDirs.push(cwd);

    const filePath = path.join(cwd, "docs", "generated", "guide.md");
    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, "Hello wrld\n", "utf-8");

    const result = await lintMarkdownFile(filePath, {
      cwd,
      exclude: ["docs/generated/**"],
      include: ["docs/**/*.md"],
    });

    expect(result.skipped).toBe(true);
    expect(result.diagnostics).toHaveLength(0);
  });
});

describe("lintMarkdownFiles", () => {
  it("checks only matched files and flattens diagnostics with file metadata", async () => {
    const cwd = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-lint-files-"));
    tempDirs.push(cwd);

    const docsDir = path.join(cwd, "docs");
    await fs.mkdir(path.join(docsDir, "generated"), { recursive: true });
    await fs.writeFile(path.join(docsDir, "guide.md"), "Hello wrld\n", "utf-8");
    await fs.writeFile(path.join(docsDir, "generated", "skip.md"), "Hello wrld\n", "utf-8");
    await fs.writeFile(path.join(cwd, "README.txt"), "Hello wrld\n", "utf-8");

    const result = await lintMarkdownFiles({
      cwd,
      exclude: ["docs/generated/**"],
      include: ["docs/**/*.md"],
    });

    expect(result.checkedFileCount).toBe(1);
    expect(result.files.map((file) => file.relativePath)).toEqual(["docs/guide.md"]);
    expect(result.diagnostics).toHaveLength(1);
    expect(result.diagnostics[0]).toMatchObject({
      filePath: path.join(docsDir, "guide.md"),
      relativePath: "docs/guide.md",
      ruleId: "spellcheck",
    });
  });

  it("supports opt-in standard dictionaries across multiple files", async () => {
    const cwd = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-lint-files-"));
    tempDirs.push(cwd);

    const docsDir = path.join(cwd, "docs");
    await fs.mkdir(docsDir, { recursive: true });
    await fs.writeFile(path.join(docsDir, "guide.md"), "Hello world\n", "utf-8");
    await fs.writeFile(path.join(docsDir, "typo.md"), "Hello wrld\n", "utf-8");

    const result = await lintMarkdownFiles({
      cwd,
      dictionary: {
        standard: {
          languages: ["en"],
        },
      },
      include: ["docs/**/*.md"],
    });

    expect(result.checkedFileCount).toBe(2);
    expect(result.files.map((file) => file.relativePath)).toEqual([
      "docs/guide.md",
      "docs/typo.md",
    ]);
    expect(result.diagnostics).toHaveLength(1);
    expect(result.diagnostics[0]).toMatchObject({
      relativePath: "docs/typo.md",
      ruleId: "spellcheck",
    });
  });
});

describe("shouldLintMarkdownFile", () => {
  it("evaluates include and ignore aliases from the consumer config", () => {
    const cwd = path.join("/workspace", "project");
    const includedFile = path.join(cwd, "docs", "guide.md");
    const ignoredFile = path.join(cwd, "docs", "generated", "guide.md");

    expect(
      shouldLintMarkdownFile(includedFile, {
        cwd,
        ignore: ["docs/generated/**"],
        include: ["docs/**/*.md"],
      }),
    ).toBe(true);

    expect(
      shouldLintMarkdownFile(ignoredFile, {
        cwd,
        ignore: ["docs/generated/**"],
        include: ["docs/**/*.md"],
      }),
    ).toBe(false);
  });
});
