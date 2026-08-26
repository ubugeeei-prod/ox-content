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

  it("discovers nested, expression, and fragment islands from the MDX AST", async () => {
    const nested = await transformMarkdownWithSvelte(
      '<Callout>\n\n# Title\n\n<Badge title="hi" />\n\n</Callout>\n',
      "/repo/docs/nested.mdx",
      createOptions({
        components: {
          Alert: "./src/components/Alert.svelte",
          Callout: "./src/components/Callout.svelte",
          Badge: "./src/components/Badge.svelte",
        },
      }),
    );
    expect(nested.usedComponents).toEqual(["Callout", "Badge"]);
    expect(nested.code).toContain("Callout");
    expect(nested.code).toContain("Badge");
    expect(nested.code).toContain("initIslands");
    expect(nested.code).toMatchSnapshot();

    const expr = await transformMarkdownWithSvelte(
      "<Alert title={foo} count={count + 1} />\n",
      "/repo/docs/expr.mdx",
      createOptions(),
    );
    expect(expr.usedComponents).toEqual(["Alert"]);
    expect(expr.code).toContain("Alert");
    expect(expr.code).toMatchSnapshot();

    const fragment = await transformMarkdownWithSvelte(
      '<>\n<Alert tone="info" />\n</>\n',
      "/repo/docs/fragment.mdx",
      createOptions(),
    );
    expect(fragment.usedComponents).toEqual(["Alert"]);
    expect(fragment.code).toMatchSnapshot();
  });

  it("keeps fenced JSX literal and skips unregistered MDX components", async () => {
    const fenced = await transformMarkdownWithSvelte(
      [
        "# Guide",
        "",
        '<Alert tone="info" />',
        "",
        "```svelte",
        '<Alert tone="code" />',
        "```",
      ].join("\n"),
      "/repo/docs/fence.mdx",
      createOptions(),
    );
    expect(fenced.usedComponents).toEqual(["Alert"]);

    const mixed = await transformMarkdownWithSvelte(
      "# Plain\n\n<Alert />\n\n<Unknown />\n",
      "/repo/docs/mixed.mdx",
      createOptions(),
    );
    expect(mixed.usedComponents).toEqual(["Alert"]);
    expect(mixed.code).not.toContain("import Unknown");

    const unknownOnly = await transformMarkdownWithSvelte(
      "# Plain\n\n<Unknown />\n",
      "/repo/docs/unknown.mdx",
      createOptions(),
    );
    expect(unknownOnly.usedComponents).toEqual([]);
    expect(unknownOnly.code).not.toContain("initIslands");
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
