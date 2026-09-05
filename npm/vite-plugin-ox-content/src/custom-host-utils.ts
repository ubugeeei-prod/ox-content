import * as fsSync from "node:fs";
import * as path from "node:path";
import type { Connect, ResolvedConfig, ViteDevServer } from "vite";
import { HTML_CONTENT_TYPE_RE } from "./custom-host-constants";
import type { OxContentCustomHostRenderResult, SerializedResponse } from "./custom-host-types";
import type { ResolvedOptions } from "./types";

export function resultBody(result: OxContentCustomHostRenderResult): string | Uint8Array {
  if (result.body != null) {
    return result.body;
  }
  if (result.html != null) {
    return result.html;
  }
  return result.text ?? "";
}

export function contentTypeFromHeaders(headers: HeadersInit | undefined): string | undefined {
  if (!headers) {
    return undefined;
  }
  return new Headers(headers).get("content-type") ?? undefined;
}

export function shouldTransformHtml(contentType: string, enabled = true): boolean {
  return enabled !== false && isHtmlContentType(contentType);
}

export function isHtmlContentType(contentType: string): boolean {
  return HTML_CONTENT_TYPE_RE.test(contentType);
}

export function routeOutputPath(outDir: string, routePath: string, contentType: string): string {
  const clean = routePath.replace(/^\/+/u, "");
  if (!clean) {
    return path.join(outDir, "index.html");
  }
  const hasExtension = path.posix.extname(clean) !== "";
  if (!isHtmlContentType(contentType) || hasExtension) {
    return resolveOutputPath(outDir, clean);
  }
  return resolveOutputPath(outDir, path.posix.join(clean, "index.html"));
}

export function resolveOutputPath(outDir: string, outputPath: string): string {
  const root = path.resolve(outDir);
  const relative = outputPath.startsWith("/") ? outputPath.slice(1) : outputPath;
  if (!isSafeRelativePath(relative)) {
    throw new Error(
      `[ox-content] Custom host output path ${JSON.stringify(outputPath)} is unsafe.`,
    );
  }
  return path.resolve(root, ...relative.split("/"));
}

export function claimOutput(
  owners: Map<string, string>,
  outputPath: string,
  routePath: string,
): void {
  const key = path.resolve(outputPath);
  const owner = owners.get(key);
  if (owner) {
    throw new Error(
      `[ox-content] Custom host route ${JSON.stringify(routePath)} conflicts with ${JSON.stringify(owner)} at ${key}.`,
    );
  }
  owners.set(key, routePath);
}

export function normalizeRoutePath(routePath: string): string {
  const pathname = safePathname(routePath);
  if (pathname === "/") {
    return "/";
  }
  const withoutIndex = pathname.endsWith("/index.html")
    ? pathname.slice(0, -"/index.html".length) || "/"
    : pathname;
  return withoutIndex.endsWith("/") ? withoutIndex.slice(0, -1) : withoutIndex;
}

export function stripBasePathname(pathname: string, base: string): string | undefined {
  const normalizedBase = normalizeBase(base);
  if (normalizedBase === "/") {
    return pathname;
  }
  const bareBase = normalizedBase.slice(0, -1);
  if (pathname === bareBase) {
    return "/";
  }
  if (!pathname.startsWith(normalizedBase)) {
    return undefined;
  }
  return `/${pathname.slice(normalizedBase.length)}`;
}

export function ssgUrlPath(routePath: string): string {
  if (routePath === "/") {
    return "/";
  }
  return routePath.replace(/^\/+/u, "").replace(/\/index\.html$/u, "");
}

export function resolveInputPath(inputPath: string | undefined, root: string): string {
  return inputPath ? path.resolve(root, inputPath) : path.join(root, "ox-content-custom-host.html");
}

export function normalizeDependencies(
  root: string,
  dependencies: readonly string[] | undefined,
): string[] {
  return (dependencies ?? [])
    .filter((dependency) => dependency.length > 0)
    .map((dependency) => resolveDependency(root, dependency));
}

export function versionedModuleId(moduleId: string, version: number): string {
  if (version === 0 || !isFileLikeModuleId(moduleId)) {
    return moduleId;
  }
  const separator = moduleId.includes("?") ? "&" : "?";
  return `${moduleId}${separator}ox_content_custom_host_v=${version}`;
}

export function isFileLikeModuleId(moduleId: string): boolean {
  return moduleId.startsWith("/") || moduleId.startsWith(".") || moduleId.includes("\\");
}

