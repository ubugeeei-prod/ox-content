#!/usr/bin/env node

import { rm, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const packageRoot = join(root, "npm", "vite-plugin-ox-content");
const dist = join(packageRoot, "dist");
const reference = '/// <reference path="./virtual.d.ts" />';

const virtualSource = await readFile(join(packageRoot, "src", "virtual.ts"), "utf8");
const virtualDeclaration = virtualSource
  .replaceAll('import("./types")', 'import("@ox-content/vite-plugin")')
  .trimEnd();

await writeFile(join(dist, "virtual.d.ts"), `${virtualDeclaration}\n`);

for (const extension of ["mts", "cts"]) {
  const declarationFile = join(dist, `index.d.${extension}`);
  let declaration = await readFile(declarationFile, "utf8");
  declaration = declaration
    .replace(new RegExp(`\\n//# sourceMappingURL=index\\.d\\.${extension}\\.map\\s*$`, "u"), "")
    .replace(/\n?\/\/#region src\/virtual\.d\.ts\n[\s\S]*?\n\/\/#endregion\n?/u, "\n")
    // Newer tsdown releases inline `src/virtual.ts` without region markers.
    // Those ambient modules live in `virtual.d.ts`, so drop the inlined copies.
    .replace(/^declare module "virtual:ox-content\/[^"]+" \{\n[\s\S]*?^\}\n/gmu, "")
    .trimStart();
  if (!declaration.startsWith(reference)) {
    declaration = `${reference}\n${declaration}`;
  }
  await writeFile(declarationFile, `${declaration.trimEnd()}\n`);
  await rm(`${declarationFile}.map`, { force: true });
}
