/**
 * Transform cache writes for page resources.
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
import { resourceCacheKey, type ResourceTransform } from "./resources";

export async function ensureTransformedCache(input: {
  sourcePath: string;
  outputFile: string;
  cacheDir: string;
  mtimeMs: number;
  transform: ResourceTransform;
}): Promise<string> {
  const key = resourceCacheKey(input.sourcePath, input.mtimeMs, input.transform);
  const ext = path.extname(input.outputFile);
  const cacheFile = path.join(input.cacheDir, `${key}${ext}`);
  try {
    await fs.access(cacheFile);
    return cacheFile;
  } catch {
    // Cache miss — process below.
  }

  const source = await fs.readFile(input.sourcePath);
  const output = transformResourceBuffer(source, input.sourcePath, input.transform);
  if (output.length > 8 * 1024 * 1024) {
    throw new Error("transform produced an oversized file");
  }
  await fs.mkdir(input.cacheDir, { recursive: true });
  await fs.writeFile(cacheFile, output);
  return cacheFile;
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
    return encodeFormat(decodePng(source), transform.format);
  }

  if (!isPng(source)) {
    throw new Error("resize/crop requires a PNG source");
  }
  return encodeFormat(applyPixelTransform(decodePng(source), transform), transform.format ?? "png");
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
