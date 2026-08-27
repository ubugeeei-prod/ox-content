/**
 * A cross-build cache for provider card metadata.
 *
 * Open Graph cards have had one of these since `ogp.persistCache`; the provider
 * fetchers had a process-local `Map`, so every clean build and every fresh CI
 * worker re-fetched Qiita, Zenn, npm, crates.io, PyPI, Docker Hub, CodePen,
 * JSFiddle, Observable, Vimeo, and Twitch.
 *
 * The envelope, the atomic replace, and the corrupt-entry handling follow
 * `ogp/cache.ts` deliberately: a second cache with different failure behaviour
 * would be a second thing to reason about.
 */

import { randomBytes, createHash } from "node:crypto";
import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import path from "node:path";

/** What one cache file holds. `null` data is a remembered miss. */
interface ProviderCacheEntry<T> {
  v: 1;
  key: string;
  cachedAt: number;
  data: T | null;
}

export interface ProviderDiskCacheOptions {
  /** Off by default, matching `ogp.persistCache`. */
  persistCache?: boolean;
  /** Defaults to `.cache/ox-content/providers`. */
  cacheDir?: string;
}

export const DEFAULT_PROVIDER_CACHE_DIR = ".cache/ox-content/providers";

/**
 * A file name for a cache key.
 *
 * Hashed rather than escaped: a key carries a provider URL, and URLs contain
 * characters a path cannot, in combinations that differ per platform.
 */
export function providerCacheFileName(key: string): string {
  return `${createHash("sha256").update(key).digest("hex")}.json`;
}

export function resolveProviderCacheDir(
  options: ProviderDiskCacheOptions | undefined,
): string | null {
  if (!options?.persistCache) return null;
  return path.resolve(options.cacheDir ?? DEFAULT_PROVIDER_CACHE_DIR);
}

function isEntry<T>(value: unknown, key: string): value is ProviderCacheEntry<T> {
  if (!value || typeof value !== "object") return false;
  const entry = value as Record<string, unknown>;
  return (
    entry.v === 1 &&
    entry.key === key &&
    typeof entry.cachedAt === "number" &&
    Number.isFinite(entry.cachedAt)
  );
}

/**
 * Reads a cached value.
 *
 * Returns `undefined` for "nothing usable here" and `null` for "this was
 * fetched before and there was nothing", which are different answers: the
 * second is a remembered miss and must not trigger another request.
 */
export async function readProviderCache<T>(
  cacheDir: string | null,
  key: string,
  ttl: number,
  now: number,
): Promise<T | null | undefined> {
  if (!cacheDir) return undefined;
  const file = path.join(cacheDir, providerCacheFileName(key));
  try {
    const parsed: unknown = JSON.parse(await readFile(file, "utf8"));
    if (!isEntry<T>(parsed, key)) {
      await discard(file);
      return undefined;
    }
    if (now - parsed.cachedAt >= ttl) return undefined;
    return parsed.data;
  } catch (error) {
    if (isEnoent(error)) return undefined;
    await discard(file);
    return undefined;
  }
}

/** Writes a value, replacing any existing entry atomically. */
export async function writeProviderCache<T>(
  cacheDir: string | null,
  key: string,
  data: T | null,
  cachedAt: number,
): Promise<void> {
  if (!cacheDir) return;
  const target = path.join(cacheDir, providerCacheFileName(key));
  const temp = path.join(
    cacheDir,
    `.${providerCacheFileName(key)}.${process.pid}.${randomBytes(8).toString("hex")}.tmp`,
  );
  const entry: ProviderCacheEntry<T> = { v: 1, key, cachedAt, data };
  try {
    await mkdir(cacheDir, { recursive: true });
    await writeFile(temp, `${JSON.stringify(entry)}\n`);
    try {
      await rename(temp, target);
    } catch {
      await writeFile(target, `${JSON.stringify(entry)}\n`);
      await rm(temp, { force: true });
    }
  } catch {
    // A cache that cannot be written is a slow build, not a broken one.
    await rm(temp, { force: true }).catch(() => undefined);
  }
}

async function discard(file: string): Promise<void> {
  console.warn(`[ox-content] Ignoring corrupt provider cache entry ${file}`);
  await rm(file, { force: true }).catch(() => undefined);
}

function isEnoent(error: unknown): boolean {
  return Boolean(error && typeof error === "object" && "code" in error && error.code === "ENOENT");
}
