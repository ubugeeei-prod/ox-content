import type { ViteDevServer } from "vite";

export function invalidateViteModules(server: ViteDevServer, file: string, all = false): void {
  for (const moduleGraph of viteModuleGraphs(server)) {
    if (all) {
      if (typeof moduleGraph.invalidateAll === "function") {
        moduleGraph.invalidateAll();
      } else {
        for (const mod of moduleGraph.idToModuleMap?.values() ?? []) {
          moduleGraph.invalidateModule?.(mod);
        }
      }
      continue;
    }
    for (const mod of moduleGraph.getModulesByFile?.(file) ?? []) {
      moduleGraph.invalidateModule?.(mod);
    }
  }
  for (const evaluatedModules of viteEvaluatedModules(server)) {
    if (all) {
      evaluatedModules.clear();
      continue;
    }
    for (const mod of evaluatedModules.getModulesByFile(file) ?? []) {
      evaluatedModules.invalidateModule(mod);
    }
  }
}

export type ViteModuleGraphLike = {
  idToModuleMap?: Map<string, unknown>;
  getModuleById?(id: string): unknown;
  getModulesByFile?(file: string): Set<unknown> | undefined;
  invalidateAll?: () => void;
  invalidateModule?(mod: unknown): void;
};

export function viteModuleGraphs(server: ViteDevServer): ViteModuleGraphLike[] {
  const graphs = new Set<ViteModuleGraphLike>();
  graphs.add(server.moduleGraph as unknown as ViteModuleGraphLike);
  const environments = (
    server as ViteDevServer & {
      environments?: Record<string, ViteEnvironmentWithCaches>;
    }
  ).environments;
  for (const environment of Object.values(environments ?? {})) {
    if (environment.moduleGraph) {
      graphs.add(environment.moduleGraph);
    }
  }
  return [...graphs];
}

type ViteEnvironmentWithCaches = {
  moduleGraph?: ViteModuleGraphLike;
  runner?: { evaluatedModules?: InvalidatableEvaluatedModules };
};

type InvalidatableEvaluatedModules = {
  getModulesByFile(file: string): Set<unknown> | undefined;
  invalidateModule(mod: unknown): void;
  clear(): void;
};

function viteEvaluatedModules(server: ViteDevServer): InvalidatableEvaluatedModules[] {
  const modules = new Set<InvalidatableEvaluatedModules>();
  const compatRunner = (
    server as ViteDevServer & {
      _ssrCompatModuleRunner?: { evaluatedModules?: InvalidatableEvaluatedModules };
    }
  )._ssrCompatModuleRunner;
  if (compatRunner?.evaluatedModules) {
    modules.add(compatRunner.evaluatedModules);
  }
  const environments = (
    server as ViteDevServer & {
      environments?: Record<string, ViteEnvironmentWithCaches>;
    }
  ).environments;
  for (const environment of Object.values(environments ?? {})) {
    if (environment.runner?.evaluatedModules) {
      modules.add(environment.runner.evaluatedModules);
    }
  }
  return [...modules];
}
