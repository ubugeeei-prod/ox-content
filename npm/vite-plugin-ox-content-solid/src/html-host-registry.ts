import fs from "node:fs/promises";
import path from "node:path";
import type { ModuleNode, Plugin, ResolvedConfig, ViteDevServer } from "vite";
import {
  collectMdxIslandNamesFromHtml,
  customHostOxContentOptions,
  discoverDocumentMdxIslands,
  intersectHydratableComponentNames,
  renderMarkdown,
  resolveContentRootPath,
  stripViteQuery,
  type MdxImport,
  type OxContentOptions,
} from "@ox-content/vite-plugin";
import { resolveComponentsGlob } from "./components";
import type { SolidHtmlHostClientModule, SolidHtmlHostModule } from "./html-host";
import {
  isBareSpecifier,
  resolveDocumentPath,
  resolveWatchFile,
  shouldInvalidate,
  toSolidHtmlHostClientModuleId,
} from "./html-host-registry-paths";
import type { ComponentsMap, ComponentsOption } from "./types";

export { toSolidHtmlHostClientModuleId } from "./html-host-registry-paths";

export const SOLID_HTML_HOST_MODULES_VIRTUAL_ID = "virtual:ox-content-solid/html-host/modules";

type MaybePromise<T> = T | Promise<T>;

export interface SolidHtmlHostIslandDocument {
  documentPath: string;
  source?: string;
  html?: string;
  imports?: readonly MdxImport[];
  components?: ComponentsMap;
  dependencies?: readonly string[];
}

export interface SolidHtmlHostIslandEntry {
  moduleId: string;
  name?: string;
  exportName?: string;
  documentPath?: string;
}

export interface SolidHtmlHostIslandRegistryContext {
  root: string;
  mode: string;
  command: "build" | "serve";
}

export interface CreateSolidHtmlHostIslandRegistryInput {
  documents?:
    | readonly SolidHtmlHostIslandDocument[]
    | ((
        context: SolidHtmlHostIslandRegistryContext,
      ) => MaybePromise<readonly SolidHtmlHostIslandDocument[]>);
  entries?:
    | readonly SolidHtmlHostIslandEntry[]
    | ((
        context: SolidHtmlHostIslandRegistryContext,
      ) => MaybePromise<readonly SolidHtmlHostIslandEntry[]>);
  components?: ComponentsOption;
  oxContent?: OxContentOptions;
  root?: string;
  watch?: readonly string[];
  virtualModuleId?: string;
}

export interface ResolvedSolidHtmlHostIslandRegistry {
  modules: SolidHtmlHostClientModule[];
  watchFiles: string[];
}

export interface SolidHtmlHostIslandRegistry {
  plugin: Plugin;
  virtualModuleId: string;
  resolve(): Promise<ResolvedSolidHtmlHostIslandRegistry>;
  resolveClientModule(module: Pick<SolidHtmlHostModule, "serverModuleId">): string;
}

export function createSolidHtmlHostIslandRegistry(
  input: CreateSolidHtmlHostIslandRegistryInput = {},
): SolidHtmlHostIslandRegistry {
  const virtualModuleId = input.virtualModuleId ?? SOLID_HTML_HOST_MODULES_VIRTUAL_ID;
  const resolvedVirtualModuleId = `\0${virtualModuleId}`;
  let config: ResolvedConfig | undefined;
  let command: "build" | "serve" = "build";
  let components: ComponentsMap | undefined;
  let cache: Promise<ResolvedSolidHtmlHostIslandRegistry> | undefined;
  let resolved: ResolvedSolidHtmlHostIslandRegistry = { modules: [], watchFiles: [] };

  const context = (): SolidHtmlHostIslandRegistryContext => ({
    root: config?.root ?? input.root ?? process.cwd(),
    mode: config?.mode ?? "production",
    command,
  });

  const resolve = async () => {
    cache ??= resolveSolidHtmlHostIslandRegistry(input, context(), components).then((next) => {
      resolved = next;
      return next;
    });
    return cache;
  };

  const invalidate = () => {
    cache = undefined;
  };

  const plugin: Plugin = {
    name: "ox-content:solid-html-host-island-registry",

    config(_config, env) {
      command = env.command;
    },

    async configResolved(resolvedConfig) {
      config = resolvedConfig;
      components = input.components
        ? await resolveComponentsGlob(input.components, resolvedConfig.root)
        : {};
      invalidate();
    },

    buildStart() {
      invalidate();
    },

    resolveId(id) {
      return id === virtualModuleId ? resolvedVirtualModuleId : null;
    },

    async load(id) {
      if (id !== resolvedVirtualModuleId) return null;
      return renderModulesVirtualModule(await resolve());
    },

    handleHotUpdate(ctx) {
      if (
        !shouldInvalidate(
          ctx.file,
          context().root,
          input.oxContent?.srcDir,
          resolved.watchFiles,
          input.watch,
        )
      ) {
        return undefined;
      }
      invalidate();
      invalidateVirtualModule(ctx.server, resolvedVirtualModuleId);
      ctx.server.ws.send({ type: "full-reload" });
      return [];
    },
  };

  return {
    plugin,
    virtualModuleId,
    resolve,
    resolveClientModule(module) {
      return toSolidHtmlHostClientModuleId(module.serverModuleId, context().root);
    },
  };
}

