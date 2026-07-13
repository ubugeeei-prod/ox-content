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

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

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
    const devServer = {
      middlewares: { use: (handler: (typeof middlewares)[number]) => middlewares.push(handler) },
      watcher: { on: () => {} },
    };
    (search?.configureServer as (server: unknown) => void)(devServer);
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
    const index = JSON.parse(hit.body ?? "") as { documents: Array<{ id: string }> };
    expect(index.documents.some((doc) => doc.id === "intro")).toBe(true);
  });
});
