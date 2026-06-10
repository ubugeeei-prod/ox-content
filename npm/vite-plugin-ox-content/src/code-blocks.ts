import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";
import { execFile } from "node:child_process";
import { importNapiModule } from "./napi";

const execFileAsync = promisify(execFile);

export interface ExtractedCodeBlock {
  language: string;
  meta: string;
  code: string;
  startLine: number;
  endLine: number;
}

export interface CodeBlockDiagnostic {
  ruleId: string;
  severity: "error" | "warning" | "info";
  message: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  language?: string;
}

export interface CodeBlockLintOptions {
  /**
   * Languages to lint. Omit to lint every fenced block language.
   * @default undefined
   */
  languages?: string[];

  /**
   * Report fences without a language identifier.
   * @default false
   */
  requireLanguage?: boolean;

  /**
   * Report trailing whitespace in code block lines.
   * @default true
   */
  trailingSpaces?: boolean;
}

export interface DocsTestOptions {
  /**
   * Fence languages to collect as runnable examples.
   * @default ['js', 'jsx', 'ts', 'tsx', 'mjs', 'mts']
   */
  languages?: string[];

  /**
   * Require fence meta such as `test`, `runnable`, `vitest`, or `docs-test`.
   * @default true
   */
  requireMeta?: boolean;
}

export interface TypecheckCodeBlockOptions {
  /**
   * Fence languages to type-check.
   * @default ['ts', 'tsx']
   */
  languages?: string[];

  /**
   * Require fence meta such as `typecheck`, `twoslash`, or `typecheck=...`.
   * @default true
   */
  requireMeta?: boolean;

  /**
   * Command used to run the TypeScript checker.
   * @default 'tsgo'
   */
  tsgoCommand?: string;
}

export async function extractCodeBlocks(source: string): Promise<ExtractedCodeBlock[]> {
  const mod = await importNapiModule();
  return mod.extractCodeBlocks(source).map(normalizeBlock);
}

export async function lintCodeBlocks(
  source: string,
  options: CodeBlockLintOptions = {},
): Promise<CodeBlockDiagnostic[]> {
  const mod = await importNapiModule();
  return mod
    .lintCodeBlocks(source, {
      enabled: true,
      languages: options.languages,
      requireLanguage: options.requireLanguage,
      trailingSpaces: options.trailingSpaces,
    })
    .map(normalizeDiagnostic);
}

export async function extractDocsTests(
  source: string,
  options: DocsTestOptions = {},
): Promise<ExtractedCodeBlock[]> {
  const mod = await importNapiModule();
  return mod
    .extractDocsTests(source, {
      enabled: true,
      languages: options.languages,
      requireMeta: options.requireMeta,
    })
    .map(normalizeBlock);
}

export async function typecheckCodeBlocks(
  source: string,
  options: TypecheckCodeBlockOptions = {},
): Promise<CodeBlockDiagnostic[]> {
  if (!source.includes("```")) {
    return [];
  }

  const languages = new Set(
    (options.languages ?? ["ts", "tsx"]).map((language) => language.toLowerCase()),
  );
  const blocks = (await extractCodeBlocks(source)).filter((block) => {
    if (!languages.has(block.language.toLowerCase())) {
      return false;
    }
    return options.requireMeta === false || hasTypecheckMeta(block.meta);
  });
  if (blocks.length === 0) {
    return [];
  }

  const temp = await mkdtemp(join(tmpdir(), "ox-content-code-blocks-"));
  try {
    const files: string[] = [];
    await Promise.all(
      blocks.map(async (block, index) => {
        const extension = block.language.toLowerCase() === "tsx" ? "tsx" : "ts";
        const file = join(temp, `snippet-${index}.${extension}`);
        files.push(file);
        await writeFile(file, block.code);
      }),
    );

    try {
      await execFileAsync(
        options.tsgoCommand ?? "tsgo",
        ["--noEmit", "--pretty", "false", ...files],
        {
          cwd: process.cwd(),
          maxBuffer: 1024 * 1024 * 4,
        },
      );
      return [];
    } catch (error) {
      const output = commandOutput(error);
      return [
        {
          ruleId: "code-block-typecheck",
          severity: "error",
          message: output || "TypeScript code block type-checking failed.",
          line: blocks[0]?.startLine ?? 1,
          column: 1,
          endLine: blocks[0]?.startLine ?? 1,
          endColumn: 1,
          language: "ts",
        },
      ];
    }
  } finally {
    await rm(temp, { recursive: true, force: true });
  }
}

function hasTypecheckMeta(meta: string): boolean {
  return meta
    .split(/\s+/)
    .some(
      (token) => token === "typecheck" || token === "twoslash" || token.startsWith("typecheck="),
    );
}

function commandOutput(error: unknown): string {
  if (!error || typeof error !== "object") {
    return "";
  }
  const value = error as { stdout?: unknown; stderr?: unknown; message?: unknown };
  return [value.stdout, value.stderr, value.message]
    .filter((part): part is string => typeof part === "string" && part.trim().length > 0)
    .join("\n")
    .trim();
}

function normalizeBlock(block: {
  language: string;
  meta: string;
  code: string;
  startLine: number;
  endLine: number;
}): ExtractedCodeBlock {
  return block;
}

function normalizeDiagnostic(diagnostic: {
  ruleId: string;
  severity: string;
  message: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  language?: string;
}): CodeBlockDiagnostic {
  return {
    ...diagnostic,
    severity:
      diagnostic.severity === "error" || diagnostic.severity === "info"
        ? diagnostic.severity
        : "warning",
  };
}
