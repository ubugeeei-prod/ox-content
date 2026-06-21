import { spawn } from "node:child_process";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import { glob } from "glob";
import { extractDocsTests, type DocsTestOptions, type ExtractedCodeBlock } from "./code-blocks";
import { extractDocs, resolveDocsOptions } from "./docs";
import type { DocEntry, DocsOptions, ExtractedDocs, ResolvedDocsOptions } from "./types";

export interface CollectedDocsTest extends ExtractedCodeBlock {
  sourcePath: string;
  relativePath: string;
  index: number;
}

export type DocsTestSource = "markdown" | "jsdoc";

export interface DocsTestHarnessOptions extends DocsTestOptions {
  /**
   * Source kind to scan for runnable examples.
   * - `markdown` scans Markdown files for fenced code blocks.
   * - `jsdoc` scans JSDoc/TSDoc `@example` blocks through ox-content's docs extractor.
   * @default "markdown"
   */
  source?: DocsTestSource;

  /**
   * Markdown glob patterns, or source-file include globs when `source` is `jsdoc`.
   */
  include?: string | string[];

  /**
   * Glob patterns to skip. For `jsdoc`, this is passed to docs extraction as `exclude`.
   */
  ignore?: string | string[];

  /**
   * Working directory used for globs and generated test files.
   * @default process.cwd()
   */
  cwd?: string;

  /**
   * Source directories to scan when `source` is `jsdoc`.
   * @default ["./src"]
   */
  src?: string | string[];

  /**
   * Additional docs extraction options for `jsdoc` source mode.
   */
  docs?: DocsOptions;
}

export interface DocsTestFileOptions extends DocsTestHarnessOptions {
  /**
   * Directory for generated Vitest files.
   * @default ".cache/ox-content-docs-tests"
   */
  generatedDir?: string;

  /**
   * Remove the generated directory before writing files.
   * @default true
   */
  clean?: boolean;

  /**
   * Optional code prepended to every generated test file.
   */
  setupCode?: string;

  /**
   * How each generated file should execute the docs block.
   * - `test` wraps the block in a generated Vitest test, similar to Cargo doctests.
   * - `module` writes the block as-is for snippets that declare their own tests.
   * @default "test"
   */
  executionMode?: "test" | "module";

  /**
   * Module used for the generated `test` import.
   * @default "vitest"
   */
  testImport?: string;

  /**
   * Optional static import specifier rewrites for generated test files.
   */
  importRewrites?: Record<string, string>;
}

export interface WrittenDocsTestFile {
  filePath: string;
  sourcePath: string;
  relativePath: string;
  startLine: number;
  endLine: number;
  language: string;
}

export interface DocsTestWriteResult {
  cwd: string;
  generatedDir: string;
  blocks: CollectedDocsTest[];
  files: WrittenDocsTestFile[];
}

export interface RunDocsTestsOptions extends DocsTestFileOptions {
  /**
   * Vitest-compatible command to run.
   * @default "vitest"
   */
  vitestCommand?: string;

  /**
   * Arguments passed before generated test file paths.
   * @default ["run"]
   */
  vitestArgs?: string[];

  /**
   * Environment overrides for the Vitest child process.
   */
  env?: NodeJS.ProcessEnv;

  /**
   * Allow a scan that finds no runnable docs tests.
   * @default false
   */
  allowEmpty?: boolean;
}

export interface DocsTestRunResult extends DocsTestWriteResult {
  command: string;
  args: string[];
  exitCode: number;
  stdout: string;
  stderr: string;
}

export class DocsTestRunError extends Error {
  readonly result: DocsTestRunResult;

  constructor(result: DocsTestRunResult) {
    const command = [result.command, ...result.args].join(" ");
    super(`[ox-content] Docs tests failed with exit code ${result.exitCode}: ${command}`);
    this.name = "DocsTestRunError";
    this.result = result;
  }
}

export async function collectDocsTests(
  options: DocsTestHarnessOptions,
): Promise<CollectedDocsTest[]> {
  const cwd = path.resolve(options.cwd ?? process.cwd());
  if ((options.source ?? "markdown") === "jsdoc") {
    return collectJsdocDocsTests(options, cwd);
  }

  return collectMarkdownDocsTests(options, cwd);
}

