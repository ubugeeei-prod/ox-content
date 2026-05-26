import { defineConfig } from "vite-plus";
import { defineConfig as definePackConfig } from "vite-plus/pack";

export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  pack: definePackConfig({
    entry: [
      "src/index.ts",
      "src/vite.ts",
      "src/webpack.ts",
      "src/rollup.ts",
      "src/esbuild.ts",
      "src/rspack.ts",
    ],
    format: ["esm"],
    dts: true,
    clean: true,
    hash: false,
    deps: {
      neverBundle: ["vite", "webpack", "rollup", "esbuild", "@ox-content/napi", "unplugin"],
    },
  }),
});
