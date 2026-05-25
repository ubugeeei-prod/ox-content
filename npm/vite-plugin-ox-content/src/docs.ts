/**
 * Source Documentation Extraction and Generation
 *
 * This module provides comprehensive tools for extracting JSDoc/TSDoc comments
 * from TypeScript/JavaScript source files and automatically generating Markdown
 * documentation.
 *
 * ## Features
 *
 * - **Automatic Extraction**: Parses JSDoc comments from functions, classes, interfaces, and types
 * - **Flexible Filtering**: Include/exclude patterns for selective documentation
 * - **Markdown Generation**: Converts extracted docs to organized Markdown files
 * - **Navigation Generation**: Auto-generates sidebar navigation metadata
 * - **GitHub Links**: Includes clickable links to source code on GitHub
 *
 * ## Supported JSDoc Tags
 *
 * - `@param {type} name - description` - Function parameter documentation
 * - `@returns {type} description` - Return value documentation
 * - `@example` - Code examples (multi-line blocks)
 * - `@private` - Mark item as private (excluded from docs if private=false)
 * - `@default value` - Default parameter value
 * - Custom tags are preserved in the `tags` field
 *
 * ## Usage Flow
 *
 * 1. Call `extractDocs()` to parse source files
 * 2. Call `generateMarkdown()` to create Markdown content
 * 3. Call `writeDocs()` to write files to output directory
 * 4. Generated nav.ts can be imported for sidebar navigation
 *
 * @example
 * ```typescript
 * import { extractDocs, generateMarkdown, writeDocs } from './docs';
 *
 * const docsOptions = {
 *   enabled: true,
 *   src: ['./src'],
 *   out: './docs/api',
 *   include: ['**\/*.ts'],
 *   exclude: ['**\/*.test.ts'],
 *   groupBy: 'file',
 *   githubUrl: 'https://github.com/user/project',
 * };
 *
 * const extracted = await extractDocs(['./src'], docsOptions);
 * const markdown = generateMarkdown(extracted, docsOptions);
 * await writeDocs(markdown, './docs/api', extracted, docsOptions);
 * ```
 */

import * as fs from "fs";
import * as path from "path";
import type { ResolvedDocsOptions, ExtractedDocs, DocEntry, ResolvedDocsEntryPoint } from "./types";
import { generateNavMetadata, generateNavCode } from "./nav-generator";
import { importNapiModule, importNapiModuleSync } from "./napi";

const DOCS_MANIFEST_FILE = ".ox-content-docs-manifest.json";
const DOCS_DATA_FILE = "docs.json";
const DEFAULT_DOCS_INCLUDE = [
  "**/*.ts",
  "**/*.tsx",
  "**/*.js",
  "**/*.jsx",
  "**/*.mts",
  "**/*.mjs",
  "**/*.cts",
  "**/*.cjs",
];

/**
 * Extracts JSDoc documentation from source files in specified directories.
 *
 * This function recursively searches directories for source files matching
 * the include/exclude patterns, then extracts all documented items (functions,
 * classes, interfaces, types) from those files.
 *
 * ## Process
 *
 * 1. **File Discovery**: Recursively walks directories, applying filters
 * 2. **File Reading**: Loads each matching file's content
 * 3. **JSDoc Extraction**: Parses JSDoc comments using the native parser
 * 4. **Declaration Matching**: Pairs JSDoc comments with source declarations
 * 5. **Result Collection**: Aggregates extracted documentation by file
 *
 * ## Include/Exclude Patterns
 *
 * Patterns support:
 * - `**` - Match any directory structure
 * - `*` - Match any filename
 * - Standard glob patterns (e.g., `**\/*.test.ts`)
 *
 * ## Performance Considerations
 *
 * - Uses filesystem I/O which can be slow for large codebases
 * - Consider using more specific include patterns to reduce file scanning
 * - Results are not cached; call once per build/dev session
 *
 * @param srcDirs - Array of source directory paths to scan
 * @param options - Documentation extraction options (filters, grouping, etc.)
 *
 * @returns Promise resolving to array of extracted documentation by file.
 *          Each ExtractedDocs object contains file path and array of DocEntry items.
 *
 * @example
 * ```typescript
 * const docs = await extractDocs(
 *   ['./packages/vite-plugin/src'],
 *   {
 *     enabled: true,
 *     src: [],
 *     out: 'docs',
 *     include: ['**\/*.ts'],
 *     exclude: ['**\/*.test.ts', '**\/*.spec.ts'],
 *     format: 'markdown',
 *     private: false,
 *     toc: true,
 *     groupBy: 'file',
 *     generateNav: true,
 *   }
 * );
 *
 * // Returns:
 * // [
 * //   {
 * //     file: '/path/to/transform.ts',
 * //     entries: [
 * //       { name: 'transformMarkdown', kind: 'function', ... },
 * //       { name: 'loadNapiBindings', kind: 'function', ... },
 * //     ]
 * //   },
 * //   ...
 * // ]
 * ```
 */
