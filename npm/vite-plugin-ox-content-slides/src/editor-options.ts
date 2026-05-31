import { normalizeRouteSegment } from "./path-utils";

/**
 * Public configuration for the browser-based slide editor.
 */
export interface SlideEditorOptions {
  /** Enables the dev-server editor route. */
  enabled?: boolean;
  /** Route mounted under the Vite dev server. Defaults to `<routeBase>/editor`. */
  route?: string;
}

/**
 * Normalized editor options used by dev-server middleware.
 */
export interface ResolvedSlideEditorOptions {
  enabled: boolean;
  route: string;
  routePrefix: string;
  apiPrefix: string;
}

/**
 * Resolves the editor option while keeping the production slide bundle untouched.
 */
export function resolveSlideEditorOptions(
  editor: SlideEditorOptions | boolean | undefined,
  routeBase: string,
): ResolvedSlideEditorOptions {
  if (editor === false) {
    return {
      enabled: false,
      route: "",
      routePrefix: "",
      apiPrefix: "",
    };
  }

  const route =
    normalizeRouteSegment(typeof editor === "object" ? (editor.route ?? "") : "") ||
    normalizeRouteSegment(`${routeBase}/editor`);
  const routePrefix = `/${route}`;

  return {
    enabled: typeof editor === "object" ? (editor.enabled ?? true) : true,
    route,
    routePrefix,
    apiPrefix: `${routePrefix}/api`,
  };
}
