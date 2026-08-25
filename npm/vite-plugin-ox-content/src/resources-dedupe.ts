/**
 * Site-wide content-addressed emit for identical page-resource bytes.
 *
 * Hashing streams the file. The first digest+extension pair writes once;
 * later pages reuse that path. Image decode is never used here.
 */

import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import * as fs from "node:fs/promises";
import * as path from "node:path";

export const DEDUPE_ASSET_DIR = path.join("assets", "content");

const TRANSFORM_QUERY_KEYS = new Set(["width", "w", "height", "h", "crop", "format"]);

export interface ResourceDedupeStore {
  /** `${digest}\\0${ext}` → absolute canonical file. */
  readonly canonical: Map<string, string>;
  /** Source+mtime+transform key → digest, so the same input is not rehashed. */
  readonly hashes: Map<string, string>;
}

export interface CanonicalResource {
  digest: string;
  ext: string;
  absolutePath: string;
  publicPath: string;
}

export function createResourceDedupeStore(): ResourceDedupeStore {
  return { canonical: new Map(), hashes: new Map() };
}

export function normalizeDedupeExt(ext: string): string {
  const value = ext.replace(/^\./, "").trim().toLowerCase();
  if (value === "jpeg") {
    return "jpg";
  }
  return value || "bin";
}

export function canonicalPublicPath(base: string, digest: string, ext: string): string {
  const prefix = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  return `${prefix}${DEDUPE_ASSET_DIR.split(path.sep).join("/")}/${digest}.${ext}`;
}

export function canonicalAbsolutePath(outDir: string, digest: string, ext: string): string {
  return path.join(outDir, DEDUPE_ASSET_DIR, `${digest}.${ext}`);
}

/**
 * SHA-256 of emitted bytes plus a NUL and the serving extension so the
 * same payload cannot be served under an incompatible media type.
 */
export async function hashResourceFile(
  filePath: string,
  ext: string,
  store: ResourceDedupeStore,
  reuseKey: string,
): Promise<string> {
  const cached = store.hashes.get(reuseKey);
  if (cached) {
    return cached;
  }
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(filePath)) {
    hash.update(chunk);
  }
  hash.update("\0");
  hash.update(ext);
  const digest = hash.digest("hex");
  store.hashes.set(reuseKey, digest);
  return digest;
}

export function hashResourceBuffer(
  bytes: Buffer,
  ext: string,
  store: ResourceDedupeStore,
  reuseKey: string,
): string {
  const cached = store.hashes.get(reuseKey);
  if (cached) {
    return cached;
  }
  const digest = createHash("sha256").update(bytes).update("\0").update(ext).digest("hex");
  store.hashes.set(reuseKey, digest);
  return digest;
}

export async function emitCanonicalResource(
  store: ResourceDedupeStore,
  input: {
    digest: string;
    ext: string;
    sourcePath: string;
    outDir: string;
    base: string;
  },
): Promise<{ asset: CanonicalResource; wrote: boolean }> {
  const key = `${input.digest}\0${input.ext}`;
  const existing = store.canonical.get(key);
  const publicPath = canonicalPublicPath(input.base, input.digest, input.ext);
  if (existing) {
    return {
      asset: {
        digest: input.digest,
        ext: input.ext,
        absolutePath: existing,
        publicPath,
      },
      wrote: false,
    };
  }
  const absolutePath = canonicalAbsolutePath(input.outDir, input.digest, input.ext);
  await fs.mkdir(path.dirname(absolutePath), { recursive: true });
  await fs.copyFile(input.sourcePath, absolutePath);
  store.canonical.set(key, absolutePath);
  return {
    asset: { digest: input.digest, ext: input.ext, absolutePath, publicPath },
    wrote: true,
  };
}

/**
 * Prefer a hard link at the original output path. `link` failure removes
 * any stale alias and copies so a shared inode is never overwritten.
 */
export async function linkOrCopyAlias(
  canonical: string,
  alias: string,
  linker: typeof fs.link = fs.link,
): Promise<"link" | "copy"> {
  await fs.mkdir(path.dirname(alias), { recursive: true });
  try {
    await linker(canonical, alias);
    return "link";
  } catch {
    await fs.rm(alias, { force: true });
  }
  try {
    await linker(canonical, alias);
    return "link";
  } catch {
    await fs.copyFile(canonical, alias);
    return "copy";
  }
}

/** Keep leftover search/hash; drop consumed transform params. */
export function rewriteToCanonicalUrl(originalSrc: string, canonicalPath: string): string {
  const hashIndex = originalSrc.indexOf("#");
  const hash = hashIndex === -1 ? "" : originalSrc.slice(hashIndex);
  const withoutHash = hashIndex === -1 ? originalSrc : originalSrc.slice(0, hashIndex);
  const queryIndex = withoutHash.indexOf("?");
  const leftover = leftoverQuery(queryIndex === -1 ? "" : withoutHash.slice(queryIndex + 1));
  return `${canonicalPath}${leftover}${hash}`;
}

function leftoverQuery(query: string): string {
  if (!query) {
    return "";
  }
  const params = new URLSearchParams(query);
  for (const key of TRANSFORM_QUERY_KEYS) {
    params.delete(key);
  }
  const next = params.toString();
  return next ? `?${next}` : "";
}
