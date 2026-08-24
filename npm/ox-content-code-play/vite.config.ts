import { defineConfig } from "vite-plus";
import { defineConfig as definePackConfig } from "vite-plus/pack";

export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  pack: definePackConfig({
    entry: ["src/index.ts", "src/plugin.ts", "src/hydrate.ts", "src/browser.ts"],
    format: ["esm"],
    dts: true,
    clean: true,
    hash: false,
    deps: {
      neverBundle: ["vite", "typescript"],
    },
  }),
});
