import * as fsSync from "node:fs";
import * as path from "node:path";
import type { Alias, ViteDevServer } from "vite";
import type { CustomHostDevImportResolver } from "./custom-host-ssr-dev-stylesheets";
import type { CustomHostDevStylesheetContentResolver } from "./custom-host-stylesheet-content";
import type { CustomHostDevModuleGraph, CustomHostDevModuleNode } from "./custom-host-stylesheets";
import { stripBasePathname } from "./custom-host-utils";
import { viteModuleGraphs } from "./custom-host-vite-invalidation";

export function createCombinedDevModuleGraph(server: ViteDevServer): CustomHostDevModuleGraph {
  return {
    get idToModuleMap() {
      const modules = new Map<string, CustomHostDevModuleNode>();
      for (const graph of viteModuleGraphs(server)) {
        for (const [id, node] of graph.idToModuleMap ?? []) {
          modules.set(id, node as CustomHostDevModuleNode);
        }
      }
      return modules;
    },
    getModuleById(id) {
      for (const graph of viteModuleGraphs(server)) {
        const node = graph.getModuleById?.(id);
        if (node) {
          return node as CustomHostDevModuleNode;
        }
      }
      return undefined;
    },
    getModulesByFile(file) {
      const modules = new Set<CustomHostDevModuleNode>();
      for (const graph of viteModuleGraphs(server)) {
        for (const node of graph.getModulesByFile?.(file) ?? []) {
          modules.add(node as CustomHostDevModuleNode);
        }
      }
      return modules.size > 0 ? modules : undefined;
    },
  };
}

export function createDevImportResolver(
  server: ViteDevServer,
): CustomHostDevImportResolver | undefined {
  const aliases = server.config.resolve.alias.filter(
    (alias): alias is Alias =>
      (typeof alias.find === "string" || alias.find instanceof RegExp) &&
      typeof alias.replacement === "string",
  );
  const tsconfigPaths = hasTsconfigPathResolver(server)
    ? readTsconfigPaths(server.config.root)
    : undefined;
  if (aliases.length === 0 && (!tsconfigPaths || tsconfigPaths.paths.length === 0)) {
    return undefined;
  }
  return (specifier) =>
    resolveAliasImport(specifier, aliases, server.config.root) ??
    resolveTsconfigPathImport(specifier, tsconfigPaths);
}

export function createDevStylesheetContentResolver(
  server: ViteDevServer,
  base: string,
): CustomHostDevStylesheetContentResolver {
  return async (href) => {
    const url = new URL(href, "http://localhost");
    const pathname =
      stripBasePathname(url.pathname, base) ??
      stripBasePathname(url.pathname, server.config.base) ??
      url.pathname;
    const absolutePath = pathname.startsWith("/@fs/")
      ? pathname.slice("/@fs".length)
      : path.join(server.config.root, pathname.replace(/^\/+/u, ""));
    const absoluteId = `${absolutePath}${url.search}`;
    if (
      isSvelteStyleRequest(absolutePath, url.searchParams) &&
      !hasFrameworkStyleBlock(absolutePath)
    ) {
      return "";
    }
    const loaded = await loadSsrStylesheetContent(server, absoluteId);
    if (loaded) {
      return loaded;
    }
    try {
      const result = await server.transformRequest(`${pathname}${url.search}`, { ssr: true });
      return result?.code ? extractViteCssContent(result.code) : undefined;
    } catch (error) {
      return extractViteCssContent(errorPluginCode(error) ?? "");
    }
  };
}

function hasTsconfigPathResolver(server: ViteDevServer): boolean {
  return (
    (
      server.config.resolve as typeof server.config.resolve & {
        tsconfigPaths?: boolean;
      }
    ).tsconfigPaths === true
  );
}

function resolveAliasImport(
  specifier: string,
  aliases: readonly Alias[],
  root: string,
): string | undefined {
  for (const alias of aliases) {
    const replaced =
      typeof alias.find === "string"
        ? replaceStringAlias(specifier, alias.find, alias.replacement)
        : specifier.replace(alias.find, alias.replacement);
    if (replaced && replaced !== specifier) {
      return resolveImportCandidate(replaced, root);
    }
  }
  return undefined;
}

function replaceStringAlias(
  specifier: string,
  find: string,
  replacement: string,
): string | undefined {
  if (specifier === find || specifier.startsWith(`${find}/`)) {
    return `${replacement}${specifier.slice(find.length)}`;
  }
  return undefined;
}

