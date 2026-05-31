import { defineConfig } from "vite-plus";
import { defineConfig as definePackConfig } from "vite-plus/pack";

export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  test: {
    include: ["test/unit/**/*.test.ts"],
  },
  pack: definePackConfig({
    entry: ["src/index.ts"],
    format: ["esm", "cjs"],
    dts: true,
    clean: true,
    hash: false,
    external: ["vite", "@ox-content/vite-plugin", "@ox-content/napi"],
  }),
});
