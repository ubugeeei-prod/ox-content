import { describe, expect, it } from "vite-plus/test";
import {
  resolveCustomHostStylesheets,
  type CustomHostDevModuleNode,
} from "./custom-host-stylesheets";

describe("resolveCustomHostStylesheets", () => {
  it("collects build CSS in dependency order and dev CSS dependencies", () => {
    const manifest = {
      "src/Island.ts": {
        file: "assets/Island.js",
        imports: ["src/prose.css", "_child.js", "src/shared.css"],
        css: ["assets/island.css"],
      },
      "src/Module.ts": { file: "assets/module.js", css: ["assets/module.css"] },
      "src/Plain.ts": { file: "assets/plain.js" },
      "_child.js": { file: "assets/child.js", css: ["assets/child.css"] },
      "src/prose.css": { file: "assets/prose.css", src: "src/prose.css" },
      "src/shared.css": { file: "assets/prose.css", src: "src/shared.css" },
    };
    const hrefs = (modules: readonly string[], base = "/") =>
      resolveCustomHostStylesheets({ modules, manifest, base }).stylesheets.map(
        (style) => style.href,
      );

    expect(hrefs(["src/prose.css"])).toEqual(["/assets/prose.css"]);
    expect(hrefs(["src/prose.css"], "docs")).toEqual(["/docs/assets/prose.css"]);
    expect(hrefs(["src/Module.ts"])).toEqual(["/assets/module.css"]);
    expect(hrefs(["/src/Island.ts"], "/docs/")).toEqual([
      "/docs/assets/prose.css",
      "/docs/assets/child.css",
      "/docs/assets/island.css",
    ]);
    expect(hrefs(["src/prose.css", "src/shared.css"])).toEqual(["/assets/prose.css"]);

    const noStyle = resolveCustomHostStylesheets({
      modules: ["src/Plain.ts", "src/Missing.ts"],
      manifest,
    });
    expect(noStyle.stylesheets).toEqual([]);
    expect(noStyle.diagnostics).toEqual([
      expect.objectContaining({ code: "missing-module", moduleId: "src/Missing.ts" }),
    ]);

    const directCss = node("/src/prose.css", [], "/repo/src/prose.css");
    const childCss = node("/src/child.module.css?used", [], "/repo/src/child.module.css");
    const island = node("/src/Island.ts", [childCss], "/repo/src/Island.ts");
    const dev = resolveCustomHostStylesheets({
      modules: ["/src/prose.css", "/src/Island.ts", "/src/Missing.ts"],
      moduleGraph: {
        getModuleById: (id) =>
          id === "/src/prose.css" ? directCss : id === "/src/Island.ts" ? island : undefined,
      },
      base: "/docs/",
      root: "/repo",
    });

    expect(dev.stylesheets).toEqual([
      { kind: "style", href: "/docs/src/prose.css", moduleId: "/src/prose.css" },
      { kind: "style", href: "/docs/src/child.module.css?used", moduleId: "/src/Island.ts" },
    ]);
    expect(dev.dependencies).toEqual([
      "/repo/src/prose.css",
      "/repo/src/Island.ts",
      "/repo/src/child.module.css",
    ]);
    expect(dev.diagnostics).toEqual([
      expect.objectContaining({ code: "missing-module", moduleId: "/src/Missing.ts" }),
    ]);
  });

  it("reports a missing resolver separately from a module with no CSS", () => {
    expect(resolveCustomHostStylesheets({ modules: ["src/Island.ts"] })).toEqual({
      stylesheets: [],
      dependencies: [],
      diagnostics: [
        {
          code: "missing-resolver",
          moduleId: "src/Island.ts",
          message:
            'No Vite manifest or development module graph was available for "src/Island.ts".',
        },
      ],
    });
  });

  it("keeps compiled framework CSS from the dev module graph", () => {
    const page = node("/src/Page.svelte", [], "/repo/src/Page.svelte");
    page.meta = { svelte: { css: { code: ".probe.svelte-abc{color:red}\n" } } };

    const result = resolveCustomHostStylesheets({
      modules: ["/src/Page.svelte"],
      moduleGraph: {
        getModuleById: (id) => (id === "/src/Page.svelte" ? page : undefined),
      },
      base: "/docs/",
      root: "/repo",
    });

    expect(result.stylesheets).toEqual([
      {
        kind: "style",
        href: "/docs/src/Page.svelte?svelte&type=style&lang.css",
        moduleId: "/src/Page.svelte",
        content: ".probe.svelte-abc{color:red}\n",
      },
    ]);
    expect(result.diagnostics).toEqual([]);
  });
});

function node(
  url: string,
  imports: CustomHostDevModuleNode[] = [],
  file?: string,
): CustomHostDevModuleNode {
  return {
    id: url,
    url,
    file,
    importedModules: imports,
  };
}
