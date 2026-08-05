/**
 * Resolves the `components` option into a name → path map, expanding glob
 * patterns against the Vite project root.
 */

import * as fs from "fs";
import * as path from "path";
import type { ComponentsMap, ComponentsOption } from "./types";

export async function resolveComponentsGlob(
  componentsOption: ComponentsOption,
  root: string,
): Promise<ComponentsMap> {
  if (typeof componentsOption === "object" && !Array.isArray(componentsOption)) {
    return componentsOption;
  }

  const patterns = Array.isArray(componentsOption) ? componentsOption : [componentsOption];
  const result: ComponentsMap = {};

  for (const pattern of patterns) {
    for (const file of await globFiles(pattern, root)) {
      const baseName = path.basename(file, path.extname(file));
      const relativePath = "./" + path.relative(root, file).replace(/\\/g, "/");
      result[toPascalCase(baseName)] = relativePath;
    }
  }

  return result;
}

async function globFiles(pattern: string, root: string): Promise<string[]> {
  const files: string[] = [];

  if (!pattern.includes("*")) {
    const fullPath = path.resolve(root, pattern);
    if (fs.existsSync(fullPath)) {
      files.push(fullPath);
    }
    return files;
  }

  const parts = pattern.split("*");
  const baseDir = path.resolve(root, parts[0]);
  // The suffix to match lives after the last `*`, so patterns that contain more
  // than one wildcard (`**/*.tsx`) still filter on the file extension instead of
  // matching every file under `baseDir`.
  const ext = parts[parts.length - 1] || "";

  if (!fs.existsSync(baseDir)) {
    return files;
  }

  if (pattern.includes("**")) {
    await walkDir(baseDir, files, ext);
  } else {
    const entries = await fs.promises.readdir(baseDir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isFile() && entry.name.endsWith(ext)) {
        files.push(path.join(baseDir, entry.name));
      }
    }
  }

  return files;
}

async function walkDir(dir: string, files: string[], ext: string): Promise<void> {
  const entries = await fs.promises.readdir(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);

    if (entry.isDirectory()) {
      await walkDir(fullPath, files, ext);
    } else if (entry.isFile() && entry.name.endsWith(ext)) {
      files.push(fullPath);
    }
  }
}

function toPascalCase(str: string): string {
  return str.replace(/[-_](\w)/g, (_, c) => c.toUpperCase()).replace(/^\w/, (c) => c.toUpperCase());
}
