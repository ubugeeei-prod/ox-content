import { describe, expect, it } from "vite-plus/test";
import {
  generateSearchModule,
  getSearchDocumentScopes,
  matchesSearchScopes,
  parseScopedSearchQuery,
  resolveSearchOptions,
} from "./search";

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

    expect(mod).toContain("const searchOptions =");
    expect(mod).toContain('fetch("/docs/search-index.json")');
    expect(mod).toContain("export async function search");
  });
});
