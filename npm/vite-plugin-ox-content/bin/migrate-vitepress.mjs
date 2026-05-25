#!/usr/bin/env node
// Thin shim so pnpm can link the bin during `pnpm install` even before the
// package has been built. The real entry lives in `dist/vitepress-cli.mjs`
// and is produced by `pnpm run build`. The shim is checked in so the
// symlink target always exists.
//
// If `dist/vitepress-cli.mjs` is missing the user is running the source
// checkout without having built it yet — give a clearer error than the
// stack trace `import` would emit.

import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const entry = resolve(here, "..", "dist", "vitepress-cli.mjs");

if (!existsSync(entry)) {
  process.stderr.write(
    "ox-content-migrate-vitepress: dist/vitepress-cli.mjs is missing.\n" +
      "Run `pnpm --filter @ox-content/vite-plugin build` (or your package's\n" +
      "build script) first.\n",
  );
  process.exit(1);
}

await import(pathToFileURL(entry).href);
