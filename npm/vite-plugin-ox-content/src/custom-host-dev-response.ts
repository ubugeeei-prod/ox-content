import type { ViteDevServer } from "vite";
import { collectDevModuleDependencies } from "./custom-host-module-deps";
import { renderRoute, responseFromResult } from "./custom-host-render";
import type {
  DevCacheEntry,
  OxContentCustomHostBaseContext,
  OxContentCustomHostDependency,
  OxContentCustomHostModule,
  OxContentCustomHostOptions,
  OxContentCustomHostRoute,
} from "./custom-host-types";
import { createLazyCustomHostMarkdownRenderer } from "./custom-host-markdown-lazy";
import {
  deserializeResponse,
  normalizeRoutePath,
  serializeResponse,
  stripBasePathname,
} from "./custom-host-utils";

export async function devResponse(input: {
  request: Request;
  server: ViteDevServer;
  input: OxContentCustomHostOptions;
  context: OxContentCustomHostBaseContext;
  loadHost(): Promise<OxContentCustomHostModule>;
  loadRoutes(): Promise<readonly OxContentCustomHostRoute[]>;
  cache: Map<string, DevCacheEntry>;
  rememberDeps(key: string, dependencies: readonly OxContentCustomHostDependency[]): void;
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
  if (hasRequestIdentityHeaders(input.request)) {
    const serialized = await renderDevResponse(input, route, host, routePath);
    if (!serialized) {
      return undefined;
    }
    input.rememberDeps(`${key}\0request-identity`, serialized.dependencies);
    return deserializeResponse(serialized, input.request.method === "HEAD");
  }

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

function hasRequestIdentityHeaders(request: Request): boolean {
  return request.headers.has("authorization") || request.headers.has("cookie");
}

export function createTrackedContext<T extends OxContentCustomHostBaseContext>(
  context: T,
  server: ViteDevServer,
  dependencies: Set<OxContentCustomHostDependency>,
): T {
  const tracked = {
    ...context,
    loadModule: async (moduleId: string) => {
      const exports = await context.loadModule(moduleId);
      for (const dependency of collectDevModuleDependencies(
        server.moduleGraph,
        moduleId,
        context.root,
      )) {
        dependencies.add(dependency);
      }
      return exports;
    },
    assets: {
      ...context.assets,
      stylesheets(input: Parameters<OxContentCustomHostBaseContext["assets"]["stylesheets"]>[0]) {
        const result = context.assets.stylesheets(input);
        for (const dependency of result.dependencies) {
          dependencies.add(dependency);
        }
        return result;
      },
      ssrStylesheets(
        input: Parameters<OxContentCustomHostBaseContext["assets"]["ssrStylesheets"]>[0],
      ) {
        const result = context.assets.ssrStylesheets(input);
        for (const dependency of result.dependencies) {
          dependencies.add(dependency);
        }
        return result;
      },
    },
  };
  return { ...tracked, markdown: createLazyCustomHostMarkdownRenderer(tracked) } as T;
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
  const trackedDependencies = new Set<OxContentCustomHostDependency>();
  const context = createTrackedContext(input.context, input.server, trackedDependencies);
  const result = await renderRoute(host, route, context, input.request);
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
    ...trackedDependencies,
    ...(input.input.dev?.dependencies ?? []),
    ...(route.dependencies ?? []),
    ...(result instanceof Response ? [] : (result.dependencies ?? [])),
  ];
  return serializeResponse(response, dependencies);
}
