import { afterEach, describe, expect, it } from "vite-plus/test";
import { oxContent } from "../index";
import { generateSearchModule, resolveSearchOptions } from "../search";
import type { SearchOptions } from "../types";

const INDEX_PATH = "/docs/search-index.json";
const SECRET_KEY = "SECRET_WRITE_OR_ADMIN_KEY";
const HOSTILE_APP_ID = `"><img src=x onerror=alert(1)></script>`;
const HOSTILE_INDEX = `</div><script>alert(1)</script>`;

const restore: Array<() => void> = [];

afterEach(() => {
  while (restore.length > 0) restore.pop()?.();
});

async function loadModule(code: string) {
  return import(`data:text/javascript;charset=utf-8,${encodeURIComponent(code)}`);
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

describe("hosted search module safety", () => {
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
    expect(runtime.createSearchUiState("anything", [], { loading: true })).toMatchObject({
      status: "loading",
      ariaLiveMessage: "Loading search results",
    });
    expect(runtime.formatSearchResultForCard({ id: "fallback", title: "Fallback" })).toMatchObject({
      id: "ox-search-result-fallback",
      role: "option",
      ariaLabel: "Fallback",
    });
    expect(JSON.stringify(warnings)).not.toContain(SECRET_KEY);
  });

  it("escapes hostile appId and indexName and never names a write key", () => {
    const code = generateSearchModule(
      resolveSearchOptions({
        provider: "hosted",
        appId: HOSTILE_APP_ID,
        indexName: HOSTILE_INDEX,
        searchKey: "public-search-key",
      }),
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