export async function extractDocs(
  srcDirs: string[],
  options: ResolvedDocsOptions,
): Promise<ExtractedDocs[]> {
  const napi = await importNapiModule();

  if (options.entryPoints?.length) {
    const extractDocsFromEntryPoints = (
      napi as {
        extractDocsFromEntryPoints?: (
          entryPoints: ResolvedDocsEntryPoint[],
          options?: { root?: string; private?: boolean; internal?: boolean },
        ) => Array<{ file: string; entries: DocEntry[] }>;
      }
    ).extractDocsFromEntryPoints;

    if (!extractDocsFromEntryPoints) {
      throw new Error(
        "[ox-content] extractDocsFromEntryPoints is not available from @ox-content/napi.",
      );
    }

    return extractDocsFromEntryPoints(options.entryPoints, {
      root: process.cwd(),
      private: options.private,
      internal: options.internal,
    }).map((doc) => ({ file: doc.file, entries: doc.entries }));
  }

  const extractFileDocEntries = (
    napi as {
      extractFileDocEntries?: (
        filePath: string,
        includePrivate?: boolean,
        includeInternal?: boolean,
      ) => DocEntry[];
    }
  ).extractFileDocEntries;

  if (!extractFileDocEntries) {
    throw new Error("[ox-content] extractFileDocEntries is not available from @ox-content/napi.");
  }

  const results: ExtractedDocs[] = [];

  for (const srcDir of srcDirs) {
    const files = napi.collectDocsSourceFiles(srcDir, options.include, options.exclude);

    for (const file of files) {
      const entries = extractFileDocEntries(file, options.private, options.internal);

      if (entries.length > 0) {
        results.push({ file, entries });
      }
    }
  }

  return results;
}

/**
 * Generates Markdown documentation from extracted docs.
 */
export function generateMarkdown(
  docs: ExtractedDocs[],
  options: ResolvedDocsOptions,
): Record<string, string> {
  const napi = importNapiModuleSync();

  if (typeof napi.generateDocsMarkdown !== "function") {
    throw new Error(
      "[ox-content] generateDocsMarkdown is not available from @ox-content/napi. Please rebuild the NAPI package.",
    );
  }

  return napi.generateDocsMarkdown(toRustDocsModules(docs), {
    groupBy: options.groupBy,
    githubUrl: options.githubUrl,
  });
}

/**
 * Writes generated documentation to the output directory.
 */
export async function writeDocs(
  docs: Record<string, string>,
  outDir: string,
  extractedDocs?: ExtractedDocs[],
  options?: ResolvedDocsOptions,
): Promise<void> {
  await fs.promises.mkdir(outDir, { recursive: true });

  const generatedFiles = new Set(Object.keys(docs));
  if (extractedDocs && options?.generateNav && options.groupBy === "file") {
    generatedFiles.add("nav.ts");
  }
  if (extractedDocs) {
    generatedFiles.add(DOCS_DATA_FILE);
  }

  const manifestPath = path.join(outDir, DOCS_MANIFEST_FILE);
  let previousFiles: string[] = [];

  try {
    previousFiles = JSON.parse(await fs.promises.readFile(manifestPath, "utf-8")) as string[];
  } catch {
    previousFiles = [];
  }

  for (const staleFile of previousFiles) {
    if (generatedFiles.has(staleFile)) {
      continue;
    }

    await fs.promises.rm(path.join(outDir, staleFile), { force: true });
  }

  for (const [fileName, content] of Object.entries(docs)) {
    const filePath = path.join(outDir, fileName);
    await fs.promises.writeFile(filePath, content, "utf-8");
  }

  // Generate and write navigation metadata if enabled
  if (extractedDocs && options?.generateNav && options.groupBy === "file") {
    const navItems = generateNavMetadata(extractedDocs, "/api");
    const navCode = generateNavCode(navItems, "apiNav");
    const navFilePath = path.join(outDir, "nav.ts");
    await fs.promises.writeFile(navFilePath, navCode, "utf-8");
  }

  if (extractedDocs) {
    const napi = importNapiModuleSync();
    await fs.promises.writeFile(
      path.join(outDir, DOCS_DATA_FILE),
      napi.generateDocsDataJson(toRustDocsModules(extractedDocs), new Date().toISOString()),
      "utf-8",
    );
  }

  await fs.promises.writeFile(
    manifestPath,
    JSON.stringify([...generatedFiles].sort(), null, 2),
    "utf-8",
  );
}

function toRustDocsModules(docs: ExtractedDocs[]) {
  return docs.map((doc) => ({
    file: doc.file,
    entries: doc.entries.map((entry) => ({
      name: entry.name,
      kind: entry.kind,
      description: entry.description,
      params: entry.params,
      returns: entry.returns,
      examples: entry.examples,
      tags: entry.tags
        ? Object.entries(entry.tags).map(([tag, value]) => ({ tag, value }))
        : undefined,
      private: entry.private ?? false,
      file: entry.file,
      line: entry.line,
      endLine: entry.endLine,
      signature: entry.signature,
      members: entry.members,
    })),
  }));
}

/**
 * Resolves docs options with defaults.
 */
export function resolveDocsOptions(
  options: import("./types").DocsOptions | false | undefined,
): ResolvedDocsOptions | false {
  if (options === false) {
    return false;
  }

  const opts = options || {};

  return {
    enabled: opts.enabled ?? true,
    src: opts.src ?? ["./src"],
    out: opts.out ?? "docs/api",
    include: opts.include ?? DEFAULT_DOCS_INCLUDE,
    exclude: opts.exclude ?? ["**/*.test.*", "**/*.spec.*", "node_modules"],
    entryPoints: opts.entryPoints?.map((entryPoint) =>
      typeof entryPoint === "string" ? { path: entryPoint } : entryPoint,
    ),
    format: opts.format ?? "markdown",
    private: opts.private ?? false,
    internal: opts.internal ?? false,
    toc: false,
    groupBy: opts.groupBy ?? "file",
    githubUrl: opts.githubUrl,
    generateNav: opts.generateNav ?? true,
  };
}
