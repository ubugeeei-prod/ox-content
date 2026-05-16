/**
 * Vite Plugin for Ox Content Svelte Integration
 *
 * Uses Vite's Environment API to enable embedding Svelte components in Markdown.
 */

import * as fs from "fs";
import * as path from "path";
import type { Plugin, PluginOption, ResolvedConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import { transformMarkdownWithSvelte } from "./transform";
import { createSvelteMarkdownEnvironment } from "./environment";
import type {
  SvelteIntegrationOptions,
  ResolvedSvelteOptions,
  ComponentsMap,
  ComponentsOption,
} from "./types";

export type {
  SvelteIntegrationOptions,
  ResolvedSvelteOptions,
  ComponentsOption,
  ComponentsMap,
  SvelteTransformResult,
  ComponentIsland,
} from "./types";

/**
 * Creates the Ox Content Svelte integration plugin.
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

    async transform(code, id) {
      if (!id.endsWith(".md")) {
        return null;
      }

      const result = await transformMarkdownWithSvelte(code, id, {
        ...resolved,
        components: Object.fromEntries(componentMap),
        root: config.root,
      });

      return {
        code: result.code,
        map: result.map,
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
        const mdModules = Array.from(server.moduleGraph.idToModuleMap.values()).filter((mod) =>
          mod.file?.endsWith(".md"),
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

  const basePlugins = oxContent(options).flatMap((plugin) =>
    Array.isArray(plugin) ? plugin : [plugin],
  ) as Plugin[];
  const environmentPlugin = basePlugins.find((plugin) => plugin.name === "ox-content:environment");
  const plugins: Plugin[] = [svelteTransformPlugin, svelteEnvironmentPlugin, svelteHmrPlugin];

  if (environmentPlugin) {
    plugins.push(environmentPlugin);
  }

  return plugins;
}

function resolveSvelteOptions(
  options: SvelteIntegrationOptions,
): Omit<ResolvedSvelteOptions, "components"> {
  return {
    srcDir: options.srcDir ?? "docs",
    outDir: options.outDir ?? "dist",
    base: options.base ?? "/",
    gfm: options.gfm ?? true,
    frontmatter: options.frontmatter ?? true,
    toc: options.toc ?? true,
    tocMaxDepth: options.tocMaxDepth ?? 3,
    codeAnnotations: resolveCodeAnnotationsOptions(options.codeAnnotations),
    runes: options.runes ?? true,
  };
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

async function resolveComponentsGlob(
  componentsOption: ComponentsOption,
  root: string,
): Promise<ComponentsMap> {
  if (typeof componentsOption === "object" && !Array.isArray(componentsOption)) {
    return componentsOption;
  }

  const patterns = Array.isArray(componentsOption) ? componentsOption : [componentsOption];

  const result: ComponentsMap = {};

  for (const pattern of patterns) {
    const files = await globFiles(pattern, root);

    for (const file of files) {
      const baseName = path.basename(file, path.extname(file));
      const componentName = toPascalCase(baseName);
      const relativePath = "./" + path.relative(root, file).replace(/\\/g, "/");

      result[componentName] = relativePath;
    }
  }

  return result;
}

async function globFiles(pattern: string, root: string): Promise<string[]> {
  const files: string[] = [];
  const isGlob = pattern.includes("*");

  if (!isGlob) {
    const fullPath = path.resolve(root, pattern);
    if (fs.existsSync(fullPath)) {
      files.push(fullPath);
    }
    return files;
  }

  const parts = pattern.split("*");
  const baseDir = path.resolve(root, parts[0]);
  const ext = parts[1] || "";

  if (!fs.existsSync(baseDir)) {
    return files;
  }

  if (pattern.includes("**")) {
    await walkDir(baseDir, files, ext);
  } else {
    const entries = await fs.promises.readdir(baseDir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isFile() && entry.name.endsWith(ext)) {
        files.push(path.join(baseDir, entry.name));
      }
    }
  }

  return files;
}

async function walkDir(dir: string, files: string[], ext: string): Promise<void> {
  const entries = await fs.promises.readdir(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);

    if (entry.isDirectory()) {
      await walkDir(fullPath, files, ext);
    } else if (entry.isFile() && entry.name.endsWith(ext)) {
      files.push(fullPath);
    }
  }
}

function toPascalCase(str: string): string {
  return str.replace(/[-_](\w)/g, (_, c) => c.toUpperCase()).replace(/^\w/, (c) => c.toUpperCase());
}

export { oxContent } from "@ox-content/vite-plugin";
