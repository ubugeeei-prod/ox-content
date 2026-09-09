import { toSvelteHtmlHostClientModuleId } from "./html-host-registry-paths";
import {
  renderSvelteHtmlHost,
  type RenderSvelteHtmlHostResult,
  type SvelteClientModuleResolver,
  type SvelteHtmlComponentRenderer,
  type SvelteHtmlHostDiagnostic,
  type SvelteServerModuleLoader,
} from "./html-host";
import type { MdxImport } from "@ox-content/vite-plugin";
import type { ComponentsMap } from "./types";

export type SvelteHtmlHostRendererDiagnosticPolicy = "throw" | "collect";

export interface CreateSvelteHtmlHostRendererInput {
  root?: string;
  srcDir?: string;
  contentRoot?: string;
  components?: ComponentsMap;
  loadModule: SvelteServerModuleLoader;
  renderComponent?: SvelteHtmlComponentRenderer;
  resolveClientModule?: SvelteClientModuleResolver;
  diagnostics?: SvelteHtmlHostRendererDiagnosticPolicy;
}

export interface SvelteHtmlHostRendererContext {
  documentPath: string;
  imports?: readonly MdxImport[];
  root?: string;
  srcDir?: string;
  contentRoot?: string;
  components?: ComponentsMap;
  renderComponent?: SvelteHtmlComponentRenderer;
  resolveClientModule?: SvelteClientModuleResolver;
}

export type SvelteHtmlHostRenderer = (
  html: string,
  context: SvelteHtmlHostRendererContext,
) => Promise<RenderSvelteHtmlHostResult>;

export class SvelteHtmlHostRenderError extends Error {
  readonly diagnostics: SvelteHtmlHostDiagnostic[];

  constructor(diagnostics: readonly SvelteHtmlHostDiagnostic[]) {
    super(formatSvelteHtmlHostDiagnostics(diagnostics));
    this.name = "SvelteHtmlHostRenderError";
    this.diagnostics = [...diagnostics];
  }
}

export function createSvelteHtmlHostRenderer(
  input: CreateSvelteHtmlHostRendererInput,
): SvelteHtmlHostRenderer {
  const policy = input.diagnostics ?? "throw";

  return async (html, context) => {
    const root = context.root ?? input.root;
    const result = await renderSvelteHtmlHost({
      html,
      documentPath: context.documentPath,
      root,
      srcDir: context.srcDir ?? input.srcDir,
      contentRoot: context.contentRoot ?? input.contentRoot,
      imports: context.imports,
      components: context.components ?? input.components,
      loadModule: input.loadModule,
      renderComponent: context.renderComponent ?? input.renderComponent,
      resolveClientModule:
        context.resolveClientModule ??
        input.resolveClientModule ??
        ((module) => toSvelteHtmlHostClientModuleId(module.serverModuleId, root)),
    });

    if (policy === "throw" && result.diagnostics.length > 0) {
      throw new SvelteHtmlHostRenderError(result.diagnostics);
    }

    return result;
  };
}

function formatSvelteHtmlHostDiagnostics(diagnostics: readonly SvelteHtmlHostDiagnostic[]): string {
  if (diagnostics.length === 0) {
    return "Svelte HTML host rendering failed.";
  }
  return diagnostics
    .map((diagnostic) => {
      const details = [
        diagnostic.documentPath,
        diagnostic.component && `component ${diagnostic.component}`,
        diagnostic.moduleId && `module ${diagnostic.moduleId}`,
      ].filter(Boolean);
      return `${diagnostic.code}: ${diagnostic.message}${
        details.length > 0 ? ` (${details.join(", ")})` : ""
      }`;
    })
    .join("\n");
}
