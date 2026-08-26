import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { oxContent } from "./index";
import {
  buildSearchIndex,
  generateSearchModule,
  getSearchDocumentScopes,
  matchesSearchScopes,
  parseScopedSearchQuery,
  resolveSearchOptions,
  writeSearchIndex,
} from "./search";

const tempDirs: string[] = [];
const restore: Array<() => void> = [];

afterEach(async () => {
  while (restore.length > 0) restore.pop()?.();
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

async function loadModule(code: string) {
  return import(`data:text/javascript;charset=utf-8,${encodeURIComponent(code)}`);
}

function mockFetchJson(json: string) {
  const calls: string[] = [];
  const original = globalThis.fetch;
  globalThis.fetch = (async (input: unknown) => {
    calls.push(String(input));
    return { json: async () => JSON.parse(json) };
  }) as typeof fetch;
  restore.push(() => {
    globalThis.fetch = original;
  });
  return calls;
}

describe("parseScopedSearchQuery", () => {
  it("separates scope prefixes from free-text terms", () => {
    expect(parseScopedSearchQuery("@api some_function_name")).toEqual({
      text: "some_function_name",
      scopes: ["api"],
    });
  });

  it("deduplicates scopes and preserves plain text", () => {
    expect(parseScopedSearchQuery("@api @api clamp util")).toEqual({
      text: "clamp util",
      scopes: ["api"],
    });
  });
});

describe("search scopes", () => {
  it("derives cumulative scopes from document ids", () => {
    expect(getSearchDocumentScopes({ id: "api/math/index", url: "/api/math/index" })).toEqual([
      "api",
      "api/math",
    ]);
  });

  it("matches documents against requested scopes", () => {
    const doc = { id: "api/utils", url: "/api/utils" };

    expect(matchesSearchScopes(doc, ["api"])).toBe(true);
    expect(matchesSearchScopes(doc, ["api/utils"])).toBe(false);
    expect(matchesSearchScopes(doc, ["guides"])).toBe(false);
  });
});

describe("generateSearchModule", () => {
  it("generates the client runtime through the native binding", () => {
    const mod = generateSearchModule(resolveSearchOptions(true), "/docs/search-index.json");

    expect(mod).toMatchSnapshot();
  });

  it("supports opt-in fuzzy matching in the local runtime", async () => {
    const srcDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-search-fuzzy-"));
    tempDirs.push(srcDir);
    await fs.writeFile(
      path.join(srcDir, "install.md"),
      "# Installation\n\nInstall the package and configure the docs.",
      "utf-8",
    );
    const indexJson = await buildSearchIndex(srcDir, "/docs/");
    const calls = mockFetchJson(indexJson);
    const runtime = await loadModule(
      generateSearchModule(resolveSearchOptions({ fuzzy: true, prefix: false }), "/search.json"),
    );

    expect(runtime.searchOptions.fuzzy).toBe(true);
    const results = await runtime.search("isntall");

    expect(calls).toEqual(["/search.json"]);
    expect(results[0]?.id).toBe("install");
  });

  it("exposes rich query, ranking, and accessible card state in the local runtime", async () => {
    const srcDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-search-rich-"));
    tempDirs.push(srcDir);
    await fs.mkdir(path.join(srcDir, "2.90", "ja", "api"), { recursive: true });
    await fs.writeFile(
      path.join(srcDir, "2.90", "ja", "api", "search.md"),
      `---
title: Query Grammar
---
# Search API

The static index includes render pipelines for the CLI.
`,
      "utf-8",
    );
    await fs.writeFile(
      path.join(srcDir, "2.90", "ja", "api", "legacy.md"),
      `---
title: Legacy Search
---
# Search API

A static page may mention an index and renderer separately.
`,
      "utf-8",
    );

    const indexJson = await buildSearchIndex(srcDir, "/");
    mockFetchJson(indexJson);
    const runtime = await loadModule(
      generateSearchModule(resolveSearchOptions({ prefix: false }), "/search.json"),
    );
    const query = String.raw`@2.90/ja/api lang:ja version:2.90 "static index" render*`;

    expect(runtime.parseSearchQuery(query)).toMatchObject({
      text: "static index render",
      terms: [],
      phrases: ["static index"],
      prefixes: ["render"],
      filters: [
        { name: "locale", value: "ja" },
        { name: "version", value: "2.90" },
      ],
      scopes: ["2.90/ja/api"],
    });

    const results = await runtime.search(query, {
      localeCodes: ["ja"],
      defaultLocale: "en",
      versionPrefixes: ["2.90"],
    });

    expect(results).toHaveLength(2);
    expect(results[0]).toMatchObject({
      id: "2.90/ja/api/search",
      scopes: ["2.90", "2.90/ja", "2.90/ja/api"],
      metadata: {
        section: "Search API",
        locale: "ja",
        version: "2.90",
      },
    });
    expect(results[0].matches).toContain("static index");
    expect(results[0].ranking.fields).toContain("body");
    expect(results[0].ranking.reasons).toContain("body phrase match: static index");
    expect(results[0].ranking.reasons).toContain("scope filter: 2.90/ja/api");
    expect(results[0].ariaLabel).toContain("language ja");

    const ui = runtime.createSearchUiState(query, results, { activeIndex: 0 });
    expect(ui.status).toBe("results");
    expect(ui.refinements.map((item: { label: string }) => item.label)).toEqual([
      "@2.90/ja/api",
      "locale:ja",
      "version:2.90",
      '"static index"',
      "render*",
    ]);
    expect(ui.cards[0]).toMatchObject({
      role: "option",
      id: "ox-search-result-2-90-ja-api-search",
      title: "Query Grammar",
    });
    expect(ui.activeDescendant).toBe("ox-search-result-2-90-ja-api-search");
    expect(runtime.createSearchUiState("", []).status).toBe("empty");
    expect(runtime.createSearchUiState("query", [], { loading: true }).status).toBe("loading");
    expect(runtime.createSearchUiState("query", []).status).toBe("no-results");
    expect(runtime.createSearchUiState("検索", [], { isComposing: true }).status).toBe("composing");
  });

  it("keeps the generated local runtime under the rich search size budget", () => {
    const mod = generateSearchModule(resolveSearchOptions(true), "/docs/search-index.json");

    expect(Buffer.byteLength(mod, "utf8")).toBeLessThan(36_000);
  });
});

describe("buildSearchIndex", () => {
  it("builds the index from Markdown files through the native binding", async () => {
    const srcDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-search-"));
    tempDirs.push(srcDir);
    await fs.mkdir(path.join(srcDir, "guide"), { recursive: true });
    await fs.writeFile(
      path.join(srcDir, "guide", "intro.markdown"),
      `---
title: Native Search
---
# Ignored heading

Body text with a searchable phrase.
`,
      "utf-8",
    );

    const index = JSON.parse(await buildSearchIndex(srcDir, "/docs/")) as {
      doc_count: number;
      documents: Array<{ id: string; title: string; url: string; body: string }>;
    };

    expect(index).toMatchSnapshot();
  });
});

describe("writeSearchIndex", () => {
  it("writes the index through the native binding", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-search-out-"));
    tempDirs.push(outDir);

    await writeSearchIndex('{"doc_count":0}', outDir);

    expect(await fs.readFile(path.join(outDir, "search-index.json"), "utf-8")).toBe(
      '{"doc_count":0}',
    );
  });
});

describe("search dev server", () => {
  it("serves the search index from the dev server", async () => {
    const srcDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-search-dev-"));
    tempDirs.push(srcDir);
    await fs.writeFile(path.join(srcDir, "intro.md"), "# Intro\n\nSearchable body.\n", "utf-8");

    const plugins = oxContent({ srcDir, search: true });
    const search = plugins.find((plugin) => plugin.name === "ox-content:search");
    expect(search).toBeDefined();

    const middlewares: Array<
      (req: unknown, res: unknown, next: (err?: unknown) => void) => Promise<void> | void
    > = [];
    let watchAll: ((event: string, file: string) => void) | undefined;
    const devServer = {
      middlewares: { use: (handler: (typeof middlewares)[number]) => middlewares.push(handler) },
      watcher: {
        on: (event: string, handler: (event: string, file: string) => void) => {
          expect(event).toBe("all");
          watchAll = handler;
        },
      },
    };
    if (!search?.configureServer) {
      throw new Error("search plugin should expose configureServer");
    }
    (search.configureServer as (server: unknown) => void)(devServer);
    expect(middlewares).toHaveLength(1);

    const request = async (url: string) => {
      const headers: Record<string, string> = {};
      let body: string | undefined;
      let fellThrough = false;
      await middlewares[0](
        { url },
        {
          setHeader: (name: string, value: string) => {
            headers[name.toLowerCase()] = value;
          },
          end: (chunk: string) => {
            body = chunk;
          },
        },
        () => {
          fellThrough = true;
        },
      );
      return { headers, body, fellThrough };
    };

    const miss = await request("/other.json");
    expect(miss.fellThrough).toBe(true);

    const hit = await request("/search-index.json");
    expect(hit.fellThrough).toBe(false);
    expect(hit.headers["content-type"]).toContain("application/json");
    const index = JSON.parse(hit.body ?? "") as { documents: Array<{ body: string; id: string }> };
    expect(index.documents.some((doc) => doc.id === "intro")).toBe(true);

    await fs.writeFile(path.join(srcDir, "intro.md"), "# Intro\n\nUpdated body.\n", "utf-8");
    watchAll?.("change", path.join(`${srcDir}-legacy`, "intro.md"));
    const siblingChange = await request("/search-index.json");
    expect(siblingChange.body).toBe(hit.body);

    watchAll?.("change", path.join(srcDir, "intro.md"));
    const changed = await request("/search-index.json");
    const changedIndex = JSON.parse(changed.body ?? "") as {
      documents: Array<{ body: string; id: string }>;
    };
    expect(changedIndex.documents.some((doc) => doc.body.includes("Updated body"))).toBe(true);
  });
});
