import { randomBytes } from "node:crypto";
import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import type { OgpData, ResolvedOgpOptions } from "./types";

export interface OgpCacheEntry {
  v: 1;
  url: string;
  cachedAt: number;
  data: OgpData | null;
}

interface MemoryRecord {
  data: OgpData | null;
  timestamp: number;
}

const memoryCache = new Map<string, MemoryRecord>();

export function clearOgpCache(): void {
  memoryCache.clear();
}

export function ogpCacheFilePath(directory: string, key: string): string {
  return path.join(directory, `${key}.json`);
}

export function isFreshOgpEntry(cachedAt: number, ttl: number, now: number): boolean {
  return now - cachedAt < ttl;
}

export function parseOgpCacheEntry(value: unknown): OgpCacheEntry | null {
  if (!value || typeof value !== "object") return null;
  const entry = value as Record<string, unknown>;
  if (entry.v !== 1) return null;
  if (typeof entry.url !== "string" || entry.url.length === 0) return null;
  if (typeof entry.cachedAt !== "number" || !Number.isFinite(entry.cachedAt)) return null;
  if (entry.data !== null && !isOgpData(entry.data)) return null;
  return { v: 1, url: entry.url, cachedAt: entry.cachedAt, data: entry.data };
}

export function readMemoryOgp(
  key: string,
  options: ResolvedOgpOptions,
  now: number,
): OgpData | null | undefined {
  const cached = memoryCache.get(key);
  if (!cached || !isFreshOgpEntry(cached.timestamp, options.cacheTTL, now)) {
    return undefined;
  }
  return cached.data;
}

export function writeMemoryOgp(
  key: string,
  data: OgpData | null,
  timestamp: number,
  options: ResolvedOgpOptions,
): void {
  if (data === null && !options.persistCache) return;
  memoryCache.set(key, { data, timestamp });
}

export async function readDiskOgp(
  key: string,
  options: ResolvedOgpOptions,
  now: number,
): Promise<OgpData | null | undefined> {
  const file = ogpCacheFilePath(options.cacheDir, key);
  try {
    const entry = parseOgpCacheEntry(JSON.parse(await readFile(file, "utf8")));
    if (!entry) {
      await discardCorruptEntry(file);
      return undefined;
    }
    if (!isFreshOgpEntry(entry.cachedAt, options.cacheTTL, now)) return undefined;
    return entry.data;
  } catch (error) {
    if (isEnoent(error)) return undefined;
    await discardCorruptEntry(file);
    return undefined;
  }
}

export async function writeDiskOgp(
  key: string,
  url: string,
  data: OgpData | null,
  options: ResolvedOgpOptions,
  cachedAt: number,
): Promise<void> {
  const directory = options.cacheDir;
  const target = ogpCacheFilePath(directory, key);
  const temp = path.join(directory, `.${key}.${process.pid}.${randomBytes(8).toString("hex")}.tmp`);
  const entry: OgpCacheEntry = { v: 1, url, cachedAt, data };
  try {
    await mkdir(directory, { recursive: true });
    await writeFile(temp, `${JSON.stringify(entry)}\n`);
    try {
      await rename(temp, target);
    } catch {
      await writeFile(target, `${JSON.stringify(entry)}\n`);
      await rm(temp, { force: true });
    }
  } catch {
    await rm(temp, { force: true }).catch(() => undefined);
  }
}

function isOgpData(value: unknown): value is OgpData {
  if (!value || typeof value !== "object") return false;
  const data = value as Record<string, unknown>;
  if (typeof data.url !== "string" || typeof data.title !== "string") return false;
  for (const field of ["description", "image", "siteName", "favicon"] as const) {
    if (data[field] !== undefined && typeof data[field] !== "string") return false;
  }
  return true;
}

async function discardCorruptEntry(file: string): Promise<void> {
  console.warn(`Ignoring corrupt Open Graph cache entry ${file}`);
  await rm(file, { force: true }).catch(() => undefined);
}

function isEnoent(error: unknown): boolean {
  return Boolean(error && typeof error === "object" && "code" in error && error.code === "ENOENT");
}
