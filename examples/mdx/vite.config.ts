/**
 * Built-in MDX example.
 *
 * `mdx` is omitted on purpose: `.mdx` files enable JSX / ESM / expressions
 * by default, while sibling `.md` files stay CommonMark + GFM.
 */

import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "src/content",
      outDir: "dist",
      gfm: true,
      tables: true,
      taskLists: true,
      strikethrough: true,
      footnotes: true,
      highlight: true,
      toc: true,
      tocMaxDepth: 3,
      docs: false,
    }),
  ],
  build: {
    ssr: false,
    outDir: "dist",
  },
});
