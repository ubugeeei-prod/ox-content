#!/usr/bin/env node

import { readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { publicDeclarationEntries } from "./public-declaration-contracts.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const requested = new Set(process.argv.slice(2));
const entries = requested.size
  ? publicDeclarationEntries.filter((entry) =>
      [entry.packageName, entry.specifier, entry.distBase].some((key) => requested.has(key)),
    )
  : publicDeclarationEntries;

if (entries.length === 0) {
  throw new Error(`No public declaration entry matched: ${[...requested].join(", ")}`);
}

for (const entry of entries) {
  for (const extension of ["mts", "cts"]) {
    const declarationFile = join(
      root,
      entry.packageDir,
      "dist",
      `${entry.distBase}.d.${extension}`,
    );
    const declaration = await readFile(declarationFile, "utf8");
    await writeFile(declarationFile, stabilizeDeclaration(declaration, entry));
    await rm(`${declarationFile}.map`, { force: true });
  }
}

function stabilizeDeclaration(declaration, entry) {
  const expectedNames = [...entry.values, ...entry.types];
  for (const name of expectedNames) {
    if (!new RegExp(`\\b${escapeRegExp(name)}\\b`).test(declaration)) {
      throw new Error(`${entry.specifier} declaration is missing ${name}`);
    }
  }

  const exportStart = declaration.lastIndexOf("export {");
  if (exportStart === -1) {
    throw new Error(`${entry.specifier} declaration is missing a final export block`);
  }

  return [
    declaration.slice(0, exportStart).trimEnd(),
    `export { ${entry.values.join(", ")} };`,
    `export type { ${entry.types.join(", ")} };`,
    "",
  ].join("\n");
}

function escapeRegExp(value) {
  return value.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
}
