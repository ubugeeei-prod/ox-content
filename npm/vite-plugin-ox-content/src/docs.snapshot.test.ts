import { describe, expect, it } from "vite-plus/test";
import { generateMarkdown, resolveDocsOptions } from "./docs";
import { transformMarkdown } from "./transform";
import { createDocsFixture, createDocsResolvedOptions } from "../test/fixtures/docs-fixture";

describe("docs generation snapshots", () => {
  it("matches the generated module and index markdown", () => {
    const markdown = generateMarkdown(
      createDocsFixture(),
      resolveDocsOptions({
        githubUrl: "https://github.com/acme/ox-content",
      })!,
    );

    expect({
      index: markdown["index.md"],
      math: markdown["math.md"],
      utils: markdown["utils.md"],
    }).toMatchSnapshot();
  });

  it("keeps generated accordion blocks as HTML through markdown transform", async () => {
    const markdown = generateMarkdown(
      createDocsFixture(),
      resolveDocsOptions({
        githubUrl: "https://github.com/acme/ox-content",
      })!,
    );

    const result = await transformMarkdown(
      markdown["utils.md"]!,
      "docs/utils.md",
      createDocsResolvedOptions(),
    );

    expect(result.html).toContain('<details id="capitalize" class="ox-api-entry">');
    expect(result.html).not.toContain("&#x3C;details");
    expect(result.html).toContain("shiki-inline");
  });
});
