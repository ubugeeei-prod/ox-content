/**
 * Vite Plugin for Ox Content Vue Integration
 *
 * Uses Vite's Environment API to enable embedding Vue components in Markdown.
 * Provides SSR and client environments for proper hydration.
 */

import * as fs from "fs";
import * as path from "path";
import type { Plugin, PluginOption, ResolvedConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import { transformMarkdownWithVue } from "./transform";
import { createVueMarkdownEnvironment } from "./environment";
import type {
  VueIntegrationOptions,
  ResolvedVueOptions,
  ComponentsMap,
  ComponentsOption,
} from "./types";

export type {
  VueIntegrationOptions,
  ResolvedVueOptions,
  ComponentsOption,
  ComponentsMap,
  VueTransformResult,
  ComponentIsland,
  ParsedMarkdownContent,
  TocEntry,
} from "./types";

/**
 * Creates the Ox Content Vue integration plugin with Environment API support.
 *
 * @example
 * ```ts
 * // vite.config.ts
 * import { defineConfig } from 'vite';
 * import vue from '@vitejs/plugin-vue';
 * import { oxContentVue } from 'vite-plugin-ox-content-vue';
 *
 * export default defineConfig({
 *   plugins: [
 *     vue(),
 *     oxContentVue({
 *       srcDir: 'docs',
 *       components: {
 *         Counter: './src/components/Counter.vue',
 *       },
 *     }),
 *   ],
 * });
 * ```
 */
export function oxContentVue(options: VueIntegrationOptions = {}): PluginOption[] {
  const resolved = resolveVueOptions(options);
  let componentMap = new Map<string, string>();
  let config: ResolvedConfig;

  // Pre-resolve components if it's a map (not glob)
  if (typeof options.components === "object" && !Array.isArray(options.components)) {
    componentMap = new Map(Object.entries(options.components));
  }

  // Main Vue transformation plugin
  const vueTransformPlugin: Plugin = {
    name: "ox-content:vue-transform",
    enforce: "pre",

    async configResolved(resolvedConfig) {
      config = resolvedConfig;

      // Resolve glob patterns for components
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

      const result = await transformMarkdownWithVue(code, id, {
        ...resolved,
        components: componentMap,
        root: config.root,
      });

      return {
        code: result.code,
        map: result.map,
      };
    },
  };

  // Environment API plugin for Vue-specific SSR/client handling
  const vueEnvironmentPlugin: Plugin = {
    name: "ox-content:vue-environment",

    config() {
      return {
        environments: {
          // SSR environment for Vue component rendering
          oxcontent_ssr: createVueMarkdownEnvironment("ssr", resolved),
          // Client environment for hydration
          oxcontent_client: createVueMarkdownEnvironment("client", resolved),
        },
      };
    },

    // Environment-specific module resolution
    resolveId: {
      order: "pre",
      async handler(id, _importer, _options) {
        // Handle virtual modules for Vue markdown runtime
        if (id === "virtual:ox-content-vue/runtime") {
          return "\0virtual:ox-content-vue/runtime";
        }

        if (id === "virtual:ox-content-vue/components") {
          return "\0virtual:ox-content-vue/components";
        }

        return null;
      },
    },

    load: {
      order: "pre",
      async handler(id) {
        if (id === "\0virtual:ox-content-vue/runtime") {
          return generateRuntimeModule(resolved);
        }

        if (id === "\0virtual:ox-content-vue/components") {
          return generateComponentsModule(componentMap);
        }

        return null;
      },
    },

    // Per-environment build hooks
    applyToEnvironment(environment) {
      return (
        environment.name === "oxcontent_ssr" ||
        environment.name === "oxcontent_client" ||
        environment.name === "client" ||
        environment.name === "ssr"
      );
    },
  };

  // HMR plugin for component updates
  const vueHmrPlugin: Plugin = {
    name: "ox-content:vue-hmr",
    apply: "serve",

    handleHotUpdate({ file, server, modules }) {
      // Check if updated file is a registered component
      const isComponent = Array.from(componentMap.values()).some((path) =>
        file.endsWith(path.replace(/^\.\//, "")),
      );

      if (isComponent) {
        // Invalidate all Markdown modules that might use this component
        const mdModules = Array.from(server.moduleGraph.idToModuleMap.values()).filter((mod) =>
          mod.file?.endsWith(".md"),
        );

        if (mdModules.length > 0) {
          server.ws.send({
            type: "custom",
            event: "ox-content:vue-update",
            data: { file },
          });
          return [...modules, ...mdModules];
        }
      }

      return modules;
    },
  };

  // Get base ox-content plugins (environment plugin only)
  const basePlugins = oxContent(options).flatMap((plugin) =>
    Array.isArray(plugin) ? plugin : [plugin],
  ) as Plugin[];
  const environmentPlugin = basePlugins.find((plugin) => plugin.name === "ox-content:environment");
  const plugins: Plugin[] = [vueTransformPlugin, vueEnvironmentPlugin, vueHmrPlugin];

  if (environmentPlugin) {
    plugins.push(environmentPlugin);
  }

  return plugins;
}

/**
 * Resolves Vue integration options with defaults.
 */
function resolveVueOptions(options: VueIntegrationOptions): ResolvedVueOptions {
  return {
    srcDir: options.srcDir ?? "docs",
    outDir: options.outDir ?? "dist",
    base: options.base ?? "/",
    gfm: options.gfm ?? true,
    frontmatter: options.frontmatter ?? true,
    toc: options.toc ?? true,
    tocMaxDepth: options.tocMaxDepth ?? 3,
    codeAnnotations: resolveCodeAnnotationsOptions(options.codeAnnotations),
    components: options.components ?? {},
    // Vue-specific options
    reactivityTransform: options.reactivityTransform ?? false,
    customBlocks: options.customBlocks ?? true,
  };
}

function resolveCodeAnnotationsOptions(
  options: VueIntegrationOptions["codeAnnotations"],
): ResolvedVueOptions["codeAnnotations"] {
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

/**
 * Generates the runtime module for Vue markdown rendering.
 */
function generateRuntimeModule(_options: ResolvedVueOptions): string {
  return `
import { h, defineComponent, ref, onMounted } from 'vue';

export const OxContentRenderer = defineComponent({
  name: 'OxContentRenderer',
  props: {
    content: { type: Object, required: true },
    components: { type: Object, default: () => ({}) },
  },
  setup(props) {
    const mounted = ref(false);

    onMounted(() => {
      mounted.value = true;
    });

    return () => {
      if (!props.content) return null;

      const { html, frontmatter, toc, islands } = props.content;

      // Render static HTML with component islands
      return h('div', {
        class: 'ox-content',
        innerHTML: mounted.value ? undefined : html,
      }, mounted.value ? renderWithIslands(html, islands, props.components) : undefined);
    };
  },
});

function renderWithIslands(html, islands, components) {
  // Parse and render islands with Vue components
  // This is a simplified version - full implementation would use proper parsing
  return h('div', { innerHTML: html });
}

export function useOxContent() {
  return {
    OxContentRenderer,
  };
}
`;
}

/**
 * Generates the components registration module.
 */
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

/**
 * Resolves component glob patterns to a component map.
 */
async function resolveComponentsGlob(
  componentsOption: ComponentsOption,
  root: string,
): Promise<ComponentsMap> {
  // If it's already a map, return as-is
  if (typeof componentsOption === "object" && !Array.isArray(componentsOption)) {
    return componentsOption;
  }

  const patterns = Array.isArray(componentsOption) ? componentsOption : [componentsOption];

  const result: ComponentsMap = {};

  for (const pattern of patterns) {
    const files = await globFiles(pattern, root);

    for (const file of files) {
      // Derive component name from file name (PascalCase)
      const baseName = path.basename(file, path.extname(file));
      const componentName = toPascalCase(baseName);
      const relativePath = "./" + path.relative(root, file).replace(/\\/g, "/");

      result[componentName] = relativePath;
    }
  }

  return result;
}

/**
 * Simple glob matcher for component files.
 */
async function globFiles(pattern: string, root: string): Promise<string[]> {
  const files: string[] = [];

  // Parse glob pattern
  const isGlob = pattern.includes("*");

  if (!isGlob) {
    // It's a direct path
    const fullPath = path.resolve(root, pattern);
    if (fs.existsSync(fullPath)) {
      files.push(fullPath);
    }
    return files;
  }

  // Handle glob patterns like './src/components/*.vue'
  const parts = pattern.split("*");
  const baseDir = path.resolve(root, parts[0]);
  const ext = parts[1] || "";

  if (!fs.existsSync(baseDir)) {
    return files;
  }

  // Handle ** recursive pattern
  if (pattern.includes("**")) {
    await walkDir(baseDir, files, ext);
  } else {
    // Single level glob
    const entries = await fs.promises.readdir(baseDir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isFile() && entry.name.endsWith(ext)) {
        files.push(path.join(baseDir, entry.name));
      }
    }
  }

  return files;
}

/**
 * Recursively walks a directory.
 */
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

/**
 * Converts a string to PascalCase.
 */
function toPascalCase(str: string): string {
  return str.replace(/[-_](\w)/g, (_, c) => c.toUpperCase()).replace(/^\w/, (c) => c.toUpperCase());
}

// Re-export
export { oxContent } from "@ox-content/vite-plugin";
