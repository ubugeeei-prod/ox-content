import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const exampleRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

export default defineConfig({
  plugins: [
    oxContent({
      attrs: true,
      codeAnnotations: { notation: "both" },
      codeImports: { rootDir: exampleRoot },
      cjkEmphasis: true,
      editThisPage: {
        repoUrl: "https://github.com/owner/repo",
        rootDir: exampleRoot,
        label: "Suggest an edit",
      },
      emojiShortcodes: {
        custom: {
          shipit: "ship it",
        },
      },
      mermaid: true,
      wikiLinks: { baseUrl: "/docs" },
    }),
  ],
});
