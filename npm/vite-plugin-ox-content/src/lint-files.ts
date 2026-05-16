import * as fs from "node:fs/promises";
import * as path from "node:path";
import { glob } from "glob";
import {
  lintMarkdownDocumentsAsync,
  lintMarkdownAsync,
  type MarkdownLintDiagnostic,
  type MarkdownLintOptions,
  type MarkdownLintResult,
} from "./lint";

const DEFAULT_LINT_FILE_INCLUDE = ["**/*.md", "**/*.markdown"] as const;
const DEFAULT_LINT_FILE_EXCLUDE = ["**/node_modules/**", "**/.git/**", "**/dist/**"] as const;

/**
 * File-oriented Markdown lint options for end-user configuration.
 *
 * This extends the content-level lint options with project-level targeting,
 * so consumers can decide which files should be checked and which paths should
 * be ignored.
 */
export interface MarkdownLintFileOptions extends MarkdownLintOptions {
  /**
   * Base directory used to resolve `include` and `exclude` patterns.
   * @default process.cwd()
   */
  cwd?: string;

  /**
   * Glob patterns for files to lint.
   * @default ['**\/*.md', '**\/*.markdown']
   */
  include?: string[];

  /**
   * Glob patterns for files to exclude from linting.
   * @default ['**\/node_modules/**', '**\/.git/**', '**\/dist/**']
   */
  exclude?: string[];

  /**
   * Alias of `exclude`.
   */
  ignore?: string[];
}

/**
 * A lint diagnostic annotated with file metadata.
 */
export interface MarkdownLintFileDiagnostic extends MarkdownLintDiagnostic {
  filePath: string;
  relativePath: string;
}

/**
 * Lint result for a single file.
 */
export interface MarkdownLintFileResult extends MarkdownLintResult {
  filePath: string;
  relativePath: string;
  skipped: boolean;
}

/**
 * Aggregated lint result for multiple files.
 */
export interface MarkdownLintFilesResult {
  checkedFileCount: number;
  diagnostics: MarkdownLintFileDiagnostic[];
  errorCount: number;
  files: MarkdownLintFileResult[];
  infoCount: number;
  warningCount: number;
}

interface ResolvedMarkdownLintFileOptions {
  cwd: string;
  exclude: string[];
  include: string[];
  lintOptions: MarkdownLintOptions;
}

interface MarkdownLintFileEntry {
  filePath: string;
  relativePath: string;
}

/**
 * Returns true if the file path is included by the configured glob filters.
 */
export function shouldLintMarkdownFile(
  filePath: string,
  options: MarkdownLintFileOptions = {},
): boolean {
  const resolvedOptions = resolveMarkdownLintFileOptions(options);
  return shouldLintAbsoluteFile(path.resolve(resolvedOptions.cwd, filePath), resolvedOptions);
}

/**
 * Lints a single Markdown file using project-style include/exclude settings.
 *
 * If the file is filtered out by `include` / `exclude`, the returned result is
 * marked as `skipped` and contains no diagnostics.
 */
export async function lintMarkdownFile(
  filePath: string,
  options: MarkdownLintFileOptions = {},
): Promise<MarkdownLintFileResult> {
  const resolvedOptions = resolveMarkdownLintFileOptions(options);
  return lintMarkdownFileWithResolvedOptions(
    path.resolve(resolvedOptions.cwd, filePath),
    resolvedOptions,
  );
}

/**
 * Lints all Markdown files matched by the configured include/exclude patterns.
 */
