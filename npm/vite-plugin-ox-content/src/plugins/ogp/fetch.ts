import { classifyError, classifyStatus, warnEmbedFailure } from "../provider-failure";
import { readDiskOgp, readMemoryOgp, writeDiskOgp, writeMemoryOgp } from "./cache";
import type { OgpData, OgpOptions, ResolvedOgpOptions } from "./types";
import { resolveOgpOptions } from "./types";
import { parseOgpFromHtml } from "./parse";
import { requestOgpPage } from "./request";
import { isSafeOgpUrl, normalizeOgpUrl, ogpCacheKey } from "./url";

const inflight = new Map<string, Promise<OgpData | null>>();

export async function fetchOgpData(url: string, options: OgpOptions = {}): Promise<OgpData | null> {
  if (!isSafeOgpUrl(url)) return null;

  const resolved = resolveOgpOptions(options);
  const key = ogpCacheKey(url);
  const pending = inflight.get(key);
  if (pending) return pending;

  const request = loadOgpData(url, key, resolved).finally(() => {
    if (inflight.get(key) === request) inflight.delete(key);
  });
  inflight.set(key, request);
  return request;
}

async function loadOgpData(
  url: string,
  key: string,
  options: ResolvedOgpOptions,
): Promise<OgpData | null> {
  const now = Date.now();
  if (options.cache && !options.refresh) {
    const memory = readMemoryOgp(key, options, now);
    if (memory !== undefined) return memory;
    if (options.persistCache) {
      const disk = await readDiskOgp(key, options, now);
      if (disk !== undefined) {
        writeMemoryOgp(key, disk, now, options);
        return disk;
      }
    }
  }

  const data = await requestOgpData(url, options);
  if (options.cache) {
    writeMemoryOgp(key, data, now, options);
    if (options.persistCache) {
      await writeDiskOgp(key, normalizeOgpUrl(url), data, options, now);
    }
  }
  return data;
}

async function requestOgpData(url: string, options: ResolvedOgpOptions): Promise<OgpData | null> {
  try {
    const result = await requestOgpPage(url, {
      timeout: options.timeout,
      headers: {
        "User-Agent": options.userAgent,
        Accept: "text/html,application/xhtml+xml",
      },
    });
    if (result.kind === "blocked") {
      return null;
    }
    if (result.kind === "status") {
      warnEmbedFailure("ogp", url, classifyStatus(result.status), "a link-only card");
      return null;
    }

    return parseOgpFromHtml(result.html, url);
  } catch (error) {
    warnEmbedFailure("ogp", url, classifyError(error), "a link-only card");
    return null;
  }
}
