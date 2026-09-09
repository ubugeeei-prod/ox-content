import type {
  CreateSvelteHtmlHostLazyHydrateInput,
  InitSvelteHtmlHostInput,
  SvelteHtmlHostClientModuleLoader,
  SvelteHtmlHostClientModules,
  SvelteHtmlHostClientRenderer,
  SvelteHtmlHostClientRuntimeLoader,
} from "./html-host-client-types";
import {
  createSvelteHtmlHostDomRenderer,
  loadSvelteHtmlHostDomRuntime,
  svelteHtmlHostDomRendererMode,
} from "./html-host-dom-renderer";
import { clientError, reportError } from "./html-host-client-errors";

export type {
  CreateSvelteHtmlHostLazyHydrateInput,
  InitSvelteHtmlHostInput,
  SvelteHtmlHostClientContext,
  SvelteHtmlHostClientDiagnosticCode,
  SvelteHtmlHostClientError,
  SvelteHtmlHostClientComponentValue,
  SvelteHtmlHostClientModuleLoader,
  SvelteHtmlHostClientModules,
  SvelteHtmlHostClientModuleValue,
  SvelteHtmlHostClientRenderer,
  SvelteHtmlHostClientRuntimeLoader,
  SvelteHtmlHostDomMode,
  SvelteHtmlHostDomRendererInput,
  SvelteHtmlHostDomRuntime,
  SvelteHtmlHostExportNameResolver,
  SvelteHtmlHostInitIslands,
  SvelteHtmlHostModuleIdResolver,
} from "./html-host-client-types";
export {
  createSvelteHtmlHostDomRenderer,
  loadSvelteHtmlHostDomRuntime,
  type SvelteHtmlHostDomRenderer,
} from "./html-host-dom-renderer";

const ISLAND_JSON_SCRIPT = /^\s*<script type="application\/json">[\s\S]*?<\/script>/;

export function initSvelteHtmlHost<TRuntime = undefined>(
  input: InitSvelteHtmlHostInput<TRuntime>,
): ReturnType<InitSvelteHtmlHostInput<TRuntime>["initIslands"]> {
  return input.initIslands(createSvelteHtmlHostLazyHydrate(input), input.options);
}

export function createSvelteHtmlHostLazyHydrate<TRuntime = undefined>(
  input: CreateSvelteHtmlHostLazyHydrateInput<TRuntime>,
): (element: HTMLElement, props: Record<string, unknown>) => () => void {
  const render = resolveRenderer(input);
  const load = resolveRuntimeLoader(input, render);
  const moduleCache = new Map<string, Promise<unknown>>();
  let runtimeCache: Promise<TRuntime> | undefined;

  return (element, props) => {
    const componentName = element.dataset.oxIsland;
    if (!componentName) {
      reportError(input, clientError("missing-island-name", element, props));
      return noop;
    }

    const moduleId =
      input.resolveModuleId?.(element, { componentName, props }) ?? element.dataset.oxModule;
    if (!moduleId) {
      reportError(input, clientError("missing-module-id", element, props, { componentName }));
      return noop;
    }
    if (!moduleLoader(input.modules, moduleId)) {
      reportError(
        input,
        clientError("unknown-module", element, props, {
          componentName,
          moduleId,
        }),
      );
      return noop;
    }

    const exportName =
      input.resolveExportName?.(element, { componentName, moduleId, props }) ??
      element.dataset.oxExport ??
      "default";
    const slotHtml = readSvelteHtmlHostSlot(element);
    let disposed = false;
    let disposeMounted: (() => void) | undefined;

    const dispose = () => {
      if (disposed) return;
      disposed = true;
      disposeMounted?.();
      disposeMounted = undefined;
    };

    void (async () => {
      let moduleExports: unknown;
      let runtime: TRuntime | undefined;
      try {
        moduleExports = await loadClientModule(input.modules, moduleId, moduleCache);
      } catch (cause) {
        if (!disposed) {
          reportError(
            input,
            clientError("module-load-failed", element, props, {
              componentName,
              moduleId,
              exportName,
              cause,
            }),
          );
        }
        return;
      }

      if (disposed) return;

      try {
        runtime = await loadRuntime(
          load,
          () => runtimeCache,
          (pending) => {
            runtimeCache = pending;
          },
        );
      } catch (cause) {
        if (!disposed) {
          reportError(
            input,
            clientError("runtime-load-failed", element, props, {
              componentName,
              moduleId,
              exportName,
              cause,
            }),
          );
        }
        return;
      }

      if (disposed) return;

      const component = exportedValue(moduleExports, exportName);
      if (component == null) {
        reportError(
          input,
          clientError("missing-export", element, props, {
            componentName,
            moduleId,
            exportName,
            cause: new Error(`Export "${exportName}" was not found.`),
          }),
        );
        return;
      }

      if (!preservesElementContents(input, render)) {
        element.innerHTML = "";
      }
      try {
        const cleanup = await render({
          component,
          componentName,
          element,
          exportName,
          moduleExports,
          moduleId,
          props,
          runtime,
          slotHtml,
        });
        disposeMounted = cleanup ? once(cleanup) : undefined;
        if (disposed) {
          disposeMounted?.();
          disposeMounted = undefined;
        }
      } catch (cause) {
        if (!disposed) {
          reportError(
            input,
            clientError("render-failed", element, props, {
              componentName,
              moduleId,
              exportName,
              cause,
            }),
          );
        }
      }
    })();

    return dispose;
  };
}

