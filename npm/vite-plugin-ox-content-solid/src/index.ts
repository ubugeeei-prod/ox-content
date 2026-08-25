/**
 * Vite Plugin for Ox Content Solid Integration
 *
 * Uses Vite's Environment API to enable embedding Solid components in Markdown.
 */

import type { Plugin, PluginOption, ResolvedConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import { resolveComponentsGlob } from "./components";
import { createSolidMarkdownEnvironment } from "./environment";
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

/**
 * Creates the Ox Content Solid integration plugin.
 *
 * Unlike the React and Svelte integrations, this plugin must be listed **before**
 * `vite-plugin-solid`, and that plugin must be told about the Markdown
 * extensions. Markdown is turned into Solid JSX here, and Solid's JSX is
 * compile-time only — `vite-plugin-solid` is what turns it into DOM or SSR
 * instructions.
 *
 * @example
 * ```ts
 * // vite.config.ts
 * import { defineConfig } from 'vite';
 * import solid from 'vite-plugin-solid';
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
 *     solid({ extensions: ['.md', '.markdown', '.mdx'] }),
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

  // `post` so every `pre`/normal plugin — vite-plugin-solid included — has
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

export { oxContent } from "@ox-content/vite-plugin";
