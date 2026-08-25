import path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { resolveDocumentComponentImports } from "./document-imports";
import type { MdxImport } from "./types";

const contentRoot = "/repo/docs";

describe("resolveDocumentComponentImports", () => {
  it("resolves a relative default import against the document directory", () => {
    const result = resolveDocumentComponentImports({
      imports: [defaultImport("GtvChart", "./gtv-chart/GtvChart.tsx")],
      documentPath: "/repo/docs/guide.mdx",
      contentRoot,
      srcDir: "docs",
    });

    expect(result.diagnostics).toEqual([]);
    expect(result.bindings).toEqual([
      {
        localName: "GtvChart",
        specifier: "./gtv-chart/GtvChart.tsx",
        resolvedPath: path.normalize("/repo/docs/gtv-chart/GtvChart.tsx"),
        importPathRelativeToDocument: "./gtv-chart/GtvChart.tsx",
        imported: "default",
        kind: "default",
      },
    ]);
  });

  it("resolves a named import alias", () => {
    const result = resolveDocumentComponentImports({
      imports: [
        {
          source: "./Chart.tsx",
          specifiers: [{ imported: "Chart", local: "Plot", kind: "named" }],
        },
      ],
      documentPath: "/repo/docs/guide.mdx",
      contentRoot,
    });

    expect(result.bindings).toEqual([
      expect.objectContaining({
        localName: "Plot",
        imported: "Chart",
        kind: "named",
        importPathRelativeToDocument: "./Chart.tsx",
      }),
    ]);
  });

  it("keeps two documents with the same local name on distinct paths", () => {
    const first = resolveDocumentComponentImports({
      imports: [defaultImport("Chart", "./a/Chart.tsx")],
      documentPath: "/repo/docs/a.mdx",
      contentRoot,
    });
    const second = resolveDocumentComponentImports({
      imports: [defaultImport("Chart", "./b/Chart.tsx")],
      documentPath: "/repo/docs/b.mdx",
      contentRoot,
    });

    expect(first.bindings[0]?.importPathRelativeToDocument).toBe("./a/Chart.tsx");
    expect(second.bindings[0]?.importPathRelativeToDocument).toBe("./b/Chart.tsx");
    expect(first.bindings[0]?.resolvedPath).not.toBe(second.bindings[0]?.resolvedPath);
  });

  it("rejects a specifier that escapes the content root", () => {
    const result = resolveDocumentComponentImports({
      imports: [defaultImport("Outside", "../../Outside.tsx")],
      documentPath: "/repo/docs/nested/page.mdx",
      contentRoot,
      srcDir: "docs",
    });

    expect(result.bindings).toEqual([]);
    expect(result.diagnostics).toEqual([
      expect.objectContaining({
        code: "escapes-root",
        specifier: "../../Outside.tsx",
        localName: "Outside",
      }),
    ]);
  });

  it("ignores bare, package, and remote specifiers", () => {
    const result = resolveDocumentComponentImports({
      imports: [
        defaultImport("Chart", "chart-pkg"),
        defaultImport("Remote", "https://example.com/Chart.js"),
      ],
      documentPath: "/repo/docs/guide.mdx",
      contentRoot,
    });

    expect(result.bindings).toEqual([]);
    expect(result.diagnostics.map((item) => item.code)).toEqual(["not-relative", "not-relative"]);
  });

  it("skips namespace imports", () => {
    const result = resolveDocumentComponentImports({
      imports: [
        {
          source: "./icons.tsx",
          specifiers: [{ imported: "*", local: "Icons", kind: "namespace" }],
        },
      ],
      documentPath: "/repo/docs/guide.mdx",
      contentRoot,
    });

    expect(result.bindings).toEqual([]);
  });

  it("drops a name bound twice in the same document", () => {
    const result = resolveDocumentComponentImports({
      imports: [defaultImport("Chart", "./A.tsx"), defaultImport("Chart", "./B.tsx")],
      documentPath: "/repo/docs/guide.mdx",
      contentRoot,
    });

    expect(result.bindings).toEqual([]);
    expect(result.diagnostics.some((item) => item.code === "duplicate-binding")).toBe(true);
  });
});

function defaultImport(local: string, source: string): MdxImport {
  return {
    source,
    specifiers: [{ imported: "default", local, kind: "default" }],
  };
}
