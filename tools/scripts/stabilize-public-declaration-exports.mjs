#!/usr/bin/env node

import { readFile, readdir, rm, writeFile } from "node:fs/promises";
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
    const { declaration, facade } = await readDeclaration(declarationFile, entry, extension);
    const stableDeclaration = stabilizeDeclaration(declaration, entry, extension, facade);
    const outputBase = entry.publicDistBase ?? entry.distBase;
    const outputFile = join(root, entry.packageDir, "dist", `${outputBase}.d.${extension}`);
    await writeFile(outputFile, stableDeclaration);
    await rm(`${outputFile}.map`, { force: true });
  }
}

async function readDeclaration(file, entry, extension) {
  const declaration = await readFile(file, "utf8");
  const names = [...entry.values, ...entry.types];
  const complete = (source) =>
    names.every((name) => new RegExp(`\\b${escapeRegExp(name)}\\b`).test(source));
  if (complete(declaration)) return { declaration, facade: false };

  // Shared type chunks can move the complete entry facade to a numbered file.
  // Keep its imports instead of assuming every public type lives in the first chunk.
  const pattern = new RegExp(`^${escapeRegExp(entry.distBase)}\\d+\\.d\\.${extension}$`);
  const candidates = await Promise.all(
    (await readdir(dirname(file)))
      .filter((name) => pattern.test(name))
      .map((name) => readFile(join(dirname(file), name), "utf8")),
  );
  const matches = candidates.filter((source) => {
    const exportStart = source.lastIndexOf("export {");
    if (!complete(source) || exportStart === -1) return false;
    const exports = parseExportedNames(source.slice(exportStart));
    return names.every((name) => exports.has(name));
  });
  if (matches.length > 1) {
    throw new Error(`${entry.specifier} has multiple complete declaration facades`);
  }
  return matches.length === 1
    ? { declaration: matches[0], facade: true }
    : { declaration, facade: false };
}

function stabilizeDeclaration(declaration, entry, extension, facade = false) {
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

  if (!facade && entry.publicDistBase && entry.publicDistBase !== entry.distBase) {
    return stableFacadeDeclaration(declaration.slice(exportStart), entry, extension);
  }

  return [
    declaration
      .slice(0, exportStart)
      .replace(/^export\s+(?=(?:declare|interface|type|abstract|class|function|const)\b)/gm, "")
      .trimEnd(),
    `export { ${entry.values.join(", ")} };`,
    `export type { ${entry.types.join(", ")} };`,
    "",
  ].join("\n");
}

function stableFacadeDeclaration(exportBlock, entry, extension) {
  const exportedNames = parseExportedNames(exportBlock);
  const source = `./${entry.distBase}.${extension === "cts" ? "cjs" : "mjs"}`;
  const values = entry.values.map((name) => exportSpecifier(exportedNames, name));
  const types = entry.types.map((name) => exportSpecifier(exportedNames, name));

  return [
    `export { ${values.join(", ")} } from "${source}";`,
    `export type { ${types.join(", ")} } from "${source}";`,
    "",
  ].join("\n");
}

function parseExportedNames(exportBlock) {
  const exports = new Map();
  const bodyStart = exportBlock.indexOf("{");
  const bodyEnd = exportBlock.indexOf("}", bodyStart);
  if (bodyStart === -1 || bodyEnd === -1) {
    throw new Error("Declaration export block is malformed");
  }

  for (const specifier of exportBlock.slice(bodyStart + 1, bodyEnd).split(",")) {
    const trimmed = specifier.trim().replace(/^type\s+/u, "");
    if (!trimmed) continue;
    const alias = /^(?<local>[$A-Z_a-z][$\w]*)\s+as\s+(?<exported>[$A-Z_a-z][$\w]*)$/.exec(trimmed);
    if (alias?.groups) {
      exports.set(alias.groups.local, alias.groups.exported);
      continue;
    }
    exports.set(trimmed, trimmed);
  }

  return exports;
}

function exportSpecifier(exportedNames, publicName) {
  const exported = exportedNames.get(publicName);
  if (!exported) {
    throw new Error(`Declaration export block is missing ${publicName}`);
  }
  return exported === publicName ? publicName : `${exported} as ${publicName}`;
}

function escapeRegExp(value) {
  return value.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
}
