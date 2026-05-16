/**
 * Vite Plugin for Ox Content React Integration
 *
 * Uses Vite's Environment API to enable embedding React components in Markdown.
 */

import * as fs from "fs";
import * as path from "path";
import type { Plugin, PluginOption, ResolvedConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import { transformMarkdownWithReact } from "./transform";
import { createReactMarkdownEnvironment } from "./environment";
import type {
  ReactIntegrationOptions,
  ResolvedReactOptions,
  ComponentsMap,
  ComponentsOption,
} from "./types";

export type {
  ReactIntegrationOptions,
  ResolvedReactOptions,
  ComponentsOption,
  ComponentsMap,
  ReactTransformResult,
  ComponentIsland,
} from "./types";

/**
 * Creates the Ox Content React integration plugin.
 *
 * @example
 * ```ts
 * // vite.config.ts
 * import { defineConfig } from 'vite';
 * import react from '@vitejs/plugin-react';
 * import { oxContentReact } from 'vite-plugin-ox-content-react';
 *
 * export default defineConfig({
 *   plugins: [
 *     react(),
 *     oxContentReact({
 *       srcDir: 'docs',
 *       components: {
 *         Counter: './src/components/Counter.tsx',
 *       },
 *     }),
 *   ],
 * });
 * ```
 */
export function oxContentReact(options: ReactIntegrationOptions = {}): PluginOption[] {
  const resolved = resolveReactOptions(options);
  let componentMap = new Map<string, string>();
  let config: ResolvedConfig;

  if (typeof options.components === "object" && !Array.isArray(options.components)) {
    componentMap = new Map(Object.entries(options.components));
  }

  const reactTransformPlugin: Plugin = {
    name: "ox-content:react-transform",
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

      const result = await transformMarkdownWithReact(code, id, {
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

  const reactEnvironmentPlugin: Plugin = {
    name: "ox-content:react-environment",

    config() {
      const envOptions = {
        ...resolved,
        components: Object.fromEntries(componentMap),
      };
      return {
        environments: {
          oxcontent_ssr: createReactMarkdownEnvironment("ssr", envOptions),
          oxcontent_client: createReactMarkdownEnvironment("client", envOptions),
        },
      };
    },

    resolveId(id) {
      if (id === "virtual:ox-content-react/runtime") {
        return "\0virtual:ox-content-react/runtime";
      }
      if (id === "virtual:ox-content-react/components") {
        return "\0virtual:ox-content-react/components";
      }
      return null;
    },

    load(id) {
      if (id === "\0virtual:ox-content-react/runtime") {
        return generateRuntimeModule();
      }
      if (id === "\0virtual:ox-content-react/components") {
        return generateComponentsModule(componentMap);
      }
      return null;
    },

    applyToEnvironment(environment) {
      return ["oxcontent_ssr", "oxcontent_client", "client", "ssr"].includes(environment.name);
    },
  };

  const reactHmrPlugin: Plugin = {
    name: "ox-content:react-hmr",
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
            event: "ox-content:react-update",
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
  const plugins: Plugin[] = [reactTransformPlugin, reactEnvironmentPlugin, reactHmrPlugin];

  if (environmentPlugin) {
    plugins.push(environmentPlugin);
  }

  return plugins;
}

function resolveReactOptions(
  options: ReactIntegrationOptions,
): Omit<ResolvedReactOptions, "components"> {
  return {
    srcDir: options.srcDir ?? "docs",
    outDir: options.outDir ?? "dist",
    base: options.base ?? "/",
    gfm: options.gfm ?? true,
    frontmatter: options.frontmatter ?? true,
    toc: options.toc ?? true,
    tocMaxDepth: options.tocMaxDepth ?? 3,
    codeAnnotations: resolveCodeAnnotationsOptions(options.codeAnnotations),
    jsxRuntime: options.jsxRuntime ?? "automatic",
  };
}

function resolveCodeAnnotationsOptions(
  options: ReactIntegrationOptions["codeAnnotations"],
): ResolvedReactOptions["codeAnnotations"] {
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
import React, { useState, useEffect } from 'react';

export function OxContentRenderer({ content, components = {} }) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!content) return null;

  const { html, frontmatter, islands } = content;

  if (!mounted) {
    return React.createElement('div', {
      className: 'ox-content',
      dangerouslySetInnerHTML: { __html: html },
    });
  }

  return React.createElement('div', { className: 'ox-content' },
    islands.map((island) => {
      const Component = components[island.name];
      return Component
        ? React.createElement(Component, { key: island.id, ...island.props })
        : null;
    })
  );
}

export function useOxContent() {
  return { OxContentRenderer };
}
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