export async function lintMarkdownFiles(
  options: MarkdownLintFileOptions = {},
): Promise<MarkdownLintFilesResult> {
  const resolvedOptions = resolveMarkdownLintFileOptions(options);
  const matchedFiles = await collectMarkdownLintFileEntries(resolvedOptions);
  const sources = await Promise.all(
    matchedFiles.map((file) => fs.readFile(file.filePath, "utf-8")),
  );
  const results = await lintMarkdownDocumentsAsync(sources, resolvedOptions.lintOptions);

  const files = matchedFiles.map(
    (file, index): MarkdownLintFileResult => ({
      ...(results[index] ?? createEmptyLintResult()),
      filePath: file.filePath,
      relativePath: file.relativePath,
      skipped: false,
    }),
  );

  const diagnostics = files.flatMap((fileResult) =>
    fileResult.diagnostics.map(
      (diagnostic): MarkdownLintFileDiagnostic => ({
        ...diagnostic,
        filePath: fileResult.filePath,
        relativePath: fileResult.relativePath,
      }),
    ),
  );

  return {
    checkedFileCount: files.length,
    diagnostics,
    errorCount: files.reduce((count, fileResult) => count + fileResult.errorCount, 0),
    files,
    infoCount: files.reduce((count, fileResult) => count + fileResult.infoCount, 0),
    warningCount: files.reduce((count, fileResult) => count + fileResult.warningCount, 0),
  };
}

function resolveMarkdownLintFileOptions(
  options: MarkdownLintFileOptions,
): ResolvedMarkdownLintFileOptions {
  return {
    cwd: path.resolve(options.cwd ?? process.cwd()),
    exclude: [
      ...new Set([...(options.exclude ?? DEFAULT_LINT_FILE_EXCLUDE), ...(options.ignore ?? [])]),
    ],
    include: [...new Set(options.include ?? DEFAULT_LINT_FILE_INCLUDE)],
    lintOptions: {
      dictionary: options.dictionary,
      languages: options.languages,
      rules: options.rules,
    },
  };
}

async function lintMarkdownFileWithResolvedOptions(
  filePath: string,
  options: ResolvedMarkdownLintFileOptions,
): Promise<MarkdownLintFileResult> {
  const absoluteFilePath = path.resolve(filePath);
  const relativePath = normalizePath(path.relative(options.cwd, absoluteFilePath));

  if (!shouldLintAbsoluteFile(absoluteFilePath, options)) {
    return {
      ...createEmptyLintResult(),
      filePath: absoluteFilePath,
      relativePath,
      skipped: true,
    };
  }

  const source = await fs.readFile(absoluteFilePath, "utf-8");
  const result = await lintMarkdownAsync(source, options.lintOptions);

  return {
    ...result,
    filePath: absoluteFilePath,
    relativePath,
    skipped: false,
  };
}

async function collectMarkdownLintFileEntries(
  options: ResolvedMarkdownLintFileOptions,
): Promise<MarkdownLintFileEntry[]> {
  const files = new Map<string, MarkdownLintFileEntry>();

  for (const pattern of options.include) {
    const matches = await glob(pattern, {
      absolute: true,
      cwd: options.cwd,
      ignore: options.exclude,
      nodir: true,
    });

    for (const filePath of matches) {
      const absoluteFilePath = path.resolve(filePath);
      if (shouldLintAbsoluteFile(absoluteFilePath, options)) {
        files.set(absoluteFilePath, {
          filePath: absoluteFilePath,
          relativePath: normalizePath(path.relative(options.cwd, absoluteFilePath)),
        });
      }
    }
  }

  return [...files.values()].sort((left, right) => left.filePath.localeCompare(right.filePath));
}

function shouldLintAbsoluteFile(
  filePath: string,
  options: ResolvedMarkdownLintFileOptions,
): boolean {
  const absolutePath = normalizePath(path.resolve(filePath));
  const relativePath = normalizePath(path.relative(options.cwd, absolutePath));

  const matches = (patterns: string[]) =>
    patterns.some((pattern) => {
      const normalizedPattern = normalizePath(pattern);
      return (
        path.matchesGlob(relativePath, normalizedPattern) ||
        path.matchesGlob(absolutePath, normalizedPattern)
      );
    });

  return matches(options.include) && !matches(options.exclude);
}

function normalizePath(value: string): string {
  return value.split(path.sep).join("/");
}

function createEmptyLintResult(): MarkdownLintResult {
  return {
    diagnostics: [],
    errorCount: 0,
    infoCount: 0,
    warningCount: 0,
  };
}
