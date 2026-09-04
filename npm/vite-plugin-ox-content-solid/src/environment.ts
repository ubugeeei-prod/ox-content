import type { EnvironmentOptions } from "vite";
import type { ResolvedSolidOptions } from "./types";

export type SolidMarkdownRuntime = "node" | "deno" | "bun" | "unknown";

export function detectSolidMarkdownRuntime(): SolidMarkdownRuntime {
  const runtime = globalThis as {
    Bun?: unknown;
    Deno?: unknown;
    process?: { versions?: { node?: string; bun?: string } };
  };
  if (runtime.Bun || runtime.process?.versions?.bun) {
    return "bun";
  }
  if (runtime.Deno) {
    return "deno";
  }
  if (runtime.process?.versions?.node) {
    return "node";
  }
  return "unknown";
}

export function createSolidRuntimeResolveConditions(runtime: SolidMarkdownRuntime): string[] {
  return runtime === "unknown" || runtime === "node" ? [] : [runtime];
}

export function mergeSolidResolveConditions(
  first: readonly string[] = [],
  second: readonly string[] = [],
): string[] {
  return [...new Set([...first, ...second])];
}

export function createSolidMarkdownEnvironment(
  mode: "ssr" | "client",
  options: ResolvedSolidOptions,
  runtime: SolidMarkdownRuntime = detectSolidMarkdownRuntime(),
): EnvironmentOptions {
  const isSSR = mode === "ssr";
  const runtimeConditions = createSolidRuntimeResolveConditions(runtime);

  return {
    build: {
      outDir: isSSR ? `${options.outDir}/.ox-content/ssr` : `${options.outDir}/.ox-content/client`,
      ssr: isSSR,
      rollupOptions: {
        output: {
          format: "esm",
          entryFileNames: isSSR ? "[name].js" : "[name].[hash].js",
        },
      },
      ...(isSSR && { target: runtime === "node" ? "node18" : "esnext", minify: false }),
    },
    resolve: {
      // `solid` must lead: Solid libraries ship their uncompiled JSX behind that
      // condition, and the server/client split then comes from `node`/`browser`.
      conditions: isSSR
        ? ["solid", ...runtimeConditions, "node", "import"]
        : ["solid", "browser", "import"],
    },
    optimizeDeps: {
      include: isSSR ? [] : ["solid-js", "@solidjs/web"],
      exclude: ["@ox-content/vite-plugin", "@ox-content/vite-plugin-solid"],
    },
  };
}
