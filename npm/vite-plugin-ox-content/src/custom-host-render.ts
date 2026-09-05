import type { ViteDevServer } from "vite";
import type {
  OxContentCustomHostBaseContext,
  OxContentCustomHostModule,
  OxContentCustomHostRenderResult,
  OxContentCustomHostRoute,
} from "./custom-host-types";
import { resultBody, shouldTransformHtml } from "./custom-host-utils";

export async function responseFromResult(
  result: OxContentCustomHostRenderResult | Response,
  routePath: string,
  server: ViteDevServer,
  transformHtml = true,
): Promise<Response> {
  if (result instanceof Response) {
    const contentType = result.headers.get("content-type") ?? "text/html";
    if (!shouldTransformHtml(contentType, transformHtml)) {
      return result;
    }
    const headers = new Headers(result.headers);
    const html = await server.transformIndexHtml(routePath, await result.text());
    return new Response(html, {
      headers,
      status: result.status,
      statusText: result.statusText,
    });
  }

  const headers = new Headers(result.headers);
  const contentType = result.contentType ?? headers.get("content-type") ?? "text/html";
  headers.set("content-type", contentType);
  for (const dep of result.dependencies ?? []) {
    headers.append("x-ox-content-dependencies", dep);
  }

  let body = resultBody(result);
  if (typeof body === "string" && shouldTransformHtml(contentType, transformHtml)) {
    body = await server.transformIndexHtml(routePath, body);
  }
  return new Response(body, {
    headers,
    status: result.status ?? 200,
    statusText: result.statusText,
  });
}

export async function renderRoute(
  host: OxContentCustomHostModule,
  route: OxContentCustomHostRoute,
  baseContext: OxContentCustomHostBaseContext,
  request: Request,
): Promise<OxContentCustomHostRenderResult | Response | undefined> {
  const url = new URL(request.url);
  const result = await route.render({ ...baseContext, route, request, url });
  return result || undefined;
}

export async function normalizeRenderResult(
  result: OxContentCustomHostRenderResult | Response,
): Promise<OxContentCustomHostRenderResult> {
  if (!(result instanceof Response)) {
    return result;
  }
  return {
    body: new Uint8Array(await result.arrayBuffer()),
    status: result.status,
    statusText: result.statusText,
    contentType: result.headers.get("content-type") ?? undefined,
    headers: result.headers,
  };
}
