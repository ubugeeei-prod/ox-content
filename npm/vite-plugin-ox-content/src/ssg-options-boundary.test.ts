import { describe, expect, it, vi } from "vite-plus/test";
import * as path from "node:path";

const state = vi.hoisted(() => ({ markdownLoads: 0, rendererCreations: 0 }));

vi.mock("./custom-host-markdown", async (importOriginal) => {
  state.markdownLoads++;
  const actual = await importOriginal<typeof import("./custom-host-markdown")>();
  return {
    ...actual,
    createCustomHostMarkdownRenderer(
      ...args: Parameters<typeof actual.createCustomHostMarkdownRenderer>
    ) {
      state.rendererCreations++;
      return actual.createCustomHostMarkdownRenderer(...args);
    },
  };
});

vi.mock("./ssg", () => {
  throw new Error("Option helpers must not initialize the built-in SSG runtime.");
});

describe("SSG option dependency boundary", () => {
  it("creates custom hosts and renders standalone Markdown without the built-in SSG runtime", async () => {
    const { createOxContentCustomHostPlugin } = await import("./custom-host");
    const { createBaseContext } = await import("./custom-host-loader");
    const { createAssetsContext } = await import("./custom-host-assets");
    const { resolveOptions } = await import("./resolve-options");
    const oxContent = {
      docs: false,
      search: false,
      embeds: false,
      highlight: false,
      ssg: false,
    } as const;
    const plugin = createOxContentCustomHostPlugin({ host: { routes: [] }, oxContent });
    expect(plugin.name).toBe("ox-content:custom-host");
    expect(state.markdownLoads).toBe(0);

    const root = process.cwd();
    const outDir = path.join(root, "dist");
    const options = resolveOptions(oxContent);
    const context = createBaseContext(
      "build",
      root,
      outDir,
      options,
      async () => {
        throw new Error("Unexpected module request.");
      },
      createAssetsContext(options, outDir, undefined, undefined),
    );
    expect(state.markdownLoads).toBe(0);
    const input = {
      source: "---\ntitle: Boundary\n---\n\n# 雪🌸\n",
      documentPath: "content/boundary.md",
    };
    const [first, second] = await Promise.all([
      context.markdown.render(input),
      context.markdown.render(input),
    ]);
    expect(first.html).toBe(second.html);
    expect(state.markdownLoads).toBe(1);
    expect(state.rendererCreations).toBe(1);

    const { createOxContentMarkdownRenderer } = await import("./standalone-markdown");
    const renderer = createOxContentMarkdownRenderer({ root: process.cwd(), oxContent });
    const result = await renderer.render(input);
    expect(result.frontmatter.title).toBe("Boundary");
    expect(result.html).toContain("雪🌸");
    expect(result.html).toBe(first.html);
  });
});
