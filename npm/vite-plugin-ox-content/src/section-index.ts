/**
 * Opt-in generated section index pages.
 *
 * Resolution and directory walking live here. Listing HTML is rendered in
 * Rust (`ox_content_ssg::render_section_index`) when the NAPI helper is
 * available; a matching TypeScript renderer covers the same escape / href
 * rules so the SSG path stays safe either way. The Vite plugin appends
 * themed HTML during SSG and never overwrites an existing index page.
 */

import * as path from "node:path";
import { importNapiModuleSync } from "./napi";
import type { ResolvedSectionIndexOptions, SectionIndexOptions, SectionIndexStyle } from "./types";

const HOSTILE_SCHEME = /^(?:javascript|data|vbscript|file):/i;

/** One built page considered when deciding indexes and children. */
export interface SectionIndexSourcePage {
  title: string;
  description?: string;
  frontmatter: Record<string, unknown>;
  inputPath?: string;
  routePaths: {
    href: string;
    urlPath: string;
    outputPath?: string;
  };
}

/** Synthetic page passed back to `generateHtmlPage`. */
export interface SectionIndexGeneratedPage {
  title: string;
  content: string;
  outputPath: string;
  urlPath: string;
  href: string;
}

/** One child link on a generated section index. */
export interface SectionIndexItem {
  title: string;
  href: string;
  description?: string;
}

/**
 * Resolves `ssg.sectionIndex` with defaults.
 *
 * `false` / omitted stays off. `true` enables card listings. An object
 * enables the feature and overrides only the fields the site set.
 */
export function resolveSectionIndexOptions(
  value: boolean | SectionIndexOptions | undefined,
): ResolvedSectionIndexOptions {
  if (!value) {
    return { enabled: false, style: "cards" };
  }
  if (value === true) {
    return { enabled: true, style: "cards" };
  }
  return {
    enabled: true,
    style: value.style === "list" ? "list" : "cards",
  };
}

/** `https:`-free, same-origin or relative href. `javascript:` is rejected. */
export function isSafeSectionHref(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed || /[\n\r\0\t]/.test(trimmed) || trimmed.startsWith("//")) {
    return false;
  }
  if (trimmed.startsWith("/")) {
    return true;
  }
  const scheme = trimmed.match(/^([a-zA-Z][a-zA-Z0-9+.-]*):/);
  if (scheme) {
    return false;
  }
  return !HOSTILE_SCHEME.test(trimmed);
}

/** Escapes text and attribute values in generated listing markup. */
export function escapeSectionIndexHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

/** Renders the listing body. Titles are escaped; hostile hrefs are dropped. */
export function renderSectionIndexHtml(
  title: string,
  items: readonly SectionIndexItem[],
  style: SectionIndexStyle,
): string {
  try {
    const napi = importNapiModuleSync() as typeof import("@ox-content/napi") & {
      renderSsgSectionIndex?: (
        title: string,
        items: Array<{ title: string; href: string; description?: string }>,
        style: string,
      ) => string;
    };
    if (typeof napi.renderSsgSectionIndex === "function") {
      return napi.renderSsgSectionIndex(
        title,
        items.map((item) => ({
          title: item.title,
          href: item.href,
          description: item.description,
        })),
        style,
      );
    }
  } catch {
    // Fall through to the local renderer when the native helper is absent.
  }
  return renderSectionIndexHtmlLocal(title, items, style);
}

function renderSectionIndexHtmlLocal(
  title: string,
  items: readonly SectionIndexItem[],
  style: SectionIndexStyle,
): string {
  const safe = items.filter((item) => isSafeSectionHref(item.href));
  const modifier = style === "list" ? "list" : "cards";
  const listClass = style === "list" ? "ox-section-index__list" : "ox-section-index__cards";
  const body = safe.map((item) => renderItem(item, style)).join("");
  return (
    `<nav class="ox-section-index ox-section-index--${modifier}" aria-label="Section pages">` +
    `<h1>${escapeSectionIndexHtml(title)}</h1>` +
    `<ul class="${listClass}">${body}</ul>` +
    `</nav>`
  );
}

function renderItem(item: SectionIndexItem, style: SectionIndexStyle): string {
  const href = escapeSectionIndexHtml(item.href.trim());
  const label = escapeSectionIndexHtml(item.title);
  if (style === "list") {
    return `<li><a href="${href}">${label}</a></li>`;
  }
  const description =
    typeof item.description === "string" && item.description.trim()
      ? `<span class="ox-section-index__desc">${escapeSectionIndexHtml(item.description)}</span>`
      : "";
  return (
    `<li class="ox-section-index__card">` +
    `<a href="${href}"><span class="ox-section-index__title">${label}</span>${description}</a>` +
    `</li>`
  );
}

