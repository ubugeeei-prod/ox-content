import { describe, expect, it } from "vite-plus/test";
import { transformMarkdownWithReact } from "./transform";
import type { ResolvedReactOptions } from "./types";

describe("document-local React islands", () => {
  it("hydrates a relative default import and emits a static import of that file", async () => {
    const result = await transformMarkdownWithReact(
      "import GtvChart from './gtv-chart/GtvChart.tsx'\n\n<GtvChart title=\"ok\" />\n",
      "/repo/docs/guide.mdx",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual(["GtvChart"]);
    expect(result.code).toContain("import GtvChart from './gtv-chart/GtvChart.tsx'");
    expect(result.code).toContain('data-ox-island=\\"GtvChart\\"');
    expect(result.code).toContain("ok");
  });

  it("keeps the same local name on two documents as distinct imports", async () => {
    const first = await transformMarkdownWithReact(
      "import Chart from './a/Chart.tsx'\n\n<Chart />\n",
      "/repo/docs/a.mdx",
      createOptions({ components: {} }),
    );
    const second = await transformMarkdownWithReact(
      "import Chart from './b/Chart.tsx'\n\n<Chart />\n",
      "/repo/docs/b.mdx",
      createOptions({ components: {} }),
    );

    expect(first.code).toContain("import Chart from './a/Chart.tsx'");
    expect(second.code).toContain("import Chart from './b/Chart.tsx'");
  });

  it("rejects an import that escapes the content root", async () => {
    const result = await transformMarkdownWithReact(
      "import Outside from '../../Outside.tsx'\n\n<Outside />\n",
      "/repo/docs/nested/page.mdx",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).not.toContain("import Outside");
  });

  it("still hydrates the global map and leaves plain .md unchanged", async () => {
    const globalMap = await transformMarkdownWithReact(
      '<Alert tone="info" />\n',
      "/repo/docs/guide.mdx",
      createOptions(),
    );
    expect(globalMap.code).toContain("import Alert from '../src/components/Alert.tsx'");

    const plain = await transformMarkdownWithReact(
      "import Chart from './Chart.tsx'\n\n<Chart />\n",
      "/repo/docs/page.md",
      createOptions({ components: {} }),
    );
    expect(plain.usedComponents).toEqual([]);
    expect(plain.code).not.toContain("import Chart from './Chart.tsx'");
  });
});

function createOptions(overrides: Partial<ResolvedReactOptions> = {}): ResolvedReactOptions {
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
    jsxRuntime: "automatic",
    embeds: { github: false, openGraph: false },
    root: "/repo",
    ...overrides,
  };
}
