/**
 * Custom OG Image Example
 *
 * Demonstrates how to use a custom template for OG image generation.
 * Frontmatter fields (category, coverColor, etc.) are passed as props.
 */

import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "src/content",
      outDir: "dist",
      gfm: true,
      highlight: true,

      // Enable OG image generation with a custom template
      ogImage: true,
      ogImageOptions: {
        template: "./og.ts",
        width: 1200,
        height: 630,
        cache: true,
      },

      ssg: {
        siteName: "My Blog",
      },
    }),
  ],

  build: {
    outDir: "dist",
  },
});
