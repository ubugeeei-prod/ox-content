/**
 * Vite Environment API configuration for Vue integration.
 */

import type { EnvironmentOptions } from "vite";
import type { ResolvedVueOptions } from "./types";

/**
 * Creates a Vite environment for Vue markdown processing.
 *
 * @param mode - 'ssr' for server-side rendering, 'client' for client hydration
 * @param options - Resolved Vue integration options
 */
export function createVueMarkdownEnvironment(
  mode: "ssr" | "client",
  options: ResolvedVueOptions,
): EnvironmentOptions {
  const isSSR = mode === "ssr";

  return {
    build: {
      outDir: isSSR ? `${options.outDir}/.ox-content/ssr` : `${options.outDir}/.ox-content/client`,

      ssr: isSSR,

      rollupOptions: {
        input: isSSR ? undefined : undefined,
        output: {
          format: isSSR ? "esm" : "esm",
          entryFileNames: isSSR ? "[name].js" : "[name].[hash].js",
          chunkFileNames: isSSR ? "chunks/[name].js" : "chunks/[name].[hash].js",
        },
      },

      // SSR-specific optimizations
      ...(isSSR && {
        target: "node18",
        minify: false,
      }),
    },

    resolve: {
      conditions: isSSR ? ["node", "import"] : ["browser", "import"],
    },

    optimizeDeps: {
      // Pre-bundle Vue for faster cold starts
      include: isSSR ? [] : ["vue"],

      // Exclude ox-content packages from optimization (they're local)
      exclude: ["@ox-content/vite-plugin", "@ox-content/vite-plugin-vue"],
    },

    // Development server options (client only)
    ...(!isSSR && {
      dev: {
        warmup: ["./src/**/*.vue", "./docs/**/*.{md,markdown,mdx}"],
      },
    }),
  };
}

/**
 * Creates environment-specific virtual modules.
 */
export function createVirtualModules(
  mode: "ssr" | "client",
  _options: ResolvedVueOptions,
): Record<string, string> {
  const isSSR = mode === "ssr";

  return {
    // Environment detection module
    "virtual:ox-content/env": `
      export const isSSR = ${isSSR};
      export const isClient = ${!isSSR};
      export const mode = '${mode}';
    `,

    // Hydration utilities
    "virtual:ox-content/hydration": isSSR
      ? `
        // SSR: No-op hydration
        export function hydrate() {}
        export function createSSRApp(component) {
          return component;
        }
      `
      : `
        import { createApp } from 'vue';

        export function hydrate(component, container, props = {}) {
          const app = createApp(component, props);
          app.mount(container);
          return app;
        }

        export function createClientApp(component) {
          return createApp(component);
        }
      `,
  };
}
