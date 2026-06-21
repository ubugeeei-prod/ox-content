import { describe, expect, it } from "vite-plus/test";
import { transformMarkdownWithVue } from "./transform";
import type { ResolvedVueOptions } from "./types";

describe("transformMarkdownWithVue", () => {
  it("turns registered components into islands and leaves fenced tags literal", async () => {
    const result = await transformMarkdownWithVue(
      [
        "---",
        "title: Vue Guide",
        "draft: false",
        "---",
        "# Vue Guide",
        "",
        '<Alert tone="info" active>Read **docs**.</Alert>',
        "",
        "```vue",
        '<Alert tone="code" />',
        "```",
      ].join("\n"),
      "/repo/docs/vue.md",
      createOptions(),
    );

    expect(result.frontmatter).toEqual({ title: "Vue Guide", draft: false });
    expect(result.usedComponents).toEqual(["Alert"]);
    expect(result.code).toMatchSnapshot();
  });

  it("uses the static html path when no registered component is present", async () => {
    const result = await transformMarkdownWithVue(
      "# Plain\n\n<Unknown />",
      "/repo/docs/plain.md",
      createOptions(),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).toMatchSnapshot();
  });

  it("honors disabled built-in embeds from framework options", async () => {
    const result = await transformMarkdownWithVue(
      '<GitHub repo="ubugeeei-prod/ox-content"></GitHub>',
      "/repo/docs/embed.md",
      createOptions({ embeds: { github: false, openGraph: false } }),
    );

    expect(result.code).toMatchSnapshot();
  });
});

function createOptions(
  overrides: Partial<
    Omit<ResolvedVueOptions, "components"> & { components: Map<string, string>; root: string }
  > = {},
): Omit<ResolvedVueOptions, "components"> & { components: Map<string, string>; root: string } {
  return {
    srcDir: "docs",
    outDir: "dist",
    base: "/",
    extensions: [".md", ".markdown", ".mdx"],
    gfm: true,
    frontmatter: true,
    toc: true,
    tocMaxDepth: 3,
    codeAnnotations: { enabled: false, metaKey: "annotate" },
    components: new Map([["Alert", "./src/components/Alert.vue"]]),
    reactivityTransform: false,
    customBlocks: true,
    embeds: { github: false, openGraph: false },
    root: "/repo",
    ...overrides,
  };
}