export function invalidateViteModules(server: ViteDevServer, file: string, all = false): void {
  const moduleGraph = server.moduleGraph as ViteDevServer["moduleGraph"] & {
    invalidateAll?: () => void;
  };
  if (all && typeof moduleGraph.invalidateAll === "function") {
    moduleGraph.invalidateAll();
    return;
  }
  for (const mod of server.moduleGraph.getModulesByFile(file) ?? []) {
    server.moduleGraph.invalidateModule(mod);
  }
}

export function resolveDependency(root: string, dependency: string): string {
  return canonicalFilePath(
    path.isAbsolute(dependency) ? dependency : path.resolve(root, dependency),
  );
}

export function canonicalFilePath(file: string): string {
  const resolved = path.resolve(file);
  try {
    return fsSync.realpathSync.native(resolved);
  } catch {
    return resolved;
  }
}

export function clearKeyDeps(
  key: string,
  keyDeps: Map<string, Set<string>>,
  depKeys: Map<string, Set<string>>,
): void {
  for (const dep of keyDeps.get(key) ?? []) {
    const keys = depKeys.get(dep);
    keys?.delete(key);
    if (keys?.size === 0) {
      depKeys.delete(dep);
    }
  }
  keyDeps.delete(key);
}

export async function serializeResponse(
  response: Response,
  dependencies: readonly string[],
): Promise<SerializedResponse> {
  return {
    status: response.status,
    statusText: response.statusText,
    headers: Array.from(response.headers.entries()).filter(
      ([name]) => name !== "x-ox-content-dependencies",
    ),
    body: new Uint8Array(await response.arrayBuffer()),
    dependencies: [...new Set(dependencies)],
  };
}

export function deserializeResponse(serialized: SerializedResponse, head: boolean): Response {
  return new Response(head ? null : serialized.body, {
    headers: serialized.headers,
    status: serialized.status,
    statusText: serialized.statusText,
  });
}

export function connectRequestToRequest(
  req: Parameters<Connect.NextHandleFunction>[0],
): Request | null {
  if (!req.url) {
    return null;
  }
  return new Request(new URL(req.url, "http://localhost"), {
    method: req.method ?? "GET",
    headers: connectRequestHeaders(req.headers),
  });
}

export function connectRequestHeaders(
  headers: Parameters<Connect.NextHandleFunction>[0]["headers"] | undefined,
): Headers {
  const result = new Headers();
  if (!headers) {
    return result;
  }
  for (const [name, value] of Object.entries(headers)) {
    if (Array.isArray(value)) {
      for (const item of value) {
        result.append(name, item);
      }
    } else if (value !== undefined) {
      result.set(name, value);
    }
  }
  return result;
}

export async function writeConnectResponse(
  response: Response,
  res: Parameters<Connect.NextHandleFunction>[1],
): Promise<void> {
  res.statusCode = response.status;
  response.headers.forEach((value, name) => {
    res.setHeader(name, value);
  });
  res.end(new Uint8Array(await response.arrayBuffer()));
}

export function patchServerClose(server: ViteDevServer, cleanup: () => void): void {
  const original = server.close.bind(server);
  let cleaned = false;
  server.close = async () => {
    if (!cleaned) {
      cleaned = true;
      cleanup();
    }
    return original();
  };
}

export function resolveOutDir(
  config: ResolvedConfig,
  options: ResolvedOptions,
  root: string,
): string {
  const outDir = config.build.outDir || options.outDir;
  return path.isAbsolute(outDir) ? outDir : path.resolve(root, outDir);
}

export function normalizeBase(base: string): string {
  if (!base || base === "/") {
    return "/";
  }
  const withLeading = base.startsWith("/") ? base : `/${base}`;
  return withLeading.endsWith("/") ? withLeading : `${withLeading}/`;
}

export function withBase(base: string, href: string): string {
  if (href.startsWith("#") || href.startsWith("//") || /^[a-z][a-z0-9+.-]*:/iu.test(href)) {
    return href;
  }
  const normalizedBase = normalizeBase(base);
  if (normalizedBase === "/") {
    return href.startsWith("/") ? href : `/${href}`;
  }
  if (href === normalizedBase.slice(0, -1) || href.startsWith(normalizedBase)) {
    return href;
  }
  return `${normalizedBase}${href.replace(/^\/+/u, "")}`;
}

function safePathname(value: string): string {
  const pathname = new URL(value, "http://localhost").pathname;
  try {
    return decodeURIComponent(pathname);
  } catch {
    return pathname;
  }
}

function isSafeRelativePath(relative: string): boolean {
  return (
    relative.length > 0 &&
    !relative.includes("\0") &&
    !relative.includes("\\") &&
    path.posix.normalize(relative) === relative &&
    !relative.startsWith("../") &&
    relative !== ".."
  );
}
