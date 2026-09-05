import path from "node:path";
import {
  applyIslandSsrHtml,
  collectMdxIslandNamesFromHtml,
  resolveContentRootPath,
  resolveDocumentComponentImports,
  type DocumentImportDiagnostic,
  type MdxImport,
} from "@ox-content/vite-plugin";
import { readIslandSlotHtml } from "@ox-content/islands";
import type { ComponentsMap } from "./types";

export interface SolidHtmlHostModule {
  name: string;
  serverModuleId: string;
  exportName: string;
  source: "document" | "components";
  clientModuleId?: string;
}

export interface SolidHtmlHostClientModule {
  name: string;
  moduleId: string;
}

export type SolidHtmlHostDiagnosticCode =
  | DocumentImportDiagnostic["code"]
  | "missing-component"
  | "module-load-failed"
  | "missing-export"
  | "ssr-failed";

export interface SolidHtmlHostDiagnostic {
  code: SolidHtmlHostDiagnosticCode;
  message: string;
  documentPath: string;
  component?: string;
  moduleId?: string;
}

export type SolidServerModuleLoader = (moduleId: string) => Promise<unknown>;
type SolidComponentValue = NonNullable<unknown>;

export type SolidHtmlComponentRenderer = (
  component: unknown,
  props: Record<string, unknown>,
  slotHtml: string | undefined,
  context: { component: string; moduleId: string; documentPath: string },
) => string | Promise<string>;

export type SolidClientModuleResolver = (
  module: SolidHtmlHostModule,
  context: { documentPath: string },
) => string | undefined;

export interface RenderSolidHtmlHostInput {
  html: string;
  documentPath: string;
  components?: ComponentsMap;
  imports?: readonly MdxImport[];
  root?: string;
  srcDir?: string;
  contentRoot?: string;
  loadModule: SolidServerModuleLoader;
  renderComponent?: SolidHtmlComponentRenderer;
  resolveClientModule?: SolidClientModuleResolver;
}

export interface RenderSolidHtmlHostResult {
  html: string;
  modules: SolidHtmlHostModule[];
  clientModules: SolidHtmlHostClientModule[];
  diagnostics: SolidHtmlHostDiagnostic[];
}

export type SolidHostHydrateRenderer = (
  component: unknown,
  props: Record<string, unknown>,
  element: HTMLElement,
  slotHtml: string | undefined,
) => void | (() => void);

export interface CreateSolidHtmlHostHydrateInput {
  components: Readonly<Record<string, unknown>> | ReadonlyMap<string, unknown>;
  render: SolidHostHydrateRenderer;
}

export async function renderSolidHtmlHost(
  input: RenderSolidHtmlHostInput,
): Promise<RenderSolidHtmlHostResult> {
  const diagnostics: SolidHtmlHostDiagnostic[] = [];
  const modules = resolveHostModules(input, diagnostics);
  const byName = new Map(modules.map((module) => [module.name, module] as const));
  const cache = new Map<string, Promise<unknown>>();
  const renderComponent = input.renderComponent ?? defaultRenderComponent;

  const html = await applyIslandSsrHtml(
    input.html,
    async (name, props, _filePath, slotHtml) => {
      const module = byName.get(name);
      if (!module) {
        diagnostics.push({
          code: "missing-component",
          message: `Solid island "${name}" is not registered for this document.`,
          documentPath: input.documentPath,
          component: name,
        });
        return slotHtml ?? "";
      }
      const component = await loadComponent(input, module, cache, diagnostics);
      if (!component) {
        return slotHtml ?? "";
      }
      try {
        return await renderComponent(component, props, slotHtml || undefined, {
          component: name,
          moduleId: module.serverModuleId,
          documentPath: input.documentPath,
        });
      } catch (error) {
        diagnostics.push({
          code: "ssr-failed",
          message: `Solid island "${name}" failed to render: ${errorMessage(error)}`,
          documentPath: input.documentPath,
          component: name,
          moduleId: module.serverModuleId,
        });
        return slotHtml ?? "";
      }
    },
    input.documentPath,
    modules.map((module) => module.name),
  );

  return {
    html,
    modules,
    clientModules: modules.flatMap((module) =>
      module.clientModuleId ? [{ name: module.name, moduleId: module.clientModuleId }] : [],
    ),
    diagnostics,
  };
}