async function collectMarkdownDocsTests(
  options: DocsTestHarnessOptions,
  cwd: string,
): Promise<CollectedDocsTest[]> {
  const include = toArray(options.include);
  const ignore = toArray(options.ignore);
  const files = new Map<string, string>();

  if (include.length === 0) {
    throw new Error("[ox-content] Docs test include patterns are required for markdown sources.");
  }

  for (const pattern of include) {
    const matches = await glob(pattern, {
      absolute: true,
      cwd,
      ignore,
      nodir: true,
    });

    for (const filePath of matches) {
      const absolutePath = path.resolve(filePath);
      files.set(absolutePath, normalizePath(path.relative(cwd, absolutePath)));
    }
  }

  const blocks: CollectedDocsTest[] = [];
  let index = 0;
  for (const [sourcePath, relativePath] of [...files.entries()].sort((left, right) =>
    left[0].localeCompare(right[0]),
  )) {
    const source = await fs.readFile(sourcePath, "utf-8");
    const extracted = await extractDocsTests(source, {
      languages: options.languages,
      requireMeta: options.requireMeta,
    });

    for (const block of extracted) {
      blocks.push({
        ...block,
        sourcePath,
        relativePath,
        index,
      });
      index += 1;
    }
  }

  return blocks;
}

async function collectJsdocDocsTests(
  options: DocsTestHarnessOptions,
  cwd: string,
): Promise<CollectedDocsTest[]> {
  const docsOptions = resolveJsdocDocsOptions(options, cwd);
  const docs = await extractDocs(docsOptions.src, docsOptions);
  const blocks: CollectedDocsTest[] = [];
  let index = 0;

  for (const doc of sortDocs(docs)) {
    for (const entry of sortEntries(doc.entries)) {
      for (const example of entry.examples ?? []) {
        const extracted = await extractDocsTests(example, {
          languages: options.languages,
          requireMeta: options.requireMeta,
        });
        const sourcePath = resolveEntrySourcePath(entry, doc, cwd);
        const relativePath = relativeSourcePath(cwd, sourcePath);

        for (const block of extracted) {
          blocks.push({
            ...block,
            sourcePath,
            relativePath,
            startLine: entry.line,
            endLine: entry.endLine,
            index,
          });
          index += 1;
        }
      }
    }
  }

  return blocks;
}

function resolveJsdocDocsOptions(
  options: DocsTestHarnessOptions,
  cwd: string,
): ResolvedDocsOptions {
  const docsOptions: DocsOptions = {
    ...options.docs,
  };

  if (options.src !== undefined) {
    docsOptions.src = toArray(options.src);
  }
  if (options.include !== undefined) {
    docsOptions.include = toArray(options.include);
  }
  if (options.ignore !== undefined) {
    docsOptions.exclude = toArray(options.ignore);
  }

  const resolved = resolveDocsOptions(docsOptions);
  return {
    ...resolved,
    src: resolved.src.map((sourceDir) => path.resolve(cwd, sourceDir)),
    entryPoints: resolved.entryPoints?.map((entryPoint) => ({
      ...entryPoint,
      path: path.resolve(cwd, entryPoint.path),
    })),
  };
}

function sortDocs(docs: ExtractedDocs[]): ExtractedDocs[] {
  return [...docs].sort((left, right) => left.file.localeCompare(right.file));
}

function sortEntries(entries: DocEntry[]): DocEntry[] {
  return [...entries].sort((left, right) => {
    const byFile = left.file.localeCompare(right.file);
    if (byFile !== 0) return byFile;
    const byLine = left.line - right.line;
    if (byLine !== 0) return byLine;
    return left.name.localeCompare(right.name);
  });
}

function resolveEntrySourcePath(entry: DocEntry, doc: ExtractedDocs, cwd: string): string {
  const sourcePath = entry.file || doc.file;
  return path.isAbsolute(sourcePath) ? path.resolve(sourcePath) : path.resolve(cwd, sourcePath);
}

