import {
  createHtmlHostIslandRegistry,
  createHtmlHostRenderer,
  renderHtmlHostMarkdown,
  type CreateHtmlHostIslandRegistryInput,
  type CreateHtmlHostRendererInput,
  type RenderHtmlHostMarkdownInput,
} from "@ox-content/vite-plugin/html-host";
import type { Plugin } from "vite";
import { createHonoServerAdapter, type HonoServerOptions } from "./server.js";

export const HONO_MODULES_ID = "virtual:ox-content-hono/modules";
type HonoJsxOptions = Omit<CreateHtmlHostIslandRegistryInput, "virtualModuleId" | "pluginName">;

/** Uses Ox Content's discovery, virtual modules and invalidation unchanged. */
export function honoJsx(options: HonoJsxOptions = {}): Plugin[] {
  return createHonoJsxPlugins(options).plugins;
}

export function createHonoJsxPlugins(options: HonoJsxOptions) {
  const registry = createHtmlHostIslandRegistry({
    ...options,
    virtualModuleId: HONO_MODULES_ID,
    pluginName: "ox-content:hono-islands",
    collectionDocuments: options.collectionDocuments ?? {
      oxContent: options.oxContent,
    },
  });
  const plugins: Plugin[] = [
    {
      name: "ox-content:hono-jsx",
      config() {
        return {
          oxc: { jsx: { runtime: "automatic", importSource: "hono/jsx" } },
          resolve: { dedupe: ["hono", "@ox-content/vite-plugin-hono"] },
          ssr: { noExternal: ["@ox-content/vite-plugin-hono", "hono"] },
        };
      },
    },
    registry.plugin,
  ];
  return { plugins, invalidateRegistry: () => registry.invalidate() };
}

export function createHonoRenderer(
  input: Omit<CreateHtmlHostRendererInput, "adapter"> & HonoServerOptions,
) {
  const { frontmatter, ...host } = input;
  return createHtmlHostRenderer({
    ...host,
    adapter: createHonoServerAdapter({ frontmatter }),
  });
}

/** Connect a custom host render callback, including Ox Content CSS/assets. */
export function renderHonoMarkdown(
  input: Omit<RenderHtmlHostMarkdownInput, "renderIslands"> & HonoServerOptions,
) {
  const { context, frontmatter, ...rest } = input;
  return renderHtmlHostMarkdown({
    ...rest,
    context,
    renderIslands: createHonoRenderer({
      loadModule: (id) => context.loadModule(id),
      frontmatter,
    }),
  });
}
