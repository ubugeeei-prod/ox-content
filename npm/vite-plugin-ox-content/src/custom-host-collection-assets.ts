import * as fs from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import * as path from "node:path";
import type { Connect, ViteDevServer } from "vite";
import type { CollectionAssetManifest } from "./collection-assets";
import type {
  OxContentCustomHostBaseContext,
  OxContentCustomHostCollectionAssetsOptions,
} from "./custom-host-types";
import { stripBasePathname } from "./custom-host-utils";
import {
  anyCustomHostDependencyMatches,
  dependencyWatchPaths,
  normalizeCustomHostDependencies,
  type NormalizedCustomHostDependency,
} from "./custom-host-watch";

export interface CustomHostCollectionAssetsDevController {
  middleware: Connect.NextHandleFunction;
  manifest(): Promise<CollectionAssetManifest | undefined>;
  invalidate(file: string): boolean;
  close(): void;
}

interface CollectionAssetsSnapshot {
  manifest: CollectionAssetManifest;
  sources: Map<string, string>;
  ownedPrefixes: string[];
  sourceDependencies: NormalizedCustomHostDependency[];
}

export function createCustomHostCollectionAssetsDevController(input: {
  server: ViteDevServer;
  options: false | OxContentCustomHostCollectionAssetsOptions | undefined;
  context: OxContentCustomHostBaseContext;
  beforeReplan(file: string): void;
  onReplanned(): void;
}): CustomHostCollectionAssetsDevController | undefined {
  if (!input.options) {
    return undefined;
  }
  const options = input.options;

  const declaredDependencies = normalizeCustomHostDependencies(input.context.root, options.watch);
  const addedWatchPaths = new Set<string>();
  let snapshot: CollectionAssetsSnapshot | undefined;
  let snapshotPromise: Promise<CollectionAssetsSnapshot> | undefined;
  let generation = 0;
  let closed = false;

  const addWatchPaths = (paths: readonly string[]) => {
    const nextPaths = paths.filter((watchPath) => !addedWatchPaths.has(watchPath));
    if (nextPaths.length === 0) {
      return;
    }
    input.server.watcher.add(nextPaths);
    for (const watchPath of nextPaths) {
      addedWatchPaths.add(watchPath);
    }
  };

  addWatchPaths(dependencyWatchPaths(declaredDependencies));

  const loadSnapshot = () => {
    snapshotPromise ??= planSnapshot({
      context: input.context,
      generation,
      options,
      declaredOwnedPrefixes: options.ownedPrefixes,
      apply: (next, expectedGeneration) => {
        if (closed || generation !== expectedGeneration) {
          return;
        }
        const hadSnapshot = snapshot !== undefined;
        snapshot = next;
        addWatchPaths(dependencyWatchPaths(next.sourceDependencies));
        if (hadSnapshot) {
          input.onReplanned();
        }
      },
      clear: (promise) => {
        if (snapshotPromise === promise) {
          snapshotPromise = undefined;
        }
      },
    });
    return snapshotPromise;
  };

  const replan = () => {
    generation += 1;
    snapshotPromise = undefined;
    void loadSnapshot().catch((error) => {
      console.warn(`[ox-content] Failed to re-plan custom-host collection assets: ${error}`);
    });
  };

  return {
    async manifest() {
      // A watcher-triggered refresh publishes atomically through `apply`.
      // Keep serving the last successful generation while it runs or fails.
      return snapshot?.manifest ?? (await loadSnapshot()).manifest;
    },
    middleware: async (req, res, next) => {
      if (req.method !== "GET" && req.method !== "HEAD") {
        next();
        return;
      }
      const publicPath = requestPath(req, input.context.base);
      if (!publicPath) {
        next();
        return;
      }

      try {
        const current = snapshot ?? (await loadSnapshot());
        await serveCollectionAsset(current, publicPath, req, res, next);
      } catch (error) {
        next(error);
      }
    },
    invalidate(file) {
      const sourceDependencies = snapshot?.sourceDependencies ?? [];
      if (
        !anyCustomHostDependencyMatches(declaredDependencies, file) &&
        !anyCustomHostDependencyMatches(sourceDependencies, file)
      ) {
        return false;
      }
      input.beforeReplan(file);
      replan();
      return true;
    },
    close() {
      closed = true;
      if (addedWatchPaths.size > 0) {
        input.server.watcher.unwatch([...addedWatchPaths]);
        addedWatchPaths.clear();
      }
    },
  };
}

