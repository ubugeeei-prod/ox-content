import { createHash } from "node:crypto";
import * as fs from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import * as path from "node:path";

/** One collection file and the public URLs that should expose it. */
export interface CollectionAssetInput {
  sourcePath: string;
  publicPath: string | readonly string[];
}

/** Inputs for a content-addressed collection asset manifest. */
export interface PlanCollectionAssetsInput {
  assets: readonly CollectionAssetInput[];
  root?: string;
  contentDir?: string;
}

/** A source file, its normalized aliases, and its content-addressed URL. */
export interface CollectionAssetManifestEntry {
  sourcePath: string;
  publicPaths: string[];
  contentPath: string;
}

/**
 * A reusable asset plan for a custom host's build writer and development
 * middleware. Source paths are absolute so both consumers use the same file.
 */
export interface CollectionAssetManifest {
  assets: CollectionAssetManifestEntry[];
}

export interface WriteCollectionAssetsInput {
  manifest: CollectionAssetManifest;
  outDir: string;
}

export interface WriteCollectionAssetsResult {
  files: string[];
}

type Next = (error?: unknown) => void;
type AssetMiddleware = (
  req: IncomingMessage,
  res: ServerResponse,
  next: Next,
) => void | Promise<void>;

/**
 * Hash collection files and normalize their public aliases without taking over
 * the host router. The returned manifest can be written at build time and
 * served directly by {@link createCollectionAssetsMiddleware} in development.
 */
export async function planCollectionAssets(
  input: PlanCollectionAssetsInput,
): Promise<CollectionAssetManifest> {
  const root = await fs.realpath(path.resolve(input.root ?? process.cwd()));
  const contentDir = normalizePublicPath(input.contentDir ?? "/assets/content");
  const paths = new Map<string, { contentPath: string; publicPath: string }>();
  const assets: CollectionAssetManifestEntry[] = [];

  for (const asset of input.assets) {
    const sourcePath = await resolveSourcePath(root, asset.sourcePath);
    const bytes = await fs.readFile(sourcePath);
    const contentPath = `${contentDir}/${contentHash(bytes)}${extension(sourcePath)}`;
    const publicPaths = uniquePaths(asset.publicPath);

    for (const publicPath of publicPaths) {
      recordPublicPath(paths, publicPath, contentPath);
    }

    assets.push({ sourcePath, publicPaths, contentPath });
  }

  for (const asset of assets) {
    recordPublicPath(paths, asset.contentPath, asset.contentPath);
  }

  return { assets };
}

/**
 * Write each content-addressed target once, then materialize its aliases with
 * hard links where supported and copies where they are not.
 */
export async function writeCollectionAssets(
  input: WriteCollectionAssetsInput,
): Promise<WriteCollectionAssetsResult> {
  const outDir = path.resolve(input.outDir);
  const written = new Map<string, string>();
  const files: string[] = [];

  for (const asset of input.manifest.assets) {
    const contentPath = normalizePublicPath(asset.contentPath);
    const contentFile = outputFile(outDir, contentPath);
    const bytes = await fs.readFile(asset.sourcePath);
    const expected = contentPathFor(asset.sourcePath, bytes, contentPath);
    if (contentPath !== expected) {
      throw new Error(
        `Collection asset ${JSON.stringify(asset.sourcePath)} changed after planning; plan it again before writing.`,
      );
    }

    const contentKey = outputCollisionKey(contentFile);
    const existingContent = written.get(contentKey);
    if (existingContent && existingContent !== contentFile) {
      throw outputCollisionError(contentFile, existingContent);
    }
    if (!existingContent) {
      await fs.mkdir(path.dirname(contentFile), { recursive: true });
      await fs.writeFile(contentFile, bytes);
      written.set(contentKey, contentFile);
      files.push(contentFile);
    }

    for (const publicPath of asset.publicPaths) {
      const aliasFile = outputFile(outDir, normalizePublicPath(publicPath));
      const aliasKey = outputCollisionKey(aliasFile);
      const existingAlias = written.get(aliasKey);
      if (existingAlias && existingAlias !== aliasFile) {
        throw outputCollisionError(aliasFile, existingAlias);
      }
      if (existingAlias) {
        continue;
      }
      await fs.mkdir(path.dirname(aliasFile), { recursive: true });
      await fs.rm(aliasFile, { force: true });
      await linkOrCopy(contentFile, aliasFile);
      written.set(aliasKey, aliasFile);
      files.push(aliasFile);
    }
  }

  return { files };
}

/**
 * Create Connect-compatible middleware that serves planned aliases and their
 * content-addressed targets during development.
 */
export function createCollectionAssetsMiddleware(
  manifest: CollectionAssetManifest,
): AssetMiddleware {
  const sources = new Map<string, string>();
  for (const asset of manifest.assets) {
    const contentPath = normalizePublicPath(asset.contentPath);
    sources.set(contentPath, asset.sourcePath);
    for (const publicPath of asset.publicPaths) {
      sources.set(normalizePublicPath(publicPath), asset.sourcePath);
    }
  }

  return async (req, res, next) => {
    if (req.method !== "GET" && req.method !== "HEAD") {
      next();
      return;
    }
    const publicPath = requestPath(req);
    const sourcePath = publicPath ? sources.get(publicPath) : undefined;
    if (!sourcePath) {
      next();
      return;
    }

    try {
      const bytes = await fs.readFile(sourcePath);
      res.statusCode = 200;
      res.setHeader("Content-Type", contentType(sourcePath));
      res.end(req.method === "HEAD" ? undefined : bytes);
    } catch (error) {
      next(error);
    }
  };
}

