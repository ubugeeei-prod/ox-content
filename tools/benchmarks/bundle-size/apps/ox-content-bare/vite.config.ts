import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";
import * as fs from "fs";
import * as path from "path";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "../../content",
      ssg: {
        bare: true,
        clean: true,
      },
    }),
    // Plugin to clean up JS assets after SSG generation
    {
      name: "cleanup-js-assets",
      enforce: "post" as const,
      closeBundle() {
        const distDir = path.resolve(__dirname, "dist");
        const assetsDir = path.join(distDir, "assets");

        // Remove assets directory (contains JS bundles)
        if (fs.existsSync(assetsDir)) {
          fs.rmSync(assetsDir, { recursive: true });
        }

        // Remove any remaining JS/CSS files in dist root
        if (fs.existsSync(distDir)) {
          const files = fs.readdirSync(distDir);
          for (const file of files) {
            if (file.endsWith(".js") || file.endsWith(".css")) {
              fs.unlinkSync(path.join(distDir, file));
            }
          }
        }
      },
    },
  ],
  build: {
    outDir: "dist",
    minify: true,
    // Use empty entry point - we only care about SSG output
    rollupOptions: {
      input: "empty-entry.js",
      output: {
        // Prevent Vite from generating index.html
        entryFileNames: "_empty.js",
      },
    },
  },
});
