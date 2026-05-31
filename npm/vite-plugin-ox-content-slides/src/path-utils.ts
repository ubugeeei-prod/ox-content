import * as path from "node:path";
import type { ResolvedSlidesPluginOptions } from "./internal-types";

/**
 * Built-in slide source extensions supported without a custom renderer.
 */
export const DEFAULT_EXTENSIONS = [".md", ".markdown", ".html"];

/**
 * Normalizes file extensions into lowercase `.ext` form.
 */
export function normalizeExtension(ext: string): string {
  return ext.startsWith(".") ? ext.toLowerCase() : `.${ext.toLowerCase()}`;
}

/**
 * Normalizes route fragments to slash-separated URL segments.
 */
export function normalizeRouteSegment(value: string): string {
  return value
    .replace(/\\/g, "/")
    .replace(/^\/+|\/+$/g, "")
    .replace(/\/{2,}/g, "/");
}

/**
 * Converts a slug-like string into a presentable title.
 */
export function formatTitle(name: string): string {
  return name
    .replace(/[-_]([a-zA-Z0-9])/g, (_, char: string) => ` ${char.toUpperCase()}`)
    .replace(/^[a-z]/, (char) => char.toUpperCase());
}

/**
 * Builds the MPA deck entry URL.
 */
export function getDeckHref(
  base: string,
  routeBase: string,
  slug: string,
  extension: string,
): string {
  const segments = [normalizeRouteSegment(base), normalizeRouteSegment(routeBase), slug]
    .filter(Boolean)
    .join("/");
  return `/${segments ? `${segments}/` : ""}index${extension}`.replace(/\/{2,}/g, "/");
}

/**
 * Builds the MPA URL for an individual slide.
 */
export function getSlideHref(
  base: string,
  routeBase: string,
  slug: string,
  slideNumber: number,
  extension: string,
): string {
  const segments = [
    normalizeRouteSegment(base),
    normalizeRouteSegment(routeBase),
    slug,
    `${slideNumber}`,
  ]
    .filter(Boolean)
    .join("/");
  return `/${segments}/index${extension}`.replace(/\/{2,}/g, "/");
}

/**
 * Builds the presenter-mode URL for a slide.
 */
export function getPresenterHref(
  base: string,
  routeBase: string,
  slug: string,
  slideNumber: number,
  extension: string,
): string {
  const segments = [
    normalizeRouteSegment(base),
    normalizeRouteSegment(routeBase),
    slug,
    "presenter",
    `${slideNumber}`,
  ]
    .filter(Boolean)
    .join("/");
  return `/${segments}/index${extension}`.replace(/\/{2,}/g, "/");
}

/**
 * Builds the output path for a deck entry page.
 */
export function getDeckOutputPath(
  outDir: string,
  routeBase: string,
  slug: string,
  extension: string,
): string {
  return path.join(outDir, routeBase, slug, `index${extension}`);
}

/**
 * Builds the output path for a deck print shell.
 */
export function getDeckPrintOutputPath(outDir: string, routeBase: string, slug: string): string {
  return path.join(outDir, routeBase, slug, "print", "index.html");
}

/**
 * Builds the output path for a deck PDF artifact.
 */
export function getDeckPdfOutputPath(
  outDir: string,
  routeBase: string,
  slug: string,
  fileName: string,
): string {
  return path.join(outDir, routeBase, slug, fileName);
}

/**
 * Builds the output path for an individual slide page.
 */
export function getSlideOutputPath(
  outDir: string,
  routeBase: string,
  slug: string,
  slideNumber: number,
  extension: string,
): string {
  return path.join(outDir, routeBase, slug, `${slideNumber}`, `index${extension}`);
}

/**
 * Builds the output path for a presenter-mode slide page.
 */
export function getPresenterOutputPath(
  outDir: string,
  routeBase: string,
  slug: string,
  slideNumber: number,
  extension: string,
): string {
  return path.join(outDir, routeBase, slug, "presenter", `${slideNumber}`, `index${extension}`);
}

/**
 * Normalizes a route to a lookup key by dropping query strings and trailing slashes.
 */
export function createRouteLookupKey(route: string): string {
  const withoutQuery = route.split("?")[0]?.split("#")[0] ?? route;
  return withoutQuery.endsWith("/") ? withoutQuery.slice(0, -1) || "/" : withoutQuery;
}

/**
 * Maps an incoming request URL back to a generated slide route key.
 */
export function getSlideRouteLookupKey(
  options: ResolvedSlidesPluginOptions,
  routeUrl: string,
): string | null {
  const normalizedRoute = createRouteLookupKey(stripBaseFromRoute(routeUrl, options.baseHref));
  const withoutIndex = normalizedRoute.endsWith("/index.html")
    ? normalizedRoute.slice(0, -"/index.html".length) || "/"
    : normalizedRoute;

  if (
    !(withoutIndex === options.routePrefix || withoutIndex.startsWith(`${options.routePrefix}/`))
  ) {
    return null;
  }

  return withoutIndex;
}

function stripBaseFromRoute(route: string, baseHref: string): string {
  if (baseHref !== "/" && route.startsWith(baseHref)) {
    return `/${route.slice(baseHref.length)}`;
  }

  return route;
}
