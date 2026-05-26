import { describe, expect, it } from "vite-plus/test";
import * as publicApi from "./index";

describe("public export surface", () => {
  it("keeps compatibility exports available from the package entrypoint", () => {
    const guardedExports = [
      "DEFAULT_HTML_TEMPLATE",
      "DEFAULT_MARKDOWN_EXTENSIONS",
      "buildSearchIndex",
      "buildSsg",
      "extractDocs",
      "generateMarkdown",
      "isMarkdownFilePath",
      "oxContent",
      "resolveDocsOptions",
      "resolveI18nOptions",
      "resolveOgImageOptions",
      "resolveSearchOptions",
      "resolveSsgOptions",
      "transformMarkdown",
      "writeDocs",
      "writeSearchIndex",
    ].sort();

    const actual = Object.keys(publicApi)
      .filter((key) => guardedExports.includes(key))
      .sort();

    expect(actual).toEqual(guardedExports);
  });
});
