#!/usr/bin/env node

import { rm, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const dist = join(root, "npm", "vite-plugin-ox-content-solid", "dist");
const reference = '/// <reference path="./virtual.d.ts" />';

const virtualDeclaration = `declare module "virtual:ox-content-solid/html-host/modules" {
  import type { SolidHtmlHostClientModules } from "@ox-content/vite-plugin-solid/html-host/client";
  import type { SolidHtmlHostClientModule } from "@ox-content/vite-plugin-solid";

  export const modules: SolidHtmlHostClientModules;
  export const clientModules: readonly SolidHtmlHostClientModule[];
  export default modules;
}
`;

await writeFile(join(dist, "virtual.d.ts"), virtualDeclaration);

for (const extension of ["mts", "cts"]) {
  const declarationFile = join(dist, `index.d.${extension}`);
  let declaration = await readFile(declarationFile, "utf8");
  declaration = declaration
    .replace(new RegExp(`\\n//# sourceMappingURL=index\\.d\\.${extension}\\.map\\s*$`, "u"), "")
    .trimStart();
  if (!declaration.startsWith(reference)) {
    declaration = `${reference}\n${declaration}`;
  }
  await writeFile(declarationFile, `${declaration.trimEnd()}\n`);
  await rm(`${declarationFile}.map`, { force: true });
}