function planSnapshot(input: {
  context: OxContentCustomHostBaseContext;
  generation: number;
  options: OxContentCustomHostCollectionAssetsOptions;
  declaredOwnedPrefixes: readonly string[] | undefined;
  apply(snapshot: CollectionAssetsSnapshot, generation: number): void;
  clear(promise: Promise<CollectionAssetsSnapshot>): void;
}): Promise<CollectionAssetsSnapshot> {
  let promise!: Promise<CollectionAssetsSnapshot>;
  promise = resolveCollectionAssetManifest(input.options, input.context)
    .then((manifest) => createSnapshot(input.context.root, manifest, input.declaredOwnedPrefixes))
    .then((snapshot) => {
      input.apply(snapshot, input.generation);
      return snapshot;
    })
    .catch((error) => {
      input.clear(promise);
      throw error;
    });
  return promise;
}

export async function resolveCollectionAssetManifest(
  options: OxContentCustomHostCollectionAssetsOptions,
  context: OxContentCustomHostBaseContext,
): Promise<CollectionAssetManifest> {
  return typeof options.manifest === "function"
    ? await options.manifest({ ...context })
    : options.manifest;
}

function createSnapshot(
  root: string,
  manifest: CollectionAssetManifest,
  declaredOwnedPrefixes: readonly string[] | undefined,
): CollectionAssetsSnapshot {
  const sources = new Map<string, string>();
  const ownedPrefixes = new Set((declaredOwnedPrefixes ?? []).map(normalizePublicPathPrefix));
  const sourceDependencies = manifest.assets.map((asset) => ({
    path: asset.sourcePath,
    kind: "file" as const,
  }));

  for (const asset of manifest.assets) {
    const contentPath = normalizePublicPath(asset.contentPath);
    sources.set(contentPath, asset.sourcePath);
    ownedPrefixes.add(parentPublicPath(contentPath));
    for (const publicPath of asset.publicPaths) {
      sources.set(normalizePublicPath(publicPath), asset.sourcePath);
    }
  }

  return {
    manifest,
    sources,
    ownedPrefixes: [...ownedPrefixes],
    sourceDependencies: normalizeCustomHostDependencies(root, sourceDependencies),
  };
}

async function serveCollectionAsset(
  snapshot: CollectionAssetsSnapshot,
  publicPath: string,
  req: IncomingMessage,
  res: ServerResponse,
  next: (error?: unknown) => void,
): Promise<void> {
  const sourcePath = snapshot.sources.get(publicPath);
  if (!sourcePath) {
    if (snapshot.ownedPrefixes.some((prefix) => ownsPublicPath(prefix, publicPath))) {
      res.statusCode = 404;
      res.setHeader("Content-Type", "text/plain; charset=utf-8");
      res.end(req.method === "HEAD" ? undefined : "Not found");
      return;
    }
    next();
    return;
  }

  const bytes = await fs.readFile(sourcePath);
  res.statusCode = 200;
  res.setHeader("Content-Type", contentType(sourcePath));
  res.end(req.method === "HEAD" ? undefined : bytes);
}

function requestPath(req: IncomingMessage, base: string): string | undefined {
  if (!req.url) return undefined;
  try {
    const pathname = new URL(req.url, "http://ox-content.local").pathname;
    return normalizePublicPath(stripBasePathname(pathname, base) ?? pathname);
  } catch {
    return undefined;
  }
}

function normalizePublicPathPrefix(value: string): string {
  if (value === "/") {
    return "/";
  }
  return normalizePublicPath(value).replace(/\/$/u, "");
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

function parentPublicPath(publicPath: string): string {
  const directory = path.posix.dirname(publicPath);
  return directory === "." ? "/" : directory;
}

function ownsPublicPath(prefix: string, publicPath: string): boolean {
  if (prefix === "/") {
    return true;
  }
  return publicPath === prefix || publicPath.startsWith(`${prefix}/`);
}

function hasControlCharacter(value: string): boolean {
  for (const character of value) {
    const code = character.charCodeAt(0);
    if (code < 32 || code === 127) return true;
  }
  return false;
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
