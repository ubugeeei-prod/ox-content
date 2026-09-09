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
  const normalized = pattern.replace(/\\/g, "/").replace(/^\.\//, "");

  if (!hasWildcard(normalized)) {
    const fullPath = path.resolve(root, normalized);
    if (fs.existsSync(fullPath)) {
      files.push(fullPath);
    }
    return files;
  }

  // Walk from the deepest wildcard-free prefix, then keep only the paths the
  // whole pattern matches. Deriving the base directory alone is not enough:
  // `src/components/**/*.tsx` must not collect the non-component files below
  // `src/components`, and the generated module would fail to import them.
  const baseDir = path.resolve(root, staticPrefix(normalized));
  if (!fs.existsSync(baseDir)) {
    return files;
  }

  // Only a pattern whose wildcards are confined to the file name can stay
  // shallow. A wildcard in a directory segment (`src/*/Button.tsx`) puts its
  // matches below `baseDir`, and `**` crosses directory boundaries even inside
  // the last segment, so both need the recursive walk.
  const segments = normalized.split("/");
  const crossesDirectories = normalized.includes("**") || segments.slice(0, -1).some(hasWildcard);

  const candidates: string[] = [];
  if (crossesDirectories) {
    await walkDir(baseDir, candidates);
  } else {
    const entries = await fs.promises.readdir(baseDir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isFile()) {
        candidates.push(path.join(baseDir, entry.name));
      }
    }
  }

  const matcher = globToRegExp(normalized);
  for (const candidate of candidates) {
    if (matcher.test(path.relative(root, candidate).replace(/\\/g, "/"))) {
      files.push(candidate);
    }
  }

  return files;
}

/** Whether a pattern (or one segment of it) contains a wildcard `globToRegExp` expands. */
function hasWildcard(pattern: string): boolean {
  return pattern.includes("*") || pattern.includes("?");
}

/** Leading path segments of a pattern that contain no wildcard. */
function staticPrefix(pattern: string): string {
  const segments: string[] = [];
  for (const segment of pattern.split("/")) {
    if (hasWildcard(segment)) break;
    segments.push(segment);
  }
  return segments.join("/");
}

/**
 * Translates a glob into an anchored `RegExp`: `**` crosses directory
 * boundaries, `*` and `?` stay within one segment.
 */
function globToRegExp(pattern: string): RegExp {
  let source = "";
  let index = 0;

  while (index < pattern.length) {
    const char = pattern[index];
    if (char === "*") {
      if (pattern[index + 1] === "*") {
        index += 2;
        if (pattern[index] === "/") {
          index += 1;
          source += "(?:[^/]+/)*";
        } else {
          source += ".*";
        }
        continue;
      }
      source += "[^/]*";
    } else if (char === "?") {
      source += "[^/]";
    } else {
      source += char.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    }
    index += 1;
  }

  return new RegExp(`^${source}$`);
}

async function walkDir(dir: string, files: string[]): Promise<void> {
  const entries = await fs.promises.readdir(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);

    if (entry.isDirectory()) {
      await walkDir(fullPath, files);
    } else if (entry.isFile()) {
      files.push(fullPath);
    }
  }
}

function toPascalCase(str: string): string {
  return str.replace(/[-_](\w)/g, (_, c) => c.toUpperCase()).replace(/^\w/, (c) => c.toUpperCase());
}
