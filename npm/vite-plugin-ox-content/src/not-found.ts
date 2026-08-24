/**
 * Opt-in custom 404 page helpers.
 *
 * HTML generation follows `ox_content_ssg::generate_not_found`. The Vite
 * plugin writes the file during SSG after the Markdown source is transformed.
 */

import * as path from "node:path";
import type { NotFoundOptions, ResolvedNotFoundOptions } from "./types";

const DEFAULT_SOURCE = "404.md";
const ROBOTS_NOINDEX = '<meta name="robots" content="noindex">';

/**
 * Resolves `notFound` with defaults.
 *
 * `false` / omitted stays off. `true` enables `404.md`. An object enables
 * the feature and overrides only the fields the site set.
 */
export function resolveNotFoundOptions(
  value: boolean | NotFoundOptions | undefined,
): ResolvedNotFoundOptions {
  if (!value) {
    return { enabled: false, source: DEFAULT_SOURCE };
  }
  if (value === true) {
    return { enabled: true, source: DEFAULT_SOURCE };
  }
  return {
    enabled: true,
    source: value.source?.trim() || DEFAULT_SOURCE,
  };
}

/** Absolute Markdown path used when the feature is on. */
export function resolveNotFoundSourcePath(
  srcDir: string,
  options: Pick<ResolvedNotFoundOptions, "source">,
): string {
  return path.resolve(srcDir, options.source);
}

/** Virtual `404.md` path so output follows the site's SSG URL style. */
export function notFoundVirtualInputPath(srcDir: string): string {
  return path.resolve(srcDir, DEFAULT_SOURCE);
}

/**
 * True when `file` is the 404 source and should not be published as a
 * regular page (disabled by default, or handled separately when enabled).
 */
export function isNotFoundSourcePath(
  file: string,
  srcDir: string,
  options?: Pick<ResolvedNotFoundOptions, "source">,
): boolean {
  const resolved = path.resolve(file);
  if (resolved === resolveNotFoundSourcePath(srcDir, options ?? { source: DEFAULT_SOURCE })) {
    return true;
  }
  return isDefaultNotFoundFile(resolved, srcDir);
}

/** Warning emitted when the feature is on but the Markdown source is missing. */
export function missingNotFoundSourceWarning(source: string): string {
  return `[ox-content] notFound is enabled but ${source} was not found; the 404 page was not written`;
}

/** Inserts a robots noindex tag so crawlers skip the 404 document. */
export function applyNotFoundNoindex(html: string): string {
  if (html.includes(ROBOTS_NOINDEX)) {
    return html;
  }
  return html.replace("<head>", `<head>\n  ${ROBOTS_NOINDEX}`);
}

function isDefaultNotFoundFile(file: string, srcDir: string): boolean {
  const relative = path.relative(srcDir, file).replace(/\\/g, "/");
  if (relative.startsWith("..") || path.isAbsolute(relative) || relative.includes("/")) {
    return false;
  }
  const name = path.basename(file).toLowerCase();
  return name === "404.md" || name === "404.mdx" || name === "404.markdown";
}
