import { createRequire } from "node:module";
import { isAbsolute } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
const require = createRequire(
  new URL("../../npm/vite-plugin-ox-content/package.json", import.meta.url),
);
const { rolldown } = await import(pathToFileURL(require.resolve("rolldown")).href);
const bundle = await rolldown({
  input: fileURLToPath(new URL("./pipeline-extensions.ts", import.meta.url)),
  platform: "node",
  external: (id) => !id.startsWith(".") && !isAbsolute(id),
});
await bundle.write({
  file: fileURLToPath(
    new URL("../../npm/vite-plugin-ox-content/.pipeline-extensions.mjs", import.meta.url),
  ),
  format: "esm",
});
await bundle.close();
