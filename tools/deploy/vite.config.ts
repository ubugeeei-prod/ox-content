import { defineConfig } from "vite-plus";

export default defineConfig({
  run: {
    tasks: {
      check: { command: "vp exec -- tsc --project tsconfig.json", cache: false },
      "docs:build": { command: "node ../scripts/build-docs-for-cloudflare.ts", cache: false },
      docs: { command: "node ../scripts/deploy-docs-to-cloudflare.ts", cache: false },
      cf: { command: "node ../scripts/cloudflare-cli.ts", cache: false },
    },
  },
});
