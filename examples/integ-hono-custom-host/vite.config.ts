import { defineConfig } from "vite";
import { oxContentCustomHost, type OxContentOptions } from "@ox-content/vite-plugin";
import { honoJsx } from "@ox-content/vite-plugin-hono/vite";

const oxContent = {
  srcDir: "content",
  outDir: "dist",
  highlight: false,
  resources: false,
  docs: false,
  search: false,
  feeds: false,
  siteMaps: false,
} satisfies OxContentOptions;
export default defineConfig({
  appType: "mpa",
  plugins: [
    ...honoJsx({ oxContent }),
    ...oxContentCustomHost({
      host: "./src/host.ts",
      oxContent,
      build: { transformHtml: false },
      dev: { routeDependencies: [{ path: "content", kind: "directory" }] },
    }),
  ],
  build: { manifest: true, rollupOptions: { input: "src/client.ts" } },
});
