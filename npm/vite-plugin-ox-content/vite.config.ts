import { defineConfig } from "vite-plus";
import { defineConfig as definePackConfig } from "vite-plus/pack";

export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  pack: definePackConfig({
    entry: [
      "src/index.ts",
      "src/vitepress-cli.ts",
      "src/incremental-dom.ts",
      // `jsxImportSource: "@ox-content/vite-plugin"` makes the JSX transform
      // emit imports of ./jsx-runtime and ./jsx-dev-runtime, so both need to
      // be real modules in dist/, not just symbols on the main entry.
      "src/jsx-runtime.ts",
      "src/jsx-dev-runtime.ts",
    ],
    format: ["esm", "cjs"],
    target: "es2022",
    dts: true,
    clean: true,
    sourcemap: true,
    hash: false,
    deps: {
      neverBundle: [
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
        "@typescript/native-preview",
      ],
    },
  }),
});
