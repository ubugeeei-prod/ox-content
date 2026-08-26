import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import { generateMarkdown, resolveDocsOptions } from "./docs";
import { transformMarkdown } from "./transform";
import { createDocsFixture, createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import type { ResolvedOptions } from "./types";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

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

    expect(result.html).toMatchSnapshot();
  });

  it("snapshots rendered authoring examples from docs content", async () => {
    const base = createDocsResolvedOptions({ highlight: false });
    const withStackBlitz = {
      ...base,
      embeds: { ...base.embeds, stackBlitz: true },
    };
    const withAnnotations = {
      ...base,
      highlight: true,
      codeAnnotations: {
        ...base.codeAnnotations,
        enabled: true,
        notation: "both" as const,
      },
    };

    const [stackBlitz, codeAnnotations] = await Promise.all([
      renderDocsContent("docs/content/examples/stackblitz-embed.md", withStackBlitz),
      renderDocsContent("docs/content/examples/code-annotations.md", withAnnotations),
    ]);

    expect({ stackBlitz, codeAnnotations }).toMatchSnapshot();
  });
});

async function renderDocsContent(relativePath: string, options: ResolvedOptions): Promise<string> {
  const absolutePath = resolve(repoRoot, relativePath);
  const source = readFileSync(absolutePath, "utf8");
  const result = await transformMarkdown(source, absolutePath, options);
  return result.html;
}