function relativeSourcePath(cwd: string, sourcePath: string): string {
  const relativePath = path.relative(cwd, sourcePath);
  if (!relativePath.startsWith("..") && !path.isAbsolute(relativePath)) {
    return normalizePath(relativePath);
  }
  return normalizePath(sourcePath);
}

export async function writeDocsTestFiles(
  options: DocsTestFileOptions,
): Promise<DocsTestWriteResult> {
  const cwd = path.resolve(options.cwd ?? process.cwd());
  const generatedDir = path.resolve(cwd, options.generatedDir ?? ".cache/ox-content-docs-tests");
  const clean = options.clean ?? true;
  const blocks = await collectDocsTests({ ...options, cwd });

  if (clean) {
    await fs.rm(generatedDir, { recursive: true, force: true });
  }
  await fs.mkdir(generatedDir, { recursive: true });

  const files = await Promise.all(
    blocks.map(async (block) => {
      const filePath = path.join(generatedDir, docsTestFileName(block));
      await fs.writeFile(filePath, renderDocsTestFile(block, options), "utf-8");
      return {
        filePath,
        sourcePath: block.sourcePath,
        relativePath: block.relativePath,
        startLine: block.startLine,
        endLine: block.endLine,
        language: block.language,
      };
    }),
  );

  return {
    cwd,
    generatedDir,
    blocks,
    files,
  };
}

export async function runDocsTests(options: RunDocsTestsOptions): Promise<DocsTestRunResult> {
  const writeResult = await writeDocsTestFiles(options);
  const command = options.vitestCommand ?? "vitest";
  const leadingArgs = options.vitestArgs ?? ["run"];
  const fileArgs = writeResult.files.map((file) => file.filePath);
  const args = [...leadingArgs, ...fileArgs];

  if (fileArgs.length === 0) {
    if (options.allowEmpty) {
      return {
        ...writeResult,
        command,
        args,
        exitCode: 0,
        stdout: "",
        stderr: "",
      };
    }
    throw new Error("[ox-content] No runnable docs test blocks were found.");
  }

  const result = await runCommand(command, args, {
    cwd: writeResult.cwd,
    env: mergeEnv(options.env),
  });
  const runResult = {
    ...writeResult,
    command,
    args,
    ...result,
  };

  if (runResult.exitCode !== 0) {
    throw new DocsTestRunError(runResult);
  }

  return runResult;
}

function renderDocsTestFile(block: CollectedDocsTest, options: DocsTestFileOptions): string {
  const parts = [
    "// Generated by @ox-content/vite-plugin docs test harness.",
    `// Source: ${block.relativePath}:${block.startLine}-${block.endLine}`,
    "",
  ];
  const setupCode = options.setupCode?.trimEnd();
  const code = rewriteImports(block.code.trimEnd(), options.importRewrites);
  if (setupCode) {
    parts.push(setupCode, "");
  }
  if ((options.executionMode ?? "test") === "module") {
    parts.push(code, "");
    return parts.join("\n");
  }

  const { imports, body } = partitionImports(code);
  parts.push(
    `import { test } from ${JSON.stringify(
      rewriteSpecifier(options.testImport ?? "vitest", options.importRewrites),
    )};`,
  );
  if (imports.length > 0) {
    parts.push(...imports);
  }
  parts.push(
    "",
    `test(${JSON.stringify(`${block.relativePath}:${block.startLine}`)}, async () => {`,
  );
  if (body.trim().length > 0) {
    parts.push(indentCode(body.trimEnd()));
  }
  parts.push("});", "");
  return parts.join("\n");
}

function partitionImports(source: string): { imports: string[]; body: string } {
  const imports: string[] = [];
  const body: string[] = [];
  const lines = source.split(/\r?\n/);
  let currentImport: string[] | undefined;

  for (const line of lines) {
    if (currentImport) {
      currentImport.push(line);
      if (endsImportDeclaration(line)) {
        imports.push(currentImport.join("\n"));
        currentImport = undefined;
      }
      continue;
    }

    if (startsStaticImport(line)) {
      if (endsImportDeclaration(line)) {
        imports.push(line);
      } else {
        currentImport = [line];
      }
      continue;
    }

    body.push(line);
  }

  if (currentImport) {
    body.push(...currentImport);
  }

  return { imports, body: body.join("\n") };
}

