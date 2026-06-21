import { describe, expect, it } from "vite-plus/test";
import { transformMarkdownWithReact } from "./transform";
import type { ResolvedReactOptions } from "./types";

describe("transformMarkdownWithReact", () => {
  it("turns registered components into islands and leaves fenced tags literal", async () => {
    const result = await transformMarkdownWithReact(
      [
        "---",
        "title: React Guide",
        "draft: false",
        "---",
        "# React Guide",
        "",
        '<Alert tone="info" active>Read **docs**.</Alert>',
        "",
        "```tsx",
        '<Alert tone="code" />',
        "```",
      ].join("\n"),
      "/repo/docs/react.md",
      createOptions(),
    );

    expect(result.frontmatter).toEqual({ title: "React Guide", draft: false });
    expect(result.usedComponents).toEqual(["Alert"]);
    expect(result.code).toMatchSnapshot();
  });

  it("uses the static html path when no registered component is present", async () => {
    const result = await transformMarkdownWithReact(
      "# Plain\n\n<Unknown />",
      "/repo/docs/plain.md",
      createOptions(),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).toMatchSnapshot();
  });

  it("honors disabled built-in embeds from framework options", async () => {
    const result = await transformMarkdownWithReact(
      '<GitHub repo="ubugeeei-prod/ox-content"></GitHub>',
      "/repo/docs/embed.md",
      createOptions({ embeds: { github: false, openGraph: false } }),
    );

    expect(result.code).toMatchSnapshot();
  });
});

function createOptions(overrides: Partial<ResolvedReactOptions> = {}): ResolvedReactOptions {
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
    components: { Alert: "./src/components/Alert.tsx" },
    jsxRuntime: "automatic",
    embeds: { github: false, openGraph: false },
    root: "/repo",
    ...overrides,
  };
}
