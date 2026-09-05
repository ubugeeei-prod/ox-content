/**
 * Vite Plugin for Ox Content Solid Integration
 *
 * Uses Vite's Environment API to enable embedding Solid components in Markdown.
 */

import type { Plugin, PluginOption, ResolvedConfig, ViteDevServer } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import { resolveComponentsGlob } from "./components";
import {
  createSolidMarkdownEnvironment,
  createSolidRuntimeResolveConditions,
  detectSolidMarkdownRuntime,
  mergeSolidResolveConditions,
} from "./environment";
import { isMarkdownFilePath, resolveSolidOptions } from "./options";
import { transformMarkdownWithSolid } from "./transform";
import {
  formatSolidPluginError,
  TRANSFORM_PLUGIN_NAME,
  UNCOMPILED_JSX_MARKER,
  verifySolidPluginOrder,
} from "./verify";
import type { SolidIntegrationOptions } from "./types";

export type {
  SolidIntegrationOptions,
  ResolvedSolidOptions,
  ComponentsOption,
  ComponentsMap,
  BuiltinEmbedOptions,
  GitHubEmbedOptions,
  OpenGraphEmbedOptions,
  ResolvedBuiltinEmbedOptions,
  SolidTransformResult,
  ComponentIsland,
} from "./types";
export type {
  MdxImport,
  MdxImportSpecifier,
  MdxImportSpecifierKind,
} from "@ox-content/vite-plugin";
export {
  createSolidHtmlHostHydrate,
  renderSolidHtmlHost,
  type CreateSolidHtmlHostHydrateInput,
  type RenderSolidHtmlHostInput,
  type RenderSolidHtmlHostResult,
  type SolidClientModuleResolver,
  type SolidHostHydrateRenderer,
  type SolidHtmlComponentRenderer,
  type SolidHtmlHostClientModule,
  type SolidHtmlHostDiagnostic,
  type SolidHtmlHostDiagnosticCode,
  type SolidHtmlHostModule,
  type SolidServerModuleLoader,
} from "./html-host";
export {
  createSolidHtmlHostLazyHydrate,
  initSolidHtmlHost,
  readSolidHtmlHostSlot,
  type CreateSolidHtmlHostLazyHydrateInput,
  type InitSolidHtmlHostInput,
  type SolidHtmlHostClientComponentValue,
  type SolidHtmlHostClientContext,
  type SolidHtmlHostClientDiagnosticCode,
  type SolidHtmlHostClientError,
  type SolidHtmlHostClientModuleLoader,
  type SolidHtmlHostClientModules,
  type SolidHtmlHostClientModuleValue,
  type SolidHtmlHostClientRenderer,
  type SolidHtmlHostClientRuntimeLoader,
  type SolidHtmlHostExportNameResolver,
  type SolidHtmlHostInitIslands,
  type SolidHtmlHostModuleIdResolver,
} from "./html-host-client";
export {
  resolveSolidIslandStylesheets,
  type ResolveSolidIslandStylesheetsInput,
  type ResolveSolidIslandStylesheetsResult,
  type SolidDevModuleGraph,
  type SolidDevModuleNode,
  type SolidIslandStylesheet,
  type SolidIslandStylesheetDiagnostic,
  type SolidStylesheetManifest,
  type SolidStylesheetManifestChunk,
} from "./stylesheets";

/**
 * Creates the Ox Content Solid integration plugin.
 *
 * Unlike the React and Svelte integrations, this plugin must be listed **before**
 * `@solidjs/vite-plugin`, and that plugin must be told about the Markdown
 * extensions. Markdown is turned into Solid JSX here, and Solid's JSX is
 * compile-time only — the Solid 2 Vite plugin is what turns it into DOM or SSR
 * instructions through its native compiler.
 *
 * @example
 * ```ts
 * // vite.config.ts
 * import { defineConfig } from 'vite';
 * import solid from '@solidjs/vite-plugin';
 * import { oxContentSolid } from '@ox-content/vite-plugin-solid';
 *
 * export default defineConfig({
 *   plugins: [
 *     oxContentSolid({
 *       srcDir: 'docs',
 *       components: {
 *         Counter: './src/components/Counter.tsx',
 *       },
 *     }),
 *     solid({ extensions: ['.md', '.markdown', '.mdx'], compiler: 'native' }),
 *   ],
 * });
 * ```
 */
