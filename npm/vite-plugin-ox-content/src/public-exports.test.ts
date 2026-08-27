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
      "generateFeeds",
      "generateMarkdown",
      "enhanceMarkdownTables",
      "isMarkdownFilePath",
      "markdownTableScrollLabel",
      "oxContent",
      "planSsgOutputs",
      "createMarkdownProcessor",
      "renderMarkdown",
      "resolveGitLastmod",
      "resolveCollectionsOptions",
      "renderHtmlToReactCreateElement",
      "renderHtmlToVueH",
      "renderMarkdownStream",
      "resolveBudouxOptions",
      "resolveDocsOptions",
      "resolveI18nOptions",
      "resolveOgImageOptions",
      "resolveSearchOptions",
      "resolveNotFoundOptions",
      "resolvePublishStateOptions",
      "resolveRedirectsOptions",
      "resolveFeedsOptions",
      "resolvePwaOptions",
      "resolveTaxonomiesOptions",
      "resolveVersionsOptions",
      "resolveResourcesOptions",
      "resolveImageGalleryOptions",
      "resolveTimelineOptions",
      "resolveTeamOptions",
      "resolveSiteMapsOptions",
      "resolveMarkdownSourceOptions",
      "resolvePermalinksOptions",
      "resolveCascadeOptions",
      "resolveSsgOptions",
      "runDocsTests",
      "transformMarkdown",
      "transformBudouxHtml",
      "writeDocs",
      "writeDocsTestFiles",
      "writeFeedFiles",
      "writeMarkdownCompanions",
      "writeResourceFiles",
      "writeSearchIndex",
      "writeSiteMapFiles",
    ].sort();

    const actual = Object.keys(publicApi)
      .filter((key) => guardedExports.includes(key))
      .sort();

    expect(actual).toEqual(guardedExports);
  });
});
