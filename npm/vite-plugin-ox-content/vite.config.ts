import { defineConfig } from "vite-plus";
import { defineConfig as definePackConfig } from "vite-plus/pack";

export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  pack: definePackConfig({
    entry: ["src/index.ts"],
    format: ["esm", "cjs"],
    target: "es2022",
    dts: true,
    clean: true,
    sourcemap: true,
    hash: false,
    external: [
      "vite",
      "@ox-content/napi",
      "playwright",
      "rolldown",
      "vue",
      "vue/server-renderer",
      "@vue/compiler-sfc",
      "@vizejs/vite-plugin",
      "svelte",
      "svelte/compiler",
      "svelte/server",
      "svelte/internal",
      "svelte/internal/server",
      "react",
      "react/jsx-runtime",
      "react/jsx-dev-runtime",
      "react-dom",
      "react-dom/server",
      "typescript",
    ],
  }),
});
