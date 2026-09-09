import type { InitIslandsOptions } from "@ox-content/islands";

export type SvelteHtmlHostClientDiagnosticCode =
  | "missing-island-name"
  | "missing-module-id"
  | "unknown-module"
  | "module-load-failed"
  | "runtime-load-failed"
  | "missing-export"
  | "render-failed";

export interface SvelteHtmlHostClientError {
  code: SvelteHtmlHostClientDiagnosticCode;
  message: string;
  element: HTMLElement;
  props: Record<string, unknown>;
  componentName?: string;
  moduleId?: string;
  exportName?: string;
  cause?: unknown;
}

export type SvelteHtmlHostClientComponentValue = unknown;
export type SvelteHtmlHostClientModuleValue = SvelteHtmlHostClientComponentValue;

export type SvelteHtmlHostClientModuleLoader<TModule = SvelteHtmlHostClientModuleValue> = () =>
  | TModule
  | PromiseLike<TModule>;

export type SvelteHtmlHostClientModules =
  | Readonly<Record<string, SvelteHtmlHostClientModuleLoader>>
  | ReadonlyMap<string, SvelteHtmlHostClientModuleLoader>;

export interface SvelteHtmlHostClientContext<TRuntime = undefined> {
  component: unknown;
  componentName: string;
  element: HTMLElement;
  exportName: string;
  moduleExports: unknown;
  moduleId: string;
  props: Record<string, unknown>;
  runtime: TRuntime | undefined;
  slotHtml: string | undefined;
}

export type SvelteHtmlHostClientRenderer<TRuntime = undefined> = (
  context: SvelteHtmlHostClientContext<TRuntime>,
) => void | (() => void) | PromiseLike<void | (() => void)>;

export type SvelteHtmlHostClientRuntimeLoader<TRuntime = undefined> = () =>
  | TRuntime
  | Promise<TRuntime>;

export type SvelteHtmlHostModuleIdResolver = (
  element: HTMLElement,
  context: { componentName: string; props: Record<string, unknown> },
) => string | undefined;

export type SvelteHtmlHostExportNameResolver = (
  element: HTMLElement,
  context: { componentName: string; moduleId: string; props: Record<string, unknown> },
) => string | undefined;

export interface SvelteHtmlHostClientBaseInput<TRuntime = undefined> {
  modules: SvelteHtmlHostClientModules;
  loadRuntime?: SvelteHtmlHostClientRuntimeLoader<TRuntime>;
  resolveModuleId?: SvelteHtmlHostModuleIdResolver;
  resolveExportName?: SvelteHtmlHostExportNameResolver;
  onError?: (error: SvelteHtmlHostClientError) => void;
}

export interface CreateSvelteHtmlHostLazyHydrateRenderInput<
  TRuntime = undefined,
> extends SvelteHtmlHostClientBaseInput<TRuntime> {
  render: SvelteHtmlHostClientRenderer<TRuntime>;
  mount?: never;
}

export interface CreateSvelteHtmlHostLazyHydrateMountInput extends SvelteHtmlHostClientBaseInput<SvelteHtmlHostDomRuntime> {
  mount: SvelteHtmlHostDomRendererInput;
  render?: never;
}

export type CreateSvelteHtmlHostLazyHydrateInput<TRuntime = undefined> =
  | CreateSvelteHtmlHostLazyHydrateRenderInput<TRuntime>
  | CreateSvelteHtmlHostLazyHydrateMountInput;

export type SvelteHtmlHostDomMode = "render" | "hydrate";

export interface SvelteHtmlHostDomRendererInput {
  mode: SvelteHtmlHostDomMode;
}

export interface SvelteHtmlHostDomRuntime {
  mount: (
    component: SvelteHtmlHostClientComponentValue,
    options: { target: HTMLElement; props?: Record<string, unknown> },
  ) => unknown;
  hydrate: (
    component: SvelteHtmlHostClientComponentValue,
    options: { target: HTMLElement; props?: Record<string, unknown> },
  ) => unknown;
  unmount: (instance: unknown) => void | Promise<void>;
  createRawSnippet: (
    factory: () => {
      render: () => string;
      setup?: (element: Element) => void;
    },
  ) => unknown;
}

export type SvelteHtmlHostInitIslands<TController = unknown> = (
  hydrate: (element: HTMLElement, props: Record<string, unknown>) => void | (() => void),
  options?: InitIslandsOptions,
) => TController;

export type InitSvelteHtmlHostInput<TRuntime = undefined> =
  CreateSvelteHtmlHostLazyHydrateInput<TRuntime> & {
    initIslands: SvelteHtmlHostInitIslands;
    options?: InitIslandsOptions;
  };
