/**
 * Opt-in page-bundle resources and build-time image processing.
 *
 * A page directory is the bundle root. Sibling images are addressable with
 * relative URLs. Resize/crop/format query transforms run at build time and
 * are cached by source mtime plus transform params. Paths that leave the
 * bundle or `srcDir` are never processed.
 */

import { createHash } from "node:crypto";
import * as path from "node:path";
import type { ResourcesOptions, ResolvedResourcesOptions } from "./types";

const DEFAULT_FORMATS = ["png", "jpeg", "webp"];
const HOSTILE_SRC = /^(?:javascript|data|vbscript):/i;

export class PageResourceError extends Error {
  readonly issues: string[];

  constructor(issues: string[]) {
    super(issues.join("\n"));
    this.name = "PageResourceError";
    this.issues = issues;
  }
}

export interface ResourceTransform {
  width?: number;
  height?: number;
  crop?: string;
  format?: string;
}

export interface ProcessPageResourcesInput {
  html: string;
  inputPath: string;
  outputPath: string;
  srcDir: string;
  options: ResolvedResourcesOptions;
  cacheDir: string;
}

export interface ProcessPageResourcesResult {
  html: string;
  files: string[];
  errors: string[];
  fatal: string[];
}

/**
 * Resolves `resources`. Omitted / `false` stay off. `true` or `{}` enables
 * defaults. An object enables the feature and overrides only set fields.
 */
export function resolveResourcesOptions(
  value: boolean | ResourcesOptions | undefined,
): ResolvedResourcesOptions {
  if (!value) {
    return {
      enabled: false,
      formats: [...DEFAULT_FORMATS],
      widths: [],
      missing: "error",
    };
  }
  if (value === true) {
    return {
      enabled: true,
      formats: [...DEFAULT_FORMATS],
      widths: [],
      missing: "error",
    };
  }
  return {
    enabled: true,
    formats: normalizeFormats(value.formats),
    widths: normalizeWidths(value.widths),
    missing: value.missing === "warn" ? "warn" : "error",
  };
}

/**
 * Cache key for a source file plus transform. Changing mtime or params
 * produces a different key so stale derivatives are not reused.
 */
export function resourceCacheKey(
  sourcePath: string,
  mtimeMs: number,
  transform: ResourceTransform,
): string {
  return createHash("sha256")
    .update(sourcePath)
    .update("\0")
    .update(String(mtimeMs))
    .update("\0")
    .update(JSON.stringify(normalizeTransform(transform)))
    .digest("hex");
}

/** True when `candidate` stays inside `root` after resolve. */
export function isInsideRoot(root: string, candidate: string): boolean {
  const resolvedRoot = path.resolve(root);
  const resolved = path.resolve(candidate);
  const relative = path.relative(resolvedRoot, resolved);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

export function parseResourceSrc(
  src: string,
): { pathname: string; transform: ResourceTransform } | undefined {
  const trimmed = src.trim();
  if (!trimmed || isRemoteOrAbsolute(trimmed) || HOSTILE_SRC.test(trimmed.replace(/\s+/g, ""))) {
    return undefined;
  }
  const withoutHash = trimmed.split("#")[0] ?? trimmed;
  const queryIndex = withoutHash.indexOf("?");
  const pathname = queryIndex === -1 ? withoutHash : withoutHash.slice(0, queryIndex);
  const query = queryIndex === -1 ? "" : withoutHash.slice(queryIndex + 1);
  if (!pathname || pathname.includes("\0")) {
    return undefined;
  }
  const params = new URLSearchParams(query);
  return {
    pathname,
    transform: {
      width: parsePositiveInt(params.get("width") ?? params.get("w")),
      height: parsePositiveInt(params.get("height") ?? params.get("h")),
      crop: params.get("crop")?.trim() || undefined,
      format: normalizeFormat(params.get("format") ?? undefined),
    },
  };
}

function isRemoteOrAbsolute(src: string): boolean {
  const compact = src.replace(/\s+/g, "");
  return (
    /^[a-z][a-z0-9+.-]*:/i.test(compact) || compact.startsWith("//") || compact.startsWith("/")
  );
}

function parsePositiveInt(raw: string | null): number | undefined {
  if (!raw) {
    return undefined;
  }
  if (!/^[0-9]+$/.test(raw)) {
    return undefined;
  }
  const value = Number(raw);
  return value > 0 ? value : undefined;
}

function normalizeFormats(formats: string[] | undefined): string[] {
  if (!formats?.length) {
    return [...DEFAULT_FORMATS];
  }
  const normalized = formats
    .map((format) => normalizeFormat(format))
    .filter((format): format is string => Boolean(format));
  return normalized.length > 0 ? [...new Set(normalized)] : [...DEFAULT_FORMATS];
}

function normalizeWidths(widths: number[] | undefined): number[] {
  if (!widths?.length) {
    return [];
  }
  return [...new Set(widths.filter((width) => Number.isInteger(width) && width > 0))];
}

function normalizeFormat(format: string | undefined): string | undefined {
  if (!format) {
    return undefined;
  }
  const value = format.trim().toLowerCase();
  if (value === "jpg") {
    return "jpeg";
  }
  return value || undefined;
}

function normalizeTransform(transform: ResourceTransform): ResourceTransform {
  return {
    width: transform.width,
    height: transform.height,
    crop: transform.crop,
    format: transform.format,
  };
}

export { processPageResources } from "./resources-process";
