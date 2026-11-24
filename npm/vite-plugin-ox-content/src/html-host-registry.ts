import fs from "node:fs/promises";
import path from "node:path";
import type { ModuleNode, Plugin, ResolvedConfig, ViteDevServer } from "vite";
import { collectMdxIslandNamesFromHtml, intersectHydratableComponentNames } from "./mdx-islands";
import { discoverDocumentMdxIslands } from "./document-islands";
import { resolveContentRootPath, stripViteQuery } from "./document-imports";
import { renderMarkdown } from "./render-markdown";
import { customHostOxContentOptions } from "./custom-host";
import type { MdxImport, OxContentOptions } from "./types";
import {
  resolveHtmlHostComponentsGlob,
  type HtmlHostComponentsMap,
  type HtmlHostComponentsOption,
} from "./html-host-components";
import type { HtmlHostClientModule, HtmlHostModule } from "./html-host";
import {
  resolveHtmlHostComponentPath,
  resolveHtmlHostDocumentPath,
  resolveHtmlHostWatchFile,
  shouldInvalidateHtmlHostRegistry,
  toHtmlHostClientModuleId,
} from "./html-host-registry-paths";
import {
  resolveHtmlHostCollectionDocuments,
  type HtmlHostCollectionDocumentsOptions,
} from "./html-host-collection-documents";

export { toHtmlHostClientModuleId } from "./html-host-registry-paths";

export const HTML_HOST_MODULES_VIRTUAL_ID = "virtual:ox-content/html-host/modules";

type MaybePromise<T> = T | Promise<T>;

export interface HtmlHostIslandDocument {
  documentPath: string;
  source?: string;
  html?: string;
  imports?: readonly MdxImport[];
  components?: HtmlHostComponentsMap;
  dependencies?: readonly string[];
}

export interface HtmlHostIslandEntry {
  moduleId: string;
  name?: string;
  exportName?: string;
  documentPath?: string;
}

export interface HtmlHostIslandRegistryContext {
  root: string;
  mode: string;
  command: "build" | "serve";
}

export interface CreateHtmlHostIslandRegistryInput {
  documents?:
    | readonly HtmlHostIslandDocument[]
    | ((context: HtmlHostIslandRegistryContext) => MaybePromise<readonly HtmlHostIslandDocument[]>);
  collectionDocuments?: false | HtmlHostCollectionDocumentsOptions;
  entries?:
    | readonly HtmlHostIslandEntry[]
    | ((context: HtmlHostIslandRegistryContext) => MaybePromise<readonly HtmlHostIslandEntry[]>);
  components?: HtmlHostComponentsOption;
  oxContent?: OxContentOptions;
  root?: string;
  watch?: readonly string[];
  virtualModuleId?: string;
  pluginName?: string;
}

export interface ResolvedHtmlHostIslandRegistry {
  modules: HtmlHostClientModule[];
  watchFiles: string[];
}

export interface HtmlHostIslandRegistry {
  plugin: Plugin;
  virtualModuleId: string;
  resolve(): Promise<ResolvedHtmlHostIslandRegistry>;
  /** Refresh discovery and the dev virtual module after a programmatic input change. */
  invalidate(): void;
  resolveClientModule(module: Pick<HtmlHostModule, "serverModuleId">): string;
}

export function createHtmlHostIslandRegistry(
  input: CreateHtmlHostIslandRegistryInput = {},
): HtmlHostIslandRegistry {
  const virtualModuleId = input.virtualModuleId ?? HTML_HOST_MODULES_VIRTUAL_ID;
  const resolvedVirtualModuleId = `\0${virtualModuleId}`;
  let config: ResolvedConfig | undefined;
  let server: ViteDevServer | undefined;
  let command: "build" | "serve" = "build";
  let components: HtmlHostComponentsMap | undefined;
  let cache: Promise<ResolvedHtmlHostIslandRegistry> | undefined;
  let resolved: ResolvedHtmlHostIslandRegistry = { modules: [], watchFiles: [] };

  const context = (): HtmlHostIslandRegistryContext => ({
    root: config?.root ?? input.root ?? process.cwd(),
    mode: config?.mode ?? "production",
    command,
  });

  const resolve = async () => {
    while (true) {
      const pending = (cache ??= resolveHtmlHostIslandRegistry(input, context(), components));
      try {
        const next = await pending;
        // Inputs can change during discovery, including from a transform that discovery requested.
        if (pending === cache) {
          resolved = next;
          return next;
        }
      } catch (error) {
        if (pending === cache) throw error;
      }
    }
  };

  const invalidate = (devServer = server) => {
    cache = undefined;
    if (devServer) invalidateVirtualModule(devServer, resolvedVirtualModuleId);
  };

  const plugin: Plugin = {
    name: input.pluginName ?? "ox-content:html-host-island-registry",

    config(_config, env) {
      command = env.command;
    },

    configureServer(devServer) {
      server = devServer;
    },

    async configResolved(resolvedConfig) {
      config = resolvedConfig;
      components = input.components
        ? await resolveHtmlHostComponentsGlob(input.components, resolvedConfig.root)
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
        !shouldInvalidateHtmlHostRegistry(
          ctx.file,
          context().root,
          input.oxContent?.srcDir,
          resolved.watchFiles,
          input.watch,
        )
      ) {
        return undefined;
      }
      invalidate(ctx.server);
      ctx.server.ws.send({ type: "full-reload" });
      return [];
    },
  };

  return {
    plugin,
    virtualModuleId,
    resolve,
    invalidate,
    resolveClientModule(module) {
      return toHtmlHostClientModuleId(module.serverModuleId, context().root);
    },
  };
}

