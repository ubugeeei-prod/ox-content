import { defineConfig } from "vite";
import { oxContentHono } from "@ox-content/vite-plugin-hono/vite";

export default defineConfig({
  plugins: [
    oxContentHono({
      oxContent: { srcDir: "content" },
      layout: "./src/Layout.tsx",
      frontmatter: ({ title }) => ({ title: title ?? "" }),
    }),
  ],
});
