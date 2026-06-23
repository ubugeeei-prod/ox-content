import { describe, expect, it } from "vite-plus/test";
import * as publicApi from "./index";

describe("public export surface", () => {
  it("keeps compatibility exports available from the package entrypoint", () => {
    const guardedExports = [
      "DEFAULT_HTML_TEMPLATE",
      "DEFAULT_MARKDOWN_EXTENSIONS",
      "DocsTestRunError",
      "IncrementalMarkdownParser",
      "IncrementalMarkdownRenderer",
      "buildSearchIndex",
      "buildCollectionManifest",
      "buildSsg",
      "collectDocsTests",
      "createFrameworkMarkdownOptions",
      "createIncrementalMarkdownParser",
      "createIncrementalMarkdownRenderer",
      "defineCollection",
      "defineCollections",
      "extractDocs",
      "extractDocsTests",
      "generateCollectionsVirtualModule",
      "generateMarkdown",
      "isMarkdownFilePath",
      "oxContent",
      "resolveCollectionsOptions",
      "renderHtmlToReactCreateElement",
      "renderHtmlToVueH",
      "renderMarkdownStream",
      "resolveDocsOptions",
      "resolveI18nOptions",
      "resolveOgImageOptions",
      "resolveSearchOptions",
      "resolveSsgOptions",
      "runDocsTests",
      "transformMarkdown",
      "writeDocs",
      "writeDocsTestFiles",
      "writeSearchIndex",
    ].sort();

    const actual = Object.keys(publicApi)
      .filter((key) => guardedExports.includes(key))
      .sort();

    expect(actual).toEqual(guardedExports);
  });
});
