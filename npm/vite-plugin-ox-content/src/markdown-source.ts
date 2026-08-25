/**
 * Opt-in Markdown source companions written beside generated HTML.
 *
 * Copies already-read source bytes. Does not re-parse Markdown to emit them.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import { importNapiModuleSync } from "./napi";
import { escapeAttribute, resolvePageRoutes } from "./permalinks";
import { classifyPublishState } from "./publish-state";
import type {
  MarkdownSourceOptions,
  ResolvedCascadeOptions,
  ResolvedMarkdownSourceOptions,
  ResolvedPermalinksOptions,
  ResolvedPublishStateOptions,
} from "./types";

/** One page that may receive a source companion. */
export interface MarkdownSourcePageInput {
  inputPath: string;
  /** Already-read source bytes. Omitted pages are skipped. */
  source?: string;
  urlPath: string;
  frontmatter: Record<string, unknown>;
}

/** Inputs for writing companions next to generated HTML. */
export interface WriteMarkdownSourceFilesInput {
  outDir: string;
  base: string;
  options?: ResolvedMarkdownSourceOptions | null;
  publishState?: ResolvedPublishStateOptions;
  pages: readonly MarkdownSourcePageInput[];
}

/** One entry in the dev-server companion index. */
export interface MarkdownSourceIndexEntry {
  source: string;
  allowed: boolean;
}

/**
 * Resolves `ssg.markdownSource` with defaults.
 *
 * `false` / omitted stays off. `true` enables companions and the alternate
 * link. An object enables the feature and overrides only the fields set.
 */
export function resolveMarkdownSourceOptions(
  value: boolean | MarkdownSourceOptions | undefined,
): ResolvedMarkdownSourceOptions {
  if (!value) {
    return { enabled: false, alternate: true };
  }
  if (value === true) {
    return { enabled: true, alternate: true };
  }
  return {
    enabled: true,
    alternate: value.alternate !== false,
  };
}

/** Companion href for one page after permalink / publish-state checks. */
export function markdownSourceHrefForPage(input: {
  source: string;
  fileUrl: string;
  frontmatter: Record<string, unknown>;
  base: string;
  permalinks?: ResolvedPermalinksOptions | null;
  cascade?: ResolvedCascadeOptions | null;
  publishState?: ResolvedPublishStateOptions;
}): string | undefined {
  if (!shouldPublishMarkdownSource(input.frontmatter, input.publishState)) {
    return undefined;
  }
  const routed = resolvePageRoutes({
    pages: [{ source: input.source, fileUrl: input.fileUrl, frontmatter: input.frontmatter }],
    permalinks: input.permalinks,
    cascade: input.cascade,
  });
  const urlPath = routed.pages[0]?.urlPath ?? input.fileUrl;
  return markdownSourceHref(urlPath, input.base);
}

/** Public companion href, including `base`. Always ends in `.md`. */
export function markdownSourceHref(urlPath: string, base: string): string | undefined {
  const relative = companionRelativePath(urlPath);
  if (!relative) {
    return undefined;
  }
  return `${normalizeBase(base)}${relative}`;
}

/** Filesystem path for a companion, or `undefined` when it would escape `outDir`. */
export function markdownSourceOutputPath(outDir: string, urlPath: string): string | undefined {
  const relative = companionRelativePath(urlPath);
  if (!relative) {
    return undefined;
  }
  return containedPath(outDir, ...relative.split("/"));
}

/**
 * Whether this page may publish a companion.
 *
 * Draft and unlisted source is never emitted. When `publishState` is on,
 * scheduled / expired pages follow that filter and `includeDrafts` is ignored
 * so preview HTML cannot leak source.
 */
export function shouldPublishMarkdownSource(
  frontmatter: Record<string, unknown>,
  publishState?: ResolvedPublishStateOptions,
): boolean {
  if (frontmatter.draft === true || frontmatter.unlisted === true) {
    return false;
  }
  if (!publishState?.enabled) {
    return true;
  }
  return classifyPublishState(frontmatter, { ...publishState, includeDrafts: false }).output;
}

/** Inserts `<link rel="alternate" type="text/markdown">` before `</head>`. */
export function injectMarkdownSourceAlternate(html: string, href: string): string {
  if (!href || !/<\/head>/i.test(html)) {
    return html;
  }
  const tag = `<link rel="alternate" type="text/markdown" href="${escapeAttribute(href)}">`;
  const index = html.toLowerCase().lastIndexOf("</head>");
  return `${html.slice(0, index)}  ${tag}\n${html.slice(index)}`;
}