export function oxContentSolid(options: SolidIntegrationOptions = {}): PluginOption[] {
  const resolved = resolveSolidOptions(options);
  let componentMap = new Map<string, string>();
  let config: ResolvedConfig;

  if (typeof options.components === "object" && !Array.isArray(options.components)) {
    componentMap = new Map(Object.entries(options.components));
  }

  const solidTransformPlugin: Plugin = {
    name: TRANSFORM_PLUGIN_NAME,
    enforce: "pre",

    async configResolved(resolvedConfig) {
      config = resolvedConfig;

      if (resolved.verifySolidPlugin) {
        verifySolidPluginOrder(resolvedConfig);
      }

      const componentsOption = options.components;
      if (componentsOption) {
        const resolvedComponents = await resolveComponentsGlob(componentsOption, config.root);
        componentMap = new Map(Object.entries(resolvedComponents));
      }
    },

    async transform(code, id) {
      if (!isMarkdownFilePath(id, resolved.extensions)) {
        return null;
      }

      const result = await transformMarkdownWithSolid(code, id, {
        ...resolved,
        components: Object.fromEntries(componentMap),
        root: config.root,
        renderIsland: options.renderIsland,
      });

      return {
        code: result.code,
        map: result.map,
      };
    },
  };

  // `post` so every `pre`/normal plugin — @solidjs/vite-plugin included — has
  // already had its turn at the module.
  const solidVerifyPlugin: Plugin = {
    name: "ox-content:solid-verify",
    enforce: "post",

    transform(code, id) {
      if (!resolved.verifySolidPlugin) return null;
      if (!isMarkdownFilePath(id, resolved.extensions)) return null;
      if (!code.includes(UNCOMPILED_JSX_MARKER)) return null;

      this.error(formatSolidPluginError("extensions"));
    },
  };

  const solidEnvironmentPlugin: Plugin = {
    name: "ox-content:solid-environment",

    config() {
      const envOptions = {
        ...resolved,
        components: Object.fromEntries(componentMap),
      };
      return {
        environments: {
          oxcontent_ssr: createSolidMarkdownEnvironment("ssr", envOptions),
          oxcontent_client: createSolidMarkdownEnvironment("client", envOptions),
        },
      };
    },

    configEnvironment(name, environmentOptions) {
      if (name !== "oxcontent_ssr" && name !== "oxcontent_client") {
        return;
      }
      const runtime = detectSolidMarkdownRuntime();
      const conditions =
        name === "oxcontent_ssr"
          ? ["solid", ...createSolidRuntimeResolveConditions(runtime), "node", "import"]
          : ["solid", "browser", "import"];
      return {
        resolve: {
          ...environmentOptions.resolve,
          conditions: mergeSolidResolveConditions(
            environmentOptions.resolve?.conditions,
            conditions,
          ),
        },
      };
    },

    resolveId(id) {
      if (id === "virtual:ox-content-solid/components") {
        return "\0virtual:ox-content-solid/components";
      }
      return null;
    },

    load(id) {
      if (id === "\0virtual:ox-content-solid/components") {
        return generateComponentsModule(componentMap);
      }
      return null;
    },

    applyToEnvironment(environment) {
      return ["oxcontent_ssr", "oxcontent_client", "client", "ssr"].includes(environment.name);
    },
  };

  const solidHmrPlugin: Plugin = {
    name: "ox-content:solid-hmr",
    apply: "serve",

    hotUpdate({ file, modules }) {
      if (!isRegisteredComponentFile(componentMap, file)) {
        return modules;
      }

      const mdModules = markdownModulesFromMap(
        this.environment.moduleGraph.idToModuleMap,
        resolved.extensions,
      );
      if (mdModules.length > 0) {
        this.environment.hot.send({
          type: "custom",
          event: "ox-content:solid-update",
          data: { file },
        });
        return [...modules, ...mdModules];
      }

      return modules;
    },

    handleHotUpdate({ file, server, modules }) {
      if (hasEnvironmentApiServer(server)) {
        return;
      }

      if (isRegisteredComponentFile(componentMap, file)) {
        const mdModules = Array.from(server.moduleGraph.idToModuleMap.values()).filter(
          (mod) => mod.file && isMarkdownFilePath(mod.file, resolved.extensions),
        );

        if (mdModules.length > 0) {
          server.ws.send({
            type: "custom",
            event: "ox-content:solid-update",
            data: { file },
          });
          return [...modules, ...mdModules];
        }
      }

      return modules;
    },
  };

  const basePlugins = oxContent(options).flatMap((plugin) =>
    Array.isArray(plugin) ? plugin : [plugin],
  ) as Plugin[];
  const environmentPlugin = basePlugins.find((plugin) => plugin.name === "ox-content:environment");
  const plugins: Plugin[] = [
    solidTransformPlugin,
    solidVerifyPlugin,
    solidEnvironmentPlugin,
    solidHmrPlugin,
  ];

  if (environmentPlugin) {
    plugins.push(environmentPlugin);
  }

  return plugins;
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

function isRegisteredComponentFile(
  componentMap: ReadonlyMap<string, string>,
  file: string,
): boolean {
  return Array.from(componentMap.values()).some((componentPath) =>
    file.endsWith(componentPath.replace(/^\.\//, "")),
  );
}

function markdownModulesFromMap<T extends { file?: string | null }>(
  moduleMap: ReadonlyMap<string, T>,
  extensions: readonly string[],
): T[] {
  return Array.from(moduleMap.values()).filter(
    (mod) => mod.file && isMarkdownFilePath(mod.file, extensions),
  );
}

function hasEnvironmentApiServer(server: ViteDevServer): boolean {
  return Boolean((server as { environments?: unknown }).environments);
}

export { oxContent } from "@ox-content/vite-plugin";
