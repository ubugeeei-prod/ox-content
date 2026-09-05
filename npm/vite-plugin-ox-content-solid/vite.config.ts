import { defineConfig } from "vite-plus";
import { defineConfig as definePackConfig } from "vite-plus/pack";

export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  pack: definePackConfig({
    entry: ["src/index.ts", "src/html-host-client.ts"],
    format: ["esm", "cjs"],
    dts: true,
    clean: true,
    hash: false,
    deps: {
      neverBundle: ["vite", "solid-js", "@solidjs/web", "@ox-content/vite-plugin"],
    },
  }),
});
