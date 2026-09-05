import { describe, expect, it } from "vite-plus/test";
import {
  resolveSolidIslandStylesheets,
  type SolidDevModuleNode,
  type SolidStylesheetManifest,
} from ".";

describe("resolveSolidIslandStylesheets", () => {
  it("collects production CSS from static imports in deterministic order", () => {
    const manifest: SolidStylesheetManifest = {
      "src/Chart.tsx": {
        file: "assets/Chart.js",
        css: ["assets/chart.css"],
        imports: ["_shared.js"],
      },
      "_shared.js": {
        file: "assets/shared.js",
        css: ["assets/base.css", "assets/shared.css"],
        imports: ["_cycle.js"],
      },
      "_cycle.js": {
        file: "assets/cycle.js",
        css: ["assets/cycle.css"],
        imports: ["_shared.js"],
      },
      "src/Badge.tsx": {
        file: "assets/Badge.js",
        css: ["assets/shared.css", "assets/badge.css"],
      },
      "src/Empty.tsx": {
        file: "assets/Empty.js",
      },
    };

    const result = resolveSolidIslandStylesheets({
      modules: ["src/Chart.tsx", "src/Badge.tsx", "src/Empty.tsx", "src/Missing.tsx"],
      manifest,
      base: "/docs/",
    });

    expect(result.stylesheets.map((stylesheet) => stylesheet.href)).toEqual([
      "/docs/assets/cycle.css",
      "/docs/assets/base.css",
      "/docs/assets/shared.css",
      "/docs/assets/chart.css",
      "/docs/assets/badge.css",
    ]);
    expect(result.diagnostics).toEqual([
      expect.objectContaining({ code: "missing-module", moduleId: "src/Missing.tsx" }),
    ]);
  });

  it("resolves production entries by manifest src and keeps absolute CSS URLs intact", () => {
    const manifest: SolidStylesheetManifest = {
      "assets/Chart.js": {
        src: "src/Chart.tsx",
        css: ["https://cdn.example.com/chart.css", "assets/chart.css"],
      },
    };

    const result = resolveSolidIslandStylesheets({
      modules: ["src/Chart.tsx"],
      manifest,
      base: "/docs/",
    });

    expect(result).toEqual({
      stylesheets: [
        { href: "https://cdn.example.com/chart.css", moduleId: "src/Chart.tsx" },
        { href: "/docs/assets/chart.css", moduleId: "src/Chart.tsx" },
      ],
      diagnostics: [],
    });
  });

  it("collects development CSS URLs from the module graph", () => {
    const reset = node("/src/reset.css?used");
    const chart = node("/src/Chart.tsx", [reset, node("/src/chart.css?direct")]);
    const plain = node("/src/Plain.tsx");
    const graph = {
      getModuleById(id: string) {
        return id === "/src/Chart.tsx" ? chart : id === "/src/Plain.tsx" ? plain : undefined;
      },
    };

    const result = resolveSolidIslandStylesheets({
      modules: ["/src/Chart.tsx", "/src/Plain.tsx", "/src/Missing.tsx"],
      moduleGraph: graph,
      base: "/base/",
    });

    expect(result.stylesheets.map((stylesheet) => stylesheet.href)).toEqual([
      "/base/src/reset.css?used",
      "/base/src/chart.css?direct",
    ]);
    expect(result.diagnostics).toEqual([
      expect.objectContaining({ code: "missing-module", moduleId: "/src/Missing.tsx" }),
    ]);
  });

  it("falls back to Vite getModulesByFile in development", () => {
    const chart = node("/@fs/repo/src/Chart.tsx", [node("/src/chart.css?t=1")]);
    const graph = {
      getModuleById() {
        return undefined;
      },
      getModulesByFile(file: string) {
        return file === "/repo/src/Chart.tsx" ? new Set([chart]) : undefined;
      },
    };

    const result = resolveSolidIslandStylesheets({
      modules: ["/repo/src/Chart.tsx"],
      moduleGraph: graph,
    });

    expect(result).toEqual({
      stylesheets: [{ href: "/src/chart.css?t=1", moduleId: "/repo/src/Chart.tsx" }],
      diagnostics: [],
    });
  });

  it("reports a missing resolver separately from a module with no CSS", () => {
    expect(resolveSolidIslandStylesheets({ modules: ["src/Chart.tsx"] })).toEqual({
      stylesheets: [],
      diagnostics: [
        {
          code: "missing-resolver",
          moduleId: "src/Chart.tsx",
          message: 'No Vite manifest or module graph was supplied for "src/Chart.tsx".',
        },
      ],
    });
  });
});

function node(url: string, imports: SolidDevModuleNode[] = []): SolidDevModuleNode {
  return {
    id: url,
    url,
    importedModules: imports,
  };
}
