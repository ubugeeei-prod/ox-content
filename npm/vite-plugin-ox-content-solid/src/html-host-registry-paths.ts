import fsSync from "node:fs";
import path from "node:path";
import { stripViteQuery } from "@ox-content/vite-plugin";

export function toSolidHtmlHostClientModuleId(moduleId: string, root = process.cwd()): string {
  const [pathname, suffix = ""] = splitModuleSuffix(moduleId);
  if (isBareSpecifier(pathname) || pathname.startsWith("/@fs/")) {
    return `${pathname}${suffix}`;
  }

  const rootPath = path.resolve(root);
  const realRoot = existingRealpath(rootPath) ?? rootPath;
  const absolute = path.isAbsolute(pathname)
    ? path.resolve(pathname)
    : path.resolve(rootPath, pathname);
  const realAbsolute = existingRealpath(absolute);
  const isInsideRealRoot = realAbsolute ? isInsideRoot(realAbsolute, realRoot) : false;
  const isInsideLexicalRoot = isInsideRoot(absolute, rootPath);

  if (path.isAbsolute(pathname) && !isInsideRealRoot && !isInsideLexicalRoot) {
    return fsSync.existsSync(pathname)
      ? `/@fs${toPosixPath(pathname)}${suffix}`
      : `${toPosixPath(pathname)}${suffix}`;
  }

  const relativeRoot = isInsideRealRoot ? realRoot : rootPath;
  const relativePath = isInsideRealRoot && realAbsolute ? realAbsolute : absolute;
  const relative = path.relative(relativeRoot, relativePath).replace(/\\/g, "/");
  return `/${relative}${suffix}`;
}

export function resolveDocumentPath(documentPath: string, root: string): string {
  return path.isAbsolute(documentPath)
    ? stripViteQuery(documentPath)
    : path.resolve(root, stripViteQuery(documentPath));
}

export function resolveWatchFile(file: string, root: string): string {
  return path.isAbsolute(file) ? stripViteQuery(file) : path.resolve(root, stripViteQuery(file));
}

export function shouldInvalidate(
  file: string,
  root: string,
  srcDir: string | undefined,
  watchFiles: readonly string[],
  extraWatchFiles: readonly string[] | undefined,
): boolean {
  const changed = path.resolve(file);
  if (watchFiles.some((watchFile) => changed === watchFile)) return true;
  if (extraWatchFiles?.some((watchFile) => changed === resolveWatchFile(watchFile, root))) {
    return true;
  }
  const contentRoot = path.resolve(root, srcDir ?? "content");
  return isInsideRoot(changed, contentRoot);
}

function splitModuleSuffix(moduleId: string): [string, string?] {
  const match = /[?#]/u.exec(moduleId);
  return match ? [moduleId.slice(0, match.index), moduleId.slice(match.index)] : [moduleId];
}

export function isBareSpecifier(moduleId: string): boolean {
  return !moduleId.startsWith(".") && !moduleId.startsWith("/") && !moduleId.includes("\\");
}

function isInsideRoot(file: string, root: string): boolean {
  const relative = path.relative(path.resolve(root), path.resolve(file));
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

function existingRealpath(file: string): string | undefined {
  try {
    return fsSync.realpathSync.native(file);
  } catch {
    return undefined;
  }
}

function toPosixPath(file: string): string {
  return file.replace(/\\/g, "/");
}
