import type { EnvironmentOptions } from "vite";
import type { ResolvedSolidOptions } from "./types";

export function createSolidMarkdownEnvironment(
  mode: "ssr" | "client",
  options: ResolvedSolidOptions,
): EnvironmentOptions {
  const isSSR = mode === "ssr";

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
      ...(isSSR && { target: "node18", minify: false }),
    },
    resolve: {
      // `solid` must lead: Solid libraries ship their uncompiled JSX behind that
      // condition, and the server/client split then comes from `node`/`browser`.
      conditions: isSSR ? ["solid", "node", "import"] : ["solid", "browser", "import"],
    },
    optimizeDeps: {
      include: isSSR ? [] : ["solid-js", "solid-js/web"],
      exclude: ["@ox-content/vite-plugin", "@ox-content/vite-plugin-solid"],
    },
  };
}