export async function resolveSolidHtmlHostIslandRegistry(
  input: CreateSolidHtmlHostIslandRegistryInput,
  context: SolidHtmlHostIslandRegistryContext,
  resolvedComponents?: ComponentsMap,
): Promise<ResolvedSolidHtmlHostIslandRegistry> {
  const components =
    resolvedComponents ??
    (input.components ? await resolveComponentsGlob(input.components, context.root) : {});
  const documents = await resolveMaybe(input.documents, context);
  const entries = await resolveMaybe(input.entries, context);
  const modules = new Map<string, SolidHtmlHostClientModule>();
  const watchFiles = new Set<string>();

  for (const file of input.watch ?? []) {
    watchFiles.add(resolveWatchFile(file, context.root));
  }

  for (const entry of entries ?? []) {
    addModule(modules, {
      name:
        entry.name ?? path.basename(stripViteQuery(entry.moduleId), path.extname(entry.moduleId)),
      moduleId: toSolidHtmlHostClientModuleId(entry.moduleId, context.root),
      exportName: entry.exportName ?? "default",
    });
    if (entry.documentPath) watchFiles.add(resolveWatchFile(entry.documentPath, context.root));
  }

  const oxContent = customHostOxContentOptions(input.oxContent ?? {});
  const srcDir = oxContent.srcDir ?? "content";
  const contentRoot = resolveContentRootPath({ root: context.root, srcDir });
  for (const document of documents ?? []) {
    const documentPath = resolveDocumentPath(document.documentPath, context.root);
    watchFiles.add(documentPath);
    for (const dependency of document.dependencies ?? []) {
      watchFiles.add(resolveWatchFile(dependency, context.root));
    }

    const materialized = await materializeDocument(document, documentPath, oxContent);
    if (!materialized) continue;

    const documentComponents = document.components ?? components;
    const discovered = await discoverDocumentMdxIslands({
      source: materialized.source,
      html: materialized.html,
      components: documentComponents,
      imports: materialized.imports,
      documentPath,
      contentRoot,
      srcDir,
      root: context.root,
    });
    const usedComponents = new Set(discovered.usedComponents);
    if (materialized.html) {
      for (const name of intersectHydratableComponentNames(
        collectMdxIslandNamesFromHtml(materialized.html),
        documentComponents,
        discovered.localBindings.keys(),
      )) {
        usedComponents.add(name);
      }
    }

    for (const name of usedComponents) {
      const local = discovered.localBindings.get(name);
      const serverModuleId = local
        ? local.resolvedPath
        : resolveComponentPath(documentComponents, name, context.root);
      if (!serverModuleId) continue;
      addModule(modules, {
        name,
        moduleId: toSolidHtmlHostClientModuleId(serverModuleId, context.root),
        exportName: local?.imported ?? "default",
      });
    }
  }

  return {
    modules: [...modules.values()].sort((a, b) =>
      `${a.moduleId}\0${a.exportName}\0${a.name}`.localeCompare(
        `${b.moduleId}\0${b.exportName}\0${b.name}`,
      ),
    ),
    watchFiles: [...watchFiles],
  };
}

function renderModulesVirtualModule(registry: ResolvedSolidHtmlHostIslandRegistry): string {
  const moduleIds = [...new Set(registry.modules.map((module) => module.moduleId))].sort();
  return [
    "export const modules = {",
    ...moduleIds.map(
      (moduleId) => `  ${JSON.stringify(moduleId)}: () => import(${JSON.stringify(moduleId)}),`,
    ),
    "};",
    `export const clientModules = ${JSON.stringify(registry.modules, null, 2)};`,
    "export default modules;",
    "",
  ].join("\n");
}

async function materializeDocument(
  document: SolidHtmlHostIslandDocument,
  documentPath: string,
  oxContent: OxContentOptions,
): Promise<{ source: string; html?: string; imports: readonly MdxImport[] } | undefined> {
  const source = document.source ?? (await readOptional(documentPath));
  if (source === undefined && document.html === undefined) return undefined;
  if (document.imports && document.html !== undefined) {
    return { source: source ?? "", html: document.html, imports: document.imports };
  }
  if (source === undefined) {
    return { source: "", html: document.html, imports: document.imports ?? [] };
  }
  const rendered = await renderMarkdown(source, documentPath, oxContent);
  return {
    source,
    html: document.html ?? rendered.html,
    imports: document.imports ?? rendered.imports,
  };
}

async function readOptional(file: string): Promise<string | undefined> {
  try {
    return await fs.readFile(file, "utf8");
  } catch {
    return undefined;
  }
}

function addModule(
  modules: Map<string, SolidHtmlHostClientModule>,
  module: SolidHtmlHostClientModule,
) {
  modules.set(`${module.moduleId}\0${module.exportName}\0${module.name}`, module);
}

function resolveMaybe<T>(
  value:
    | readonly T[]
    | ((context: SolidHtmlHostIslandRegistryContext) => MaybePromise<readonly T[]>)
    | undefined,
  context: SolidHtmlHostIslandRegistryContext,
): MaybePromise<readonly T[] | undefined> {
  return typeof value === "function" ? value(context) : value;
}

function resolveComponentPath(
  components: ComponentsMap,
  name: string,
  root: string,
): string | undefined {
  const specifier = components[name];
  if (!specifier) return undefined;
  if (isBareSpecifier(specifier) || specifier.startsWith("/@fs/")) return specifier;
  if (specifier.startsWith("/") && !path.isAbsolute(specifier)) return specifier;
  return path.isAbsolute(specifier) ? specifier : path.resolve(root, specifier);
}

function invalidateVirtualModule(server: ViteDevServer, resolvedVirtualModuleId: string): void {
  const mod = server.moduleGraph.getModuleById(resolvedVirtualModuleId);
  if (mod) server.moduleGraph.invalidateModule(mod as ModuleNode);
}
