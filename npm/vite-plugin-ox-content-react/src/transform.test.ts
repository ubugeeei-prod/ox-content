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

  it("discovers nested, expression, and fragment islands from the MDX AST", async () => {
    const nested = await transformMarkdownWithReact(
      '<Callout>\n\n# Title\n\n<Badge title="hi" />\n\n</Callout>\n',
      "/repo/docs/nested.mdx",
      createOptions({
        components: {
          Alert: "./src/components/Alert.tsx",
          Callout: "./src/components/Callout.tsx",
          Badge: "./src/components/Badge.tsx",
        },
      }),
    );
    expect(nested.usedComponents).toEqual(["Callout", "Badge"]);
    expect(nested.code).toContain("import Callout from '../src/components/Callout.tsx'");
    expect(nested.code).toContain("import Badge from '../src/components/Badge.tsx'");
    expect(nested.code).toContain('data-ox-island=\\"Callout\\"');
    expect(nested.code).toContain('data-ox-island=\\"Badge\\"');
    expect(nested.code).toContain("readIslandSlotHtml");

    const expr = await transformMarkdownWithReact(
      "<Alert title={foo} count={count + 1} />\n",
      "/repo/docs/expr.mdx",
      createOptions(),
    );
    expect(expr.usedComponents).toEqual(["Alert"]);
    expect(expr.code).toContain('data-ox-island=\\"Alert\\"');
    expect(expr.code).toMatch(/count \+ 1|count \\u002b 1/);

    const fragment = await transformMarkdownWithReact(
      '<>\n<Alert tone="info" />\n</>\n',
      "/repo/docs/fragment.mdx",
      createOptions(),
    );
    expect(fragment.usedComponents).toEqual(["Alert"]);
    expect(fragment.code).toContain('data-ox-island=\\"Alert\\"');
  });

  it("keeps fenced JSX literal and skips unregistered MDX components", async () => {
    const fenced = await transformMarkdownWithReact(
      ["# Guide", "", '<Alert tone="info" />', "", "```tsx", '<Alert tone="code" />', "```"].join(
        "\n",
      ),
      "/repo/docs/fence.mdx",
      createOptions(),
    );
    expect(fenced.usedComponents).toEqual(["Alert"]);
    expect(fenced.code.match(/data-ox-island=\\"Alert\\"/g)?.length).toBe(1);
    expect(fenced.code).toContain("&lt;Alert");

    const mixed = await transformMarkdownWithReact(
      "# Plain\n\n<Alert />\n\n<Unknown />\n",
      "/repo/docs/mixed.mdx",
      createOptions(),
    );
    expect(mixed.usedComponents).toEqual(["Alert"]);
    expect(mixed.code).toContain("import Alert from '../src/components/Alert.tsx'");
    expect(mixed.code).not.toContain("import Unknown");

    const unknownOnly = await transformMarkdownWithReact(
      "# Plain\n\n<Unknown />\n",
      "/repo/docs/unknown.mdx",
      createOptions(),
    );
    expect(unknownOnly.usedComponents).toEqual([]);
    expect(unknownOnly.code).not.toContain("initIslands");
    expect(unknownOnly.code).toContain('data-ox-island=\\"Unknown\\"');
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
