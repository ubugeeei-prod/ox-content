import { defineConfig } from "vite-plus";
import vue from "@vitejs/plugin-vue";
import { oxContent } from "@ox-content/vite-plugin";
import { oxContentVue } from "@ox-content/vite-plugin-vue";

export default defineConfig({
  plugins: [
    vue(),
    oxContent({
      srcDir: "../../content",
    }),
    oxContentVue(),
  ],
  build: {
    outDir: "dist",
    minify: true,
  },
});