/** Maps a generated section index onto the SSG render shape. */
export function toSectionIndexProcessResult(page: SectionIndexGeneratedPage): {
  inputPath: string;
  routePaths: {
    outputPath: string;
    urlPath: string;
    href: string;
    ogImagePath: string;
    ogImageUrl: string;
  };
  transformedHtml: string;
  title: string;
  frontmatter: Record<string, unknown>;
  toc: [];
} {
  return {
    inputPath: page.outputPath,
    routePaths: {
      outputPath: page.outputPath,
      urlPath: page.urlPath,
      href: page.href,
      ogImagePath: "",
      ogImageUrl: "",
    },
    transformedHtml: page.content,
    title: page.title,
    frontmatter: {},
    toc: [],
  };
}

/** Appends generated section indexes for directories that have no real index. */
export async function appendSectionIndexPages(input: {
  generatedPages: Array<{ inputPath: string; outputPath: string; html: string }>;
  collectedPages: readonly SectionIndexSourcePage[];
  listedPages: readonly SectionIndexSourcePage[];
  options?: ResolvedSectionIndexOptions;
  outDir: string;
  base: string;
  extension: string;
  errors: string[];
  render: (page: SectionIndexGeneratedPage) => Promise<string>;
}): Promise<void> {
  if (!input.options?.enabled) {
    return;
  }

  const existingOutputs = new Set(
    input.generatedPages.map((page) => path.normalize(page.outputPath)),
  );
  for (const spec of sectionIndexSpecs(
    input.collectedPages,
    input.listedPages,
    input.options,
    input.outDir,
    input.base,
    input.extension,
  )) {
    if (existingOutputs.has(path.normalize(spec.outputPath))) {
      continue;
    }
    try {
      const html = await input.render(spec);
      input.generatedPages.push({
        inputPath: spec.outputPath,
        outputPath: spec.outputPath,
        html,
      });
      existingOutputs.add(path.normalize(spec.outputPath));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      input.errors.push(`Failed to generate section index ${spec.href}: ${message}`);
    }
  }
}

function sectionIndexSpecs(
  collected: readonly SectionIndexSourcePage[],
  listed: readonly SectionIndexSourcePage[],
  options: ResolvedSectionIndexOptions,
  outDir: string,
  base: string,
  extension: string,
): SectionIndexGeneratedPage[] {
  const occupied = new Set<string>();
  for (const page of collected) {
    occupied.add(normalizeUrlPath(page.routePaths.urlPath));
  }
  for (const page of collected) {
    const output = page.routePaths.outputPath;
    if (output) {
      occupied.add(dirFromOutputPath(output, outDir));
    }
  }

  const visible = listed.filter((page) => !isHiddenByFlags(page.frontmatter));
  const childrenByDir = new Map<string, SectionIndexItem[]>();

  for (const page of visible) {
    const urlPath = normalizeUrlPath(page.routePaths.urlPath);
    const parent = parentDir(urlPath);
    if (parent === null) {
      continue;
    }
    pushChild(childrenByDir, parent, {
      title: pageTitle(page),
      href: page.routePaths.href,
      description: page.description,
    });

    let ancestor = parent;
    while (ancestor !== "") {
      const grand = parentDir(ancestor);
      if (grand === null) {
        break;
      }
      const nested = firstChildDir(urlPath, grand);
      if (nested) {
        pushUniqueDir(childrenByDir, grand, nested, visible, base, extension);
      }
      ancestor = grand;
    }
  }

  const pages: SectionIndexGeneratedPage[] = [];
  const dirs = [...childrenByDir.keys()].sort();
  for (const dir of dirs) {
    if (occupied.has(dir)) {
      continue;
    }
    const children = uniqueItems(childrenByDir.get(dir) ?? []).filter((item) =>
      isSafeSectionHref(item.href),
    );
    if (children.length === 0) {
      continue;
    }
    children.sort((left, right) => {
      const titleCmp = left.title.localeCompare(right.title);
      return titleCmp !== 0 ? titleCmp : left.href.localeCompare(right.href);
    });
    const outputPath = sectionOutputPath(outDir, dir, extension);
    if (!outputPath) {
      continue;
    }
    const title = sectionTitle(dir);
    pages.push({
      title,
      content: renderSectionIndexHtml(title, children, options.style),
      outputPath,
      urlPath: dir || "/",
      href: sectionHref(base, dir, extension),
    });
  }
  return pages;
}

function pushChild(
  map: Map<string, SectionIndexItem[]>,
  dir: string,
  item: SectionIndexItem,
): void {
  const list = map.get(dir);
  if (list) {
    list.push(item);
    return;
  }
  map.set(dir, [item]);
}

