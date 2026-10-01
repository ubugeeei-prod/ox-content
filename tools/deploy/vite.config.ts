import { defineConfig } from "vite-plus";

export default defineConfig({
  run: {
    tasks: {
      docs: { command: "node ../scripts/deploy-docs-to-cloudflare.mjs", cache: false },
      cf: { command: "node ../scripts/cloudflare-cli.mjs", cache: false },
    },
  },
});
