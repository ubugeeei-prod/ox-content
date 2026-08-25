import { describe, expect, it } from "vite-plus/test";
import { transformMarkdownWithVue } from "./transform";
import type { ResolvedVueOptions } from "./types";

describe("document-local Vue islands", () => {
  it("hydrates a relative default import and emits a static import of that file", async () => {
    const result = await transformMarkdownWithVue(
      "import GtvChart from './gtv-chart/GtvChart.vue'\n\n<GtvChart title=\"ok\" />\n",
      "/repo/docs/guide.mdx",
      createOptions({ components: new Map() }),
    );

    expect(result.usedComponents).toEqual(["GtvChart"]);
    expect(result.code).toContain("import GtvChart from './gtv-chart/GtvChart.vue'");
    expect(result.code).toContain('data-ox-island=\\"GtvChart\\"');
    expect(result.code).toContain("ok");
  });

  it("keeps the same local name on two documents as distinct imports", async () => {
    const first = await transformMarkdownWithVue(
      "import Chart from './a/Chart.vue'\n\n<Chart />\n",
      "/repo/docs/a.mdx",
      createOptions({ components: new Map() }),
    );
    const second = await transformMarkdownWithVue(
      "import Chart from './b/Chart.vue'\n\n<Chart />\n",
      "/repo/docs/b.mdx",
      createOptions({ components: new Map() }),
    );

    expect(first.code).toContain("import Chart from './a/Chart.vue'");
    expect(second.code).toContain("import Chart from './b/Chart.vue'");
  });

  it("rejects an import that escapes the content root", async () => {
    const result = await transformMarkdownWithVue(
      "import Outside from '../../Outside.vue'\n\n<Outside />\n",
      "/repo/docs/nested/page.mdx",
      createOptions({ components: new Map() }),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).not.toContain("import Outside");
  });

  it("still hydrates the global map and leaves plain .md unchanged", async () => {
    const globalMap = await transformMarkdownWithVue(
      '<Alert tone="info" />\n',
      "/repo/docs/guide.mdx",
      createOptions(),
    );
    expect(globalMap.code).toContain("import Alert from '../src/components/Alert.vue'");

    const plain = await transformMarkdownWithVue(
      "import Chart from './Chart.vue'\n\n<Chart />\n",
      "/repo/docs/page.md",
      createOptions({ components: new Map() }),
    );
    expect(plain.usedComponents).toEqual([]);
    expect(plain.code).not.toContain("import Chart from './Chart.vue'");
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
    autolinks: true,
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
