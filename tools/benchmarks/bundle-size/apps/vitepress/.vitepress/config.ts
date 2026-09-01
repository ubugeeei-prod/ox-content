import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Benchmark",
  description: "Bundle size benchmark",
  themeConfig: {
    sidebar: [
      { text: "Home", link: "/" },
      { text: "Getting Started", link: "/getting-started" },
      { text: "API", link: "/api" },
      { text: "Examples", link: "/examples" },
    ],
  },
});
