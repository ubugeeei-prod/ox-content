import path from "node:path";
import { fileURLToPath } from "node:url";
import { build, type UserConfig } from "vite";

const root = fileURLToPath(new URL("..", import.meta.url));

export async function bundleBrowserClient(outDir = path.join(root, "dist")): Promise<void> {
  const config = {
    configFile: false,
    root,
    publicDir: false,
    logLevel: "warn",
    build: {
      emptyOutDir: false,
      minify: false,
      outDir,
      lib: {
        entry: path.join(root, "src/browser.ts"),
        formats: ["es"],
        fileName: () => "browser",
      },
      rollupOptions: {
        external: (id: string) => id.startsWith("node:"),
        output: {
          codeSplitting: false,
          entryFileNames: "browser.mjs",
        },
      },
    },
  } as UserConfig;
  await build(config);
}

function isDirectRun(): boolean {
  const entry = process.argv[1];
  return Boolean(entry) && fileURLToPath(import.meta.url) === path.resolve(entry);
}

if (isDirectRun()) {
  await bundleBrowserClient();
}