export async function resolveHtmlHostIslandRegistry(
  input: CreateHtmlHostIslandRegistryInput,
  context: HtmlHostIslandRegistryContext,
  resolvedComponents?: HtmlHostComponentsMap,
): Promise<ResolvedHtmlHostIslandRegistry> {
  const components =
    resolvedComponents ??
    (input.components ? await resolveHtmlHostComponentsGlob(input.components, context.root) : {});
  const documents = [
    ...((await resolveMaybe(input.documents, context)) ?? []),
    ...((await resolveInputCollectionDocuments(input, context)) ?? []),
  ];
  const entries = await resolveMaybe(input.entries, context);
  const modules = new Map<string, HtmlHostClientModule>();
  const watchFiles = new Set<string>();

  for (const file of input.watch ?? []) {
    watchFiles.add(resolveHtmlHostWatchFile(file, context.root));
  }

  for (const entry of entries ?? []) {
    addModule(modules, {
      name:
        entry.name ?? path.basename(stripViteQuery(entry.moduleId), path.extname(entry.moduleId)),
      moduleId: toHtmlHostClientModuleId(entry.moduleId, context.root),
      exportName: entry.exportName ?? "default",
    });
    if (entry.documentPath)
      watchFiles.add(resolveHtmlHostWatchFile(entry.documentPath, context.root));
  }

  const oxContent = customHostOxContentOptions({ ...input.oxContent, embeds: false });
  const srcDir = oxContent.srcDir ?? "content";
  const contentRoot = resolveContentRootPath({ root: context.root, srcDir });
  for (const document of documents) {
    const documentPath = resolveHtmlHostDocumentPath(document.documentPath, context.root);
    watchFiles.add(documentPath);
    for (const dependency of document.dependencies ?? []) {
      watchFiles.add(resolveHtmlHostWatchFile(dependency, context.root));
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
        : resolveHtmlHostComponentPath(documentComponents, name, context.root);
      if (!serverModuleId) continue;
      addModule(modules, {
        name,
        moduleId: toHtmlHostClientModuleId(serverModuleId, context.root),
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

function resolveInputCollectionDocuments(
  input: CreateHtmlHostIslandRegistryInput,
  context: HtmlHostIslandRegistryContext,
): Promise<readonly HtmlHostIslandDocument[]> | undefined {
  if (!input.collectionDocuments) return undefined;
  const collectionInput =
    input.collectionDocuments.oxContent === undefined
      ? { ...input.collectionDocuments, oxContent: input.oxContent }
      : input.collectionDocuments;
  return resolveHtmlHostCollectionDocuments(collectionInput, context);
}

function renderModulesVirtualModule(registry: ResolvedHtmlHostIslandRegistry): string {
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
  document: HtmlHostIslandDocument,
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

function addModule(modules: Map<string, HtmlHostClientModule>, module: HtmlHostClientModule) {
  modules.set(`${module.moduleId}\0${module.exportName}\0${module.name}`, module);
}

function resolveMaybe<T>(
  value:
    | readonly T[]
    | ((context: HtmlHostIslandRegistryContext) => MaybePromise<readonly T[]>)
    | undefined,
  context: HtmlHostIslandRegistryContext,
): MaybePromise<readonly T[] | undefined> {
  return typeof value === "function" ? value(context) : value;
}

function invalidateVirtualModule(server: ViteDevServer, resolvedVirtualModuleId: string): void {
  const mod = server.moduleGraph.getModuleById(resolvedVirtualModuleId);
  if (mod) server.moduleGraph.invalidateModule(mod as ModuleNode);
}
