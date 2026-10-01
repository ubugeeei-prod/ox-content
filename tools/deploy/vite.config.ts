import { defineConfig } from "vite-plus";

export default defineConfig({
  run: {
    tasks: {
      "docs:build": { command: "node ../scripts/build-docs-for-cloudflare.mjs", cache: false },
      docs: { command: "node ../scripts/deploy-docs-to-cloudflare.mjs", cache: false },
      cf: { command: "node ../scripts/cloudflare-cli.mjs", cache: false },
    },
  },
});