async function resolveSourcePath(root: string, sourcePath: string): Promise<string> {
  if (!sourcePath || sourcePath.includes("\0")) {
    throw new Error("Collection asset sourcePath must be a non-empty file path.");
  }
  const candidate = path.resolve(root, sourcePath);
  if (!isWithin(root, candidate)) {
    throw new Error(
      `Collection asset sourcePath ${JSON.stringify(sourcePath)} must stay within root.`,
    );
  }
  const resolved = await fs.realpath(candidate);
  if (!isWithin(root, resolved)) {
    throw new Error(
      `Collection asset sourcePath ${JSON.stringify(sourcePath)} must stay within root.`,
    );
  }
  return resolved;
}

function uniquePaths(value: string | readonly string[]): string[] {
  const values = typeof value === "string" ? [value] : value;
  if (values.length === 0) {
    throw new Error("Collection asset publicPath must contain at least one URL path.");
  }
  return [...new Set(values.map(normalizePublicPath))];
}

function recordPublicPath(
  paths: Map<string, { contentPath: string; publicPath: string }>,
  publicPath: string,
  contentPath: string,
): void {
  const key = outputCollisionKey(publicPath);
  const existing = paths.get(key);
  if (existing && (existing.publicPath !== publicPath || existing.contentPath !== contentPath)) {
    throw new Error(
      `Collection asset path ${JSON.stringify(publicPath)} collides with ${JSON.stringify(existing.publicPath)} on case-insensitive filesystems.`,
    );
  }
  paths.set(key, { contentPath, publicPath });
}

function normalizePublicPath(value: string): string {
  if (!value.startsWith("/") || value.startsWith("//") || value.includes("\0")) {
    throw new Error(
      `Collection asset public path ${JSON.stringify(value)} must be an absolute URL path.`,
    );
  }
  const segments = value.slice(1).split("/");
  if (segments.length === 0 || segments.some((segment) => !segment)) {
    throw new Error(
      `Collection asset public path ${JSON.stringify(value)} must not contain empty segments.`,
    );
  }
  const decoded = segments.map((segment) => decodePathSegment(segment, value));
  return `/${decoded.map(encodeURIComponent).join("/")}`;
}

function decodePathSegment(segment: string, value: string): string {
  let decoded: string;
  try {
    decoded = decodeURIComponent(segment);
  } catch {
    throw new Error(
      `Collection asset public path ${JSON.stringify(value)} contains invalid URL encoding.`,
    );
  }
  if (
    !decoded ||
    decoded === "." ||
    decoded === ".." ||
    decoded.includes("/") ||
    decoded.includes("\\") ||
    hasControlCharacter(decoded)
  ) {
    throw new Error(`Collection asset public path ${JSON.stringify(value)} is unsafe.`);
  }
  return decoded;
}

function hasControlCharacter(value: string): boolean {
  for (const character of value) {
    const code = character.charCodeAt(0);
    if (code < 32 || code === 127) return true;
  }
  return false;
}

function outputFile(outDir: string, publicPath: string): string {
  const file = path.resolve(outDir, ...publicPath.slice(1).split("/").map(decodeURIComponent));
  if (!isWithin(outDir, file)) {
    throw new Error(`Collection asset public path ${JSON.stringify(publicPath)} escapes outDir.`);
  }
  return file;
}

function outputCollisionKey(value: string): string {
  return value.normalize("NFC").toLocaleLowerCase("en-US");
}

function outputCollisionError(pathname: string, existing: string): Error {
  return new Error(
    `Collection asset output ${JSON.stringify(pathname)} collides with ${JSON.stringify(existing)} on case-insensitive filesystems.`,
  );
}

function requestPath(req: IncomingMessage): string | undefined {
  if (!req.url) return undefined;
  try {
    return normalizePublicPath(new URL(req.url, "http://ox-content.local").pathname);
  } catch {
    return undefined;
  }
}

function contentPathFor(sourcePath: string, bytes: Uint8Array, contentPath: string): string {
  const directory = contentPath.slice(0, contentPath.lastIndexOf("/"));
  return `${directory}/${contentHash(bytes)}${extension(sourcePath)}`;
}

function contentHash(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function extension(sourcePath: string): string {
  const value = path.extname(sourcePath).toLowerCase();
  return /^[.][a-z0-9]{1,16}$/u.test(value) ? value : ".bin";
}

function isWithin(root: string, file: string): boolean {
  const relative = path.relative(root, file);
  return (
    relative !== "" &&
    !relative.startsWith(`..${path.sep}`) &&
    relative !== ".." &&
    !path.isAbsolute(relative)
  );
}

async function linkOrCopy(source: string, destination: string): Promise<void> {
  try {
    await fs.link(source, destination);
  } catch {
    await fs.copyFile(source, destination);
  }
}

function contentType(sourcePath: string): string {
  switch (path.extname(sourcePath).toLowerCase()) {
    case ".avif":
      return "image/avif";
    case ".gif":
      return "image/gif";
    case ".jpg":
    case ".jpeg":
      return "image/jpeg";
    case ".png":
      return "image/png";
    case ".svg":
      return "image/svg+xml";
    case ".webp":
      return "image/webp";
    default:
      return "application/octet-stream";
  }
}
