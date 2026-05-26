import { defineConfig } from "vite-plus";

/**
 * Vite+ config for `vscode-ox-content`. We don't bundle the extension
 * (it ships as the raw tsc output for VSIX packaging), but the workspace
 * still needs a config so `vp test` and `vp fmt` resolve here.
 */
export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**", "out/**"],
  },
  test: {
    // Only run the pure-node unit suite from `vp test`; the integration
    // suite (`pnpm test`) needs a real VS Code Electron host and is
    // driven by `@vscode/test-cli`.
    include: ["src/test/unit/**/*.test.ts"],
  },
});
