import { describe, expect, it, vi } from "vite-plus/test";

vi.mock("./ssg", () => {
  throw new Error("Option helpers must not initialize the built-in SSG runtime.");
});

describe("SSG option dependency boundary", () => {
  it("creates custom hosts and renders standalone Markdown without the built-in SSG runtime", async () => {
    const { createOxContentCustomHostPlugin } = await import("./custom-host");
    const { createOxContentMarkdownRenderer } = await import("./standalone-markdown");
    const oxContent = {
      docs: false,
      search: false,
      embeds: false,
      highlight: false,
      ssg: false,
    } as const;
    const plugin = createOxContentCustomHostPlugin({ host: { routes: [] }, oxContent });
    expect(plugin.name).toBe("ox-content:custom-host");

    const renderer = createOxContentMarkdownRenderer({ root: process.cwd(), oxContent });
    const result = await renderer.render({
      source: "---\ntitle: Boundary\n---\n\n# 雪🌸\n",
      documentPath: "content/boundary.md",
    });
    expect(result.frontmatter.title).toBe("Boundary");
    expect(result.html).toContain("雪🌸");
  });
});
