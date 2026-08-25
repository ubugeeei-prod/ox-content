import { mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { clearOgpCache, fetchOgpData, transformOgp } from "./ogp";
import { ogpCacheFilePath, parseOgpCacheEntry } from "./ogp/cache";
import { normalizeOgpUrl, ogpCacheKey } from "./ogp/url";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  clearOgpCache();
});

function htmlCard(title: string): string {
  return `<html><head><meta property="og:title" content="${title}"></head></html>`;
}

function okResponse(body: string): Response {
  return { ok: true, status: 200, text: async () => body } as Response;
}

function notFoundResponse(): Response {
  return { ok: false, status: 404, text: async () => "" } as Response;
}

async function withCacheDir<T>(run: (cacheDir: string) => Promise<T>): Promise<T> {
  const root = await mkdtemp(path.join(tmpdir(), "ox-content-ogp-"));
  try {
    return await run(path.join(root, "cache"));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

describe("Open Graph persistent cache", () => {
  it("derives deterministic keys from a normalized URL", () => {
    expect(normalizeOgpUrl("https://Example.com:443/post/#frag")).toBe("https://example.com/post");
    expect(ogpCacheKey("https://Example.com/post/")).toBe(ogpCacheKey("https://example.com/post"));
  });

  it("reuses in-memory hits without persist", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      return okResponse(htmlCard("Mem"));
    };
    const url = "https://example.com/memory";
    await expect(fetchOgpData(url)).resolves.toMatchObject({ title: "Mem" });
    await expect(fetchOgpData(url)).resolves.toMatchObject({ title: "Mem" });
    expect(requests).toBe(1);
  });

  it("does not write disk files when persistent cache is off", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      return okResponse(htmlCard("Example"));
    };
    await withCacheDir(async (cacheDir) => {
      await fetchOgpData("https://example.com/post", { cacheDir });
      await expect(readdir(path.dirname(cacheDir))).resolves.not.toContain("cache");
      expect(requests).toBe(1);
    });
  });

  it("reuses a seeded disk entry without fetching", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      return okResponse(htmlCard("Fresh"));
    };
    await withCacheDir(async (cacheDir) => {
      const url = "https://example.com/seeded";
      const key = ogpCacheKey(url);
      await writeFile(
        path.join(await mkdirp(cacheDir), `${key}.json`),
        `${JSON.stringify({
          v: 1,
          url,
          cachedAt: Date.now(),
          data: { url, title: "Seeded" },
        })}\n`,
      );

      const data = await fetchOgpData(url, { persistCache: true, cacheDir });
      expect(data).toEqual({ url, title: "Seeded" });
      expect(requests).toBe(0);
    });
  });

  it("stores negative entries so failed URLs are not retried", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      return notFoundResponse();
    };
    await withCacheDir(async (cacheDir) => {
      const url = "https://example.com/missing";
      const options = { persistCache: true, cacheDir };
      await expect(fetchOgpData(url, options)).resolves.toBeNull();
      clearOgpCache();
      await expect(fetchOgpData(url, options)).resolves.toBeNull();
      expect(requests).toBe(1);

      const raw = await readFile(ogpCacheFilePath(cacheDir, ogpCacheKey(url)), "utf8");
      expect(parseOgpCacheEntry(JSON.parse(raw))).toMatchObject({ v: 1, url, data: null });
    });
  });

  it("refetches after TTL expiry or refresh and replaces the file", async () => {
    let title = "First";
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      return okResponse(htmlCard(title));
    };
    await withCacheDir(async (cacheDir) => {
      const url = "https://example.com/refresh";
      const file = ogpCacheFilePath(cacheDir, ogpCacheKey(url));
      const options = { persistCache: true, cacheDir, cacheTTL: 60_000 };
      await expect(fetchOgpData(url, options)).resolves.toMatchObject({ title: "First" });
      title = "Second";
      clearOgpCache();
      await expect(fetchOgpData(url, options)).resolves.toMatchObject({ title: "First" });

      const stale = parseOgpCacheEntry(JSON.parse(await readFile(file, "utf8")));
      expect(stale).not.toBeNull();
      await writeFile(
        file,
        `${JSON.stringify({ ...stale, cachedAt: Date.now() - options.cacheTTL - 1 })}\n`,
      );
      clearOgpCache();
      await expect(fetchOgpData(url, options)).resolves.toMatchObject({ title: "Second" });

      title = "Third";
      clearOgpCache();
      await expect(fetchOgpData(url, { ...options, refresh: true })).resolves.toMatchObject({
        title: "Third",
      });
      expect(requests).toBe(3);
    });
  });

  it("ignores corrupt entries and does not poison later builds", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      return okResponse(htmlCard("Recovered"));
    };
    await withCacheDir(async (cacheDir) => {
      const url = "https://example.com/corrupt";
      const file = ogpCacheFilePath(await mkdirp(cacheDir), ogpCacheKey(url));
      await writeFile(file, "{not-json\n");
      await expect(fetchOgpData(url, { persistCache: true, cacheDir })).resolves.toMatchObject({
        title: "Recovered",
      });
      expect(requests).toBe(1);
      expect(parseOgpCacheEntry(JSON.parse(await readFile(file, "utf8")))).toMatchObject({
        data: { title: "Recovered" },
      });
    });
  });

  it("coalesces concurrent fetches into one valid JSON file", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      await new Promise((resolve) => setTimeout(resolve, 20));
      return okResponse(htmlCard("Parallel"));
    };
    await withCacheDir(async (cacheDir) => {
      const url = "https://example.com/parallel";
      const options = { persistCache: true, cacheDir };
      const [left, right] = await Promise.all([
        fetchOgpData(url, options),
        fetchOgpData(url, options),
      ]);
      expect(left).toEqual(right);
      expect(left?.title).toBe("Parallel");
      expect(requests).toBe(1);
      JSON.parse(await readFile(ogpCacheFilePath(cacheDir, ogpCacheKey(url)), "utf8"));
    });
  });

  it("lets caller-provided metadata win over fetch and cache", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      return okResponse(htmlCard("Fetched"));
    };
    await withCacheDir(async (cacheDir) => {
      const url = "https://example.com/caller";
      await writeFile(
        path.join(await mkdirp(cacheDir), `${ogpCacheKey(url)}.json`),
        `${JSON.stringify({
          v: 1,
          url,
          cachedAt: Date.now(),
          data: { url, title: "Cached" },
        })}\n`,
      );
      const html = await transformOgp(
        `<OgCard url="${url}"></OgCard>`,
        new Map([[url, { url, title: "Caller" }]]),
        { persistCache: true, cacheDir },
      );
      expect(html).toContain("Caller");
      expect(html).not.toContain("Cached");
      expect(html).not.toContain("Fetched");
      expect(requests).toBe(0);
    });
  });

  it("retries failed URLs in memory-only mode", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      return notFoundResponse();
    };
    await expect(fetchOgpData("https://example.com/retry")).resolves.toBeNull();
    await expect(fetchOgpData("https://example.com/retry")).resolves.toBeNull();
    expect(requests).toBe(2);
  });
});

async function mkdirp(directory: string): Promise<string> {
  await mkdir(directory, { recursive: true });
  return directory;
}
