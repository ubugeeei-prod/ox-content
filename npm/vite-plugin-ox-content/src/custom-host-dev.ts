import type { ViteDevServer } from "vite";
import { createAssetsContext, themeTokenMiddleware } from "./custom-host-assets";
import {
  createBaseContext,
  loadHost,
  loadRoutes,
  normalizeHostModuleId,
} from "./custom-host-loader";
import { renderRoute, responseFromResult } from "./custom-host-render";
import type {
  DevCacheEntry,
  OxContentCustomHostBaseContext,
  OxContentCustomHostModule,
  OxContentCustomHostOptions,
  OxContentCustomHostRoute,
  ResolvedThemeTokens,
} from "./custom-host-types";
import {
  canonicalFilePath,
  clearKeyDeps,
  connectRequestToRequest,
  deserializeResponse,
  invalidateViteModules,
  normalizeDependencies,
  normalizeRoutePath,
  patchServerClose,
  resolveDependency,
  resolveOutDir,
  serializeResponse,
  stripBasePathname,
  versionedModuleId,
  writeConnectResponse,
} from "./custom-host-utils";
import type { ResolvedOptions } from "./types";

export function configureDevServer(
  server: ViteDevServer,
  input: OxContentCustomHostOptions,
  options: ResolvedOptions,
  themeTokens: ResolvedThemeTokens | undefined,
): void {
  const root = server.config.root;
  const outDir = resolveOutDir(server.config, options, root);
  const assets = createAssetsContext(
    options,
    outDir,
    undefined,
    themeTokens,
    server.moduleGraph,
    root,
  );
  let ssrVersion = 0;
  const loadModule = (moduleId: string) =>
    server.ssrLoadModule(versionedModuleId(moduleId, ssrVersion));
  const baseContext = createBaseContext("serve", root, outDir, options, loadModule, assets);
  const cache = new Map<string, DevCacheEntry>();
  const keyDeps = new Map<string, Set<string>>();
  const depKeys = new Map<string, Set<string>>();
  let hostPromise: Promise<OxContentCustomHostModule> | undefined;
  let routesPromise: Promise<readonly OxContentCustomHostRoute[]> | undefined;
  let reloadTimer: ReturnType<typeof setTimeout> | undefined;

  const scheduleReload = () => {
    const delay = input.dev?.reloadDebounceMs ?? 80;
    if (reloadTimer) {
      clearTimeout(reloadTimer);
    }
    reloadTimer = setTimeout(() => {
      reloadTimer = undefined;
      server.ws.send({ type: "full-reload" });
    }, delay);
  };

  const clearKey = (key: string) => {
    cache.delete(key);
    for (const dep of keyDeps.get(key) ?? []) {
      const keys = depKeys.get(dep);
      keys?.delete(key);
      if (keys?.size === 0) {
        depKeys.delete(dep);
      }
    }
    keyDeps.delete(key);
  };

  const rememberDeps = (key: string, dependencies: readonly string[]) => {
    clearKeyDeps(key, keyDeps, depKeys);
    const normalized = new Set(
      dependencies.map((dependency) => resolveDependency(root, dependency)),
    );
    keyDeps.set(key, normalized);
    for (const dep of normalized) {
      const keys = depKeys.get(dep) ?? new Set<string>();
      keys.add(key);
      depKeys.set(dep, keys);
    }
  };

  const clearHost = () => {
    hostPromise = undefined;
    routesPromise = undefined;
    cache.clear();
    keyDeps.clear();
    depKeys.clear();
  };

  const loadCachedHost = async () => {
    hostPromise ??= loadHost(input.host, loadModule, root);
    return hostPromise;
  };

  const loadCachedRoutes = async () => {
    routesPromise ??= loadCachedHost().then((host) => loadRoutes(host, baseContext));
    return routesPromise;
  };

  const onChange = (_event: string, file: string) => {
    const changed = canonicalFilePath(file);
    if (typeof input.host === "string" && changed === normalizeHostModuleId(input.host, root)) {
      ssrVersion += 1;
      invalidateViteModules(server, changed, true);
      clearHost();
      scheduleReload();
      return;
    }

    let invalidated = false;
    for (const key of depKeys.get(changed) ?? []) {
      clearKey(key);
      invalidated = true;
    }
    if (invalidated) {
      ssrVersion += 1;
      invalidateViteModules(server, changed, true);
      scheduleReload();
    }
  };
  const onAdd = (file: string) => onChange("add", file);
  const onFileChange = (file: string) => onChange("change", file);
  const onUnlink = (file: string) => onChange("unlink", file);

  server.watcher.on("all", onChange);
  server.watcher.on("add", onAdd);
  server.watcher.on("change", onFileChange);
  server.watcher.on("unlink", onUnlink);
  server.middlewares.use(themeTokenMiddleware(themeTokens));
  server.middlewares.use(async (req, res, next) => {
    const request = connectRequestToRequest(req);
    if (!request || (request.method !== "GET" && request.method !== "HEAD")) {
      next();
      return;
    }

    try {
      const response = await devResponse({
        request,
        server,
        input,
        context: baseContext,
        loadHost: loadCachedHost,
        loadRoutes: loadCachedRoutes,
        cache,
        rememberDeps,
      });
      if (!response) {
        next();
        return;
      }
      await writeConnectResponse(response, res);
    } catch (error) {
      next(error);
    }
  });

  patchServerClose(server, () => {
    if (reloadTimer) {
      clearTimeout(reloadTimer);
      reloadTimer = undefined;
    }
    server.watcher.off("all", onChange);
    server.watcher.off("add", onAdd);
    server.watcher.off("change", onFileChange);
    server.watcher.off("unlink", onUnlink);
    cache.clear();
    keyDeps.clear();
    depKeys.clear();
  });
}