function startsStaticImport(line: string): boolean {
  const trimmed = line.trimStart();
  return trimmed.startsWith("import ") && !trimmed.startsWith("import(");
}

function endsImportDeclaration(line: string): boolean {
  const trimmed = line.trim();
  return (
    trimmed.endsWith(";") ||
    /^import\s+["'][^"']+["']$/.test(trimmed) ||
    /\sfrom\s+["'][^"']+["']$/.test(trimmed)
  );
}

function indentCode(source: string): string {
  return source
    .split("\n")
    .map((line) => (line.length > 0 ? `  ${line}` : line))
    .join("\n");
}

function rewriteImports(source: string, rewrites: Record<string, string> | undefined): string {
  if (!rewrites) {
    return source;
  }

  let result = source;
  for (const [from, to] of Object.entries(rewrites)) {
    const escaped = escapeRegExp(from);
    result = result
      .replace(new RegExp(`(from\\s+["'])${escaped}(["'])`, "g"), `$1${to}$2`)
      .replace(new RegExp(`(import\\s+["'])${escaped}(["'])`, "g"), `$1${to}$2`)
      .replace(new RegExp(`(import\\(\\s*["'])${escaped}(["']\\s*\\))`, "g"), `$1${to}$2`);
  }
  return result;
}

/**
 * Applies the same import rewrite table to harness-owned imports.
 *
 * @internal
 * @example
 * ```ts docs-test
 * import { expect } from "vitest";
 * import { extractDocsTests } from "../../src/code-blocks";
 *
 * const markdown = [
 *   "```ts docs-test",
 *   "expect(1 + 1).toBe(2);",
 *   "```",
 * ].join("\n");
 *
 * const blocks = await extractDocsTests(markdown);
 *
 * expect(blocks).toHaveLength(1);
 * expect(blocks[0]?.code).toMatchInlineSnapshot('"expect(1 + 1).toBe(2);"');
 * ```
 */
function rewriteSpecifier(specifier: string, rewrites: Record<string, string> | undefined): string {
  return rewrites?.[specifier] ?? specifier;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function docsTestFileName(block: CollectedDocsTest): string {
  const baseName =
    block.relativePath
      .replace(/^\.\//, "")
      .replace(/[^A-Za-z0-9._-]+/g, "-")
      .replace(/^-+|-+$/g, "") || "docs-test";
  return `${baseName}-L${block.startLine}-${block.index + 1}.test.${extensionForLanguage(
    block.language,
  )}`;
}

function extensionForLanguage(language: string): string {
  switch (language.toLowerCase()) {
    case "jsx":
      return "jsx";
    case "tsx":
      return "tsx";
    case "mjs":
      return "mjs";
    case "mts":
      return "mts";
    case "js":
      return "js";
    default:
      return "ts";
  }
}

function toArray(value: string | string[] | undefined): string[] {
  if (!value) return [];
  return Array.isArray(value) ? value : [value];
}

function normalizePath(value: string): string {
  return value.split(path.sep).join("/");
}

function mergeEnv(overrides: NodeJS.ProcessEnv | undefined): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = { ...process.env };
  for (const [key, value] of Object.entries(overrides ?? {})) {
    if (value === undefined) {
      delete env[key];
    } else {
      env[key] = value;
    }
  }
  return env;
}

function runCommand(
  command: string,
  args: string[],
  options: { cwd: string; env: NodeJS.ProcessEnv },
): Promise<{ exitCode: number; stdout: string; stderr: string }> {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: options.cwd,
      env: options.env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";

    if (child.stdout) {
      child.stdout.setEncoding("utf-8");
      child.stdout.on("data", (chunk) => {
        stdout += chunk;
      });
    }
    if (child.stderr) {
      child.stderr.setEncoding("utf-8");
      child.stderr.on("data", (chunk) => {
        stderr += chunk;
      });
    }
    child.on("error", reject);
    child.on("close", (exitCode) => {
      resolve({ exitCode: exitCode ?? 1, stdout, stderr });
    });
  });
}
