import { describe, expect, it } from "vite-plus/test";
import { transformMarkdownWithSvelte } from "./transform";
import type { ResolvedSvelteOptions } from "./types";

describe("transformMarkdownWithSvelte", () => {
  it("turns registered components into islands and leaves fenced tags literal", async () => {
    const result = await transformMarkdownWithSvelte(
      [
        "---",
        "title: Svelte Guide",
        "draft: false",
        "---",
        "# Svelte Guide",
        "",
        '<Alert tone="info" active>Read **docs**.</Alert>',
        "",
        "```svelte",
        '<Alert tone="code" />',
        "```",
      ].join("\n"),
      "/repo/docs/svelte.md",
      createOptions(),
    );

    expect(result.frontmatter).toEqual({ title: "Svelte Guide", draft: false });
    expect(result.usedComponents).toEqual(["Alert"]);
    expect(result.code).toMatchSnapshot();
  });

  it("uses the static html path when no registered component is present", async () => {
    const result = await transformMarkdownWithSvelte(
      "# Plain\n\n<Unknown />",
      "/repo/docs/plain.md",
      createOptions(),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).toMatchSnapshot();
  });

  it("honors disabled built-in embeds from framework options", async () => {
    const result = await transformMarkdownWithSvelte(
      '<GitHub repo="ubugeeei-prod/ox-content"></GitHub>',
      "/repo/docs/embed.md",
      createOptions({ embeds: { github: false, openGraph: false } }),
    );

    expect(result.code).toMatchSnapshot();
  });
});

function createOptions(overrides: Partial<ResolvedSvelteOptions> = {}): ResolvedSvelteOptions {
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
    components: { Alert: "./src/components/Alert.svelte" },
    runes: true,
    embeds: { github: false, openGraph: false },
    root: "/repo",
    ...overrides,
  };
}
