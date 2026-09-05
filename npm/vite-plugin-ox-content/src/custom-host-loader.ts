import * as fsSync from "node:fs";
import * as path from "node:path";
import { createServer, type ResolvedConfig, type ViteDevServer } from "vite";
import { LOAD_ENV } from "./custom-host-constants";
import type {
  OxContentCustomHostAssetsContext,
  OxContentCustomHostBaseContext,
  OxContentCustomHostModule,
  OxContentCustomHostRoute,
  OxContentCustomHostRoutesContext,
} from "./custom-host-types";
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

export function createBaseContext(
  mode: "build" | "serve",
  root: string,
  outDir: string,
  options: ResolvedOptions,
  loadModule: (moduleId: string) => Promise<unknown>,
  assets: OxContentCustomHostAssetsContext,
): OxContentCustomHostBaseContext {
  return {
    mode,
    root,
    outDir,
    base: options.base,
    options,
    loadModule,
    assets,
  };
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
