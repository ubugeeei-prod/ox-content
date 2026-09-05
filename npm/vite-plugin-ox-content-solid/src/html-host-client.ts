import type {
  CreateSolidHtmlHostLazyHydrateInput,
  InitSolidHtmlHostInput,
  SolidHtmlHostClientDiagnosticCode,
  SolidHtmlHostClientError,
  SolidHtmlHostClientModuleLoader,
  SolidHtmlHostClientModules,
} from "./html-host-client-types";

export type {
  CreateSolidHtmlHostLazyHydrateInput,
  InitSolidHtmlHostInput,
  SolidHtmlHostClientContext,
  SolidHtmlHostClientDiagnosticCode,
  SolidHtmlHostClientError,
  SolidHtmlHostClientComponentValue,
  SolidHtmlHostClientModuleLoader,
  SolidHtmlHostClientModules,
  SolidHtmlHostClientModuleValue,
  SolidHtmlHostClientRenderer,
  SolidHtmlHostClientRuntimeLoader,
  SolidHtmlHostExportNameResolver,
  SolidHtmlHostInitIslands,
  SolidHtmlHostModuleIdResolver,
} from "./html-host-client-types";

const ISLAND_JSON_SCRIPT = /^\s*<script type="application\/json">[\s\S]*?<\/script>/;

export function initSolidHtmlHost<TRuntime = undefined>(
  input: InitSolidHtmlHostInput<TRuntime>,
): ReturnType<InitSolidHtmlHostInput<TRuntime>["initIslands"]> {
  return input.initIslands(createSolidHtmlHostLazyHydrate(input), input.options);
}

export function createSolidHtmlHostLazyHydrate<TRuntime = undefined>(
  input: CreateSolidHtmlHostLazyHydrateInput<TRuntime>,
): (element: HTMLElement, props: Record<string, unknown>) => () => void {
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
    const slotHtml = readSolidHtmlHostSlot(element);
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
          input,
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

      element.innerHTML = "";
      try {
        const cleanup = input.render({
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

export function readSolidHtmlHostSlot(
  element: Pick<HTMLElement, "dataset" | "innerHTML">,
): string | undefined {
  const fromAttr = element.dataset.oxContent;
  if (fromAttr) return fromAttr;
  if (element.dataset.oxSsr === "true") return undefined;

  const slotHtml = element.innerHTML.replace(ISLAND_JSON_SCRIPT, "");
  return slotHtml || undefined;
}

async function loadClientModule(
  modules: SolidHtmlHostClientModules,
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
  input: CreateSolidHtmlHostLazyHydrateInput<TRuntime>,
  getCached: () => Promise<TRuntime> | undefined,
  setCached: (pending: Promise<TRuntime> | undefined) => void,
): Promise<TRuntime | undefined> {
  const load = input.loadRuntime;
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

function moduleLoader(
  modules: SolidHtmlHostClientModules,
  moduleId: string,
): SolidHtmlHostClientModuleLoader | undefined {
  return isReadonlyMap(modules) ? modules.get(moduleId) : modules[moduleId];
}

function isReadonlyMap(
  value: SolidHtmlHostClientModules,
): value is ReadonlyMap<string, SolidHtmlHostClientModuleLoader> {
  return typeof (value as ReadonlyMap<string, SolidHtmlHostClientModuleLoader>).get === "function";
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

function reportError(
  input: Pick<CreateSolidHtmlHostLazyHydrateInput, "onError">,
  error: SolidHtmlHostClientError,
): void {
  error.element.classList?.add("ox-island-error");
  error.element.dataset.oxError = error.message;
  input.onError?.(error);

  if (typeof CustomEvent === "function" && typeof error.element.dispatchEvent === "function") {
    error.element.dispatchEvent(
      new CustomEvent("ox-content-solid-html-host:error", { detail: error }),
    );
  }
}

function clientError(
  code: SolidHtmlHostClientDiagnosticCode,
  element: HTMLElement,
  props: Record<string, unknown>,
  context: {
    componentName?: string;
    moduleId?: string;
    exportName?: string;
    cause?: unknown;
  } = {},
): SolidHtmlHostClientError {
  return {
    code,
    element,
    props,
    ...context,
    message: clientErrorMessage(code, context),
  };
}

function clientErrorMessage(
  code: SolidHtmlHostClientDiagnosticCode,
  context: { componentName?: string; moduleId?: string; exportName?: string; cause?: unknown },
): string {
  const component = context.componentName
    ? `Solid island "${context.componentName}"`
    : "Solid island";
  const reason = causeMessage(context.cause);
  switch (code) {
    case "missing-island-name":
      return "Solid island element is missing data-ox-island.";
    case "missing-module-id":
      return `${component} is missing data-ox-module.`;
    case "unknown-module":
      return `${component} references unknown module "${context.moduleId ?? ""}".`;
    case "module-load-failed":
      return `${component} module "${context.moduleId ?? ""}" failed to load: ${reason}`;
    case "runtime-load-failed":
      return `Solid runtime failed to load: ${reason}`;
    case "missing-export":
      return `${component} module "${context.moduleId ?? ""}" is missing export "${context.exportName ?? "default"}".`;
    case "render-failed":
      return `${component} failed to render: ${reason}`;
  }
}

function causeMessage(cause: unknown): string {
  if (cause == null) return "";
  if (cause instanceof Error) return cause.message;
  if (typeof cause === "string") return cause;
  if (typeof cause === "number" || typeof cause === "boolean") return cause.toString();
  try {
    return JSON.stringify(cause);
  } catch {
    return Object.prototype.toString.call(cause);
  }
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
