import * as fsSync from "node:fs";
import * as path from "node:path";
import { createServer, type ResolvedConfig, type ViteDevServer } from "vite";
import { LOAD_ENV } from "./custom-host-constants";
import type {
  OxContentCustomHostAssetsContext,
  OxContentCustomHostBaseContext,
  OxContentCustomHostMemo,
  OxContentCustomHostModule,
  OxContentCustomHostOutputData,
  OxContentCustomHostOutputsContext,
  OxContentCustomHostRoute,
  OxContentCustomHostRoutesContext,
} from "./custom-host-types";
import { createLazyCustomHostMarkdownRenderer } from "./custom-host-markdown-lazy";
import { canonicalFilePath } from "./custom-host-utils";
import type { ResolvedOptions } from "./types";

export async function createHostLoaderServer(config: ResolvedConfig): Promise<ViteDevServer> {
  const previous = process.env[LOAD_ENV];
  process.env[LOAD_ENV] = "1";
  try {
    return await createServer({
      root: config.root,
      mode: config.mode,
      configFile: config.configFile ?? false,
      appType: "custom",
      logLevel: config.logLevel,
      server: { middlewareMode: true },
      plugins: [
        {
          name: "ox-content:custom-host-build-loader",
          enforce: "post",
          configResolved(resolved) {
            // Vite's config merge skips null overrides inside consumer server
            // options. Disable the build-only watcher after that merge.
            resolved.server.watch = null;
          },
        },
      ],
    });
  } finally {
    if (previous === undefined) {
      delete process.env[LOAD_ENV];
    } else {
      process.env[LOAD_ENV] = previous;
    }
  }
}

export async function loadHost(
  host: string | OxContentCustomHostModule,
  loadModule: (moduleId: string) => Promise<unknown>,
  root = process.cwd(),
): Promise<OxContentCustomHostModule> {
  if (typeof host !== "string") {
    return host;
  }
  const moduleId = normalizeHostModuleId(host, root);
  const exports = await loadModule(moduleId);
  return normalizeHostExports(exports, host);
}

export function normalizeHostModuleId(moduleId: string, root: string): string {
  if (path.isAbsolute(moduleId)) {
    if (fsSync.existsSync(moduleId)) {
      return canonicalFilePath(moduleId);
    }
    return canonicalFilePath(path.resolve(root, `.${moduleId}`));
  }
  if (moduleId.startsWith(".") || moduleId.includes("/") || moduleId.includes("\\")) {
    return canonicalFilePath(path.resolve(root, moduleId));
  }
  return moduleId;
}

export async function loadRoutes(
  host: OxContentCustomHostModule,
  context: OxContentCustomHostRoutesContext,
): Promise<readonly OxContentCustomHostRoute[]> {
  return typeof host.routes === "function" ? await host.routes(context) : host.routes;
}

export async function loadOutputs(
  host: OxContentCustomHostModule,
  context: OxContentCustomHostOutputsContext,
): Promise<OxContentCustomHostOutputData | undefined> {
  if (!host.outputs) {
    return undefined;
  }
  const output = typeof host.outputs === "function" ? await host.outputs(context) : host.outputs;
  return output || undefined;
}

export function createBaseContext(
  mode: "build" | "serve",
  root: string,
  outDir: string,
  options: ResolvedOptions,
  loadModule: (moduleId: string) => Promise<unknown>,
  assets: OxContentCustomHostAssetsContext,
): OxContentCustomHostBaseContext {
  const context = {
    mode,
    root,
    outDir,
    base: options.base,
    options,
    loadModule,
    assets,
  };
  return { ...context, markdown: createLazyCustomHostMarkdownRenderer(context) };
}

export function createContextMemo(): OxContentCustomHostMemo {
  const values = new Map<string, Promise<unknown>>();
  return async <T>(key: string, load: () => Promise<T> | T): Promise<T> => {
    const existing = values.get(key);
    if (existing) {
      return existing as Promise<T>;
    }
    const current = Promise.resolve().then(load);
    values.set(key, current);
    try {
      return await current;
    } catch (error) {
      if (values.get(key) === current) {
        values.delete(key);
      }
      throw error;
    }
  };
}

export function createRoutesContext(
  base: OxContentCustomHostBaseContext,
  memo = createContextMemo(),
): OxContentCustomHostRoutesContext {
  return { ...base, memo };
}

export function createOutputsContext(
  base: OxContentCustomHostBaseContext,
  routes: readonly OxContentCustomHostRoute[],
  memo: OxContentCustomHostMemo,
): OxContentCustomHostOutputsContext {
  return { ...base, routes, memo };
}

function normalizeHostExports(exports: unknown, moduleId: string): OxContentCustomHostModule {
  const record = exports as {
    default?: unknown;
    host?: unknown;
    routes?: unknown;
  };
  const candidate = record.default ?? record.host ?? exports;
  if (!candidate || typeof candidate !== "object" || !("routes" in candidate)) {
    throw new Error(
      `[ox-content] Custom host module ${JSON.stringify(moduleId)} does not export routes.`,
    );
  }
  return candidate as OxContentCustomHostModule;
}
