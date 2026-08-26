import { existsSync, readFileSync, realpathSync, statSync } from "node:fs";
import path from "node:path";
import type { ParsedProjectOptions } from "./authoring";
import { normalizeProjectPath } from "./project-sandbox";
import type { ProjectSandboxFile } from "./types";

export interface ProjectFileCollectionContext {
  documentPath?: string;
  sourceRoot?: string;
  maxFiles?: number;
  maxBytes?: number;
}

export interface ProjectFileCollection {
  files: ProjectSandboxFile[];
  warnings: string[];
}

const DEFAULT_MAX_FILES = 32;
const DEFAULT_MAX_BYTES = 256 * 1024;

export function collectProjectFiles(
  project: ParsedProjectOptions | undefined,
  context: ProjectFileCollectionContext = {},
): ProjectFileCollection {
  const warnings: string[] = [];
  if (!project?.files.length) {
    return { files: [], warnings };
  }
  if (!context.documentPath) {
    warnings.push("Skipped project files because the Markdown source path is unavailable.");
    return { files: [], warnings };
  }

  const documentDir = path.dirname(context.documentPath);
  const sourceRoot = path.resolve(context.sourceRoot ?? documentDir);
  const realSourceRoot = realpathIfExists(sourceRoot) ?? sourceRoot;
  const files: ProjectSandboxFile[] = [];
  for (const requested of project.files) {
    if (files.length >= (context.maxFiles ?? DEFAULT_MAX_FILES)) {
      warnings.push(`Skipped project file after ${files.length} files: ${requested}`);
      continue;
    }
    const safePath = normalizeProjectPath(requested, warnings);
    if (!safePath) {
      continue;
    }
    const absolute = path.resolve(documentDir, safePath);
    if (!pathInside(absolute, sourceRoot)) {
      warnings.push(`Skipped project file outside source root: ${safePath}`);
      continue;
    }
    const realAbsolute = realpathIfExists(absolute);
    if (!realAbsolute) {
      warnings.push(`Skipped missing project file: ${safePath}`);
      continue;
    }
    if (!pathInside(realAbsolute, realSourceRoot)) {
      warnings.push(`Skipped project file outside real source root: ${safePath}`);
      continue;
    }
    const stat = statSync(realAbsolute);
    if (!stat.isFile()) {
      warnings.push(`Skipped non-file project path: ${safePath}`);
      continue;
    }
    if (stat.size > (context.maxBytes ?? DEFAULT_MAX_BYTES)) {
      warnings.push(`Skipped large project file: ${safePath}`);
      continue;
    }
    files.push({ path: safePath, code: readFileSync(realAbsolute, "utf8") });
  }
  return { files, warnings };
}

function realpathIfExists(file: string): string | undefined {
  if (!existsSync(file)) {
    return undefined;
  }
  return realpathSync(file);
}

function pathInside(file: string, root: string): boolean {
  const relative = path.relative(root, file);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}