function pushUniqueDir(
  map: Map<string, SectionIndexItem[]>,
  parent: string,
  childDir: string,
  visible: readonly SectionIndexSourcePage[],
  base: string,
  extension: string,
): void {
  const href = sectionHref(base, childDir, extension);
  const existing = map.get(parent);
  if (existing?.some((item) => item.href === href)) {
    return;
  }
  const indexPage = visible.find((page) => normalizeUrlPath(page.routePaths.urlPath) === childDir);
  pushChild(map, parent, {
    title: indexPage ? pageTitle(indexPage) : sectionTitle(childDir),
    href: indexPage?.routePaths.href ?? href,
    description: indexPage?.description,
  });
}

function uniqueItems(items: SectionIndexItem[]): SectionIndexItem[] {
  const seen = new Set<string>();
  const unique: SectionIndexItem[] = [];
  for (const item of items) {
    if (seen.has(item.href)) {
      continue;
    }
    seen.add(item.href);
    unique.push(item);
  }
  return unique;
}

function isHiddenByFlags(frontmatter: Record<string, unknown>): boolean {
  return frontmatter.draft === true || frontmatter.unlisted === true;
}

function pageTitle(page: SectionIndexSourcePage): string {
  if (page.title.trim()) {
    return page.title;
  }
  const stem = path.basename(page.inputPath ?? page.routePaths.urlPath).replace(/\.[^.]+$/, "");
  return formatSectionTitle(stem || page.routePaths.urlPath);
}

function sectionTitle(dir: string): string {
  if (!dir) {
    return "Home";
  }
  const segment = dir.slice(dir.lastIndexOf("/") + 1);
  return formatSectionTitle(segment);
}

function formatSectionTitle(name: string): string {
  try {
    return importNapiModuleSync().formatSsgTitle(name);
  } catch {
    if (!name) {
      return "Untitled";
    }
    return name.charAt(0).toUpperCase() + name.slice(1).replace(/[-_]+/g, " ");
  }
}

function normalizeUrlPath(urlPath: string): string {
  if (!urlPath || urlPath === "/") {
    return "";
  }
  return urlPath.replace(/^\/+|\/+$/g, "");
}

function parentDir(urlPath: string): string | null {
  const normalized = normalizeUrlPath(urlPath);
  if (!normalized) {
    return null;
  }
  const index = normalized.lastIndexOf("/");
  return index === -1 ? "" : normalized.slice(0, index);
}

function firstChildDir(urlPath: string, parent: string): string | undefined {
  const normalized = normalizeUrlPath(urlPath);
  if (!normalized) {
    return undefined;
  }
  if (!parent) {
    const slash = normalized.indexOf("/");
    return slash === -1 ? undefined : normalized.slice(0, slash);
  }
  const prefix = `${parent}/`;
  if (!normalized.startsWith(prefix) || normalized === parent) {
    return undefined;
  }
  const rest = normalized.slice(prefix.length);
  const slash = rest.indexOf("/");
  return slash === -1 ? undefined : `${parent}/${rest.slice(0, slash)}`;
}

function sectionHref(base: string, dir: string, extension: string): string {
  const prefix = !base || base === "/" ? "/" : base.endsWith("/") ? base : `${base}/`;
  const ext = extension.startsWith(".") ? extension : `.${extension}`;
  return dir ? `${prefix}${dir}/index${ext}` : `${prefix}index${ext}`;
}

function sectionOutputPath(outDir: string, dir: string, extension: string): string | undefined {
  const ext = extension.startsWith(".") ? extension : `.${extension}`;
  const segments = dir ? [...dir.split("/").filter(Boolean), `index${ext}`] : [`index${ext}`];
  return containedPath(outDir, ...segments);
}

function dirFromOutputPath(outputPath: string, outDir: string): string {
  const relative = path.relative(path.resolve(outDir), path.resolve(outputPath)).replaceAll(
    path.sep,
    "/",
  );
  return relative
    .replace(/\/index\.[^/]+$/u, "")
    .replace(/^index\.[^/]+$/u, "")
    .replace(/^\/+|\/+$/g, "");
}

function containedPath(outDir: string, ...segments: string[]): string | undefined {
  const root = path.resolve(outDir);
  const resolved = path.resolve(root, ...segments);
  const prefix = root.endsWith(path.sep) ? root : `${root}${path.sep}`;
  if (resolved !== root && !resolved.startsWith(prefix)) {
    return undefined;
  }
  if (segments.some((segment) => segment === ".." || segment.includes("\0"))) {
    return undefined;
  }
  return resolved;
}