export function readSvelteHtmlHostSlot(
  element: Pick<HTMLElement, "dataset" | "innerHTML">,
): string | undefined {
  const fromAttr = element.dataset.oxContent;
  if (fromAttr) return fromAttr;
  if (element.dataset.oxSsr === "true") return undefined;

  const slotHtml = element.innerHTML.replace(ISLAND_JSON_SCRIPT, "");
  return slotHtml || undefined;
}

async function loadClientModule(
  modules: SvelteHtmlHostClientModules,
  moduleId: string,
  cache: Map<string, Promise<unknown>>,
): Promise<unknown> {
  const cached = cache.get(moduleId);
  if (cached) return cached;

  const loader = moduleLoader(modules, moduleId);
  if (!loader) {
    throw new Error(`Unknown module "${moduleId}".`);
  }

  const pending = Promise.resolve()
    .then(loader)
    .catch((cause: unknown) => {
      cache.delete(moduleId);
      throw cause;
    });
  cache.set(moduleId, pending);
  return pending;
}

async function loadRuntime<TRuntime>(
  load: SvelteHtmlHostClientRuntimeLoader<TRuntime> | undefined,
  getCached: () => Promise<TRuntime> | undefined,
  setCached: (pending: Promise<TRuntime> | undefined) => void,
): Promise<TRuntime | undefined> {
  if (!load) return undefined;

  const cached = getCached();
  if (cached) return cached;

  const pending = Promise.resolve()
    .then(load)
    .catch((cause: unknown) => {
      setCached(undefined);
      throw cause;
    });
  setCached(pending);
  return pending;
}

function resolveRenderer<TRuntime>(
  input: CreateSvelteHtmlHostLazyHydrateInput<TRuntime>,
): SvelteHtmlHostClientRenderer<TRuntime> {
  if (input.render) return input.render;
  if (input.mount) {
    return createSvelteHtmlHostDomRenderer(input.mount) as SvelteHtmlHostClientRenderer<TRuntime>;
  }
  throw new Error("initSvelteHtmlHost requires either render or mount.");
}

function resolveRuntimeLoader<TRuntime>(
  input: CreateSvelteHtmlHostLazyHydrateInput<TRuntime>,
  render: SvelteHtmlHostClientRenderer<TRuntime>,
): SvelteHtmlHostClientRuntimeLoader<TRuntime> | undefined {
  if (input.loadRuntime) {
    return input.loadRuntime as SvelteHtmlHostClientRuntimeLoader<TRuntime>;
  }
  if (
    input.mount ||
    svelteHtmlHostDomRendererMode(render as SvelteHtmlHostClientRenderer<unknown>)
  ) {
    return loadSvelteHtmlHostDomRuntime as SvelteHtmlHostClientRuntimeLoader<TRuntime>;
  }
  return undefined;
}

function preservesElementContents<TRuntime>(
  input: CreateSvelteHtmlHostLazyHydrateInput<TRuntime>,
  render: SvelteHtmlHostClientRenderer<TRuntime>,
): boolean {
  const mode =
    input.mount?.mode ??
    svelteHtmlHostDomRendererMode(render as unknown as SvelteHtmlHostClientRenderer<unknown>);
  return mode === "hydrate";
}

function moduleLoader(
  modules: SvelteHtmlHostClientModules,
  moduleId: string,
): SvelteHtmlHostClientModuleLoader | undefined {
  return isReadonlyMap(modules) ? modules.get(moduleId) : modules[moduleId];
}

function isReadonlyMap(
  value: SvelteHtmlHostClientModules,
): value is ReadonlyMap<string, SvelteHtmlHostClientModuleLoader> {
  return typeof (value as ReadonlyMap<string, SvelteHtmlHostClientModuleLoader>).get === "function";
}

function exportedValue(moduleExports: unknown, exportName: string): unknown {
  if (exportName === "default" && typeof moduleExports === "function") {
    return moduleExports;
  }
  if (!moduleExports || typeof moduleExports !== "object") {
    return undefined;
  }
  return (moduleExports as Record<string, unknown>)[exportName];
}

function once(cleanup: () => void): () => void {
  let called = false;
  return () => {
    if (called) return;
    called = true;
    cleanup();
  };
}

function noop(): void {}
