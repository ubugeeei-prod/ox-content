import { mkdtemp, rm, readdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { clearProviderArticleCache, enrichProviderArticleEmbeds } from "./provider-articles";
import { providerCacheFileName } from "./provider-disk-cache";

afterEach(() => {
  clearProviderArticleCache();
});

const TAG = '<Qiita url="https://qiita.com/ubugeeei/items/abcdef123456"></Qiita>';

function countingFetch(body: unknown, counter: { calls: number }) {
  return (async () => {
    counter.calls += 1;
    return { ok: true, status: 200, json: async () => body } as Response;
  }) as unknown as typeof fetch;
}

async function withCacheDir<T>(run: (dir: string) => Promise<T>): Promise<T> {
  const root = await mkdtemp(path.join(tmpdir(), "ox-provider-cache-"));
  try {
    return await run(path.join(root, "providers"));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

describe("provider metadata disk cache", () => {
  it("serves a second build from disk without fetching again", async () => {
    await withCacheDir(async (cacheDir) => {
      const counter = { calls: 0 };
      const body = { title: "Static cards", user: { id: "ubugeeei" }, likes_count: 42 };
      const options = { persistCache: true, cacheDir };

      const first = await enrichProviderArticleEmbeds(
        TAG,
        { qiita: options },
        countingFetch(body, counter),
      );
      expect(counter.calls).toBe(1);

      // A fresh process would have an empty memory map; clearing models that.
      clearProviderArticleCache();

      const second = await enrichProviderArticleEmbeds(
        TAG,
        { qiita: options },
        countingFetch(body, counter),
      );
      expect(counter.calls).toBe(1);
      expect(second).toBe(first);
    });
  });

  it("writes nothing when persistence is off", async () => {
    await withCacheDir(async (cacheDir) => {
      const counter = { calls: 0 };
      await enrichProviderArticleEmbeds(
        TAG,
        { qiita: { cacheDir } },
        countingFetch({ title: "T" }, counter),
      );
      await expect(readdir(cacheDir)).rejects.toThrow();
    });
  });

  // A remembered miss must not become another request; a provider that is down
  // would otherwise be retried once per embed on every build.
  it("remembers that a lookup found nothing", async () => {
    await withCacheDir(async (cacheDir) => {
      const counter = { calls: 0 };
      const missing = (async () => {
        counter.calls += 1;
        return { ok: false, status: 404, json: async () => ({}) } as Response;
      }) as unknown as typeof fetch;
      const options = { persistCache: true, cacheDir };

      await enrichProviderArticleEmbeds(TAG, { qiita: options }, missing);
      expect(counter.calls).toBe(1);
      clearProviderArticleCache();
      await enrichProviderArticleEmbeds(TAG, { qiita: options }, missing);
      expect(counter.calls).toBe(1);
    });
  });

  it("re-fetches instead of failing when an entry is corrupt", async () => {
    await withCacheDir(async (cacheDir) => {
      const counter = { calls: 0 };
      const options = { persistCache: true, cacheDir };
      const body = { title: "Static cards" };

      await enrichProviderArticleEmbeds(TAG, { qiita: options }, countingFetch(body, counter));
      const [file] = await readdir(cacheDir);
      await writeFile(path.join(cacheDir, file!), "{ not json");
      clearProviderArticleCache();

      const html = await enrichProviderArticleEmbeds(
        TAG,
        { qiita: options },
        countingFetch(body, counter),
      );
      expect(counter.calls).toBe(2);
      expect(html).toContain("Static cards");
    });
  });

  it("keys a cache file by hash, so a URL cannot escape the directory", () => {
    expect(providerCacheFileName("qiita:https://qiita.com/../../etc/passwd")).toMatch(
      /^[0-9a-f]{64}\.json$/,
    );
  });
});
