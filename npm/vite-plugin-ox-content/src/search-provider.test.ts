import { afterEach, describe, expect, it } from "vite-plus/test";
import { oxContent } from "./index";
import { generateSearchModule, resolveSearchOptions, writeSearchIndex } from "./search";
import type { SearchOptions } from "./types";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";

const INDEX_PATH = "/docs/search-index.json";
const PUBLIC_KEY = "pub_search_only_key";
const SECRET_KEY = "SECRET_WRITE_OR_ADMIN_KEY";
const HOSTILE_APP_ID = `"><img src=x onerror=alert(1)></script>`;
const HOSTILE_INDEX = `</div><script>alert(1)</script>`;

const tempDirs: string[] = [];
const restore: Array<() => void> = [];

afterEach(async () => {
  while (restore.length > 0) restore.pop()?.();
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

function hostedOptions(overrides: SearchOptions = {}): SearchOptions {
  return {
    provider: "hosted",
    appId: "docs-app",
    indexName: "site-index",
    searchKey: PUBLIC_KEY,
    endpoint: "https://search.example.test/query",
    ...overrides,
  };
}

function withForbiddenKey(field: "adminKey" | "writeKey" | "apiKey"): SearchOptions {
  return { ...hostedOptions(), [field]: SECRET_KEY } as SearchOptions;
}

function captureWarn(): unknown[][] {
  const messages: unknown[][] = [];
  const original = console.warn;
  console.warn = (...args: unknown[]) => {
    messages.push(args);
  };
  restore.push(() => {
    console.warn = original;
  });
  return messages;
}

async function loadModule(code: string) {
  return import(`data:text/javascript;charset=utf-8,${encodeURIComponent(code)}`);
}

function mockFetch(handler: (input: unknown, init?: RequestInit) => unknown) {
  const calls: Array<{ input: unknown; init?: RequestInit }> = [];
  const original = globalThis.fetch;
  globalThis.fetch = (async (input: unknown, init?: RequestInit) => {
    calls.push({ input, init });
    return handler(input, init);
  }) as typeof fetch;
  restore.push(() => {
    globalThis.fetch = original;
  });
  return calls;
}

describe("resolveSearchOptions provider", () => {
  it("defaults omitted options and provider: local to the BM25 backend", () => {
    expect(resolveSearchOptions(undefined).provider).toBe("local");
    expect(resolveSearchOptions(true).provider).toBe("local");
    expect(resolveSearchOptions({}).provider).toBe("local");
    expect(resolveSearchOptions({ provider: "local" }).provider).toBe("local");
    expect(resolveSearchOptions({ provider: "local" }).enabled).toBe(true);
    expect(resolveSearchOptions({ provider: "local" }).searchKey).toBeUndefined();
  });

  it("keeps search: false disabled", () => {
    expect(resolveSearchOptions(false)).toMatchObject({
      enabled: false,
      provider: "local",
      placeholder: "Search documentation...",
      hotkey: "/",
    });
    expect(
      resolveSearchOptions({ enabled: false, provider: "hosted", searchKey: PUBLIC_KEY }),
    ).toMatchObject({
      enabled: false,
    });
    expect(
      resolveSearchOptions({ enabled: false, provider: "hosted", searchKey: PUBLIC_KEY }).searchKey,
    ).toBeUndefined();
  });

  it("wires hosted credentials from config or env", () => {
    const fromConfig = resolveSearchOptions(hostedOptions());
    expect(fromConfig.provider).toBe("hosted");
    expect(fromConfig.appId).toBe("docs-app");
    expect(fromConfig.indexName).toBe("site-index");
    expect(fromConfig.searchKey).toBe(PUBLIC_KEY);
    expect(fromConfig.endpoint).toBe("https://search.example.test/query");

    const envKeys = [
      "OX_CONTENT_SEARCH_APP_ID",
      "OX_CONTENT_SEARCH_INDEX_NAME",
      "OX_CONTENT_SEARCH_KEY",
      "OX_CONTENT_SEARCH_ENDPOINT",
    ] as const;
    const previous = Object.fromEntries(envKeys.map((key) => [key, process.env[key]]));
    restore.push(() => {
      for (const key of envKeys) {
        if (previous[key] === undefined) delete process.env[key];
        else process.env[key] = previous[key];
      }
    });
    process.env.OX_CONTENT_SEARCH_APP_ID = "env-app";
    process.env.OX_CONTENT_SEARCH_INDEX_NAME = "env-index";
    process.env.OX_CONTENT_SEARCH_KEY = "env-search-key";
    process.env.OX_CONTENT_SEARCH_ENDPOINT = "https://search.example.test/env";
    const fromEnv = resolveSearchOptions({ provider: "hosted" });
    expect(fromEnv).toMatchObject({
      provider: "hosted",
      appId: "env-app",
      indexName: "env-index",
      searchKey: "env-search-key",
      endpoint: "https://search.example.test/env",
    });

    const fromPublicKey = resolveSearchOptions({
      provider: "hosted",
      appId: "docs-app",
      indexName: "site-index",
      publicKey: "alias-public-key",
    });
    expect(fromPublicKey.searchKey).toBe("alias-public-key");
  });

  it("fails closed when hosted fields are missing and never logs secrets", () => {
    const warnings = captureWarn();
    const missing = [
      resolveSearchOptions({ provider: "hosted", searchKey: SECRET_KEY }),
      resolveSearchOptions({ provider: "hosted", appId: "docs-app", searchKey: SECRET_KEY }),
      resolveSearchOptions({ provider: "hosted", appId: "docs-app", indexName: "site-index" }),
    ];

    for (const resolved of missing) {
      expect(resolved.provider).toBe("hosted");
      expect(resolved.appId).toBeUndefined();
      expect(resolved.searchKey).toBeUndefined();
      expect(JSON.stringify(resolved)).not.toContain(SECRET_KEY);
    }
    expect(JSON.stringify(warnings)).not.toContain(SECRET_KEY);
  });

  it("rejects adminKey, writeKey, and apiKey even when a public key is also set", () => {
    const warnings = captureWarn();
    for (const field of ["adminKey", "writeKey", "apiKey"] as const) {
      const resolved = resolveSearchOptions(withForbiddenKey(field));
      expect(resolved.provider).toBe("hosted");
      expect(resolved.searchKey).toBeUndefined();
      expect(resolved.appId).toBeUndefined();
      expect(JSON.stringify(resolved)).not.toContain(SECRET_KEY);
    }
    expect(JSON.stringify(warnings)).not.toContain(SECRET_KEY);
  });
});

describe("generateSearchModule hosted adapter", () => {
  it("keeps the local BM25 module when provider is omitted or local", () => {
    const omitted = generateSearchModule(resolveSearchOptions(true), INDEX_PATH);
    const local = generateSearchModule(resolveSearchOptions({ provider: "local" }), INDEX_PATH);
    expect(local).toBe(omitted);
    expect(omitted).toContain("search-index.json");
    expect(omitted).toContain("BM25");
    expect(omitted).toContain('"placeholder":"Search documentation..."');
    expect(omitted).toContain('"hotkey":"/"');
  });

  it("wires a hosted client without changing the local index writer", async () => {
    const resolved = resolveSearchOptions(
      hostedOptions({ placeholder: "Find a page", hotkey: "s" }),
    );
    const mod = generateSearchModule(resolved, INDEX_PATH);

    expect(mod).not.toContain("search-index.json");
    expect(mod).not.toContain("BM25");
    expect(mod).toContain("fetch(");
    expect(mod).toContain("https://search.example.test/query");
    expect(mod).toContain("searchKey");
    expect(mod).not.toMatch(/adminKey|writeKey|apiKey/);
    expect(mod).toContain("Find a page");
    expect(mod).toContain('"hotkey":"s"');

    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-search-hosted-"));
    tempDirs.push(outDir);
    await writeSearchIndex('{"doc_count":0,"documents":[]}', outDir);
    expect(await fs.readFile(path.join(outDir, "search-index.json"), "utf-8")).toBe(
      '{"doc_count":0,"documents":[]}',
    );
  });

  it("queries the hosted HTTP adapter and maps hits", async () => {
    const calls = mockFetch(() => ({
      ok: true,
      json: async () => ({
        hits: [{ id: "guide", title: "Guide", url: "/guide", score: 2, snippet: "hello" }],
      }),
    }));
    const runtime = await loadModule(
      generateSearchModule(resolveSearchOptions(hostedOptions()), INDEX_PATH),
    );
    const results = await runtime.search("guide", { limit: 3 });

    expect(runtime.searchOptions.placeholder).toBe("Search documentation...");
    expect(runtime.searchOptions.hotkey).toBe("/");
    expect(runtime.searchOptions.provider).toBe("hosted");
    expect(calls).toHaveLength(1);
    expect(calls[0]?.input).toBe("https://search.example.test/query");
    const headers = new Headers(calls[0]?.init?.headers);
    expect(headers.get("x-search-key")).toBe(PUBLIC_KEY);
    expect(headers.get("x-app-id")).toBe("docs-app");
    expect(headers.has("x-admin-key")).toBe(false);
    expect(headers.has("x-write-key")).toBe(false);
    expect(headers.has("x-api-key")).toBe(false);
    expect(calls[0]?.init?.body).toBe(
      JSON.stringify({ query: "guide", limit: 3, indexName: "site-index" }),
    );
    expect(results).toEqual([
      { id: "guide", title: "Guide", url: "/guide", score: 2, matches: [], snippet: "hello" },
    ]);
  });

  it("fails closed without calling a broken endpoint or leaking secrets", async () => {
    const warnings = captureWarn();
    const calls = mockFetch(() => {
      throw new Error("hosted adapter should not fetch");
    });
    const resolved = resolveSearchOptions({
      provider: "hosted",
      adminKey: SECRET_KEY,
    } as SearchOptions);
    const code = generateSearchModule(resolved, INDEX_PATH);
    const runtime = await loadModule(code);
    const results = await runtime.search("anything");

    expect(results).toEqual([]);
    expect(calls).toHaveLength(0);
    expect(code).not.toContain("fetch(");
    expect(code).not.toContain(SECRET_KEY);
    expect(code).not.toMatch(/adminKey|writeKey|apiKey/);
    expect(runtime.searchOptions.placeholder).toBe("Search documentation...");
    expect(runtime.searchOptions.hotkey).toBe("/");
    expect(JSON.stringify(warnings)).not.toContain(SECRET_KEY);
  });

  it("escapes hostile appId and indexName and never names a write key", () => {
    const code = generateSearchModule(
      resolveSearchOptions(
        hostedOptions({
          appId: HOSTILE_APP_ID,
          indexName: HOSTILE_INDEX,
        }),
      ),
      INDEX_PATH,
    );

    expect(code).not.toContain("<script>");
    expect(code).not.toContain("<img");
    expect(code).toContain("\\u003c");
    expect(code).toContain("searchKey");
    expect(code).not.toMatch(/adminKey|writeKey|apiKey/);
    expect(code).not.toContain("write key");
  });
});

describe("search plugin disable path", () => {
  it("still disables the virtual module when search is false", async () => {
    const plugins = oxContent({ search: false });
    const search = plugins.find((plugin) => plugin.name === "ox-content:search");
    const load = search?.load as (id: string) => Promise<string | null> | string | null;
    const code = await load.call(search, "\0virtual:ox-content/search");

    expect(code).toContain("enabled: false");
    expect(code).not.toContain("fetch(");
    expect(code).not.toContain("BM25");
  });
});