/** Writes enabled companions from already-read source bytes. */
export async function writeMarkdownSourceFiles(
  input: WriteMarkdownSourceFilesInput,
): Promise<{ files: string[]; errors: string[] }> {
  if (!input.options?.enabled) {
    return { files: [], errors: [] };
  }

  const files: string[] = [];
  const errors: string[] = [];
  const seen = new Map<string, string>();
  for (const page of input.pages) {
    if (page.source == null || !shouldPublishMarkdownSource(page.frontmatter, input.publishState)) {
      continue;
    }
    const outputPath = markdownSourceOutputPath(input.outDir, page.urlPath);
    if (!outputPath) {
      errors.push(`[ox-content] markdownSource skipped path-escape for ${page.inputPath}`);
      continue;
    }
    const previous = seen.get(outputPath);
    if (previous) {
      errors.push(
        `[ox-content] markdownSource collision: ${page.inputPath} and ${previous} both map to ${outputPath}`,
      );
      continue;
    }
    seen.set(outputPath, page.inputPath);
    await fs.mkdir(path.dirname(outputPath), { recursive: true });
    await fs.writeFile(outputPath, page.source);
    files.push(outputPath);
  }
  return { files, errors };
}

/** True when the request pathname is a `.md` companion URL. */
export function isMarkdownSourceRequest(pathname: string): boolean {
  const clean = stripSearch(pathname);
  return clean.toLowerCase().endsWith(".md") && !clean.includes("\\");
}

/** Builds a companion index from source files without transforming Markdown. */
export async function buildMarkdownSourceIndex(input: {
  files: readonly string[];
  srcDir: string;
  permalinks?: ResolvedPermalinksOptions | null;
  cascade?: ResolvedCascadeOptions | null;
  publishState?: ResolvedPublishStateOptions;
}): Promise<Map<string, MarkdownSourceIndexEntry>> {
  const loaded = await Promise.all(
    input.files.map(async (file) => {
      const source = await fs.readFile(file, "utf8");
      return {
        source: file,
        fileUrl: importNapiModuleSync().getSsgUrlPath(file, input.srcDir),
        frontmatter: parseSourceFrontmatter(source),
        body: source,
      };
    }),
  );
  const routed = resolvePageRoutes({
    pages: loaded.map(({ source, fileUrl, frontmatter }) => ({ source, fileUrl, frontmatter })),
    permalinks: input.permalinks,
    cascade: input.cascade,
  });
  const bodies = new Map(loaded.map((page) => [page.source, page.body]));
  const index = new Map<string, MarkdownSourceIndexEntry>();
  for (const page of routed.pages) {
    const href = markdownSourceHref(page.urlPath, "/");
    const body = bodies.get(page.source);
    if (!href || body == null) {
      continue;
    }
    index.set(normalizePathname(href), {
      source: body,
      allowed: shouldPublishMarkdownSource(page.frontmatter, input.publishState),
    });
  }
  return index;
}

/** Looks up a companion after the site `base` has been stripped. */
export function resolveMarkdownSourceRequest(
  pathname: string,
  index: ReadonlyMap<string, MarkdownSourceIndexEntry>,
): MarkdownSourceIndexEntry | undefined {
  if (!isMarkdownSourceRequest(pathname)) {
    return undefined;
  }
  return index.get(normalizePathname(stripSearch(pathname)));
}

/** Frontmatter keys only — not a Markdown parse. */
export function parseSourceFrontmatter(source: string): Record<string, unknown> {
  const match = source.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match?.[1]) {
    return {};
  }
  const result: Record<string, unknown> = {};
  for (const line of match[1].split("\n")) {
    const kv = line.match(/^([A-Za-z_][\w-]*)\s*:\s*(.*)$/);
    if (!kv) {
      continue;
    }
    result[kv[1]] = parseFrontmatterScalar(kv[2].trim());
  }
  return result;
}

function parseFrontmatterScalar(value: string): unknown {
  if (value === "true") {
    return true;
  }
  if (value === "false") {
    return false;
  }
  if (
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    return value.slice(1, -1);
  }
  return value;
}

function companionRelativePath(urlPath: string): string | undefined {
  const trimmed = urlPath === "/" || !urlPath ? "index" : urlPath.replace(/^\/+|\/+$/gu, "");
  if (!trimmed) {
    return undefined;
  }
  const segments = trimmed.split("/");
  if (segments.some((segment) => !segment || segment === "." || segment === "..")) {
    return undefined;
  }
  return `${trimmed}.md`;
}

function containedPath(outDir: string, ...segments: string[]): string | undefined {
  const root = path.resolve(outDir);
  const resolved = path.resolve(root, ...segments);
  const prefix = root.endsWith(path.sep) ? root : `${root}${path.sep}`;
  if (resolved === root || !resolved.startsWith(prefix)) {
    return undefined;
  }
  return resolved;
}

function normalizeBase(base: string): string {
  if (!base || base === "/") {
    return "/";
  }
  return base.endsWith("/") ? base : `${base}/`;
}

function stripSearch(pathname: string): string {
  return pathname.split("?")[0]?.split("#")[0] ?? pathname;
}

function normalizePathname(pathname: string): string {
  const clean = stripSearch(pathname);
  if (!clean || clean === "/") {
    return "/";
  }
  const withSlash = clean.startsWith("/") ? clean : `/${clean}`;
  return withSlash.length > 1 && withSlash.endsWith("/") ? withSlash.slice(0, -1) : withSlash;
}
