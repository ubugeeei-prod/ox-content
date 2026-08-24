import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";
import { codePlay } from "@ox-content/code-play";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",
      outDir: "dist",
      highlight: true,
    }),
    codePlay({
      languages: {
        javascript: true,
        typescript: { execute: true, typecheck: true },
      },
      srcDir: "content",
      outDir: "dist",
    }),
  ],
  server: {
    port: 4177,
  },
  preview: {
    port: 4177,
  },
  build: {
    outDir: "dist",
  },
});