type TsconfigPaths = {
  baseUrl: string;
  paths: { pattern: string; targets: string[] }[];
};

function readTsconfigPaths(root: string): TsconfigPaths | undefined {
  try {
    const config = JSON.parse(fsSync.readFileSync(path.join(root, "tsconfig.json"), "utf8")) as {
      compilerOptions?: {
        baseUrl?: unknown;
        paths?: Record<string, unknown>;
      };
    };
    const rawPaths = config.compilerOptions?.paths;
    if (!rawPaths) {
      return undefined;
    }
    return {
      baseUrl: path.resolve(
        root,
        typeof config.compilerOptions?.baseUrl === "string" ? config.compilerOptions.baseUrl : ".",
      ),
      paths: Object.entries(rawPaths)
        .map(([pattern, targets]) => ({
          pattern,
          targets: Array.isArray(targets)
            ? targets.filter((target): target is string => typeof target === "string")
            : [],
        }))
        .filter((entry) => entry.targets.length > 0),
    };
  } catch {
    return undefined;
  }
}

function resolveTsconfigPathImport(
  specifier: string,
  tsconfigPaths: TsconfigPaths | undefined,
): string | undefined {
  if (!tsconfigPaths) {
    return undefined;
  }
  for (const { pattern, targets } of tsconfigPaths.paths) {
    const wildcard = tsconfigWildcard(pattern, specifier);
    if (wildcard == null) {
      continue;
    }
    for (const target of targets) {
      return path.resolve(tsconfigPaths.baseUrl, target.replace("*", wildcard));
    }
  }
  return undefined;
}

function tsconfigWildcard(pattern: string, specifier: string): string | undefined {
  if (!pattern.includes("*")) {
    return pattern === specifier ? "" : undefined;
  }
  const [prefix, suffix = ""] = pattern.split("*", 2);
  if (!specifier.startsWith(prefix) || !specifier.endsWith(suffix)) {
    return undefined;
  }
  return specifier.slice(prefix.length, specifier.length - suffix.length);
}

function resolveImportCandidate(specifier: string, root: string): string {
  if (specifier.startsWith("/@fs/")) {
    return specifier.slice("/@fs".length);
  }
  return path.isAbsolute(specifier) ? specifier : path.resolve(root, specifier);
}

function isSvelteStyleRequest(file: string, params: URLSearchParams): boolean {
  return file.endsWith(".svelte") && params.get("type") === "style";
}

function hasFrameworkStyleBlock(file: string): boolean {
  try {
    return /<style(?:\s[^>]*)?>[\s\S]*?<\/style>/iu.test(fsSync.readFileSync(file, "utf8"));
  } catch {
    return true;
  }
}

async function loadSsrStylesheetContent(
  server: ViteDevServer,
  id: string,
): Promise<string | undefined> {
  const environment = (
    server as ViteDevServer & {
      environments?: Record<string, { pluginContainer?: { load(id: string): Promise<unknown> } }>;
    }
  ).environments?.ssr;
  const loaded = await environment?.pluginContainer?.load(id);
  return extractViteCssContent(loadResultCode(loaded) ?? "");
}

function loadResultCode(result: unknown): string | undefined {
  if (typeof result === "string") {
    return result;
  }
  if (!result || typeof result !== "object") {
    return undefined;
  }
  const code = (result as { code?: unknown }).code;
  return typeof code === "string" ? code : undefined;
}

function errorPluginCode(error: unknown): string | undefined {
  if (!error || typeof error !== "object") {
    return undefined;
  }
  const pluginCode = (error as { pluginCode?: unknown }).pluginCode;
  return typeof pluginCode === "string" ? pluginCode : undefined;
}

function extractViteCssContent(code: string): string | undefined {
  const trimmed = code.trim();
  if (!trimmed) {
    return undefined;
  }
  if (trimmed.includes("__vite__updateStyle")) {
    const match = /const\s+__vite__css\s*=\s*("(?:\\.|[^"\\])*")/u.exec(trimmed);
    if (!match) {
      return undefined;
    }
    try {
      const css = JSON.parse(match[1]) as unknown;
      return typeof css === "string" && !isRawSvelteSource(css) ? css : undefined;
    } catch {
      return undefined;
    }
  }
  return isRawSvelteSource(trimmed) || isLikelyJsModule(trimmed) ? undefined : trimmed;
}

function isRawSvelteSource(content: string): boolean {
  return /<\/(?:script|style)>/iu.test(content);
}

function isLikelyJsModule(content: string): boolean {
  return /^(?:import|export)\s/mu.test(content);
}
