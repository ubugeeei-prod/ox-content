/**
 * Page-resource HTML rewriting and transform writes.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import {
  coverCrop,
  cropImage,
  decodePng,
  encodeJpeg,
  encodePng,
  isPng,
  resizeNearest,
  type RgbaImage,
} from "./resources-image";
import {
  isInsideRoot,
  parseResourceSrc,
  resourceCacheKey,
  type ProcessPageResourcesInput,
  type ProcessPageResourcesResult,
  type ResourceTransform,
} from "./resources";
import type { ResolvedResourcesOptions } from "./types";

const IMG_TAG = /<img\b[^>]*>/gi;
const SRC_ATTR = /\bsrc\s*=\s*(?:"([^"]*)"|'([^']*)')/i;

export async function processPageResources(
  input: ProcessPageResourcesInput,
): Promise<ProcessPageResourcesResult> {
  if (!input.options.enabled) {
    return { html: input.html, files: [], errors: [], fatal: [] };
  }

  const bundleRoot = path.dirname(input.inputPath);
  const outputDir = path.dirname(input.outputPath);
  const files: string[] = [];
  const errors: string[] = [];
  const fatal: string[] = [];
  let html = input.html;

  const tags = input.html.match(IMG_TAG) ?? [];
  for (const tag of tags) {
    const srcMatch = tag.match(SRC_ATTR);
    const rawSrc = srcMatch?.[1] ?? srcMatch?.[2];
    if (!rawSrc) {
      continue;
    }
    const src = unescapeHtml(rawSrc);
    const parsed = parseResourceSrc(src);
    if (!parsed) {
      continue;
    }

    const resolved = resolveBundlePath(parsed.pathname, bundleRoot, input.srcDir);
    if (!resolved.ok) {
      const message = `[ox-content] page resource ${JSON.stringify(src)} on ${input.inputPath} is outside the page bundle`;
      errors.push(message);
      fatal.push(message);
      continue;
    }

    let stat: Awaited<ReturnType<typeof fs.stat>>;
    try {
      stat = await fs.stat(resolved.absolute);
    } catch {
      const message = `[ox-content] missing page resource ${JSON.stringify(parsed.pathname)} on ${input.inputPath}`;
      errors.push(message);
      if (input.options.missing === "error") {
        fatal.push(message);
      }
      continue;
    }

    const transformError = validateTransform(parsed.transform, input.options);
    if (transformError) {
      const message = `[ox-content] ${transformError} for ${JSON.stringify(src)} on ${input.inputPath}`;
      errors.push(message);
      fatal.push(message);
      continue;
    }

    const hasTransform = hasPixelOrFormatTransform(parsed.transform);
    const outputName = hasTransform
      ? transformedFileName(
          parsed.pathname,
          parsed.transform,
          resourceCacheKey(resolved.absolute, stat.mtimeMs, parsed.transform),
        )
      : path.basename(resolved.absolute);
    const outputFile = path.join(outputDir, outputName);

    try {
      if (hasTransform) {
        await writeTransformedResource({
          sourcePath: resolved.absolute,
          outputFile,
          cacheDir: input.cacheDir,
          mtimeMs: stat.mtimeMs,
          transform: parsed.transform,
        });
      } else {
        await fs.mkdir(outputDir, { recursive: true });
        await fs.copyFile(resolved.absolute, outputFile);
      }
      files.push(outputFile);
      const rewritten = tag.replace(rawSrc, escapeAttribute(outputName));
      html = html.replace(tag, rewritten);
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      const message = `[ox-content] failed to process page resource ${JSON.stringify(src)} on ${input.inputPath}: ${detail}`;
      errors.push(message);
      fatal.push(message);
    }
  }

  return { html, files, errors, fatal };
}

function resolveBundlePath(
  pathname: string,
  bundleRoot: string,
  contentRoot: string,
): { ok: true; absolute: string } | { ok: false } {
  if (path.isAbsolute(pathname) || pathname.includes("\0")) {
    return { ok: false };
  }
  const absolute = path.resolve(bundleRoot, pathname);
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

async function writeTransformedResource(input: {
  sourcePath: string;
  outputFile: string;
  cacheDir: string;
  mtimeMs: number;
  transform: ResourceTransform;
}): Promise<void> {
  const key = resourceCacheKey(input.sourcePath, input.mtimeMs, input.transform);
  const ext = path.extname(input.outputFile);
  const cacheFile = path.join(input.cacheDir, `${key}${ext}`);
  try {
    await fs.copyFile(cacheFile, input.outputFile);
    return;
  } catch {
    // Cache miss — process below.
  }

  const source = await fs.readFile(input.sourcePath);
  const output = transformResourceBuffer(source, input.sourcePath, input.transform);
  if (output.length > 8 * 1024 * 1024) {
    throw new Error("transform produced an oversized file");
  }
  await fs.mkdir(path.dirname(input.outputFile), { recursive: true });
  await fs.mkdir(input.cacheDir, { recursive: true });
  await fs.writeFile(cacheFile, output);
  await fs.writeFile(input.outputFile, output);
}

function transformResourceBuffer(
  source: Buffer,
  sourcePath: string,
  transform: ResourceTransform,
): Buffer {
  const needsPixels = Boolean(transform.width || transform.height || transform.crop);
  if (!needsPixels && !transform.format) {
    return source;
  }
  if (!needsPixels && transform.format) {
    if (!isPng(source)) {
      if (transform.format === formatFromPath(sourcePath)) {
        return source;
      }
      throw new Error(
        `cannot convert ${path.extname(sourcePath) || "source"} to ${transform.format}`,
      );
    }
    const image = decodePng(source);
    return encodeFormat(image, transform.format);
  }

  if (!isPng(source)) {
    throw new Error("resize/crop requires a PNG source");
  }
  const encoded = encodeFormat(
    applyPixelTransform(decodePng(source), transform),
    transform.format ?? "png",
  );
  return encoded;
}

function applyPixelTransform(image: RgbaImage, transform: ResourceTransform): RgbaImage {
  const crop = transform.crop;
  if (crop && crop !== "center") {
    const parts = crop.split(",").map((part) => Number(part.trim()));
    if (parts.length === 4 && parts.every((part) => Number.isFinite(part))) {
      return cropImage(image, parts[0]!, parts[1]!, parts[2]!, parts[3]!);
    }
    throw new Error(`invalid crop ${crop}`);
  }

  const width = transform.width;
  const height = transform.height;
  if (crop === "center") {
    if (!width || !height) {
      throw new Error("crop=center requires width and height");
    }
    return coverCrop(image, width, height);
  }
  if (width && height) {
    return resizeNearest(image, width, height);
  }
  if (width) {
    return resizeNearest(
      image,
      width,
      Math.max(1, Math.round((image.height * width) / image.width)),
    );
  }
  if (height) {
    return resizeNearest(
      image,
      Math.max(1, Math.round((image.width * height) / image.height)),
      height,
    );
  }
  return image;
}

function encodeFormat(image: RgbaImage, format: string): Buffer {
  if (format === "jpeg") {
    return encodeJpeg(image);
  }
  if (format === "png") {
    return encodePng(image);
  }
  if (format === "webp") {
    throw new Error("webp encoding requires a webp source without pixel transforms");
  }
  throw new Error(`unsupported format ${format}`);
}

function formatFromPath(filePath: string): string {
  const ext = path.extname(filePath).slice(1).toLowerCase();
  return ext === "jpg" ? "jpeg" : ext;
}

function unescapeHtml(value: string): string {
  return value
    .replaceAll("&amp;", "&")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">");
}

function escapeAttribute(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}
