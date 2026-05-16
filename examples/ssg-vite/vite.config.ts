/**
 * Vite SSG Example Configuration
 *
 * Demonstrates how to use Ox Content with Vite's Environment API
 * for static site generation.
 */

import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    // Ox Content plugin with Environment API
    oxContent({
      // Source directory for Markdown files
      srcDir: "src/content",

      // Output directory
      outDir: "dist",

      // Enable all GFM features
      gfm: true,
      tables: true,
      taskLists: true,
      strikethrough: true,
      footnotes: true,

      // Enable syntax highlighting
      highlight: true,
      highlightTheme: "github-dark",
      codeAnnotations: {
        notation: "both",
      },

      // Generate table of contents
      toc: true,
      tocMaxDepth: 3,

      // Enable OG image generation
      ogImage: true,
      ogImageOptions: {
        background: "#1a1a2e",
        textColor: "#ffffff",
        accentColor: "#e94560",
      },
    }),
  ],

  // Build configuration
  build: {
    // Enable SSG mode
    ssr: false,
    outDir: "dist",
  },
});
