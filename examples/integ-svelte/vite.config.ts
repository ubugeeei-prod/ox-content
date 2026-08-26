import { defineConfig } from "vite-plus";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { oxContentSvelte } from "@ox-content/vite-plugin-svelte";

export default defineConfig({
  plugins: [
    svelte(),
    oxContentSvelte({
      srcDir: "docs",
      ssg: false,
      // Auto-discover components using glob pattern
      components: "./src/components/*.svelte",
      mdxDocumentProps: true,
    }),
  ],
});
