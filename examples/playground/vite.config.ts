import { defineConfig } from "vite-plus";

const productionBase = "/playground/";

export function resolvePlaygroundBase(mode: string, configuredBase?: string): string {
  return configuredBase ?? (mode === "production" ? productionBase : "/");
}

export default defineConfig(({ mode }) => ({
  base: resolvePlaygroundBase(mode, process.env.OX_CONTENT_PLAYGROUND_BASE),
  server: {
    port: 5173,
  },
  build: {
    outDir: "dist",
  },
}));
