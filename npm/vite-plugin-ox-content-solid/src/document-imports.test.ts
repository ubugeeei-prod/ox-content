import { describe, expect, it } from "vite-plus/test";
import { transformMarkdownWithSolid } from "./transform";
import type { ResolvedSolidOptions } from "./types";

describe("document-local Solid islands", () => {
  it("hydrates a relative default import and emits a static import of that file", async () => {
    const result = await transformMarkdownWithSolid(
      "import GtvChart from './gtv-chart/GtvChart.tsx'\n\n<GtvChart title=\"ok\" />\n",
      "/repo/docs/guide.mdx",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual(["GtvChart"]);
    expect(result.code).toContain("import GtvChart from './gtv-chart/GtvChart.tsx'");
    expect(result.code).toContain('data-ox-island=\\"GtvChart\\"');
    expect(result.code).toContain("ok");
    expect(result.code).toContain("initIslands");
  });

  it("keeps the same local name on two documents as distinct imports", async () => {
    const first = await transformMarkdownWithSolid(
      "import Chart from './a/Chart.tsx'\n\n<Chart />\n",
      "/repo/docs/a.mdx",
      createOptions({ components: {} }),
    );
    const second = await transformMarkdownWithSolid(
      "import Chart from './b/Chart.tsx'\n\n<Chart />\n",
      "/repo/docs/b.mdx",
      createOptions({ components: {} }),
    );

    expect(first.code).toContain("import Chart from './a/Chart.tsx'");
    expect(second.code).toContain("import Chart from './b/Chart.tsx'");
    expect(first.code).not.toContain("./b/Chart.tsx");
    expect(second.code).not.toContain("./a/Chart.tsx");
  });

  it("rejects an import that escapes the content root", async () => {
    const result = await transformMarkdownWithSolid(
      "import Outside from '../../Outside.tsx'\n\n<Outside />\n",
      "/repo/docs/nested/page.mdx",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).not.toContain("import Outside");
    expect(result.code).not.toContain("initIslands");
  });

  it("resolves a nested relative path", async () => {
    const result = await transformMarkdownWithSolid(
      "import Foo from './widgets/Foo.tsx'\n\n<Foo />\n",
      "/repo/docs/guide.mdx",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual(["Foo"]);
    expect(result.code).toContain("import Foo from './widgets/Foo.tsx'");
  });

  it("keeps serialized literal props on the island payload", async () => {
    const result = await transformMarkdownWithSolid(
      "import GtvChart from './gtv-chart/GtvChart.tsx'\n\n<GtvChart title=\"ok\" count={3} />\n",
      "/repo/docs/guide.mdx",
      createOptions({ components: {} }),
    );

    expect(result.code).toContain("data-ox-island");
    expect(result.code).toContain("title");
    expect(result.code).toContain("ok");
    expect(result.code).toMatch(/data-ox-props|application\\\/json/);
  });

  it("still hydrates the global components map when the document has no import", async () => {
    const result = await transformMarkdownWithSolid(
      '<Alert tone="info" />\n',
      "/repo/docs/guide.mdx",
      createOptions(),
    );

    expect(result.usedComponents).toEqual(["Alert"]);
    expect(result.code).toContain("import Alert from '../src/components/Alert.tsx'");
  });

  it("lets a document-local binding override a global name", async () => {
    const result = await transformMarkdownWithSolid(
      "import Alert from './local/Alert.tsx'\n\n<Alert />\n",
      "/repo/docs/guide.mdx",
      createOptions(),
    );

    expect(result.code).toContain("import Alert from './local/Alert.tsx'");
    expect(result.code).not.toContain("../src/components/Alert.tsx");
  });

  it("leaves unregistered JSX without a matching import static", async () => {
    const result = await transformMarkdownWithSolid(
      "# Plain\n\n<Unknown />\n",
      "/repo/docs/unknown.mdx",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).not.toContain("import Unknown");
    expect(result.code).not.toContain("initIslands");
    expect(result.code).toContain('data-ox-island=\\"Unknown\\"');
  });

  it("does not create document-import islands from ESM in plain .md", async () => {
    const result = await transformMarkdownWithSolid(
      "import Chart from './Chart.tsx'\n\n<Chart />\n",
      "/repo/docs/page.md",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).not.toContain("import Chart from './Chart.tsx'");
    expect(result.code).not.toContain("initIslands");
  });

  it("applies an optional SSR hook without a framework SSR runtime in core", async () => {
    const result = await transformMarkdownWithSolid(
      "import GtvChart from './gtv-chart/GtvChart.tsx'\n\n<GtvChart title=\"ok\" />\n",
      "/repo/docs/guide.mdx",
      createOptions({
        components: {},
        renderIsland: (name, props) => `<span class="ssr">${name}:${String(props.title)}</span>`,
      }),
    );

    expect(result.code).toContain("GtvChart:ok");
    expect(result.code).toContain("import GtvChart from './gtv-chart/GtvChart.tsx'");
    expect(result.code).not.toContain("svelte/server");
    expect(result.code).not.toContain("react-dom/server");
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
