/**
 * Vite Environment API integration for Ox Content.
 *
 * Creates a dedicated environment for Markdown processing,
 * enabling SSG-style rendering with separate client/server contexts.
 */

import type { EnvironmentOptions } from "vite";
import type { ResolvedOptions } from "./types";
import { isMarkdownFilePath } from "./markdown";

/**
 * Creates the Markdown processing environment configuration.
 *
 * This environment is used for:
 * - Server-side rendering of Markdown files
 * - Static site generation
 * - Pre-rendering at build time
 *
 * @example
 * ```ts
 * // In your vite.config.ts
 * export default defineConfig({
 *   environments: {
 *     markdown: createMarkdownEnvironment({
 *       srcDir: 'content',
 *       gfm: true,
 *     }),
 *   },
 * });
 * ```
 */
export function createMarkdownEnvironment(options: ResolvedOptions): EnvironmentOptions {
  return {
    // Consumer type for this environment
    consumer: "server",

    // Build configuration
    build: {
      // Output to a separate directory
      outDir: `${options.outDir}/.markdown`,

      // Emit assets for SSG
      emitAssets: true,

      // Create manifest for asset tracking
      manifest: true,

      // SSR-like externalization
      rollupOptions: {
        external: [
          // Externalize Node.js built-ins
          /^node:/,
          // Externalize native modules
          /\.node$/,
        ],
      },
    },

    // Resolve configuration
    resolve: {
      // Handle Markdown-like files
      extensions: options.extensions,

      // Conditions for module resolution
      conditions: ["markdown", "node", "import"],

      // Don't dedupe - each environment gets its own modules
      dedupe: [],
    },

    // Optimize dependencies
    optimizeDeps: {
      // Include ox-content dependencies
      include: [],
      // Exclude native modules
      exclude: ["@ox-content/napi"],
    },
  };
}

/**
 * Environment-specific module transformer.
 *
 * This is called during the transform phase to process
 * Markdown files within the environment context.
 */
export interface EnvironmentTransformContext {
  /**
   * Current environment name.
   */
  environment: string;

  /**
   * Whether we're in development mode.
   */
  isDev: boolean;

  /**
   * Whether this is a server-side render.
   */
  isSSR: boolean;

  /**
   * The resolved Vite config.
   */
  config: unknown;
}

/**
 * Creates environment-aware transform options.
 */
export function createTransformOptions(
  ctx: EnvironmentTransformContext,
  options: ResolvedOptions,
): ResolvedOptions {
  return {
    ...options,
    // Adjust options based on environment
    highlight: ctx.isSSR ? options.highlight : false,
    ogImage: ctx.isSSR ? options.ogImage : false,
  };
}

/**
 * Runs pre-render for SSG.
 *
 * This function is called during build to pre-render all Markdown files.
 */
export async function prerender(
  files: string[],
  _options: ResolvedOptions,
): Promise<Map<string, string>> {
  const results = new Map<string, string>();

  for (const file of files) {
    // In production, this would use the Ox Content parser
    // For now, we just mark the file as needing processing
    results.set(file, `/* Pre-rendered: ${file} */`);
  }

  return results;
}

/**
 * Environment plugin factory.
 *
 * Creates plugins specific to the Markdown environment.
 */
export function createEnvironmentPlugins(options: ResolvedOptions) {
  return [
    {
      name: "ox-content:markdown-env",

      // Only apply to markdown environment
      applyToEnvironment(name: string) {
        return name === "markdown";
      },

      // Transform within the environment
      transform(code: string, id: string) {
        if (!isMarkdownFilePath(id, options.extensions)) {
          return null;
        }

        // Environment-specific transformation
        return {
          code: `
            // Transformed in markdown environment
            ${code}
          `,
        };
      },
    },
  ];
}
