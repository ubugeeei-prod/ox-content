import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const runtime = readFileSync(join(packageRoot, "../../crates/ox_content_ssg/src/ssg.js"), "utf8");

interface SearchResult {
  url: string;
  title: string;
  scopes?: string[];
  snippet?: string;
}

type RenderResults = (
  elements: { searchResults: { innerHTML: string } },
  state: { results: SearchResult[]; selectedIdx: number },
) => void;

function loadRenderResults(): RenderResults {
  const start = runtime.indexOf("const OX_CONTENT_SEARCH_HTML_ENTITIES");
  const end = runtime.indexOf("const scoreOxContentSearchTerms");
  expect(start).toBeGreaterThan(-1);
  expect(end).toBeGreaterThan(start);
  const factory = new Function(
    `${runtime.slice(start, end)}\nreturn renderOxContentSearchResults;`,
  ) as () => RenderResults;
  return factory();
}

describe("SSG search result rendering", () => {
  it("renders index titles, snippets, and scopes as text", () => {
    const render = loadRenderResults();
    const elements = { searchResults: { innerHTML: "" } };
    render(elements, {
      selectedIdx: 0,
      results: [
        {
          url: '/guide/"x',
          title: "Guide <b>bold</b>",
          scopes: ["api<b>"],
          snippet: "Use `<img src=x>` & more",
        },
      ],
    });

    const html = elements.searchResults.innerHTML;
    expect(html).toBe(
      '<a href="/guide/&quot;x" class="search-result selected">' +
        '<div class="search-result-title">Guide &lt;b&gt;bold&lt;/b&gt;' +
        '<span class="search-result-scope">@api&lt;b&gt;</span></div>' +
        '<div class="search-result-snippet">Use `&lt;img src=x&gt;` &amp; more</div></a>',
    );
  });
});
