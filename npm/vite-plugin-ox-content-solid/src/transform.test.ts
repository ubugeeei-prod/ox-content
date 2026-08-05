import { describe, expect, it } from "vite-plus/test";
import { transformMarkdownWithSolid } from "./transform";
import type { ResolvedSolidOptions } from "./types";

describe("transformMarkdownWithSolid", () => {
  it("turns registered components into islands and leaves fenced tags literal", async () => {
    const result = await transformMarkdownWithSolid(
      [
        "---",
        "title: Solid Guide",
        "draft: false",
        "---",
        "# Solid Guide",
        "",
        '<Alert tone="info" active>Read **docs**.</Alert>',
        "",
        "```tsx",
        '<Alert tone="code" />',
        "```",
      ].join("\n"),
      "/repo/docs/solid.md",
      createOptions(),
    );

    expect(result.frontmatter).toEqual({ title: "Solid Guide", draft: false });
    expect(result.usedComponents).toEqual(["Alert"]);
    expect(result.code).toMatchSnapshot();
  });

  it("uses the static html path when no registered component is present", async () => {
    const result = await transformMarkdownWithSolid(
      "# Plain\n\n<Unknown />",
      "/repo/docs/plain.md",
      createOptions(),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).toMatchSnapshot();
  });

  it("honors disabled built-in embeds from framework options", async () => {
    const result = await transformMarkdownWithSolid(
      '<GitHub repo="ubugeeei-prod/ox-content"></GitHub>',
      "/repo/docs/embed.md",
      createOptions({ embeds: { github: false, openGraph: false } }),
    );

    expect(result.code).toMatchSnapshot();
  });

  it("emits Solid JSX rather than a runtime element factory", async () => {
    const result = await transformMarkdownWithSolid(
      "# Static",
      "/repo/docs/static.md",
      createOptions(),
    );

    // Solid has no runtime JSX factory: the generated module stays JSX and is
    // compiled by vite-plugin-solid. The `class` (not `className`) attribute is
    // part of that contract.
    expect(result.code).toContain('<div class="ox-content" innerHTML={rawHtml} />');
    expect(result.code).not.toContain("createElement");
  });

  it("keeps the frontmatter block in the document when frontmatter parsing is off", async () => {
    const source = ["---", "title: Solid Guide", "---", "# Solid Guide"].join("\n");

    const result = await transformMarkdownWithSolid(
      source,
      "/repo/docs/no-frontmatter.md",
      createOptions({ frontmatter: false }),
    );

    expect(result.frontmatter).toEqual({});
    expect(result.code).toContain("export const frontmatter = {};");
    expect(result.code).toContain("title: Solid Guide");
  });

  it("mounts islands through solid-js/web render", async () => {
    const result = await transformMarkdownWithSolid(
      '<Alert tone="info">Body</Alert>',
      "/repo/docs/island.md",
      createOptions(),
    );

    expect(result.code).toContain(`import { render } from 'solid-js/web';`);
    expect(result.code).toContain(`import { onCleanup, onMount } from 'solid-js';`);
    expect(result.code).toContain("initIslands(createSolidHydrate()");
  });
});

function createOptions(overrides: Partial<ResolvedSolidOptions> = {}): ResolvedSolidOptions {
  return {
    srcDir: "docs",
    outDir: "dist",
    base: "/",
    extensions: [".md", ".markdown", ".mdx"],
    gfm: true,
    autolinks: true,
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    codeAnnotations: { enabled: false, metaKey: "annotate" },
    components: { Alert: "./src/components/Alert.tsx" },
    verifySolidPlugin: true,
    embeds: { github: false, openGraph: false },
    root: "/repo",
    ...overrides,
  };
}
