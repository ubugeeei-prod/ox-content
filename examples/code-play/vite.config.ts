import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";
import { codePlay } from "@ox-content/code-play";

const env = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env;
const pythonEndpoint = env?.OX_CODE_PLAY_PYTHON_ENDPOINT;

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
        rust: true,
        go: true,
        python: pythonEndpoint ? { endpoint: pythonEndpoint } : true,
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
