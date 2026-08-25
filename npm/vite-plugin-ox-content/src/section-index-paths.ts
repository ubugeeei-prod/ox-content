/**
 * Section-index URL, title, and output-path helpers.
 */

import * as path from "node:path";
import { importNapiModuleSync } from "./napi";

export function pageTitle(page: {
  title: string;
  inputPath?: string;
  routePaths: { urlPath: string };
}): string {
  if (page.title.trim()) {
    return page.title;
  }
  const stem = path.basename(page.inputPath ?? page.routePaths.urlPath).replace(/\.[^.]+$/, "");
  return formatSectionTitle(stem || page.routePaths.urlPath);
}

export function sectionTitle(dir: string): string {
  if (!dir) {
    return "Home";
  }
  const segment = dir.slice(dir.lastIndexOf("/") + 1);
  return formatSectionTitle(segment);
}

export function formatSectionTitle(name: string): string {
  try {
    return importNapiModuleSync().formatSsgTitle(name);
  } catch {
    if (!name) {
      return "Untitled";
    }
    return name.charAt(0).toUpperCase() + name.slice(1).replace(/[-_]+/g, " ");
  }
}

export function normalizeUrlPath(urlPath: string): string {
  if (!urlPath || urlPath === "/") {
    return "";
  }
  return urlPath.replace(/^\/+|\/+$/g, "");
}

export function parentDir(urlPath: string): string | null {
  const normalized = normalizeUrlPath(urlPath);
  if (!normalized) {
    return null;
  }
  const index = normalized.lastIndexOf("/");
  return index === -1 ? "" : normalized.slice(0, index);
}

export function firstChildDir(urlPath: string, parent: string): string | undefined {
  const normalized = normalizeUrlPath(urlPath);
  if (!normalized) {
    return undefined;
  }
  if (!parent) {
    const slash = normalized.indexOf("/");
    return slash === -1 ? undefined : normalized.slice(0, slash);
  }
  const prefix = `${parent}/`;
  if (!normalized.startsWith(prefix) || normalized === parent) {
    return undefined;
  }
  const rest = normalized.slice(prefix.length);
  const slash = rest.indexOf("/");
  return slash === -1 ? undefined : `${parent}/${rest.slice(0, slash)}`;
}

export function sectionHref(base: string, dir: string, extension: string): string {
  const prefix = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  const ext = extension.startsWith(".") ? extension : `.${extension}`;
  return dir ? `${prefix}${dir}/index${ext}` : `${prefix}index${ext}`;
}

export function sectionOutputPath(
  outDir: string,
  dir: string,
  extension: string,
): string | undefined {
  const ext = extension.startsWith(".") ? extension : `.${extension}`;
  const segments = dir ? [...dir.split("/").filter(Boolean), `index${ext}`] : [`index${ext}`];
  return containedPath(outDir, ...segments);
}

export function dirFromOutputPath(outputPath: string, outDir: string): string {
  const relative = path
    .relative(path.resolve(outDir), path.resolve(outputPath))
    .replaceAll(path.sep, "/");
  return relative
    .replace(/\/index\.[^/]+$/u, "")
    .replace(/^index\.[^/]+$/u, "")
    .replace(/^\/+|\/+$/g, "");
}

function containedPath(outDir: string, ...segments: string[]): string | undefined {
  const root = path.resolve(outDir);
  const resolved = path.resolve(root, ...segments);
  const prefix = root.endsWith(path.sep) ? root : `${root}${path.sep}`;
  if (resolved !== root && !resolved.startsWith(prefix)) {
    return undefined;
  }
  if (segments.some((segment) => segment === ".." || segment.includes("\0"))) {
    return undefined;
  }
  return resolved;
}
