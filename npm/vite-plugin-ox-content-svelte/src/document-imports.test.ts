import { describe, expect, it } from "vite-plus/test";
import { transformMarkdownWithSvelte } from "./transform";
import type { ResolvedSvelteOptions } from "./types";

describe("document-local Svelte islands", () => {
  it("hydrates a relative default import and emits a static import of that file", async () => {
    const result = await transformMarkdownWithSvelte(
      "import GtvChart from './gtv-chart/GtvChart.svelte'\n\n<GtvChart title=\"ok\" />\n",
      "/repo/docs/guide.mdx",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual(["GtvChart"]);
    expect(result.code).toContain("import GtvChart from './gtv-chart/GtvChart.svelte'");
    expect(result.code).toContain("initIslands");
  });

  it("keeps the same local name on two documents as distinct imports", async () => {
    const first = await transformMarkdownWithSvelte(
      "import Chart from './a/Chart.svelte'\n\n<Chart />\n",
      "/repo/docs/a.mdx",
      createOptions({ components: {} }),
    );
    const second = await transformMarkdownWithSvelte(
      "import Chart from './b/Chart.svelte'\n\n<Chart />\n",
      "/repo/docs/b.mdx",
      createOptions({ components: {} }),
    );

    expect(first.code).toContain("import Chart from './a/Chart.svelte'");
    expect(second.code).toContain("import Chart from './b/Chart.svelte'");
    expect(first.code).not.toContain("./b/Chart.svelte");
    expect(second.code).not.toContain("./a/Chart.svelte");
  });

  it("rejects an import that escapes the content root", async () => {
    const result = await transformMarkdownWithSvelte(
      "import Outside from '../../Outside.svelte'\n\n<Outside />\n",
      "/repo/docs/nested/page.mdx",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).not.toContain("import Outside");
    expect(result.code).not.toContain("initIslands");
  });

  it("resolves a nested relative path and keeps literal props", async () => {
    const result = await transformMarkdownWithSvelte(
      "import Foo from './widgets/Foo.svelte'\n\n<Foo title=\"ok\" />\n",
      "/repo/docs/guide.mdx",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual(["Foo"]);
    expect(result.code).toContain("import Foo from './widgets/Foo.svelte'");
    expect(result.code).toContain("ok");
  });

  it("still hydrates the global components map when the document has no import", async () => {
    const result = await transformMarkdownWithSvelte(
      '<Alert tone="info" />\n',
      "/repo/docs/guide.mdx",
      createOptions(),
    );

    expect(result.usedComponents).toEqual(["Alert"]);
    expect(result.code).toContain("import Alert from '../src/components/Alert.svelte'");
  });

  it("lets a document-local binding override a global name", async () => {
    const result = await transformMarkdownWithSvelte(
      "import Alert from './local/Alert.svelte'\n\n<Alert />\n",
      "/repo/docs/guide.mdx",
      createOptions(),
    );

    expect(result.code).toContain("import Alert from './local/Alert.svelte'");
    expect(result.code).not.toContain("../src/components/Alert.svelte");
  });

  it("leaves unregistered JSX without a matching import static", async () => {
    const result = await transformMarkdownWithSvelte(
      "# Plain\n\n<Unknown />\n",
      "/repo/docs/unknown.mdx",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).not.toContain("import Unknown");
    expect(result.code).not.toContain("initIslands");
  });

  it("does not create document-import islands from ESM in plain .md", async () => {
    const result = await transformMarkdownWithSvelte(
      "import Chart from './Chart.svelte'\n\n<Chart />\n",
      "/repo/docs/page.md",
      createOptions({ components: {} }),
    );

    expect(result.usedComponents).toEqual([]);
    expect(result.code).not.toContain("import Chart from './Chart.svelte'");
    expect(result.code).not.toContain("initIslands");
  });

  it("applies an optional SSR hook without importing svelte/server", async () => {
    const result = await transformMarkdownWithSvelte(
      "import GtvChart from './gtv-chart/GtvChart.svelte'\n\n<GtvChart title=\"ok\" />\n",
      "/repo/docs/guide.mdx",
      createOptions({
        components: {},
        renderIsland: (name, props) => `<span class="ssr">${name}:${String(props.title)}</span>`,
      }),
    );

    expect(result.code).toContain("GtvChart:ok");
    expect(result.code).toContain("import GtvChart from './gtv-chart/GtvChart.svelte'");
    expect(result.code).not.toContain("svelte/server");
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
