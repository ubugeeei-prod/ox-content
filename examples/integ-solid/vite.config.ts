import { defineConfig } from "vite-plus";
import solid from "vite-plugin-solid";
import { oxContentSolid } from "@ox-content/vite-plugin-solid";

export default defineConfig({
  plugins: [
    // Order matters: oxContentSolid() turns Markdown into Solid JSX, and
    // vite-plugin-solid compiles that JSX. Both are `enforce: 'pre'`, so the
    // array order is what decides which one sees the file first.
    oxContentSolid({
      srcDir: "docs",
      // Auto-discover components using glob pattern
      components: "./src/components/*.tsx",
    }),
    // Solid's JSX is compile-time only, so the Markdown extensions have to be
    // listed here for the generated modules to be compiled at all.
    solid({ extensions: [".md", ".markdown", ".mdx"] }),
  ],
});
