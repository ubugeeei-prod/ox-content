/**
 * Vite Plugin for Ox Content Svelte Integration
 *
 * Uses Vite's Environment API to enable embedding Svelte components in Markdown.
 */

import type { Plugin, PluginOption, ResolvedConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import { transformMarkdownWithSvelte } from "./transform";
import { createSvelteMarkdownEnvironment } from "./environment";
import { resolveComponentsGlob } from "./components";
import type {
  SvelteIntegrationOptions,
  ResolvedSvelteOptions,
  BuiltinEmbedOptions,
  SvelteCompilerWarning,
} from "./types";

const DEFAULT_MARKDOWN_EXTENSIONS = [".md", ".markdown", ".mdx"] as const;

function normalizeMarkdownExtensions(extensions?: readonly string[]): string[] {
  const values = extensions?.length ? extensions : DEFAULT_MARKDOWN_EXTENSIONS;
  return Array.from(
    new Map(
      values.map((extension) => {
        const value = extension.startsWith(".") ? extension : `.${extension}`;
        return [value.toLowerCase(), value] as const;
      }),
    ).values(),
  );
}

function isMarkdownFilePath(filePath: string, extensions: readonly string[]): boolean {
  const pathname = filePath.split("?")[0].split("#")[0].toLowerCase();
  return extensions.some((extension) => pathname.endsWith(extension.toLowerCase()));
}

function resolveBuiltinEmbedOptions(
  options: BuiltinEmbedOptions | false | undefined,
): ResolvedSvelteOptions["embeds"] {
  if (options === false) return { github: false, openGraph: false };
  return {
    github: resolveSingleEmbedOptions(options?.github),
    openGraph: resolveSingleEmbedOptions(options?.openGraph),
  };
}

function resolveSingleEmbedOptions<T extends object>(options: boolean | T | undefined): T | false {
  if (options === false) return false;
  if (options === true || options === undefined) return {} as T;
  return options;
}

export type {
  SvelteIntegrationOptions,
  ResolvedSvelteOptions,
  ComponentsOption,
  ComponentsMap,
  BuiltinEmbedOptions,
  MdxDocumentPropsOption,
  GitHubEmbedOptions,
  OpenGraphEmbedOptions,
  ResolvedBuiltinEmbedOptions,
  SvelteTransformResult,
  SvelteCompilerOption,
  SvelteCompilerWarning,
  SvelteCompilerOptions,
  SvelteCompilerResult,
  SvelteCompileFunction,
  ComponentIsland,
} from "./types";
export type {
  MdxImport,
  MdxImportSpecifier,
  MdxImportSpecifierKind,
} from "@ox-content/vite-plugin";
export {
  createSvelteHtmlHostHydrate,
  renderSvelteHtmlHost,
  type CreateSvelteHtmlHostHydrateInput,
  type RenderSvelteHtmlHostInput,
  type RenderSvelteHtmlHostResult,
  type SvelteClientModuleResolver,
  type SvelteHostHydrateRenderer,
  type SvelteHtmlComponentRenderer,
  type SvelteHtmlHostClientModule,
  type SvelteHtmlHostDiagnostic,
  type SvelteHtmlHostDiagnosticCode,
  type SvelteHtmlHostModule,
  type SvelteServerModuleLoader,
} from "./html-host";
export {
  SvelteHtmlHostRenderError,
  createSvelteHtmlHostRenderer,
  type CreateSvelteHtmlHostRendererInput,
  type SvelteHtmlHostRenderer,
  type SvelteHtmlHostRendererContext,
  type SvelteHtmlHostRendererDiagnosticPolicy,
} from "./html-host-renderer";
export {
  createSvelteHtmlHostDomRenderer,
  createSvelteHtmlHostLazyHydrate,
  initSvelteHtmlHost,
  loadSvelteHtmlHostDomRuntime,
  readSvelteHtmlHostSlot,
  type CreateSvelteHtmlHostLazyHydrateInput,
  type InitSvelteHtmlHostInput,
  type SvelteHtmlHostClientComponentValue,
  type SvelteHtmlHostClientContext,
  type SvelteHtmlHostClientDiagnosticCode,
  type SvelteHtmlHostClientError,
  type SvelteHtmlHostClientModuleLoader,
  type SvelteHtmlHostClientModules,
  type SvelteHtmlHostClientModuleValue,
  type SvelteHtmlHostClientRenderer,
  type SvelteHtmlHostClientRuntimeLoader,
  type SvelteHtmlHostDomMode,
  type SvelteHtmlHostDomRenderer,
  type SvelteHtmlHostDomRendererInput,
  type SvelteHtmlHostDomRuntime,
  type SvelteHtmlHostExportNameResolver,
  type SvelteHtmlHostInitIslands,
  type SvelteHtmlHostModuleIdResolver,
} from "./html-host-client";
export {
  SVELTE_HTML_HOST_MODULES_VIRTUAL_ID,
  createSvelteHtmlHostIslandRegistry,
  resolveSvelteHtmlHostIslandRegistry,
  toSvelteHtmlHostClientModuleId,
  type CreateSvelteHtmlHostIslandRegistryInput,
  type ResolvedSvelteHtmlHostIslandRegistry,
  type SvelteHtmlHostIslandDocument,
  type SvelteHtmlHostIslandEntry,
  type SvelteHtmlHostIslandRegistry,
  type SvelteHtmlHostIslandRegistryContext,
} from "./html-host-registry";
export {
  createSvelteHtmlHostCollectionDocuments,
  resolveSvelteHtmlHostCollectionDocuments,
  type SvelteHtmlHostCollectionDocument,
  type SvelteHtmlHostCollectionDocumentsOptions,
} from "./html-host-collection-documents";

/**
 * Creates the Ox Content Svelte integration plugin.
 *
 * Forwards core options such as `ssg`, `redirects`, `feeds`, and `siteMaps`.
 * The Svelte Markdown transform and environments replace the generic core
 * transform/`markdown` environment; other build plugins are kept.
 *
 * @example
 * ```ts
 * // vite.config.ts
 * import { defineConfig } from 'vite';
 * import { svelte } from '@sveltejs/vite-plugin-svelte';
 * import { oxContentSvelte } from 'vite-plugin-ox-content-svelte';
 *
 * export default defineConfig({
 *   plugins: [
 *     svelte(),
 *     oxContentSvelte({
 *       srcDir: 'docs',
 *       components: {
 *         Counter: './src/components/Counter.svelte',
 *       },
 *     }),
 *   ],
 * });
 * ```
 */
export function oxContentSvelte(options: SvelteIntegrationOptions = {}): PluginOption[] {
  const resolved = resolveSvelteOptions(options);
  let componentMap = new Map<string, string>();
  let config: ResolvedConfig;

  if (typeof options.components === "object" && !Array.isArray(options.components)) {
    componentMap = new Map(Object.entries(options.components));
  }

  const svelteTransformPlugin: Plugin = {
    name: "ox-content:svelte-transform",
    enforce: "pre",

    async configResolved(resolvedConfig) {
      config = resolvedConfig;

      const componentsOption = options.components;
      if (componentsOption) {
        const resolvedComponents = await resolveComponentsGlob(componentsOption, config.root);
        componentMap = new Map(Object.entries(resolvedComponents));
      }
    },

    async transform(code, id, transformOptions) {
      if (!isMarkdownFilePath(id, resolved.extensions)) {
        return null;
      }

      const result = await transformMarkdownWithSvelte(code, id, {
        ...resolved,
        components: Object.fromEntries(componentMap),
        root: config.root,
        renderIsland: options.renderIsland,
        ssr: transformOptions?.ssr,
      });

      for (const warning of result.warnings) {
        this.warn(formatSvelteCompilerWarning(warning));
      }

      return {
        code: result.code,
        map: result.map as never,
      };
    },
  };

  const svelteEnvironmentPlugin: Plugin = {
    name: "ox-content:svelte-environment",

    config() {
      return {
        environments: {
          oxcontent_ssr: createSvelteMarkdownEnvironment("ssr", resolved),
          oxcontent_client: createSvelteMarkdownEnvironment("client", resolved),
        },
      };
    },

    resolveId(id) {
      if (id === "virtual:ox-content-svelte/runtime") {
        return "\0virtual:ox-content-svelte/runtime";
      }
      if (id === "virtual:ox-content-svelte/components") {
        return "\0virtual:ox-content-svelte/components";
      }
      return null;
    },

    load(id) {
      if (id === "\0virtual:ox-content-svelte/runtime") {
        return generateRuntimeModule();
      }
      if (id === "\0virtual:ox-content-svelte/components") {
        return generateComponentsModule(componentMap);
      }
      return null;
    },

    applyToEnvironment(environment) {
      return ["oxcontent_ssr", "oxcontent_client", "client", "ssr"].includes(environment.name);
    },
  };

  const svelteHmrPlugin: Plugin = {
    name: "ox-content:svelte-hmr",
    apply: "serve",

    handleHotUpdate({ file, server, modules }) {
      const isComponent = Array.from(componentMap.values()).some((path) =>
        file.endsWith(path.replace(/^\.\//, "")),
      );

      if (isComponent) {
        const mdModules = Array.from(server.moduleGraph.idToModuleMap.values()).filter(
          (mod) => mod.file && isMarkdownFilePath(mod.file, resolved.extensions),
        );

        if (mdModules.length > 0) {
          server.ws.send({
            type: "custom",
            event: "ox-content:svelte-update",
            data: { file },
          });
          return [...modules, ...mdModules];
        }
      }

      return modules;
    },
  };

  const replacedCorePluginNames = new Set(["ox-content", "ox-content:environment"]);
  const corePlugins = (
    oxContent(options).flatMap((plugin) => (Array.isArray(plugin) ? plugin : [plugin])) as Plugin[]
  ).filter((plugin) => !replacedCorePluginNames.has(plugin.name));

  return [svelteTransformPlugin, svelteEnvironmentPlugin, svelteHmrPlugin, ...corePlugins];
}

function resolveSvelteOptions(
  options: SvelteIntegrationOptions,
): Omit<ResolvedSvelteOptions, "components"> {
  return {
    srcDir: options.srcDir ?? "docs",
    outDir: options.outDir ?? "dist",
    base: options.base ?? "/",
    extensions: normalizeMarkdownExtensions(options.extensions),
    gfm: options.gfm ?? true,
    autolinks: options.autolinks ?? options.gfm ?? true,
    frontmatter: options.frontmatter ?? true,
    toc: options.toc ?? true,
    tocMaxDepth: options.tocMaxDepth ?? 3,
    codeAnnotations: resolveCodeAnnotationsOptions(options.codeAnnotations),
    runes: options.runes ?? true,
    compiler: options.compiler,
    embeds: resolveBuiltinEmbedOptions(options.embeds),
    mdx: options.mdx,
    mdxDocumentProps: options.mdxDocumentProps ?? false,
  };
}

function formatSvelteCompilerWarning(warning: SvelteCompilerWarning): string {
  const code = warning.code ? `[${warning.code}] ` : "";
  const message = `${code}${warning.message}`;
  return warning.frame ? `${message}\n${warning.frame}` : message;
}

function resolveCodeAnnotationsOptions(
  options: SvelteIntegrationOptions["codeAnnotations"],
): ResolvedSvelteOptions["codeAnnotations"] {
  if (!options) {
    return {
      enabled: false,
      metaKey: "annotate",
    };
  }

  if (options === true) {
    return {
      enabled: true,
      metaKey: "annotate",
    };
  }

  return {
    enabled: true,
    metaKey: options.metaKey ?? "annotate",
  };
}

function generateRuntimeModule(): string {
  return `
// Svelte 5 runtime for ox-content
export { mount, unmount } from 'svelte';
`;
}

function generateComponentsModule(componentMap: Map<string, string>): string {
  const imports: string[] = [];
  const exports: string[] = [];

  componentMap.forEach((path, name) => {
    imports.push(`import ${name} from '${path}';`);
    exports.push(`  ${name},`);
  });

  return `
${imports.join("\n")}

export const components = {
${exports.join("\n")}
};

export default components;
`;
}

export { oxContent, renderHead } from "@ox-content/vite-plugin";
export type { HeadInput, RenderedHead } from "@ox-content/vite-plugin";
