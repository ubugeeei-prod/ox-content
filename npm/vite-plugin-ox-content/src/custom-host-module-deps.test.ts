import * as path from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { collectDevModuleDependencies } from "./custom-host-module-deps";
import type { CustomHostDevModuleGraph, CustomHostDevModuleNode } from "./custom-host-stylesheets";

describe("custom host module dependency inference", () => {
  it("skips virtual module ids as watch dependencies while traversing their imports", () => {
    const root = path.resolve("fixtures/custom-host");
    const realTransitive = node({ file: path.join(root, "src", "collections.ts") });
    const virtualCollection = node({
      id: "\0virtual:ox-content/collections",
      url: "/@id/__x00__virtual:ox-content/collections",
      imports: [realTransitive],
    });
    const host = node({
      id: "/src/host.ts",
      imports: [virtualCollection],
    });
    const moduleGraph = graph([host, virtualCollection, realTransitive]);

    const dependencies = collectDevModuleDependencies(moduleGraph, "/src/host.ts", root);

    expect(dependencies).toContain(normalize(path.join(root, "src", "host.ts")));
    expect(dependencies).toContain(normalize(path.join(root, "src", "collections.ts")));
    expect(dependencies).not.toContain(
      normalize(path.join(root, "\0virtual:ox-content/collections")),
    );
    expect(dependencies.every((dependency) => !dependency.includes("\0"))).toBe(true);
    expect(dependencies.every((dependency) => !dependency.includes("__x00__"))).toBe(true);
  });

  it("keeps real files whose names contain Vite escape text", () => {
    const root = path.resolve("fixtures/custom-host");
    const file = path.join(root, "src", "__x00__literal.ts");
    const moduleGraph = graph([node({ file })]);

    const dependencies = collectDevModuleDependencies(moduleGraph, file, root);

    expect(dependencies).toEqual([normalize(file)]);
  });

  it("starts from the source module when Svelte style request nodes share its file", () => {
    const root = path.resolve("fixtures/custom-host");
    const pageFile = path.join(root, "src", "Page.svelte");
    const sharedFile = path.join(root, "src", "shared.ts");
    const styleRequest = node({
      id: `${pageFile}?direct&svelte&type=style&lang.css`,
      file: pageFile,
    });
    const shared = node({ file: sharedFile });
    const page = node({ file: pageFile, imports: [shared] });
    const moduleGraph = graph([styleRequest, page, shared]);

    const dependencies = collectDevModuleDependencies(moduleGraph, "/src/Page.svelte", root);

    expect(dependencies).toEqual([normalize(pageFile), normalize(sharedFile)]);
  });
});

function node(input: {
  id?: string;
  url?: string;
  file?: string;
  imports?: CustomHostDevModuleNode[];
}): CustomHostDevModuleNode {
  return {
    id: input.id,
    url: input.url,
    file: input.file,
    importedModules: input.imports,
  };
}

function graph(nodes: CustomHostDevModuleNode[]): CustomHostDevModuleGraph {
  const modulesByFile = new Map<string, Set<CustomHostDevModuleNode>>();
  for (const moduleNode of nodes) {
    if (!moduleNode.file) {
      continue;
    }
    const file = normalize(moduleNode.file);
    const fileModules = modulesByFile.get(file) ?? new Set();
    fileModules.add(moduleNode);
    modulesByFile.set(file, fileModules);
  }
  return {
    idToModuleMap: new Map(
      nodes.flatMap((moduleNode) => (moduleNode.id ? [[moduleNode.id, moduleNode]] : [])),
    ),
    getModuleById(id) {
      return nodes.find((moduleNode) => moduleNode.id === id);
    },
    getModulesByFile(file) {
      return modulesByFile.get(normalize(file));
    },
  };
}

function normalize(file: string): string {
  return path.resolve(file).replace(/\\/g, "/");
}