export function createSolidHtmlHostHydrate(
  input: CreateSolidHtmlHostHydrateInput,
): (element: HTMLElement, props: Record<string, unknown>) => void | (() => void) {
  return (element, props) => {
    const name = element.dataset.oxIsland;
    if (!name) {
      return undefined;
    }
    const component = componentFromRegistry(input.components, name);
    if (!component) {
      return undefined;
    }
    const slotHtml = readIslandSlotHtml(element);
    element.innerHTML = "";
    return input.render(component, props, element, slotHtml || undefined);
  };
}

function resolveHostModules(
  input: RenderSolidHtmlHostInput,
  diagnostics: SolidHtmlHostDiagnostic[],
): SolidHtmlHostModule[] {
  const names = collectMdxIslandNamesFromHtml(input.html);
  const local = resolveDocumentComponentImports({
    imports: input.imports ?? [],
    documentPath: input.documentPath,
    contentRoot: input.contentRoot ?? resolveContentRootPath(input),
    srcDir: input.srcDir,
  });
  for (const diagnostic of local.diagnostics) {
    diagnostics.push({ ...diagnostic, documentPath: input.documentPath });
  }

  const localBindings = new Map(local.bindings.map((binding) => [binding.localName, binding]));
  const modules: SolidHtmlHostModule[] = [];
  for (const name of names) {
    const localBinding = localBindings.get(name);
    const serverModuleId = localBinding
      ? localBinding.resolvedPath
      : componentPath(input.components ?? {}, name, input.root);
    if (!serverModuleId) {
      diagnostics.push({
        code: "missing-component",
        message: `Solid island "${name}" is not registered for this document.`,
        documentPath: input.documentPath,
        component: name,
      });
      continue;
    }
    const module: SolidHtmlHostModule = {
      name,
      serverModuleId,
      exportName: localBinding?.imported ?? "default",
      source: localBinding ? "document" : "components",
    };
    const clientModuleId = input.resolveClientModule?.(module, {
      documentPath: input.documentPath,
    });
    modules.push(clientModuleId ? { ...module, clientModuleId } : module);
  }
  return modules;
}

async function loadComponent(
  input: RenderSolidHtmlHostInput,
  module: SolidHtmlHostModule,
  cache: Map<string, Promise<unknown>>,
  diagnostics: SolidHtmlHostDiagnostic[],
): Promise<SolidComponentValue | undefined> {
  let pending = cache.get(module.serverModuleId);
  if (!pending) {
    pending = input.loadModule(module.serverModuleId);
    cache.set(module.serverModuleId, pending);
  }
  let exports: unknown;
  try {
    exports = await pending;
  } catch (error) {
    diagnostics.push({
      code: "module-load-failed",
      message: `Solid island module "${module.serverModuleId}" failed to load: ${errorMessage(error)}`,
      documentPath: input.documentPath,
      component: module.name,
      moduleId: module.serverModuleId,
    });
    return undefined;
  }
  const component = exportedValue(exports, module.exportName);
  if (!component) {
    diagnostics.push({
      code: "missing-export",
      message: `Solid island "${module.name}" could not find export "${module.exportName}".`,
      documentPath: input.documentPath,
      component: module.name,
      moduleId: module.serverModuleId,
    });
  }
  return component == null ? undefined : component;
}

function componentPath(
  components: ComponentsMap,
  name: string,
  root = process.cwd(),
): string | undefined {
  const specifier = components[name];
  if (!specifier) {
    return undefined;
  }
  return path.resolve(root, specifier.replace(/^\.\//, ""));
}

function exportedValue(exports: unknown, exportName: string): unknown {
  if (!exports || typeof exports !== "object") {
    return undefined;
  }
  return (exports as Record<string, unknown>)[exportName];
}

function componentFromRegistry(
  registry: CreateSolidHtmlHostHydrateInput["components"],
  name: string,
): unknown {
  return isReadonlyMap(registry) ? registry.get(name) : registry[name];
}

function isReadonlyMap(
  value: CreateSolidHtmlHostHydrateInput["components"],
): value is ReadonlyMap<string, unknown> {
  return typeof (value as ReadonlyMap<string, unknown>).get === "function";
}

async function defaultRenderComponent(
  component: unknown,
  props: Record<string, unknown>,
  slotHtml: string | undefined,
): Promise<string> {
  const [{ renderToString, ssr }, { createComponent }] = await Promise.all([
    import("@solidjs/web"),
    import("solid-js"),
  ]);
  const componentProps = slotHtml ? { ...props, children: ssr([slotHtml]) } : props;
  return renderToString(() => createComponent(component as never, componentProps as never));
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