async function devResponse(input: {
  request: Request;
  server: ViteDevServer;
  input: OxContentCustomHostOptions;
  context: OxContentCustomHostBaseContext;
  loadHost(): Promise<OxContentCustomHostModule>;
  loadRoutes(): Promise<readonly OxContentCustomHostRoute[]>;
  cache: Map<string, DevCacheEntry>;
  rememberDeps(key: string, dependencies: readonly string[]): void;
}): Promise<Response | undefined> {
  const url = new URL(input.request.url);
  const routePath = normalizeRoutePath(
    stripBasePathname(url.pathname, input.context.base) ?? url.pathname,
  );
  const routes = await input.loadRoutes();
  const route = routes.find((candidate) => normalizeRoutePath(candidate.path) === routePath);
  const host = await input.loadHost();

  if (!route) {
    const result = await host.notFound?.({ ...input.context, request: input.request, url });
    if (!result) {
      return undefined;
    }
    return responseFromResult(result, routePath, input.server, input.input.dev?.transformHtml);
  }

  const key = `${input.request.method}\0${routePath}\0${url.search}`;
  let entry = input.cache.get(key);
  if (!entry) {
    const current: DevCacheEntry = {
      promise: renderDevResponse(input, route, host, routePath)
        .then((serialized) => {
          if (serialized && input.cache.get(key) === current) {
            input.rememberDeps(key, serialized.dependencies);
          }
          return serialized;
        })
        .catch((error) => {
          if (input.cache.get(key) === current) {
            input.cache.delete(key);
          }
          throw error;
        }),
    };
    entry = current;
    input.cache.set(key, entry);
  }

  const serialized = await entry.promise;
  if (!serialized) {
    return undefined;
  }
  return deserializeResponse(serialized, input.request.method === "HEAD");
}

async function renderDevResponse(
  input: {
    request: Request;
    server: ViteDevServer;
    input: OxContentCustomHostOptions;
    context: OxContentCustomHostBaseContext;
  },
  route: OxContentCustomHostRoute,
  host: OxContentCustomHostModule,
  routePath: string,
) {
  const result = await renderRoute(host, route, input.context, input.request);
  if (!result) {
    return undefined;
  }
  const response = await responseFromResult(
    result,
    routePath,
    input.server,
    input.input.dev?.transformHtml,
  );
  const dependencies = [
    ...normalizeDependencies(input.context.root, route.dependencies),
    ...(result instanceof Response
      ? []
      : normalizeDependencies(input.context.root, result.dependencies)),
  ];
  return serializeResponse(response, dependencies);
}
