/**
 * Page-resource HTML rewriting and transform writes.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import {
  createResourceDedupeStore,
  emitCanonicalResource,
  hashResourceFile,
  linkOrCopyAlias,
  normalizeDedupeExt,
  rewriteToCanonicalUrl,
  type ResourceDedupeStore,
} from "./resources-dedupe";
import {
  collectResourceTags,
  escapeAttribute,
  replaceAttributeRaw,
  type ResourceAttrName,
} from "./resources-html";
import {
  isInsideRoot,
  parseResourceSrc,
  resourceCacheKey,
  type ProcessPageResourcesInput,
  type ProcessPageResourcesResult,
  type ResourceTransform,
} from "./resources";
import { ensureTransformedCache } from "./resources-write";
import type { ResolvedResourcesOptions } from "./types";

const PAGE_EXTS = new Set([".md", ".markdown", ".mdx", ".html", ".htm"]);

export async function processPageResources(
  input: ProcessPageResourcesInput,
): Promise<ProcessPageResourcesResult> {
  if (!input.options.enabled) {
    return { html: input.html, files: [], errors: [], fatal: [] };
  }
  if (input.options.dedupe && !input.outDir) {
    const message = "[ox-content] resources.dedupe requires outDir";
    return { html: input.html, files: [], errors: [message], fatal: [message] };
  }

  const bundleRoot = path.dirname(input.inputPath);
  const outputDir = path.dirname(input.outputPath);
  const files: string[] = [];
  const errors: string[] = [];
  const fatal: string[] = [];
  let html = input.html;
  const store = input.options.dedupe
    ? (input.dedupeStore ?? createResourceDedupeStore())
    : undefined;

  for (const { tag, refs } of collectResourceTags(input.html)) {
    let nextTag = tag;
    for (const ref of refs) {
      const result = await processResourceRef(input, {
        bundleRoot,
        outputDir,
        ref: ref.attr,
        src: ref.value,
        store,
      });
      errors.push(...result.errors);
      fatal.push(...result.fatal);
      files.push(...result.files);
      if (result.rewrite) {
        nextTag = replaceAttributeRaw(nextTag, ref.raw, escapeAttribute(result.rewrite));
      }
    }
    if (nextTag !== tag) {
      html = html.replace(tag, nextTag);
    }
  }

  return { html, files, errors, fatal };
}

async function processResourceRef(
  input: ProcessPageResourcesInput,
  ctx: {
    bundleRoot: string;
    outputDir: string;
    ref: ResourceAttrName;
    src: string;
    store: ResourceDedupeStore | undefined;
  },
): Promise<{ files: string[]; errors: string[]; fatal: string[]; rewrite?: string }> {
  const parsed = parseResourceSrc(ctx.src);
  if (!parsed) {
    return { files: [], errors: [], fatal: [] };
  }

  const resolved = resolveBundlePath(parsed.pathname, ctx.bundleRoot, input.srcDir);
  if (!resolved.ok) {
    if (ctx.ref === "href") {
      return { files: [], errors: [], fatal: [] };
    }
    const message = `[ox-content] page resource ${JSON.stringify(ctx.src)} on ${input.inputPath} is outside the page bundle`;
    return { files: [], errors: [message], fatal: [message] };
  }

  let stat: Awaited<ReturnType<typeof fs.stat>>;
  try {
    stat = await fs.stat(resolved.absolute);
  } catch {
    if (ctx.ref === "href") {
      return { files: [], errors: [], fatal: [] };
    }
    const message = `[ox-content] missing page resource ${JSON.stringify(parsed.pathname)} on ${input.inputPath}`;
    return {
      files: [],
      errors: [message],
      fatal: input.options.missing === "error" ? [message] : [],
    };
  }
  const hrefToPage =
    ctx.ref === "href" && PAGE_EXTS.has(path.extname(resolved.absolute).toLowerCase());
  if (!stat.isFile() || hrefToPage) {
    return { files: [], errors: [], fatal: [] };
  }

  const transformError = validateTransform(parsed.transform, input.options);
  if (transformError) {
    const message = `[ox-content] ${transformError} for ${JSON.stringify(ctx.src)} on ${input.inputPath}`;
    return { files: [], errors: [message], fatal: [message] };
  }

  const hasTransform = hasPixelOrFormatTransform(parsed.transform);
  const outputName = hasTransform
    ? transformedFileName(
        parsed.pathname,
        parsed.transform,
        resourceCacheKey(resolved.absolute, stat.mtimeMs, parsed.transform),
      )
    : decodedBasename(parsed.pathname) || path.basename(resolved.absolute);
  const outputFile = path.join(ctx.outputDir, outputName);

  try {
    const materialized = hasTransform
      ? await ensureTransformedCache({
          sourcePath: resolved.absolute,
          outputFile,
          cacheDir: input.cacheDir,
          mtimeMs: stat.mtimeMs,
          transform: parsed.transform,
        })
      : resolved.absolute;
    if (ctx.store && input.outDir) {
      return await emitDedupedResource({
        store: ctx.store,
        materialized,
        outputFile,
        src: ctx.src,
        sourcePath: resolved.absolute,
        mtimeMs: stat.mtimeMs,
        transform: parsed.transform,
        hasTransform,
        outDir: input.outDir,
        base: input.base ?? "/",
      });
    }
    if (hasTransform) {
      await fs.mkdir(ctx.outputDir, { recursive: true });
      await fs.copyFile(materialized, outputFile);
    } else {
      await fs.mkdir(ctx.outputDir, { recursive: true });
      await fs.copyFile(resolved.absolute, outputFile);
    }
    return { files: [outputFile], errors: [], fatal: [], rewrite: outputName };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    const message = `[ox-content] failed to process page resource ${JSON.stringify(ctx.src)} on ${input.inputPath}: ${detail}`;
    return { files: [], errors: [message], fatal: [message] };
  }
}

async function emitDedupedResource(input: {
  store: ResourceDedupeStore;
  materialized: string;
  outputFile: string;
  src: string;
  sourcePath: string;
  mtimeMs: number;
  transform: ResourceTransform;
  hasTransform: boolean;
  outDir: string;
  base: string;
}): Promise<{ files: string[]; errors: string[]; fatal: string[]; rewrite: string }> {
  const ext = normalizeDedupeExt(path.extname(input.outputFile));
  const reuseKey = input.hasTransform
    ? resourceCacheKey(input.sourcePath, input.mtimeMs, input.transform)
    : `${input.sourcePath}\0${input.mtimeMs}\0copy`;
  const digest = await hashResourceFile(input.materialized, ext, input.store, reuseKey);
  const { asset, wrote } = await emitCanonicalResource(input.store, {
    digest,
    ext,
    sourcePath: input.materialized,
    outDir: input.outDir,
    base: input.base,
  });
  await linkOrCopyAlias(asset.absolutePath, input.outputFile);
  return {
    files: wrote ? [asset.absolutePath, input.outputFile] : [input.outputFile],
    errors: [],
    fatal: [],
    rewrite: rewriteToCanonicalUrl(input.src, asset.publicPath),
  };
}

function resolveBundlePath(
  pathname: string,
  bundleRoot: string,
  contentRoot: string,
): { ok: true; absolute: string } | { ok: false } {
  let decoded: string;
  try {
    decoded = decodeURIComponent(pathname);
  } catch {
    decoded = pathname;
  }
  if (path.isAbsolute(decoded) || decoded.includes("\0")) {
    return { ok: false };
  }
  const absolute = path.resolve(bundleRoot, decoded);
  if (!isInsideRoot(bundleRoot, absolute) || !isInsideRoot(contentRoot, absolute)) {
    return { ok: false };
  }
  return { ok: true, absolute };
}

function validateTransform(
  transform: ResourceTransform,
  options: ResolvedResourcesOptions,
): string | undefined {
  if (transform.width && options.widths.length > 0 && !options.widths.includes(transform.width)) {
    return `width ${transform.width} is not in resources.widths`;
  }
  if (transform.format && !options.formats.includes(transform.format)) {
    return `format ${transform.format} is not in resources.formats`;
  }
  return undefined;
}

function hasPixelOrFormatTransform(transform: ResourceTransform): boolean {
  return Boolean(transform.width || transform.height || transform.crop || transform.format);
}

function transformedFileName(
  pathname: string,
  transform: ResourceTransform,
  cacheKey: string,
): string {
  const base = path.basename(pathname);
  const stem = base.replace(/\.[^.]+$/, "") || "resource";
  const ext = outputExtension(pathname, transform.format);
  return `${stem}.${cacheKey.slice(0, 12)}.${ext}`;
}

function outputExtension(pathname: string, format: string | undefined): string {
  if (format === "jpeg") {
    return "jpg";
  }
  if (format) {
    return format;
  }
  const ext = path.extname(pathname).slice(1).toLowerCase();
  return ext === "jpeg" ? "jpg" : ext || "png";
}

function decodedBasename(pathname: string): string {
  try {
    return path.basename(decodeURIComponent(pathname));
  } catch {
    return path.basename(pathname);
  }
}
