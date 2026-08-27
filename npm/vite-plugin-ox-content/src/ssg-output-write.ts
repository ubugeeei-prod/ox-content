/**
 * Writers for composable SSG outputs. Custom hosts call these without `buildSsg()`.
 */

import * as path from "node:path";
import type {
  FeedItemInput,
  RenderedFeedFile,
  RenderFeedFilesInput,
  RenderFeedFilesResult,
  WriteFeedFilesInput,
} from "./feeds";
import { writeMarkdownSourceFiles, type WriteMarkdownSourceFilesInput } from "./markdown-source";
import { importNapiModuleSync } from "./napi";
import { PageResourceError, createResourceDedupeStore, processPageResources } from "./resources";
import type { WriteSiteMapFilesInput } from "./site-maps";
import type { ResolvedResourcesOptions } from "./types";

/** One host-rendered page that may receive resource fingerprinting. */
export interface WriteResourceFilesPage {
  html: string;
  inputPath: string;
  outputPath: string;
}

/** Inputs for writing fingerprinted page resources from host HTML. */
export interface WriteResourceFilesInput {
  pages: readonly WriteResourceFilesPage[];
  srcDir: string;
  outDir: string;
  root?: string;
  base?: string;
  options?: ResolvedResourcesOptions | null;
  cacheDir?: string;
}

/** Rewritten host pages plus emitted resource paths. */
export interface WriteResourceFilesResult {
  pages: WriteResourceFilesPage[];
  files: string[];
  errors: string[];
}

/**
 * Fingerprint, rewrite, and emit page resources for host-rendered HTML.
 *
 * Uses the same `resources` option object and emit path as `buildSsg()`.
 * Throws `PageResourceError` when `missing: "error"` hits a fatal issue.
 */
export async function writeResourceFiles(
  input: WriteResourceFilesInput,
): Promise<WriteResourceFilesResult> {
  const options = input.options;
  if (!options?.enabled) {
    return { pages: input.pages.map(clonePage), files: [], errors: [] };
  }

  const cacheDir =
    input.cacheDir ?? path.join(input.root ?? process.cwd(), ".cache", "ox-content-resources");
  const files: string[] = [];
  const errors: string[] = [];
  const fatal: string[] = [];
  const pages: WriteResourceFilesPage[] = [];
  const dedupeStore = options.dedupe ? createResourceDedupeStore() : undefined;

  for (const page of input.pages) {
    const processed = await processPageResources({
      html: page.html,
      inputPath: page.inputPath,
      outputPath: page.outputPath,
      srcDir: input.srcDir,
      options,
      cacheDir,
      outDir: input.outDir,
      base: input.base,
      dedupeStore,
    });
    pages.push({ html: processed.html, inputPath: page.inputPath, outputPath: page.outputPath });
    files.push(...processed.files);
    errors.push(...processed.errors);
    fatal.push(...processed.fatal);
  }

  if (fatal.length > 0) {
    throw new PageResourceError(fatal);
  }
  return { pages, files, errors };
}

/**
 * Write Markdown companions for host-rendered pages.
 *
 * Reuses `writeMarkdownSourceFiles` from the copy-as-markdown pipeline.
 */
export function writeMarkdownCompanions(
  input: WriteMarkdownSourceFilesInput,
): Promise<{ files: string[]; errors: string[] }> {
  return writeMarkdownSourceFiles(input);
}

/**
 * Git last-commit time for `filePath` in milliseconds.
 *
 * Same lookup `buildSsg()` uses for `ssg.lastUpdated` and sitemap `<lastmod>`.
 * Returns `undefined` when `root` is missing, Git has no history, or NAPI is unavailable.
 */
export function resolveGitLastmod(filePath: string, root?: string): number | undefined {
  if (!root) {
    return undefined;
  }
  try {
    const value = importNapiModuleSync().getGitLastUpdated(filePath, root);
    return typeof value === "number" && Number.isFinite(value) ? value : undefined;
  } catch {
    return undefined;
  }
}

export type {
  FeedItemInput,
  RenderedFeedFile,
  RenderFeedFilesInput,
  RenderFeedFilesResult,
  WriteFeedFilesInput,
  WriteMarkdownSourceFilesInput,
  WriteSiteMapFilesInput,
};

function clonePage(page: WriteResourceFilesPage): WriteResourceFilesPage {
  return { html: page.html, inputPath: page.inputPath, outputPath: page.outputPath };
}
